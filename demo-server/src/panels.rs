//! Assembles `demo/index.html` from `demo/index.template.html` and the per-panel fragments in `demo/panels/`.
//!
//! The gallery used to be one hand-edited, ever-growing `index.html`. Every demo's `<section>` now lives in its own
//! file under `demo/panels/`, and this module stitches them back into the single static file `demo-server` actually
//! serves — `Files::new` in `main.rs` resolves `/demo/` to `demo/index.html` on disk, so that file has to exist
//! there, generated or not.
//!
//! [`MANIFEST`] is the single source of truth for panel order (matching the menu's own category order in
//! `demo/index.template.html`); add a new demo by adding its id here and creating the matching
//! `demo/panels/{id}.html` fragment.

use std::{fs, path::Path, process};

const PLACEHOLDER: &str = "{{PANELS}}";

/// Every category divider in the source uses this exact comment text, verified against the original hand-written
/// `index.html` when this module was split out — see `docs/design_notes/` for how, if that history matters later.
const DIVIDER: &str = "            <!-- = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = = -->";

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// One entry in the gallery's section order: either a category divider comment, or a single panel whose markup
/// lives in `demo/panels/{id}.html`.
enum Entry {
    Category(&'static str),
    Panel(&'static str),
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[rustfmt::skip]
const MANIFEST: &[Entry] = &[
    Entry::Category("Basic Shapes"),
    Entry::Panel("panel-rect"),
    Entry::Panel("panel-circle"),
    Entry::Panel("panel-ellipse"),
    Entry::Panel("panel-line"),
    Entry::Panel("panel-poly"),
    Entry::Panel("panel-group"),
    Entry::Category("Path"),
    Entry::Panel("panel-path"),
    Entry::Category("Text"),
    Entry::Panel("panel-text"),
    Entry::Panel("panel-tspan"),
    Entry::Panel("panel-text-path"),
    Entry::Category("Structural & Reusable Elements"),
    Entry::Panel("panel-marker"),
    Entry::Panel("panel-marker-view-box"),
    Entry::Panel("panel-use"),
    Entry::Panel("panel-image"),
    Entry::Panel("panel-symbol"),
    Entry::Panel("panel-anchor"),
    Entry::Panel("panel-switch"),
    Entry::Panel("panel-view"),
    Entry::Panel("panel-style"),
    Entry::Panel("panel-foreign-object"),
    Entry::Category("Paint Servers"),
    Entry::Panel("panel-linear-gradient"),
    Entry::Panel("panel-radial-gradient"),
    Entry::Panel("panel-pattern"),
    Entry::Category("Clipping & Masking"),
    Entry::Panel("panel-clip-path"),
    Entry::Panel("panel-mask"),
    Entry::Category("Filters"),
    Entry::Panel("panel-filter"),
    Entry::Panel("panel-color-matrix"),
    Entry::Panel("panel-blend"),
    Entry::Panel("panel-component-transfer"),
    Entry::Panel("panel-turbulence"),
    Entry::Panel("panel-morphology"),
    Entry::Panel("panel-fe-image"),
    Entry::Panel("panel-fe-tile"),
    Entry::Panel("panel-convolve-matrix"),
    Entry::Panel("panel-lighting"),
    Entry::Panel("panel-light-sources"),
    Entry::Category("Core Operations"),
    Entry::Panel("panel-view-box"),
    Entry::Panel("panel-tree-nav"),
    Entry::Panel("panel-accessibility"),
    Entry::Category("Animation"),
    Entry::Panel("panel-anim"),
    Entry::Category("Event Handling"),
    Entry::Panel("panel-events-click"),
    Entry::Panel("panel-events-colour"),
    Entry::Panel("panel-events-modifiers"),
    Entry::Panel("panel-events-press"),
    Entry::Panel("panel-events-group"),
    Entry::Panel("panel-events-pointer"),
    Entry::Panel("panel-events-keyboard-wheel"),
    Entry::Panel("panel-events-drag-drop-touch"),
    Entry::Panel("panel-events-passive"),
    Entry::Panel("panel-events-classlist"),
    Entry::Category("Geometry Read-back"),
    Entry::Panel("panel-geometry-path-follow"),
    Entry::Panel("panel-geometry-bbox"),
];

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Rebuilds `demo/index.html` from `demo/index.template.html` and `demo/panels/*.html`.
///
/// `root` is the project root (the directory containing `demo/`). A failure here is fatal for the same reason a
/// failed wasm build is: rather than silently serving a stale or partial `index.html`, the error is reported and
/// the process exits.
pub fn assemble(root: &Path) {
    let demo_dir = root.join("demo");
    let panels_dir = demo_dir.join("panels");
    let template_path = demo_dir.join("index.template.html");
    let out_path = demo_dir.join("index.html");

    let template = read_or_die(&template_path);

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
            Entry::Panel(id) => {
                let fragment = read_or_die(&panels_dir.join(format!("{id}.html")));
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

    if !template.contains(PLACEHOLDER) {
        eprintln!("aborting: {} is missing the {PLACEHOLDER} placeholder", template_path.display());
        process::exit(1);
    }
    let assembled = template.replacen(PLACEHOLDER, body.trim_end(), 1);

    if let Err(err) = fs::write(&out_path, assembled) {
        eprintln!("aborting: could not write {} ({err})", out_path.display());
        process::exit(1);
    }
}

fn read_or_die(path: &Path) -> String {
    match fs::read_to_string(path) {
        Ok(s) => s,
        Err(err) => {
            eprintln!("aborting: could not read {} ({err})", path.display());
            process::exit(1);
        },
    }
}
