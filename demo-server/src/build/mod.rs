//! The gallery's full build pipeline — resolving staging paths, validating the catalogue, assembling `index.html`,
//! copying static assets, and rebuilding the wasm package — factored out of `main` so it can run, and be tested,
//! without ever touching Actix or the network.
//!
//! Each phase returns a `Result` rather than terminating the process itself, the same policy [`panels::assemble`]
//! and [`validate::validate`] already followed; this module extends that discipline to the two phases that used to
//! call `process::exit` directly (copying assets, running `wasm-pack`).
//!
//! The pipeline is split into two layers rather than one:
//!  - [`prepare_gallery`] runs every phase except the wasm build: validate the catalogue, assemble `index.html`,
//!    substitute the port placeholder, copy static assets. This is the part `wasm-pack build demo-app ...` on its
//!    own — the invocation CI's `wasm` job runs to build the wasm package — does not exercise at all.
//!  - [`build_gallery`] runs [`prepare_gallery`] and then rebuilds the wasm package, which is what `cargo demo`
//!    actually needs to serve the gallery.
//!
//! Separating them means CI (via `demo-server --prepare-only`, see `main`) or a test can exercise catalogue
//! validation, fragment validation, template assembly, and asset staging together — the actual `demo-server`
//! pipeline, not just each phase's own unit tests in isolation — without paying for a full wasm build every time.
//! Deciding what a failure means for the process — report to stderr, `exit(1)` — stays `main`'s job alone.
use crate::{panels, validate};
use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Where the gallery's generated `index.html` and wasm package are staged (see `main`'s own doc comment for the
/// full directory layout). Bundled into one struct since every phase below needs some subset of these same three
/// paths, derived together from a single `target_dir`.
pub struct StagePaths {
    pub stage_dir: PathBuf,
    pub demo_dir: PathBuf,
    pub pkg_dir: PathBuf,
}

impl StagePaths {
    /// `target_dir` is the already-resolved Cargo target directory (respecting a `CARGO_TARGET_DIR` override —
    /// see `main`), not necessarily `root.join("target")`.
    pub fn new(target_dir: &Path) -> Self {
        let stage_dir = target_dir.join("demo-gallery");
        let demo_dir = stage_dir.join("demo");
        let pkg_dir = stage_dir.join("pkg");
        Self { stage_dir, demo_dir, pkg_dir }
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Everything that can go wrong building the gallery, across every phase [`build_gallery`] runs. `main` reports
/// whichever variant it gets via `Display` and exits — see this module's own doc comment for why building itself
/// never does that.
#[derive(Debug)]
pub enum BuildError {
    /// The staging directory could not be created.
    CreateStageDir { path: PathBuf, source: io::Error },
    /// A static asset (`style.css`, `view-demo.svg`) could not be copied into the staging directory.
    CopyAsset { src: PathBuf, dest: PathBuf, source: io::Error },
    /// The panel manifest and `demo_gallery!` have drifted apart, or the source catalogue is otherwise invalid —
    /// see [`validate::ValidationError`].
    Validate(validate::ValidationError),
    /// `index.html` could not be assembled — see [`panels::AssembleError`].
    Assemble(panels::AssembleError),
    /// `wasm-pack` could not even be started (e.g. not on `PATH`).
    WasmSpawn(io::Error),
    /// `wasm-pack` ran but exited with a non-success status.
    WasmBuildFailed(ExitStatus),
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CreateStageDir { path, source } => write!(f, "could not create {} ({source})", path.display()),
            Self::CopyAsset { src, dest, source } => {
                write!(f, "could not copy {} to {} ({source})", src.display(), dest.display())
            },
            Self::Validate(err) => write!(f, "{err}"),
            Self::Assemble(err) => write!(f, "{err}"),
            Self::WasmSpawn(err) => write!(f, "could not run wasm-pack ({err})"),
            Self::WasmBuildFailed(status) => write!(f, "wasm-pack exited with {status}"),
        }
    }
}

