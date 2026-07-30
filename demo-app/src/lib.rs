//! Interactive feature gallery for the browser.
//!
//! This is a separate workspace crate, not a feature of `svg-dom` itself: it depends on `svg-dom` the same way any
//! external consumer would (a plain `Cargo.toml` path dependency), reaching it only through its public API. This
//! keeps the demo gallery's build out of the main library's own dependency graph and — since a private/internal
//! item this crate accidentally relied on would simply fail to compile — doubles as a standing check that the
//! library's public surface is actually sufficient to build a real, full-featured application against.
//!
//! Build and serve with:
//! ```sh
//! cargo demo
//! ```
//! which rebuilds this crate's wasm package and serves it via the Actix `demo-server` crate at
//! <http://127.0.0.1:8080/demo/>.

mod colours;
mod events;
mod foreign_html;
mod geometry;
mod highlight;
mod paint;
mod shapes;
mod structure;
mod texts;

use std::{cell::RefCell, rc::Rc};
use svg_dom::{AnimationLoop, CachedAttr, Error, SvgAttrs, SvgNode, SvgRoot, root::utils::Point};
use wasm_bindgen::prelude::*;

use colours::*;

/// A raw `web_sys` event-listener closure, as stored in [`LIVE_DEMO_CLOSURES`].
type DemoClosure = Closure<dyn FnMut(web_sys::Event)>;

