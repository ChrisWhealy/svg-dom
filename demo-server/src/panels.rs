//! Assembles `demo/index.html` using `demo/index.template.html` and the various panel fragments in `demo/panels/`.
//!
//! The gallery used to be a single, hand-edited, ever-growing `index.html`. Now, every demo's `<section>` lives in its
//! own file under `demo/panels/` and the job of this module is to stitch them back into the single static file
//! that `demo-server` will then serve.
//!
//! `Files::new` in `main.rs` resolves `/demo/` to `demo/index.html` on disk, so that file has to exist there, generated
//! or not.
//!
//! [`MANIFEST`] acts as the gallery's single source of truth for both panel *order* and *labelling*: it drives both the
//! generated `<nav>` menu and the generated panel body, so the two can never disagree about:
//!
//!  - which panels exist
//!  - what category each belongs to, or
//!  - what order they come in.
//!
//! It does not know anything about the Rust demo functions that build each panel's content — that mapping lives in
//! `demo-app/src/lib.rs`'s `demo_gallery!` invocation instead, which holds a separate list for a separate reason (see
//! that macro's doc comment). The job of [`super::validate`] is to keep the two ids in step.
//!
//! # Adding A New Demo
//!
//! Add a new demo by adding its id and label here, creating the matching `demo/panels/{id}.html` fragment, and
//! adding the matching `demo_gallery!` entry in `demo-app/src/lib.rs`.
//!
//! [`assemble`] checks the catalogue's internal consistency, not just that referenced files exist and are readable, but
//! that [`MANIFEST`], the generated menu and the fragments directory all agree on the same set of panels — before it
//! ever writes `index.html`.
//!
//! See [`AssembleError`] for exactly what it checks and why each of those checks exists.

use std::{
    collections::HashSet,
    fmt, fs, io,
    path::{Path, PathBuf},
};

const PANELS_PLACEHOLDER: &str = "{{PANELS}}";
const MENU_PLACEHOLDER: &str = "{{MENU}}";
const SERVER_PORT: &str = "{{SERVER_PORT}}";

