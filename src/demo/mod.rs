//! Interactive element gallery for the browser.
//!
//! Build and serve with:
//! ```sh
//! cargo demo
//! ```
//! which rebuilds the wasm package and serves it via the Actix `demo-server` crate at <http://127.0.0.1:8080/demo/>.
//!
//! The `demo` feature excludes this code from the normal library build.

mod colours;
mod events;
mod geometry;
mod highlight;
mod paint;
mod shapes;
mod structure;
mod texts;

use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;

use crate::{AnimationLoop, CachedAttr, Error, SvgAttrs, SvgNode, SvgRoot, root::utils::Point};
use colours::*;

thread_local! {
    /// Demo-only owner for interactive nodes whose managed listeners must remain attached after `run_demo` returns.
    ///
    /// The library intentionally removes a DOM listener when the last `SvgNode` handle is dropped.  Browser demos are
    /// long-lived, so they keep listener-owning nodes here instead of leaking individual `Closure`s.
    static LIVE_DEMO_NODES: RefCell<Vec<SvgNode>> = const { RefCell::new(Vec::new()) };

    /// Demo-only owner for the running `AnimationLoop`.
    ///
    /// `AnimationLoop` stops on `Drop`, so the demo parks it here to keep it running for the page's lifetime.  Storing
    /// it (rather than leaking it with `mem::forget`) keeps it cancellable: clearing or replacing this slot stops the
    /// animation cleanly.
    static LIVE_DEMO_ANIM: RefCell<Option<AnimationLoop>> = const { RefCell::new(None) };
}

fn keep_demo_node(node: SvgNode) {
    LIVE_DEMO_NODES.with(|nodes| nodes.borrow_mut().push(node));
}