impl std::error::Error for BuildError {}

impl From<validate::ValidationError> for BuildError {
    fn from(err: validate::ValidationError) -> Self {
        Self::Validate(err)
    }
}

impl From<panels::AssembleError> for BuildError {
    fn from(err: panels::AssembleError) -> Self {
        Self::Assemble(err)
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Runs every gallery-build phase except the wasm rebuild: validates the catalogue, assembles `index.html`
/// (substituting the port placeholder), and copies the static assets it references — everything needed to stage a
/// servable `demo/` directory except `pkg/`. Returns as soon as any phase fails, via `?`, the same short-circuiting
/// [`panels::assemble`] and [`validate::validate`] already do individually: nothing after a failed phase runs, so
/// (for example) a stale catalogue is caught before `index.html` is ever written.
pub fn prepare_gallery(root: &Path, stage: &StagePaths, port: u16) -> Result<(), BuildError> {
    fs::create_dir_all(&stage.demo_dir).map_err(|source| BuildError::CreateStageDir {
        path: stage.demo_dir.clone(),
        source,
    })?;

    // Check whether the panel manifest is out of sync with the demo gallery before it produces a broken or
    // incomplete gallery, rather than after.
    validate::validate(root)?;

    // Rebuild index.html from the source template and panel fragments before wasm-pack or the server ever touch
    // it, so both always see the current assembled file, not some stale one from a previous run. The source demo/
    // directory is read from, never written to — panels::assemble takes its input and output paths independently.
    let source_demo_dir = root.join("demo");
    panels::assemble(&source_demo_dir, &stage.demo_dir.join("index.html"), port)?;

    // style.css and view-demo.svg are not generated — they are static assets index.html references by a plain
    // relative path, so they need to sit alongside the generated file in the staging directory too.
    copy_asset(&source_demo_dir.join("style.css"), &stage.demo_dir.join("style.css"))?;
    copy_asset(&source_demo_dir.join("view-demo.svg"), &stage.demo_dir.join("view-demo.svg"))?;

    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Runs [`prepare_gallery`] and then rebuilds the wasm package, leaving only starting the Actix server itself to
/// `main`. This is the full pipeline `cargo demo` needs; `main --prepare-only` runs [`prepare_gallery`] alone
/// instead (see this module's own doc comment for why that split exists).
pub fn build_gallery(root: &Path, stage: &StagePaths, port: u16) -> Result<(), BuildError> {
    prepare_gallery(root, stage, port)?;
    build_wasm(root, &stage.pkg_dir)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Rebuilds the `svg-dom-demo` crate's wasm package into `out_dir` so the served `pkg/` is up to date.
fn build_wasm(root: &Path, out_dir: &Path) -> Result<(), BuildError> {
    println!(
        "Building wasm package: wasm-pack build demo-app --target web --out-dir {}",
        out_dir.display()
    );

    // `demo-app` is a separate workspace crate (`svg-dom-demo`) consuming `svg-dom` only through its public API —
    // see that crate's own doc comment for why. `--out-dir` is given as an absolute path so it lands exactly at
    // `out_dir` regardless of `demo-app`'s own location, rather than relying on relative-path arithmetic from it.
    let status = Command::new("wasm-pack")
        .current_dir(root)
        .arg("build")
        .arg("demo-app")
        .args(["--target", "web", "--out-dir"])
        .arg(out_dir)
        .status()
        .map_err(BuildError::WasmSpawn)?;

    if status.success() { Ok(()) } else { Err(BuildError::WasmBuildFailed(status)) }
}

fn copy_asset(src: &Path, dest: &Path) -> Result<(), BuildError> {
    fs::copy(src, dest).map(|_| ()).map_err(|source| BuildError::CopyAsset {
        src: src.to_path_buf(),
        dest: dest.to_path_buf(),
        source,
    })
}

#[cfg(test)]
mod unit_tests;