/// Every category divider in the source uses this exact comment text, verified against the original hand-written
/// `index.html` when this module was split out — see `docs/design_notes/` for how, if that history matters later.
const DIVIDER: &str = "            <!-- = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = -->";

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// One entry in the gallery's section order: either a category divider (also a menu group), or a single panel
/// whose body markup lives in `demo/panels/{id}.html` and whose menu button reads `label`.
enum Entry {
    Category(&'static str),
    Panel { id: &'static str, label: &'static str },
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[rustfmt::skip]
const MANIFEST: &[Entry] = &[
    Entry::Category("Basic Shapes"),
    Entry::Panel { id: "panel-rect", label: "rect" },
    Entry::Panel { id: "panel-circle", label: "circle" },
    Entry::Panel { id: "panel-ellipse", label: "ellipse" },
    Entry::Panel { id: "panel-line", label: "line" },
    Entry::Panel { id: "panel-poly", label: "polygon / polyline" },
    Entry::Panel { id: "panel-group", label: "group" },
    Entry::Category("Path"),
    Entry::Panel { id: "panel-path", label: "path" },
    Entry::Category("Text"),
    Entry::Panel { id: "panel-text", label: "text" },
    Entry::Panel { id: "panel-tspan", label: "tspan" },
    Entry::Panel { id: "panel-text-path", label: "textPath" },
    Entry::Category("Structural & Reusable Elements"),
    Entry::Panel { id: "panel-marker", label: "defs / marker" },
    Entry::Panel { id: "panel-marker-view-box", label: "marker set_view_box" },
    Entry::Panel { id: "panel-use", label: "use" },
    Entry::Panel { id: "panel-image", label: "image" },
    Entry::Panel { id: "panel-symbol", label: "symbol" },
    Entry::Panel { id: "panel-anchor", label: "a" },
    Entry::Panel { id: "panel-switch", label: "switch" },
    Entry::Panel { id: "panel-view", label: "view" },
    Entry::Panel { id: "panel-style", label: "style" },
    Entry::Panel { id: "panel-foreign-object", label: "foreignObject" },
    Entry::Category("Paint Servers"),
    Entry::Panel { id: "panel-linear-gradient", label: "linearGradient" },
    Entry::Panel { id: "panel-radial-gradient", label: "radialGradient" },
    Entry::Panel { id: "panel-pattern", label: "pattern" },
    Entry::Category("Clipping & Masking"),
    Entry::Panel { id: "panel-clip-path", label: "clipPath" },
    Entry::Panel { id: "panel-mask", label: "mask" },
    Entry::Category("Filters"),
    Entry::Panel { id: "panel-filter", label: "filter" },
    Entry::Panel { id: "panel-color-matrix", label: "feColorMatrix" },
    Entry::Panel { id: "panel-blend", label: "feBlend" },
    Entry::Panel { id: "panel-component-transfer", label: "feComponentTransfer" },
    Entry::Panel { id: "panel-turbulence", label: "feTurbulence / feDisplacementMap" },
    Entry::Panel { id: "panel-morphology", label: "feMorphology" },
    Entry::Panel { id: "panel-fe-image", label: "feImage" },
    Entry::Panel { id: "panel-fe-tile", label: "feTile" },
    Entry::Panel { id: "panel-convolve-matrix", label: "feConvolveMatrix" },
    Entry::Panel { id: "panel-lighting", label: "feDiffuseLighting / feSpecularLighting" },
    Entry::Panel { id: "panel-light-sources", label: "LightSource comparison" },
    Entry::Category("Core Operations"),
    Entry::Panel { id: "panel-view-box", label: "set_view_box" },
    Entry::Panel { id: "panel-tree-nav", label: "tree navigation" },
    Entry::Panel { id: "panel-accessibility", label: "desc / title" },
    Entry::Category("Animation"),
    Entry::Panel { id: "panel-anim", label: "AnimationLoop" },
    Entry::Category("Event Handling"),
    Entry::Panel { id: "panel-events-click", label: "Click counter" },
    Entry::Panel { id: "panel-events-colour", label: "Colour wheel" },
    Entry::Panel { id: "panel-events-modifiers", label: "Modifier keys" },
    Entry::Panel { id: "panel-events-press", label: "Press state" },
    Entry::Panel { id: "panel-events-group", label: "Bubbling" },
    Entry::Panel { id: "panel-events-pointer", label: "Pointer lifecycle" },
    Entry::Panel { id: "panel-events-keyboard-wheel", label: "Keyboard & wheel" },
    Entry::Panel { id: "panel-events-drag-drop-touch", label: "Native drag/touch" },
    Entry::Panel { id: "panel-events-passive", label: "Passive Event Listeners" },
    Entry::Panel { id: "panel-events-classlist", label: "CSS class toggle" },
    Entry::Category("Geometry Read-back"),
    Entry::Panel { id: "panel-geometry-path-follow", label: "Path follow" },
    Entry::Panel { id: "panel-geometry-bbox", label: "Bounding box" },
];

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Everything that can go wrong assembling `index.html`, from a missing file through to the catalogue disagreeing
/// with itself. None of these checks need an HTML parser — every one of them is a targeted check against the
/// specific conventions this gallery already relies on (one placeholder per token, one `id="..."` per fragment,
/// filenames that match panel ids), not a general claim about well-formed HTML.
#[derive(Debug)]
pub enum AssembleError {
    /// A file could not be read, or the assembled result could not be written.
    Io { path: PathBuf, source: io::Error },
    /// `template` does not contain `placeholder` at all.
    MissingPlaceholder { template_path: PathBuf, placeholder: &'static str },
    /// `template` contains `placeholder` more than once — `str::replacen(..., 1)` would silently leave every
    /// occurrence after the first one sitting untouched in the output.
    DuplicatePlaceholder {
        template_path: PathBuf,
        placeholder: &'static str,
        count: usize,
    },
    /// The same panel id appears in [`MANIFEST`] more than once.
    DuplicateManifestId(&'static str),
    /// A fragment's own content does not contain `id="{id}"` for the id it is filed under — it may have been
    /// copy-pasted from another panel and never updated, or renamed on disk without updating its content.
    FragmentIdMismatch { id: &'static str, fragment_path: PathBuf },
    /// [`MANIFEST`]'s panel ids, the generated menu's `data-target` ids, and `demo/panels/*.html`'s filenames are
    /// not all the same set. In this pipeline the menu is generated directly from `MANIFEST`, so a mismatch there
    /// points at a bug in [`render_menu`] rather than independent drift — but a mismatch against the fragments
    /// directory is exactly how an orphaned fragment (removed from `MANIFEST` but left on disk) or a missing
    /// fragment (added to `MANIFEST` but never created) gets caught.
    CatalogueMismatch(String),
    /// The fully assembled output still contains a `{{...}}` token after both placeholders were substituted —
    /// evidence of a typo'd or unexpected placeholder that no check above already caught.
    LeftoverPlaceholder(String),
}

impl fmt::Display for AssembleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => write!(f, "{} ({source})", path.display()),
            Self::MissingPlaceholder { template_path, placeholder } => {
                write!(f, "{} is missing the {placeholder} placeholder", template_path.display())
            },
            Self::DuplicatePlaceholder {
                template_path,
                placeholder,
                count,
            } => {
                write!(
                    f,
                    "{} contains {placeholder} {count} times, expected exactly once",
                    template_path.display()
                )
            },
            Self::DuplicateManifestId(id) => write!(f, "MANIFEST contains the panel id {id:?} more than once"),
            Self::FragmentIdMismatch { id, fragment_path } => {
                write!(
                    f,
                    "{} does not contain id=\"{id}\", the id it is filed under",
                    fragment_path.display()
                )
            },
            Self::CatalogueMismatch(detail) => {
                write!(
                    f,
                    "MANIFEST, the generated menu, and demo/panels/ disagree about which panels exist:\n{detail}"
                )
            },
            Self::LeftoverPlaceholder(context) => {
                write!(
                    f,
                    "assembled index.html still contains an unresolved placeholder near: {context:?}"
                )
            },
        }
    }
}

impl std::error::Error for AssembleError {}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Builds the gallery's `index.html` from `source_demo_dir`'s `index.template.html` and `panels/*.html` fragments,
/// and writes the result to `out_path`, returning that same path.
///
/// `source_demo_dir` and `out_path` are independent — the caller decides where the generated file actually lands
/// (see `main.rs`'s doc comment for why that is no longer the same directory the source template and fragments
/// live in). Every check runs, and the complete assembled output is validated, before anything is written — a
/// call that returns `Err` never touches `out_path` on disk, so a broken run never overwrites a last-known-good
/// file with a worse one.
pub fn assemble(source_demo_dir: &Path, out_path: &Path, port: u16) -> Result<PathBuf, AssembleError> {
    let panels_dir = source_demo_dir.join("panels");
    let template_path = source_demo_dir.join("index.template.html");

    check_unique_manifest_ids(MANIFEST)?;

    let template = read_to_string(&template_path)?;
    check_placeholder_count(&template, &template_path, PANELS_PLACEHOLDER)?;
    check_placeholder_count(&template, &template_path, MENU_PLACEHOLDER)?;

    let fragment_filenames = list_fragment_filenames(&panels_dir)?;
    let panels_body = render_panels(&panels_dir)?;
    let menu_body = render_menu();
    check_catalogue_consistency(&menu_body, &fragment_filenames)?;

    let assembled = template.replacen(SERVER_PORT, &port.to_string(), 1);
    let assembled = assembled.replacen(PANELS_PLACEHOLDER, &panels_body, 1);
    let assembled = assembled.replacen(MENU_PLACEHOLDER, &menu_body, 1);
    check_no_leftover_placeholders(&assembled)?;

    fs::write(out_path, &assembled).map_err(|source| AssembleError::Io {
        path: out_path.to_path_buf(),
        source,
    })?;
    Ok(out_path.to_path_buf())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Checks
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Every panel id among `entries` must be unique — a duplicate would silently insert the same fragment twice and
/// leave one of the two ids' "real" position undefined. Takes the entry list as a parameter, rather than reading
/// [`MANIFEST`] directly, purely so the tests below can exercise the duplicate-detection logic itself against a
/// small synthetic list, independently of whether the real `MANIFEST` happens to have a duplicate right now.
fn check_unique_manifest_ids(entries: &[Entry]) -> Result<(), AssembleError> {
    let mut seen = HashSet::new();
    for entry in entries {
        if let Entry::Panel { id, .. } = entry
            && !seen.insert(*id)
        {
            return Err(AssembleError::DuplicateManifestId(id));
        }
    }
    Ok(())
}

/// `template` must contain `placeholder` exactly once: zero means nothing will ever be substituted in, and more
/// than one means `replacen(..., 1)` would leave every occurrence after the first sitting in the output untouched.
fn check_placeholder_count(
    template: &str,
    template_path: &Path,
    placeholder: &'static str,
) -> Result<(), AssembleError> {
    match template.matches(placeholder).count() {
        0 => Err(AssembleError::MissingPlaceholder {
            template_path: template_path.to_path_buf(),
            placeholder,
        }),
        1 => Ok(()),
        count => Err(AssembleError::DuplicatePlaceholder {
            template_path: template_path.to_path_buf(),
            placeholder,
            count,
        }),
    }
}

/// [`MANIFEST`]'s panel ids, the generated menu's `data-target` ids, and the filenames actually present in
/// `demo/panels/` must all be the same set. Reports every set difference found, not just the first, since a
/// caller fixing one of these by hand benefits from seeing the whole picture at once.
fn check_catalogue_consistency(menu_body: &str, fragment_filenames: &HashSet<String>) -> Result<(), AssembleError> {
    let manifest_ids: HashSet<String> = panel_ids().into_iter().map(str::to_string).collect();
    let menu_targets = extract_data_targets(menu_body);

    let mut problems = Vec::new();
    report_set_difference(&mut problems, "MANIFEST", &manifest_ids, "the generated menu", &menu_targets);
    report_set_difference(&mut problems, "the generated menu", &menu_targets, "MANIFEST", &manifest_ids);
    report_set_difference(&mut problems, "MANIFEST", &manifest_ids, "demo/panels/", fragment_filenames);
    report_set_difference(&mut problems, "demo/panels/", fragment_filenames, "MANIFEST", &manifest_ids);

    if problems.is_empty() {
        Ok(())
    } else {
        Err(AssembleError::CatalogueMismatch(problems.join("\n")))
    }
}

fn report_set_difference(
    problems: &mut Vec<String>,
    left_name: &str,
    left: &HashSet<String>,
    right_name: &str,
    right: &HashSet<String>,
) {
    let mut only_in_left: Vec<&String> = left.difference(right).collect();
    if only_in_left.is_empty() {
        return;
    }
    only_in_left.sort();
    problems.push(format!("  in {left_name} but not {right_name}: {only_in_left:?}"));
}

/// Extracts every `data-target="..."` value from generated menu markup this module itself just built — a genuine
/// check of [`render_menu`]'s output, not a restatement of its input, since a future bug in that function (skipping
/// an entry, say) would show up here as a real mismatch against [`MANIFEST`].
fn extract_data_targets(menu_body: &str) -> HashSet<String> {
    const NEEDLE: &str = "data-target=\"";
    let mut targets = HashSet::new();
    let mut rest = menu_body;
    while let Some(start) = rest.find(NEEDLE) {
        let after_open_quote = &rest[start + NEEDLE.len()..];
        let Some(close) = after_open_quote.find('"') else { break };
        targets.insert(after_open_quote[..close].to_string());
        rest = &after_open_quote[close + 1..];
    }
    targets
}

/// The assembled output must not contain any `{{...}}` token once both placeholders have been substituted — a
/// leftover one means a typo'd or unexpected placeholder that none of the checks above already caught.
fn check_no_leftover_placeholders(assembled: &str) -> Result<(), AssembleError> {
    if let Some(start) = assembled.find("{{") {
        let end = (start + 40).min(assembled.len());
        return Err(AssembleError::LeftoverPlaceholder(assembled[start..end].to_string()));
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Rendering
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Concatenates every panel fragment (and category divider comment) from [`MANIFEST`], in order. Checks each
/// fragment contains `id="{id}"` for its own id as it reads it, rather than in a separate pass, since it has
/// already paid the cost of reading the file at that point.
fn render_panels(panels_dir: &Path) -> Result<String, AssembleError> {
    let mut body = String::new();
    for (i, entry) in MANIFEST.iter().enumerate() {
        match entry {
            Entry::Category(label) => {
                body.push_str(DIVIDER);
                body.push('\n');
                body.push_str(&format!("            <!-- {label} -->\n"));
                body.push_str(DIVIDER);
                body.push('\n');
            },
            Entry::Panel { id, .. } => {
                let fragment_path = panels_dir.join(format!("{id}.html"));
                let fragment = read_to_string(&fragment_path)?;
                if !fragment.contains(&format!("id=\"{id}\"")) {
                    return Err(AssembleError::FragmentIdMismatch { id, fragment_path });
                }
                body.push_str(fragment.trim_end());
                body.push('\n');
            },
        }
        // A blank separator line between entries, but not trailing after the very last one — matches how the
        // hand-written file used to be laid out.
        if i + 1 != MANIFEST.len() {
            body.push('\n');
        }
    }
    Ok(body.trim_end().to_string())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Builds the `<details class="menu-group">...</details>` blocks from [`MANIFEST`], one per category, each holding
/// a `<button class="menu-item" data-target="...">` per panel in that category.
fn render_menu() -> String {
    let mut menu = String::new();
    let mut open = false;

    for entry in MANIFEST {
        match entry {
            Entry::Category(label) => {
                if open {
                    menu.push_str("            </details>\n\n");
                }
                menu.push_str("            <details class=\"menu-group\">\n");
                menu.push_str(&format!("                <summary>{}</summary>\n", escape_amp(label)));
                open = true;
            },
            Entry::Panel { id, label } => {
                menu.push_str(&format!(
                    "                <button type=\"button\" class=\"menu-item\" data-target=\"{id}\">{}</button>\n",
                    escape_amp(label)
                ));
            },
        }
    }
    if open {
        menu.push_str("            </details>");
    }
    menu
}

/// `&` is the only special character any category label or panel label actually contains (checked against
/// [`MANIFEST`] by [`super::validate`]), so this only handles that one case rather than pulling in a general HTML
/// escaper for text this module itself fully controls.
fn escape_amp(s: &str) -> String {
    s.replace('&', "&amp;")
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// I/O helpers
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

fn read_to_string(path: &Path) -> Result<String, AssembleError> {
    fs::read_to_string(path).map_err(|source| AssembleError::Io {
        path: path.to_path_buf(),
        source,
    })
}

/// Every `*.html` filename present in `demo/panels/`, with the extension stripped, so it can be compared directly
/// against [`MANIFEST`]'s panel ids — this is what catches a fragment orphaned by removing its `MANIFEST` entry
/// (present on disk, absent from the id set) as well as a `MANIFEST` entry with no fragment ever created for it
/// (absent from disk, present in the id set).
fn list_fragment_filenames(panels_dir: &Path) -> Result<HashSet<String>, AssembleError> {
    let entries = fs::read_dir(panels_dir).map_err(|source| AssembleError::Io {
        path: panels_dir.to_path_buf(),
        source,
    })?;
    let mut names = HashSet::new();
    for entry in entries {
        let entry = entry.map_err(|source| AssembleError::Io {
            path: panels_dir.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "html")
            && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
        {
            names.insert(stem.to_string());
        }
    }
    Ok(names)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Every panel id in [`MANIFEST`], in order — used by [`super::validate`] to cross-check against
/// `demo-app/src/lib.rs`'s `demo_gallery!` list.
pub fn panel_ids() -> Vec<&'static str> {
    MANIFEST
        .iter()
        .filter_map(|entry| match entry {
            Entry::Panel { id, .. } => Some(*id),
            Entry::Category(_) => None,
        })
        .collect()
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    /// The real project's `demo/` directory — the actual, current catalogue this whole module exists to assemble.
    /// Tests only ever read from here; nothing in this module writes into it. Locating it via `CARGO_MANIFEST_DIR`
    /// (rather than a relative path, which would depend on the test binary's working directory) is what makes this
    /// work the same way under `cargo test`, `cargo test -p demo-server`, and `cargo llvm-cov`/`nextest` alike.
    fn project_demo_dir() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("demo-server has a parent directory")
            .join("demo")
    }

    /// Copies the real project's `index.template.html` and every `panels/*.html` fragment into `dest`, so a test
    /// can then corrupt exactly one file in isolation without ever touching the real ones. Only the tests that
    /// deliberately break something need this — [`assembles_the_real_catalogue_without_error`] below reads
    /// straight from [`project_demo_dir`] instead, since it has nothing to corrupt.
    fn seed_fixtures(dest: &Path) {
        let src = project_demo_dir();
        fs::create_dir_all(dest.join("panels")).expect("create panels dir");
        fs::copy(src.join("index.template.html"), dest.join("index.template.html")).expect("copy template");
        for entry in fs::read_dir(src.join("panels")).expect("read real panels dir") {
            let entry = entry.expect("dir entry");
            fs::copy(entry.path(), dest.join("panels").join(entry.file_name())).expect("copy fragment");
        }
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // Catalogue invariants, checked directly against the real MANIFEST/menu/fragments — these are the tests that
    // actually protect against the class of drift this module exists to catch; everything below them instead
    // targets one check's logic in isolation with a small synthetic fixture.
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

    #[test]
    fn real_manifest_has_no_duplicate_ids() {
        assert!(check_unique_manifest_ids(MANIFEST).is_ok());
    }

    #[test]
    fn assembles_the_real_catalogue_without_error() {
        let tmp = tempfile::tempdir().expect("create temp dir");
        let out_path = tmp.path().join("index.html");

        let result = assemble(&project_demo_dir(), &out_path);
        assert!(result.is_ok(), "assemble failed against the real catalogue: {:?}", result.err());

        let html = fs::read_to_string(&out_path).expect("read assembled output");
        assert!(
            html.contains(r#"id="panel-rect""#),
            "assembled output is missing a known real panel"
        );
        assert!(
            !html.contains("{{"),
            "assembled output still contains an unresolved placeholder"
        );
        // One <section> and one canvas <div> per panel, and nothing left over from a stray duplicate id.
        assert_eq!(html.matches(r#"class="section" id="panel-rect""#).count(), 1);
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // assemble, exercised against deliberately corrupted copies of the real fixtures
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

    #[test]
    fn rejects_a_duplicate_placeholder() {
        let tmp = tempfile::tempdir().expect("create temp dir");
        seed_fixtures(tmp.path());

        let template_path = tmp.path().join("index.template.html");
        let template = fs::read_to_string(&template_path).expect("read template");
        let doubled = template.replacen(PANELS_PLACEHOLDER, &format!("{PANELS_PLACEHOLDER}\n{PANELS_PLACEHOLDER}"), 1);
        fs::write(&template_path, doubled).expect("write corrupted template");

        let out_path = tmp.path().join("index.html");
        let err = assemble(tmp.path(), &out_path).expect_err("a doubled placeholder must be rejected");
        assert!(
            matches!(err, AssembleError::DuplicatePlaceholder { .. }),
            "wrong error variant: {err}"
        );
        assert!(!out_path.exists(), "must not write output after a failed assembly");
    }

    #[test]
    fn rejects_a_missing_placeholder() {
        let tmp = tempfile::tempdir().expect("create temp dir");
        seed_fixtures(tmp.path());

        let template_path = tmp.path().join("index.template.html");
        let template = fs::read_to_string(&template_path).expect("read template");
        fs::write(&template_path, template.replace(MENU_PLACEHOLDER, "")).expect("write corrupted template");

        let out_path = tmp.path().join("index.html");
        let err = assemble(tmp.path(), &out_path).expect_err("a missing placeholder must be rejected");
        assert!(
            matches!(err, AssembleError::MissingPlaceholder { .. }),
            "wrong error variant: {err}"
        );
    }

    #[test]
    fn rejects_a_fragment_id_mismatch() {
        let tmp = tempfile::tempdir().expect("create temp dir");
        seed_fixtures(tmp.path());

        let fragment_path = tmp.path().join("panels").join("panel-rect.html");
        let fragment = fs::read_to_string(&fragment_path).expect("read fragment");
        fs::write(
            &fragment_path,
            fragment.replace(r#"id="panel-rect""#, r#"id="panel-rect-oops""#),
        )
        .expect("write corrupted fragment");

        let out_path = tmp.path().join("index.html");
        let err = assemble(tmp.path(), &out_path).expect_err("a fragment/id mismatch must be rejected");
        assert!(
            matches!(err, AssembleError::FragmentIdMismatch { .. }),
            "wrong error variant: {err}"
        );
    }

    #[test]
    fn rejects_an_orphaned_fragment() {
        let tmp = tempfile::tempdir().expect("create temp dir");
        seed_fixtures(tmp.path());

        // A fragment file with no matching MANIFEST entry — the "removed from MANIFEST but left on disk" case.
        fs::write(
            tmp.path().join("panels").join("panel-orphan.html"),
            "<section class=\"section\" id=\"panel-orphan\"></section>\n",
        )
        .expect("write orphaned fragment");

        let out_path = tmp.path().join("index.html");
        let err = assemble(tmp.path(), &out_path).expect_err("an orphaned fragment must be rejected");
        assert!(matches!(err, AssembleError::CatalogueMismatch(_)), "wrong error variant: {err}");
    }

    #[test]
    fn rejects_a_missing_fragment_file() {
        let tmp = tempfile::tempdir().expect("create temp dir");
        seed_fixtures(tmp.path());

        // A MANIFEST entry with no fragment ever created for it — the "added to MANIFEST but never created" case.
        fs::remove_file(tmp.path().join("panels").join("panel-rect.html")).expect("remove fragment");

        let out_path = tmp.path().join("index.html");
        let err = assemble(tmp.path(), &out_path).expect_err("a missing fragment file must be rejected");
        assert!(matches!(err, AssembleError::Io { .. }), "wrong error variant: {err}");
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // Smaller unit tests of individual checks, against synthetic input rather than the real catalogue
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

    #[test]
    fn detects_a_duplicate_manifest_id() {
        const DUPLICATED: &[Entry] = &[
            Entry::Panel { id: "panel-a", label: "a" },
            Entry::Panel { id: "panel-b", label: "b" },
            Entry::Panel {
                id: "panel-a",
                label: "a again",
            },
        ];
        let err = check_unique_manifest_ids(DUPLICATED).expect_err("a duplicated id must be rejected");
        assert!(
            matches!(err, AssembleError::DuplicateManifestId("panel-a")),
            "wrong error variant: {err}"
        );
    }

    #[test]
    fn accepts_unique_manifest_ids() {
        const UNIQUE: &[Entry] = &[
            Entry::Category("Category"),
            Entry::Panel { id: "panel-a", label: "a" },
            Entry::Panel { id: "panel-b", label: "b" },
        ];
        assert!(check_unique_manifest_ids(UNIQUE).is_ok());
    }

    #[test]
    fn placeholder_count_zero_is_missing() {
        let err = check_placeholder_count("no placeholder here", Path::new("t.html"), PANELS_PLACEHOLDER)
            .expect_err("zero occurrences must be rejected");
        assert!(matches!(err, AssembleError::MissingPlaceholder { .. }));
    }

    #[test]
    fn placeholder_count_one_is_ok() {
        assert!(check_placeholder_count("one {{PANELS}} here", Path::new("t.html"), PANELS_PLACEHOLDER).is_ok());
    }

    #[test]
    fn placeholder_count_two_is_duplicate() {
        let err = check_placeholder_count("{{PANELS}} and {{PANELS}}", Path::new("t.html"), PANELS_PLACEHOLDER)
            .expect_err("two occurrences must be rejected");
        assert!(matches!(err, AssembleError::DuplicatePlaceholder { count: 2, .. }));
    }

    #[test]
    fn extracts_every_data_target() {
        let menu = r#"<button data-target="panel-a">a</button><button data-target="panel-b">b</button>"#;
        let targets = extract_data_targets(menu);
        assert_eq!(targets, HashSet::from(["panel-a".to_string(), "panel-b".to_string()]));
    }

    #[test]
    fn escapes_ampersands_only() {
        assert_eq!(escape_amp("Clipping & Masking"), "Clipping &amp; Masking");
        assert_eq!(escape_amp("plain text"), "plain text");
    }

    #[test]
    fn no_leftover_placeholder_in_clean_output() {
        assert!(check_no_leftover_placeholders("<html>no placeholders left</html>").is_ok());
    }

    #[test]
    fn detects_a_leftover_placeholder() {
        let err =
            check_no_leftover_placeholders("<html>{{OOPS}}</html>").expect_err("a leftover token must be rejected");
        assert!(matches!(err, AssembleError::LeftoverPlaceholder(_)));
    }
}