fn keep_demo_anim(anim: AnimationLoop) {
    LIVE_DEMO_ANIM.with(|slot| *slot.borrow_mut() = Some(anim));
}

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
// Public entry point
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Create a labelled SVG demo for each supported element type and inject each into a `<div>` placeholders in
/// `demo/index.html`.
///
/// Call this from JavaScript after `await init()`:
/// ```js
/// import init, { run_demo } from '../pkg/svg_dom.js';
/// await init();
/// run_demo();
/// ```
#[wasm_bindgen]
pub fn run_demo() -> Result<(), JsValue> {
    let e = |err: Error| JsValue::from_str(&err.to_string());
    shapes::demo_rect().map_err(e)?;
    shapes::demo_circle().map_err(e)?;
    shapes::demo_ellipse().map_err(e)?;
    shapes::demo_line().map_err(e)?;
    shapes::demo_poly().map_err(e)?;
    shapes::demo_path().map_err(e)?;
    texts::demo_text().map_err(e)?;
    structure::demo_group().map_err(e)?;
    structure::demo_anim().map_err(e)?;
    structure::demo_marker().map_err(e)?;
    structure::demo_marker_view_box().map_err(e)?;
    structure::demo_use().map_err(e)?;
    structure::demo_image().map_err(e)?;
    structure::demo_symbol().map_err(e)?;
    structure::demo_view_box().map_err(e)?;
    structure::demo_tree_nav().map_err(e)?;
    structure::demo_accessibility().map_err(e)?;
    paint::demo_linear_gradient().map_err(e)?;
    paint::demo_radial_gradient().map_err(e)?;
    paint::demo_clip_path().map_err(e)?;
    paint::demo_mask().map_err(e)?;
    paint::demo_pattern().map_err(e)?;
    paint::demo_filter().map_err(e)?;
    paint::demo_color_matrix().map_err(e)?;
    paint::demo_blend().map_err(e)?;
    paint::demo_component_transfer().map_err(e)?;
    paint::demo_turbulence().map_err(e)?;
    texts::demo_tspan().map_err(e)?;
    texts::demo_text_path().map_err(e)?;

    // Event-handling gallery
    events::demo_events_click().map_err(e)?;
    events::demo_events_colour().map_err(e)?;
    events::demo_events_modifiers().map_err(e)?;
    events::demo_events_press().map_err(e)?;
    events::demo_events_group().map_err(e)?;
    events::demo_events_pointer_lifecycle().map_err(e)?;
    events::demo_events_keyboard_wheel().map_err(e)?;
    events::demo_events_drag_drop_touch().map_err(e)?;
    events::demo_events_passive().map_err(e)?;
    events::demo_events_classlist().map_err(e)?;

    // Geometry read-back gallery
    geometry::demo_geometry_path_follow().map_err(e)?;
    geometry::demo_geometry_bounding_box().map_err(e)?;

    // Below each demo, show the Rust source of the function that produced it.
    inject_source_frames().map_err(e)?;
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Source-code frames
//
// Each demo panel gets a collapsible frame below its canvas showing the formatted Rust source of the function that
// built it. The source is embedded at compile time from each sub-module file, so it is always in step with the code
// actually running — there is nothing to keep manually in sync.
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// All demo sub-module sources concatenated at build time so each panel can display the exact code that drives it.
const DEMO_SRC: &str = concat!(
    include_str!("shapes.rs"),
    include_str!("texts.rs"),
    include_str!("structure.rs"),
    include_str!("paint.rs"),
    include_str!("events.rs"),
    include_str!("geometry.rs"),
);

/// `(panel id, demo function name)` for every demo, in menu order.
/// The panel ids must match the <button class="menu-item"> elements listed in `demo/index.html` and the function names
/// must match the functions in this module that implement that particular demo.
///
/// The function names are looked up in [`DEMO_SRC`].
const DEMO_SOURCES: &[(&str, &str)] = &[
    ("panel-rect", "demo_rect"),
    ("panel-circle", "demo_circle"),
    ("panel-ellipse", "demo_ellipse"),
    ("panel-line", "demo_line"),
    ("panel-poly", "demo_poly"),
    ("panel-path", "demo_path"),
    ("panel-text", "demo_text"),
    ("panel-group", "demo_group"),
    ("panel-anim", "demo_anim"),
    ("panel-marker", "demo_marker"),
    ("panel-marker-view-box", "demo_marker_view_box"),
    ("panel-use", "demo_use"),
    ("panel-image", "demo_image"),
    ("panel-symbol", "demo_symbol"),
    ("panel-view-box", "demo_view_box"),
    ("panel-tree-nav", "demo_tree_nav"),
    ("panel-accessibility", "demo_accessibility"),
    ("panel-linear-gradient", "demo_linear_gradient"),
    ("panel-radial-gradient", "demo_radial_gradient"),
    ("panel-clip-path", "demo_clip_path"),
    ("panel-mask", "demo_mask"),
    ("panel-pattern", "demo_pattern"),
    ("panel-filter", "demo_filter"),
    ("panel-color-matrix", "demo_color_matrix"),
    ("panel-blend", "demo_blend"),
    ("panel-component-transfer", "demo_component_transfer"),
    ("panel-turbulence", "demo_turbulence"),
    ("panel-tspan", "demo_tspan"),
    ("panel-text-path", "demo_text_path"),
    ("panel-events-click", "demo_events_click"),
    ("panel-events-colour", "demo_events_colour"),
    ("panel-events-modifiers", "demo_events_modifiers"),
    ("panel-events-press", "demo_events_press"),
    ("panel-events-group", "demo_events_group"),
    ("panel-events-pointer", "demo_events_pointer_lifecycle"),
    ("panel-events-keyboard-wheel", "demo_events_keyboard_wheel"),
    ("panel-events-drag-drop-touch", "demo_events_drag_drop_touch"),
    ("panel-events-passive", "demo_events_passive"),
    ("panel-events-classlist", "demo_events_classlist"),
    ("panel-geometry-path-follow", "demo_geometry_path_follow"),
    ("panel-geometry-bbox", "demo_geometry_bounding_box"),
];

/// Appends a source frame to every panel listed in [`DEMO_SOURCES`].
fn inject_source_frames() -> Result<(), Error> {
    let document = web_sys::window()
        .and_then(|w| w.document())
        .ok_or_else(|| Error::Dom("no document".into()))?;

    for (panel_id, fn_name) in DEMO_SOURCES {
        append_source_frame(&document, panel_id, fn_name)?;
    }
    Ok(())
}

/// Builds `<details class="source"><summary>...</summary><pre><code>...</code></pre></details>` and appends it to the
/// panel `<section>`. A missing panel or missing function is skipped rather than treated as an error, so the gallery
/// still renders if `index.html` and this module drift apart.
fn append_source_frame(document: &web_sys::Document, panel_id: &str, fn_name: &str) -> Result<(), Error> {
    let Some(section) = document.get_element_by_id(panel_id) else {
        return Ok(());
    };
    let Some(source) = demo_fn_source(fn_name) else {
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

/// Returns the source text of the top-level `fn {name}` item in [`DEMO_SRC`], from the signature line through its
/// closing brace, or `None` if it cannot be located.
///
/// This relies on `rustfmt`'s guarantee that a top-level item's closing brace sits in column 0 while every brace nested
/// inside the body (including those inside `format!` strings such as `"translate({x}, {y})"`) is indented. Scanning for
/// the first line that is exactly `}` after the signature therefore finds the function's end without having to parse
/// string literals or balance braces.
fn demo_fn_source(name: &str) -> Option<&'static str> {
    let needle = format!("fn {name}(");
    let hit = DEMO_SRC.find(&needle)?;
    let start = DEMO_SRC[..hit].rfind('\n').map_or(0, |i| i + 1);

    let tail = &DEMO_SRC[hit..];
    let mut from = 0;
    loop {
        let rel = tail[from..].find("\n}")?;
        let close = from + rel + 1; // index of the '}' within `tail`
        let after = close + 1;
        // A genuine top-level close: the '}' stands alone on its line (next byte is a newline or end of file).
        if tail.as_bytes().get(after).is_none_or(|&b| b == b'\n') {
            return Some(&DEMO_SRC[start..hit + after]);
        }
        from = after;
    }
}

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