pub(crate) const NS_XHTML: &str = "http://www.w3.org/1999/xhtml";

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
thread_local! {
    /// Demo-only owner for interactive nodes whose managed listeners must remain attached after the demo function
    /// that created them returns.
    ///
    /// The library intentionally removes a DOM listener when the last `SvgNode` handle is dropped.  Browser demos are
    /// long-lived, so they keep listener-owning nodes here instead of leaking individual `Closure`s.
    static LIVE_DEMO_NODES: RefCell<Vec<SvgNode>> = const { RefCell::new(Vec::new()) };

    /// Demo-only owner for every running `AnimationLoop`.
    ///
    /// `AnimationLoop` stops on `Drop`, so each demo parks its loop here to keep it running for the page's lifetime.
    /// This has to be a collection, not a single slot: more than one demo (`structure::demo_anim` and
    /// `geometry::demo_geometry_path_follow`) runs its own independent `AnimationLoop`, and a single `Option` slot
    /// would let the second call silently drop and stop the first one's animation.
    static LIVE_DEMO_ANIMS: RefCell<Vec<AnimationLoop>> = const { RefCell::new(Vec::new()) };

    /// Demo-only owner for raw `web_sys` event-listener `Closure`s attached outside `svg_dom`'s own managed `on_*`
    /// wrappers — e.g. for an `<input>` element contained within a [`SvgRoot::foreign_object`]', the raw `as_element()`
    /// escape hatch has no `SvgNode` of its own in which to hold a managed listener (like [`LIVE_DEMO_NODES`] does).
    ///
    /// Attempting to use `Closure::forget` as a workaround would leak these permanently.
    /// Parking them here for the page's lifetime instead follows the same explicit, non-leaking ownership seen in
    /// [`LIVE_DEMO_NODES`]/[`LIVE_DEMO_ANIMS`].
    ///
    /// # Why page-lifetime ownership is the right call here, not just the convenient one
    ///
    /// Each closure parked here (e.g. `texts.rs`'s radio/slider handlers) strongly owns the `SvgNode` it updates — the
    /// text node whose `text-anchor`, `dominant-baseline`, or `startOffset` the listener writes on every event.
    ///
    /// That is *not* the self-referential cycle the library's own managed `on_*` listeners exist to avoid: a managed
    /// listener is owned by the very `SvgNode` it is attached to, so a node capturing a clone of itself can keep itself
    /// alive forever. Here the listener is owned by the raw HTML `<input>` (via this thread-local, not via the input
    /// itself), and it captures a *different* node (the SVG text/textPath element it updates), so there is no cycle for
    /// anything to leak through in the first place; the ownership graph forms a straight line (this thread-local →
    /// closure → SVG text node), not a loop.
    ///
    /// The other half of "is this actually bounded" is [`init_panel`]'s own idempotency via [`PANEL_STATE_ATTR`]: each
    /// panel's build function (and therefore every `keep_demo_closure` call it makes) runs at most once per page load,
    /// no matter how many times the user navigates back to that panel. So this `Vec` does not experience unbounded
    /// growth over the course of a session; it grows once per interactive control that ever gets built, then stops.
    ///
    /// ***IMPORTANT***
    ///
    /// Neither property is free, though: both come from specifically from the fact that this is a small, single-page,
    /// never-torn-down demo gallery. Treat this as *gallery*-lifetime ownership, not a pattern to copy unchanged into
    /// an application that repeatedly mounts and unmounts the same component: there, the closures (and the SVG nodes
    /// they capture) would need to be dropped on unmount — e.g. by handing the caller a guard/handle that removes
    /// the listener and clears this `Vec`'s corresponding entry — rather than accumulating here for as long as the
    /// page happens to stay open.
    static LIVE_DEMO_CLOSURES: RefCell<Vec<DemoClosure>> = const { RefCell::new(Vec::new()) };
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn keep_demo_node(node: SvgNode) {
    LIVE_DEMO_NODES.with(|nodes| nodes.borrow_mut().push(node));
}

fn keep_demo_anim(anim: AnimationLoop) {
    LIVE_DEMO_ANIMS.with(|anims| anims.borrow_mut().push(anim));
}

fn keep_demo_closure(closure: DemoClosure) {
    LIVE_DEMO_CLOSURES.with(|closures| closures.borrow_mut().push(closure));
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Event-type-generic handler that routes a constant `last: {name}` label through a shared `CachedAttr`, so a burst of
// identical labels (a stream of pointermove/touchmove events) skips the DOM write after the first. Every writer to a
// given readout must share the *same* cache for it to stay coherent.
fn cached_label<E>(readout: SvgNode, cache: Rc<RefCell<CachedAttr>>, name: &'static str) -> impl Fn(E) + 'static {
    // The label is constant; format it once so the cached write is fully allocation-free per event.
    let label = format!("last: {name}");
    move |_| {
        let _ = cache.borrow_mut().set_text(&readout, &label);
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Constants
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Single source of truth for the demo canvas dimensions. `create_in` writes these to each <svg>'s width/height
// attributes, which size the rendered element — so style.css deliberately does not repeat them.
const W: f64 = 800.0; // SVG width shared across all demo canvases
const H: f64 = 180.0; // SVG height shared across all demo canvases

// The demos are composed within a BAND-tall region. PAD_Y is the offset that vertically centres that band in the
// canvas, so every demo's content stays in the middle whatever H is. (PAD_Y is 0 when H == BAND.)
const BAND: f64 = 130.0;
const PAD_Y: f64 = (H - BAND) / 2.0;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Panel registry (DEMO_PANELS, generated below by demo_gallery!) and the lazy per-panel entry point that uses it
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Every demo sub-module's source, embedded at build time, keyed by its own module path so each panel can display
/// the exact code that drives it. A single flat `concat!`-ed blob (searched by bare function name alone) is not
/// enough once more than one file can define a same-named function — as `texts` now does, split into
/// `texts::demo_text`, `texts::demo_tspan`, `texts::demo_text_path`, each exporting its own `fn demo()` — so
/// [`demo_fn_source`] instead searches only the one file named here, selected by the module path half of
/// [`DEMO_PANELS`]'s own recorded `stringify!($path)` (see [`split_demo_path`]).
///
/// Placed here, above [`demo_gallery`], because [`demo_fn_source`] needs it and macros expand in place: `DEMO_PANELS`,
/// generated by the invocation below, only sees items already declared lexically above it in this file.
///
/// Each key is `stringify!($module_path)` for a demo module's path exactly as `demo_gallery!` writes it — e.g.
/// `"shapes"` or `"texts :: demo_text"` (`stringify!` spaces multi-segment paths out; [`split_demo_path`] produces
/// the identical spacing from `DEMO_PANELS`'s own `stringify!($path)`, so the two always agree without this list
/// needing to match by hand). Adding a new module to `demo_gallery!` below means adding its file here too —
/// nothing enforces that automatically, the same as any other `include_str!` list.
const DEMO_SOURCE_FILES: &[(&str, &str)] = &[
    (stringify!(shapes), include_str!("shapes.rs")),
    (stringify!(texts::demo_text), include_str!("texts/demo_text.rs")),
    (stringify!(texts::demo_tspan), include_str!("texts/demo_tspan.rs")),
    (stringify!(texts::demo_text_path), include_str!("texts/demo_text_path.rs")),
    (stringify!(structure), include_str!("structure.rs")),
    (stringify!(paint), include_str!("paint.rs")),
    (stringify!(events), include_str!("events.rs")),
    (stringify!(geometry), include_str!("geometry.rs")),
];

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Generates [`DEMO_PANELS`] from one list, so a panel's id, its build function, and the function name shown in its
/// source frame can never drift apart the way three independently hand-maintained lists could.
///
/// This file's own module-grouped order (shapes, then structure, then paint, ...) is what `DEMO_PANELS` ends up in.
/// That no longer means anything operationally — every panel is looked up by id via [`init_panel`], not iterated in
/// order — and it is not, and has no need to be, the menu's category order. The menu is built separately, from
/// `demo-server`'s own manifest (`demo-server/src/panels.rs`); the two lists serve different purposes and have no
/// reason to match.
///
/// The build function is matched as a single `$path:path` fragment — accepting any number of `::`-separated
/// segments — rather than the fixed `$module:ident :: $func:ident` an earlier version of this macro used. That
/// fixed-depth grammar could not parse a path like `texts::demo_text::demo` at all: a demo module nested one level
/// deeper than a bare top-level file, which is exactly what splitting a demo module like `texts` into its own
/// per-function sub-modules produces. Splitting `$path` into "which file" and "which function" (see
/// [`split_demo_path`]) happens at runtime instead, on the stringified path, specifically so that *this* grammar
/// keeps working unchanged the next time some other module gets split the same way — accepting `path` already
/// covers any nesting depth, so nothing here needs to change again for that.
macro_rules! demo_gallery {
    ( $( $panel_id:literal => $path:path ),+ $(,)? ) => {
        /// `(panel id, demo build function, demo module path)` for every demo, generated by the `demo_gallery!`
        /// invocation below. [`init_panel`] looks a panel up here by id to build it the first time it is
        /// selected; the module path is what [`demo_fn_source`] (via [`split_demo_path`]) uses to find the
        /// right entry in [`DEMO_SOURCE_FILES`] and display it alongside the panel.
        const DEMO_PANELS: &[(&str, fn() -> Result<(), Error>, &str)] = &[
            $( ($panel_id, $path, stringify!($path)) ),+
        ];
    };
}

demo_gallery! {
    "panel-rect" => shapes::demo_rect,
    "panel-circle" => shapes::demo_circle,
    "panel-ellipse" => shapes::demo_ellipse,
    "panel-line" => shapes::demo_line,
    "panel-poly" => shapes::demo_poly,
    "panel-path" => shapes::demo_path,
    "panel-group" => structure::demo_group,
    "panel-anim" => structure::demo_anim,
    "panel-marker" => structure::demo_marker,
    "panel-marker-view-box" => structure::demo_marker_view_box,
    "panel-use" => structure::demo_use,
    "panel-image" => structure::demo_image,
    "panel-symbol" => structure::demo_symbol,
    "panel-anchor" => structure::demo_anchor,
    "panel-switch" => structure::demo_switch,
    "panel-view" => structure::demo_view,
    "panel-style" => structure::demo_style,
    "panel-foreign-object" => structure::demo_foreign_object,
    "panel-view-box" => structure::demo_view_box,
    "panel-tree-nav" => structure::demo_tree_nav,
    "panel-accessibility" => structure::demo_accessibility,
    "panel-linear-gradient" => paint::demo_linear_gradient,
    "panel-radial-gradient" => paint::demo_radial_gradient,
    "panel-clip-path" => paint::demo_clip_path,
    "panel-mask" => paint::demo_mask,
    "panel-pattern" => paint::demo_pattern,
    "panel-filter" => paint::demo_filter,
    "panel-color-matrix" => paint::demo_color_matrix,
    "panel-blend" => paint::demo_blend,
    "panel-component-transfer" => paint::demo_component_transfer,
    "panel-turbulence" => paint::demo_turbulence,
    "panel-morphology" => paint::demo_morphology,
    "panel-fe-image" => paint::demo_fe_image,
    "panel-fe-tile" => paint::demo_fe_tile,
    "panel-convolve-matrix" => paint::demo_convolve_matrix,
    "panel-lighting" => paint::demo_lighting,
    "panel-light-sources" => paint::demo_light_sources,
    "panel-text" => texts::demo_text::demo,
    "panel-tspan" => texts::demo_tspan::demo,
    "panel-text-path" => texts::demo_text_path::demo,
    "panel-events-click" => events::demo_events_click,
    "panel-events-colour" => events::demo_events_colour,
    "panel-events-modifiers" => events::demo_events_modifiers,
    "panel-events-press" => events::demo_events_press,
    "panel-events-group" => events::demo_events_group,
    "panel-events-pointer" => events::demo_events_pointer_lifecycle,
    "panel-events-keyboard-wheel" => events::demo_events_keyboard_wheel,
    "panel-events-drag-drop-touch" => events::demo_events_drag_drop_touch,
    "panel-events-passive" => events::demo_events_passive,
    "panel-events-classlist" => events::demo_events_classlist,
    "panel-geometry-path-follow" => geometry::demo_geometry_path_follow,
    "panel-geometry-bbox" => geometry::demo_geometry_bounding_box,
}

/// Marks a panel's `<section>` with its initialisation outcome, so [`init_panel`] can tell "never attempted"
/// (attribute absent) apart from "already attempted" (attribute present, `"ready"` or `"failed"`) and skip
/// rebuilding a panel that has already been built once — rebuilding it again would duplicate its contents rather
/// than replace them, since `SvgRoot::create_in` always appends a fresh `<svg>` into the canvas `<div>`.
const PANEL_STATE_ATTR: &str = "data-panel-state";

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Builds one demo panel's content the first time it is selected, rather than every panel eagerly at page load —
/// which is what this gallery used to do, even though at most one panel is ever visible at once (every other
/// `<section>` sits behind `display: none`; see `.section`/`.section.active` in `style.css`). That eager approach
/// meant, among other costs, that both of this gallery's `AnimationLoop`-driven demos
/// (`structure::demo_anim`, `geometry::demo_geometry_path_follow`) started ticking at page load and ran for the
/// rest of the page's lifetime, animating a hidden panel nobody was looking at.
///
/// This only defers that cost, it does not remove it: once an `AnimationLoop`-driven panel has been visited, its
/// loop is parked in [`LIVE_DEMO_ANIMS`] and keeps ticking for the rest of the page's lifetime even after the user
/// navigates to a different panel — lazy construction fixes "running before anyone asked for it", not "still
/// running once nobody is looking anymore".
///
/// For the two lightweight animations currently in this gallery, the added complexity of a pause/resume lifecycle
/// (e.g. an `activate_panel`/`deactivate_panel` pair, or getting each animation callback to check its own section's
/// `.active` class), is not worth the effort.
///
/// If the number of animations grows, it would be worth revisiting this design decision.
///
/// Call this from JavaScript each time a panel becomes the active one (see `demo/index.template.html`'s
/// `selectDemo`). It is idempotent — see [`PANEL_STATE_ATTR`] — so calling it again for an already-initialised
/// panel, e.g. navigating back to one visited earlier, is a no-op rather than a duplicate rebuild.
///
/// A demo that fails to build is recorded as `"failed"` rather than retried on a later visit; [`run_panel`] and
/// [`report_panel_error`] are what surface that failure in the gallery itself. A missing or mismatched panel id is
/// a different kind of problem — a catalogue error rather than a demo runtime error — and is caught at server
/// startup instead, before the gallery is ever served (see `demo-server/src/validate.rs` and
/// `demo-server/src/panels.rs`'s `assemble`), so `init_panel` never has to distinguish the two: by the time it
/// runs, the catalogue has already been validated.
#[wasm_bindgen]
pub fn init_panel(panel_id: &str) -> Result<(), JsValue> {
    let e = |err: Error| JsValue::from_str(&err.to_string());
    let document = web_sys::window()
        .and_then(|w| w.document())
        .ok_or_else(|| Error::Dom("no document".into()))
        .map_err(e)?;

    let Some(section) = document.get_element_by_id(panel_id) else { return Ok(()) };
    if section.has_attribute(PANEL_STATE_ATTR) {
        return Ok(());
    }
    let Some(&(_, build, demo_path)) = DEMO_PANELS.iter().find(|(id, ..)| *id == panel_id) else {
        return Ok(());
    };

    let state = if run_panel(panel_id, build) { "ready" } else { "failed" };
    section.set_attribute(PANEL_STATE_ATTR, state).map_err(dom_err).map_err(e)?;

    // Show the Rust source of the function that just built this panel — including when it failed to build; the
    // source is still worth seeing even when the demo itself broke.
    append_source_frame(&document, panel_id, demo_path).map_err(e)?;
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Runs one demo's build function, catching its error rather than letting it propagate, and reports whether it
/// succeeded so [`init_panel`] can record that as the panel's state. A broken demo is this function's own runtime
/// behaviour, not a missing or mismatched panel — see [`init_panel`]'s doc comment for why those are handled
/// differently.
fn run_panel(panel_id: &str, build: fn() -> Result<(), Error>) -> bool {
    match build() {
        Ok(()) => true,
        Err(err) => {
            report_panel_error(panel_id, &err.to_string());
            false
        },
    }
}

/// Appends a visible failure banner to `panel_id`'s own `<section>`, so a demo that fails to build is immediately
/// obvious in the gallery itself instead of just leaving that panel's canvas blank. This is a development and
/// verification tool; a silently empty panel is exactly the outcome worth avoiding here.
///
/// Silently does nothing if the panel section itself cannot be found — by construction (see [`run_panel`]'s doc
/// comment) that cannot happen through the normal `cargo demo` pipeline, so this mirrors [`append_source_frame`]'s
/// own defensive fallback rather than treating an already-impossible case as fatal.
fn report_panel_error(panel_id: &str, message: &str) {
    let Some(document) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    let Some(section) = document.get_element_by_id(panel_id) else { return };
    let Ok(banner) = create_element(&document, "p") else { return };
    banner.set_class_name("demo-error");
    banner.set_text_content(Some(&format!("This demo failed to build: {message}")));
    let _ = section.append_child(&banner);
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Source-code frames
//
// Each panel containing a demo example gets a collapsible frame below its canvas showing the formatted Rust source of
// the function that built it, injected by init_panel the first time that panel is selected. The source TEXT itself is
// embedded at compile time from each sub-module file (via DEMO_SOURCE_FILES/include_str!), so that part genuinely
// never drifts from what is actually running — there is no separate copy of it to keep in sync by hand.
//
// The (panel id -> function) MAPPING that decides which source shows up in which panel is a different matter: it is
// the hand-written `demo_gallery!` invocation above, not something derived automatically.
//
// demo-server's `validate` step will catch any mismatches when the normal `cargo demo` pipeline starts (see
// `demo-server/src/validate.rs`), and `unit_tests::every_registered_demo_has_extractable_source` in
// `demo-app/src/unit_tests.rs` verifies that `demo_fn_source` can actually locate every registered function's source.
//
// However, `append_source_frame` below still fails silently rather than erroring if a panel or function genuinely
// cannot be found, as a fallback for anything run outside that validated pipeline.
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Builds `<details class="source"><summary>...</summary><pre><code>...</code></pre></details>` and appends it to the
/// panel `<section>`. Missing panels or source functions are skipped defensively for callers running the WASM module
/// outside the validated `cargo demo` pipeline.
///
/// Normal gallery builds reject catalogue mismatches during server startup and unit tests verify source extraction —
/// see this section's own doc comment above.
///
/// `demo_path` is `DEMO_PANELS`'s recorded `stringify!($path)` for this panel's build function — e.g.
/// `"texts :: demo_text :: demo"` — not yet split into "which file" and "which function name"; see
/// [`split_demo_path`].
fn append_source_frame(document: &web_sys::Document, panel_id: &str, demo_path: &str) -> Result<(), Error> {
    let Some(section) = document.get_element_by_id(panel_id) else {
        return Ok(());
    };
    let (module_path, fn_name) = split_demo_path(demo_path);
    let Some(source) = demo_fn_source(module_path, fn_name) else {
        return Ok(());
    };

    let details = create_element(document, "details")?;
    details.set_class_name("source");
    details.set_attribute("open", "").map_err(dom_err)?;

    let summary = create_element(document, "summary")?;
    summary.set_text_content(Some(&format!("Rust source — fn {fn_name}")));
    details.append_child(&summary).map_err(dom_err)?;

    let pre = create_element(document, "pre")?;
    let code = create_element(document, "code")?;
    // `rust_to_html` returns `<span>`-wrapped, HTML-escaped tokens, so angle brackets and ampersands in the code still
    // render verbatim while keywords, strings, etc. are coloured by the CSS classes.
    code.set_inner_html(&highlight::rust_to_html(source));
    pre.append_child(&code).map_err(dom_err)?;
    details.append_child(&pre).map_err(dom_err)?;

    section.append_child(&details).map_err(dom_err)?;
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Splits a `DEMO_PANELS`-recorded `stringify!($path)` (e.g. `"texts :: demo_text :: demo"` or `"shapes ::
/// demo_rect"`) into `(module_path, fn_name)` (e.g. `("texts :: demo_text", "demo")`), trimmed of the whitespace
/// `stringify!` inserts around each `::`.
///
/// A demo path always has at least one `::` — `demo_gallery!`'s own grammar requires `$path:path`, so a bare
/// function name with no module at all is not a valid entry — hence the `expect` rather than an `Option`/`Result`:
/// a path that fails to split this way would mean `stringify!` itself stopped inserting `::`, not a normal runtime
/// condition this function should have to handle gracefully.
fn split_demo_path(demo_path: &str) -> (&str, &str) {
    let (module_path, fn_name) = demo_path
        .rsplit_once("::")
        .expect("a demo_gallery! path always has at least one `::`");
    (module_path.trim(), fn_name.trim())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Returns the source text of the top-level `fn {name}` item in whichever [`DEMO_SOURCE_FILES`] entry matches
/// `module_path`, from the signature line through its closing brace, or `None` if either the module path is not in
/// [`DEMO_SOURCE_FILES`] or the function cannot be located within it.
///
/// Searching only the one matching file, rather than one flat blob of every demo module's source concatenated
/// together, is what lets two different files each define a same-named function (as `texts`'s three sub-modules,
/// each just `fn demo()`, now do) without colliding — see [`DEMO_SOURCE_FILES`]'s own doc comment.
///
/// This relies on `rustfmt`'s guarantee that a top-level item's closing brace will always sit in column 0 while every
/// brace nested inside the body (including those inside `format!` strings such as `"translate({x}, {y})"`) will be
/// indented.
///
/// Scanning for the first line that is exactly `}` after the signature therefore finds the function's end without
/// having to parse any string literals or resort to any brace-balancing shenanigans.
fn demo_fn_source(module_path: &str, name: &str) -> Option<&'static str> {
    let source = DEMO_SOURCE_FILES.iter().find(|(path, _)| *path == module_path)?.1;

    let needle = format!("fn {name}(");
    let hit = source.find(&needle)?;
    let start = source[..hit].rfind('\n').map_or(0, |i| i + 1);

    let tail = &source[hit..];
    let mut from = 0;
    loop {
        let rel = tail[from..].find("\n}")?;
        let close = from + rel + 1; // index of the '}' within `tail`
        let after = close + 1;
        // A genuine top-level close: the '}' stands alone on its line (next byte is a newline or end of file).
        if tail.as_bytes().get(after).is_none_or(|&b| b == b'\n') {
            return Some(&source[start..hit + after]);
        }
        from = after;
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Creates an element, mapping any DOM failure into [`Error::Dom`].
fn create_element(document: &web_sys::Document, tag: &str) -> Result<web_sys::Element, Error> {
    document.create_element(tag).map_err(dom_err)
}

fn dom_err(e: JsValue) -> Error {
    Error::Dom(format!("{e:?}"))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Shared helper
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Appends a small grey caption below an element at horizontal centre `cx`.
fn caption(svg: &SvgRoot, cx: f64, text: &str) -> Result<(), Error> {
    let t = svg.text(Point::new(cx, PAD_Y + BAND - 6.0), text)?;
    let mut attrs = SvgAttrs::new();
    t.attrs(&mut attrs)
        .fill(CAPTION)?
        .apply([("font-size", "11"), ("text-anchor", "middle")])?;
    Ok(())
}

#[cfg(test)]
mod unit_tests;

/// A wasm-bindgen-test browser suite for the interactive controls added to `texts.rs` (text-anchor/
/// dominant-baseline radios, startOffset slider) — `unit_tests` above only proves every demo function's source is
/// extractable, not that these controls actually dispatch to the right SVG attribute. See `browser_tests.rs`'s
/// own doc comment for how it is run and why it is a separate module from `unit_tests`.
#[cfg(test)]
mod browser_tests;
