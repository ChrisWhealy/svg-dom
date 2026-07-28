//! Assembles `demo/index.html` using `demo/index.template.html` and the various panel fragments in `demo/panels/`.
//!
//! The demo gallery is composed from multiple `<section>` fragment files found in `demo/panels/`.  The job of this
//! module is to stitch them together into the single static file that `demo-server` then serves.
//!
//! `Files::new` in `main.rs` resolves `/demo/` to `target/demo-gallery/demo/index.html` on disk, so that file has to
//! exist there, generated or not.
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
    /// A panel id does not match `panel-[a-z0-9-]+` — see [`check_panel_id_format`] for why that pattern is enforced up
    /// front rather than escaped at each place an id is emitted.
    InvalidPanelId(&'static str),
    /// A fragment's own content does not contain `id="{id}"` for the id it is filed under — it may have been
    /// copy-pasted from another panel and never updated, or renamed on disk without updating its content.
    FragmentIdMismatch { id: &'static str, fragment_path: PathBuf },
    /// [`MANIFEST`]'s panel ids, the generated menu's `data-target` ids, and `demo/panels/*.html`'s filenames are not
    /// all the same set. In this pipeline the menu is generated directly from `MANIFEST`, so a mismatch there points to
    /// a bug in [`render_menu`] rather than independent drift — but a mismatch against the fragments directory is
    /// exactly how an orphaned fragment (removed from `MANIFEST` but left on disk) or a missing fragment (added to
    /// `MANIFEST` but never created) gets caught.
    CatalogueMismatch(String),
    /// The fully assembled output still contains a `{{...}}` token after both placeholders were substituted — evidence
    /// of a typo'd or unexpected placeholder that no check above already caught.
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
            Self::InvalidPanelId(id) => write!(
                f,
                "MANIFEST contains the panel id {id:?}, which does not match panel-[a-z0-9-]+"
            ),
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
    check_panel_id_format(MANIFEST)?;

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

/// Every panel id among `entries` must be unique as a duplicate would silently overwrite the existing fragment and
/// leave one of the two ids' "real" position undefined.
///
/// Takes the entry list as a parameter, rather than reading [`MANIFEST`] directly, purely so the tests below can
/// exercise the duplicate-detection logic against a small synthetic list, independently of whether the real `MANIFEST`
/// happens to contain a duplicate.
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

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Every panel id among `entries` must match `panel-[a-z0-9-]+`.
///
/// Ids are emitted verbatim into both HTML attribute values and Rust match arms:
/// * `id="{id}"` in each fragment
/// * `data-target="{id}"` in the generated menu
/// * `"panel-..." => module::func` in `demo-app/src/lib.rs`'s `demo_gallery!`
///
/// This restricts them to a known-safe ASCII pattern up front, meaning that downstream — [`render_menu`], a fragment
/// file, `demo_gallery!` never have to escape or re-validate an id thenselves.
///
/// Unlike labels (see [`escape_text`]), ids are not free text, so a fixed character set is the natural fit rather than
/// an escaper.
fn check_panel_id_format(entries: &[Entry]) -> Result<(), AssembleError> {
    for entry in entries {
        if let Entry::Panel { id, .. } = entry
            && !is_valid_panel_id(id)
        {
            return Err(AssembleError::InvalidPanelId(id));
        }
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn is_valid_panel_id(id: &str) -> bool {
    id.strip_prefix("panel-").is_some_and(|rest| {
        !rest.is_empty() && rest.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    })
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `template` must contain `placeholder` exactly once: zero means nothing will ever be substituted in, and more than
/// one means `replacen(..., 1)` would leave every occurrence after the first sitting in the output untouched.
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

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// [`MANIFEST`]'s panel ids, the generated menu's `data-target` ids, and the filenames actually present in
/// `demo/panels/` must all be the same set. Reports every set difference found, not just the first, since a caller
/// fixing one of these by hand benefits from seeing the whole picture at once.
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

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
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

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Extracts every `data-target="..."` value from generated menu markup this module itself just built — a genuine check
/// of [`render_menu`]'s output, not a restatement of its input, since a future bug in that function (skipping an entry,
/// say) would show up here as a real mismatch against [`MANIFEST`].
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

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// The assembled output must not contain any `{{...}}` token once both placeholders have been substituted — a leftover
/// one means a typo'd or unexpected placeholder that none of the checks above already caught.
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

/// Concatenates every panel fragment (and category divider comment) from [`MANIFEST`], in order. Checks each fragment
/// contains `id="{id}"` for its own id as it reads it, rather than in a separate pass, since it has already paid the
/// cost of reading the file at that point.
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
                menu.push_str(&format!("                <summary>{}</summary>\n", escape_text(label)));
                open = true;
            },
            Entry::Panel { id, label } => {
                menu.push_str(&format!(
                    "                <button type=\"button\" class=\"menu-item\" data-target=\"{id}\">{}</button>\n",
                    escape_text(label)
                ));
            },
        }
    }
    if open {
        menu.push_str("            </details>");
    }
    menu
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Escapes `&`, `<` and `>` for safe insertion into an HTML text node — the full set that matters there. Labels are
/// trusted source code, not user input, so this is not a security boundary; it exists so a future label (e.g. one
/// containing `<`) is rendered as text rather than parsed as markup, without relying on every label happening to avoid
/// HTML's special characters. Attribute values (panel ids) go through [`check_panel_id_format`] instead, which
/// restricts them to a known-safe character set up front rather than escaping them here.
fn escape_text(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
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

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
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
mod unit_tests;
