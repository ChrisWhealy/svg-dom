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
mod highlight;

use std::{
    cell::{Cell, RefCell},
    fmt::Write,
    rc::Rc,
};
use wasm_bindgen::prelude::*;

use crate::{
    AnimationLoop, CachedAttr, DominantBaseline, Error, SvgAttrs, SvgNode, SvgRoot, TextAnchor,
    root::{
        gradient::SpreadMethod,
        utils::{Point, Size},
    },
};
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
    demo_rect().map_err(e)?;
    demo_circle().map_err(e)?;
    demo_ellipse().map_err(e)?;
    demo_line().map_err(e)?;
    demo_poly().map_err(e)?;
    demo_path().map_err(e)?;
    demo_text().map_err(e)?;
    demo_group().map_err(e)?;
    demo_anim().map_err(e)?;
    demo_marker().map_err(e)?;
    demo_use().map_err(e)?;
    demo_image().map_err(e)?;
    demo_linear_gradient().map_err(e)?;
    demo_radial_gradient().map_err(e)?;
    demo_clip_path().map_err(e)?;
    demo_tspan().map_err(e)?;

    // Event-handling gallery
    demo_events_click().map_err(e)?;
    demo_events_colour().map_err(e)?;
    demo_events_modifiers().map_err(e)?;
    demo_events_press().map_err(e)?;
    demo_events_group().map_err(e)?;
    demo_events_pointer_lifecycle().map_err(e)?;
    demo_events_keyboard_wheel().map_err(e)?;
    demo_events_drag_drop_touch().map_err(e)?;
    demo_events_passive().map_err(e)?;

    // Below each demo, show the Rust source of the function that produced it.
    inject_source_frames().map_err(e)?;
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Source-code frames
//
// Each demo panel gets a collapsible frame below its canvas showing the formatted Rust source of the function that
// built it. The source is the crate's own demo module, embedded at compile time, so it is always in step with the code
// actually running — there is nothing to keep manually in sync.
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// This module's own source, embedded at build time so each panel can display the exact code that drives it.
const DEMO_SRC: &str = include_str!("mod.rs");

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
    ("panel-use", "demo_use"),
    ("panel-image", "demo_image"),
    ("panel-linear-gradient", "demo_linear_gradient"),
    ("panel-radial-gradient", "demo_radial_gradient"),
    ("panel-clip-path", "demo_clip_path"),
    ("panel-tspan", "demo_tspan"),
    ("panel-events-click", "demo_events_click"),
    ("panel-events-colour", "demo_events_colour"),
    ("panel-events-modifiers", "demo_events_modifiers"),
    ("panel-events-press", "demo_events_press"),
    ("panel-events-group", "demo_events_group"),
    ("panel-events-pointer", "demo_events_pointer_lifecycle"),
    ("panel-events-keyboard-wheel", "demo_events_keyboard_wheel"),
    ("panel-events-drag-drop-touch", "demo_events_drag_drop_touch"),
    ("panel-events-passive", "demo_events_passive"),
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

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// rect
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn demo_rect() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-rect", Size::new(W, H))?;

    // 1. Plain fill
    let r1 = svg.rect(Point::new(10.0, 10.0 + PAD_Y), Size::new(130.0, 90.0))?;
    r1.set_fill(STEELBLUE)?;
    caption(&svg, 75.0, "fill")?;

    // 2. Stroke-only (no fill)
    let r2 = svg.rect(Point::new(155.0, 10.0 + PAD_Y), Size::new(130.0, 90.0))?;
    let mut attrs = SvgAttrs::new();
    r2.attrs(&mut attrs).fill(NONE)?.stroke(CORAL)?.stroke_width(3.0)?;
    caption(&svg, 220.0, "stroke")?;

    // 3. Rounded corners via rx attribute
    let r3 = svg.rect(Point::new(300.0, 10.0 + PAD_Y), Size::new(130.0, 90.0))?;
    r3.set_fill(MEDIUM_SEA_GREEN)?;
    r3.set_attr("rx", "20")?;
    caption(&svg, 365.0, "rounded (rx)")?;

    // 4. Hover: fill swaps on pointerenter / pointerleave
    // Strong self-captures are intentional here: these demo nodes live for the page lifetime,
    // so the reference cycle is harmless.  In application code prefer `downgrade()`/`upgrade()`.
    let r4 = svg.rect(Point::new(445.0, 10.0 + PAD_Y), Size::new(130.0, 90.0))?;
    r4.set_fill(GOLDENROD)?;
    r4.set_attr("style", "cursor:pointer")?;
    let r4b = r4.clone();
    r4.on_pointerenter(move |_| {
        let _ = r4b.set_fill(GOLD);
    })?;
    let r4c = r4.clone();
    r4.on_pointerleave(move |_| {
        let _ = r4c.set_fill(GOLDENROD);
    })?;
    caption(&svg, 510.0, "hover")?;

    // 5. Click: toggles between two fills
    let r5 = svg.rect(Point::new(590.0, 10.0 + PAD_Y), Size::new(130.0, 90.0))?;
    r5.set_fill(SLATE_GRAY)?;
    r5.set_attr("style", "cursor:pointer")?;
    let toggled = Rc::new(Cell::new(false));
    let r5b = r5.clone();
    r5.on_click(move |_| {
        let next = !toggled.get();
        toggled.set(next);
        let _ = r5b.set_fill(if next { CORAL } else { SLATE_GRAY });
    })?;
    caption(&svg, 655.0, "click (toggle)")?;

    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// circle
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn demo_circle() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-circle", Size::new(W, H))?;

    // 1. Solid fill
    let c1 = svg.circle(Point::new(70.0, 57.0 + PAD_Y), 47.0)?;
    c1.set_fill(TOMATO)?;
    caption(&svg, 70.0, "fill")?;

    // 2. Stroke-only
    let c2 = svg.circle(Point::new(210.0, 57.0 + PAD_Y), 47.0)?;
    c2.set_fill(NONE)?;
    c2.set_stroke(ORCHID)?;
    c2.set_stroke_width(4.0)?;
    caption(&svg, 210.0, "stroke")?;

    // 3. Hover: radius grows / shrinks
    // Strong self-captures are intentional: page-lifetime demo nodes, harmless cycle.
    let c3 = svg.circle(Point::new(360.0, 57.0 + PAD_Y), 35.0)?;
    c3.set_fill(LIGHT_SKY_BLUE)?;
    c3.set_attr("style", "cursor:pointer")?;
    let c3b = c3.clone();
    c3.on_pointerenter(move |_| {
        let _ = c3b.set_attr("r", "50");
    })?;
    let c3c = c3.clone();
    c3.on_pointerleave(move |_| {
        let _ = c3c.set_attr("r", "35");
    })?;
    caption(&svg, 360.0, "hover (radius)")?;

    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// ellipse
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn demo_ellipse() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-ellipse", Size::new(W, H))?;

    // 1. Wide ellipse — independent radii (rx > ry), something <circle> cannot do
    let e1 = svg.ellipse(Point::new(110.0, 57.0 + PAD_Y), Size::new(90.0, 45.0))?;
    e1.set_fill(MEDIUM_ORCHID)?;
    caption(&svg, 110.0, "wide (rx > ry)")?;

    // 2. Tall ellipse, stroke only
    let e2 = svg.ellipse(Point::new(320.0, 57.0 + PAD_Y), Size::new(45.0, 52.0))?;
    e2.set_fill(NONE)?;
    e2.set_stroke(LIGHT_SKY_BLUE)?;
    e2.set_stroke_width(4.0)?;
    caption(&svg, 320.0, "tall stroke (ry > rx)")?;

    // 3. Hover: both radii grow on pointerenter and shrink back on pointerleave.
    //
    // The hover ellipse (90 x 55) fully contains the resting one (60 x 35), so the boundary only ever moves *outward*
    // under the pointer. A hover effect that instead shrank a radius would pull the edge back past a stationary pointer
    // — re-triggering pointerleave, then pointerenter as it grew again — and the ellipse would flicker between states.
    // Strong self-captures are intentional: page-lifetime demo nodes, harmless cycle.
    let e3 = svg.ellipse(Point::new(560.0, 57.0 + PAD_Y), Size::new(60.0, 35.0))?;
    e3.set_fill(GOLDENROD)?;
    e3.set_attr("style", "cursor:pointer")?;
    let e3b = e3.clone();
    e3.on_pointerenter(move |_| {
        let _ = e3b.set_attr("rx", "90");
        let _ = e3b.set_attr("ry", "55");
    })?;
    let e3c = e3.clone();
    e3.on_pointerleave(move |_| {
        let _ = e3c.set_attr("rx", "60");
        let _ = e3c.set_attr("ry", "35");
    })?;
    caption(&svg, 560.0, "hover (grow radii)")?;

    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// line
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn demo_line() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-line", Size::new(W, H))?;

    // Horizontal
    let l1 = svg.line(Point::new(10.0, 55.0 + PAD_Y), Point::new(230.0, 55.0 + PAD_Y))?;
    l1.set_stroke(WIRE)?;
    l1.set_stroke_width(2.0)?;
    caption(&svg, 120.0, "horizontal")?;
    // Diagonal
    let l2 = svg.line(Point::new(270.0, 10.0 + PAD_Y), Point::new(470.0, 110.0 + PAD_Y))?;
    l2.set_stroke(CORAL)?;
    l2.set_stroke_width(2.0)?;
    caption(&svg, 370.0, "diagonal")?;

    // Thick
    let l3 = svg.line(Point::new(510.0, 55.0 + PAD_Y), Point::new(790.0, 55.0 + PAD_Y))?;
    l3.set_stroke(GOLDENROD)?;
    l3.set_stroke_width(18.0)?;
    caption(&svg, 650.0, "thick stroke")?;

    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// polygon / polyline
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn demo_poly() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-poly", Size::new(W, H))?;

    // 1. Polygon — closed, filled triangle (last point auto-joins to the first)
    let tri = svg.polygon(&[
        Point::new(110.0, 12.0 + PAD_Y),
        Point::new(175.0, 100.0 + PAD_Y),
        Point::new(45.0, 100.0 + PAD_Y),
    ])?;
    tri.set_fill(STEELBLUE)?;
    caption(&svg, 110.0, "polygon (closed)")?;

    // 2. Polyline — open zig-zag: stroked, fill explicitly "none"
    let zig = svg.polyline(&[
        Point::new(290.0, 100.0 + PAD_Y),
        Point::new(320.0, 20.0 + PAD_Y),
        Point::new(350.0, 100.0 + PAD_Y),
        Point::new(380.0, 20.0 + PAD_Y),
        Point::new(410.0, 100.0 + PAD_Y),
    ])?;
    zig.set_fill(NONE)?;
    zig.set_stroke(TEAL)?;
    zig.set_stroke_width(3.0)?;
    caption(&svg, 350.0, "polyline (open, fill:none)")?;

    // 3. Polyline — same shape, but left to fill: the open path is filled as if closed
    let filled = svg.polyline(&[
        Point::new(530.0, 100.0 + PAD_Y),
        Point::new(560.0, 20.0 + PAD_Y),
        Point::new(590.0, 100.0 + PAD_Y),
        Point::new(620.0, 20.0 + PAD_Y),
        Point::new(650.0, 100.0 + PAD_Y),
    ])?;
    filled.set_fill(CORAL)?;
    caption(&svg, 590.0, "polyline (filled)")?;

    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// path
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn demo_path() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-path", Size::new(W, H))?;
    // The path data is authored in the BAND; this transform vertically centres each path in the canvas.
    let shift = format!("translate(0,{PAD_Y})");

    // Closed triangle (M / L / Z)
    let tri = svg.path("M 70 10 L 130 110 L 10 110 Z")?;
    let mut attrs = SvgAttrs::new();
    tri.attrs(&mut attrs)
        .fill(STEELBLUE)?
        .stroke(WHITE)?
        .stroke_width(2.0)?
        .set("transform", &shift)?;
    caption(&svg, 70.0, "triangle (M L Z)")?;

    // Quadratic Bézier wave (Q)
    let wave = svg.path("M 180 65 Q 245 10 310 65 Q 375 120 440 65")?;
    wave.set_fill(NONE)?;
    wave.set_stroke(MEDIUM_ORCHID)?;
    wave.set_stroke_width(3.0)?;
    wave.set_attr("transform", &shift)?;
    caption(&svg, 310.0, "Bézier wave (Q)")?;

    // Elliptical arc — open semicircle (A)
    let arc = svg.path("M 510 65 A 60 60 0 1 1 630 65")?;
    arc.set_fill(NONE)?;
    arc.set_stroke(CORAL)?;
    arc.set_stroke_width(3.0)?;
    arc.set_attr("transform", &shift)?;
    caption(&svg, 570.0, "arc (A)")?;

    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// text
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn demo_text() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-text", Size::new(W, H))?;

    // ── text-anchor ───────────────────────────────────────────────────────────────
    // Vertical dashed guide at x=100; three labels anchored at that x position each
    // using a different TextAnchor variant so the visual spread makes the effect clear.
    let vguide = svg.line(Point::new(100.0, 35.0), Point::new(100.0, 138.0))?;
    vguide.set_stroke(GUIDE)?;
    vguide.set_attr("stroke-dasharray", "4 3")?;

    let ta_s = svg.text(Point::new(100.0, 58.0), "start")?;
    ta_s.set_fill(PLAIN_TEXT)?;
    ta_s.set_font_size(13.0)?;
    ta_s.set_text_anchor(TextAnchor::Start)?;

    let ta_m = svg.text(Point::new(100.0, 90.0), "middle")?;
    ta_m.set_fill(STEELBLUE)?;
    ta_m.set_font_size(13.0)?;
    ta_m.set_text_anchor(TextAnchor::Middle)?;

    let ta_e = svg.text(Point::new(100.0, 122.0), "end")?;
    ta_e.set_fill(CORAL)?;
    ta_e.set_font_size(13.0)?;
    ta_e.set_text_anchor(TextAnchor::End)?;

    caption(&svg, 100.0, "text-anchor")?;

    // ── font-size ─────────────────────────────────────────────────────────────────
    // Three labels at the same x, each rendered at a different font size.
    let fs_s = svg.text(Point::new(215.0, 60.0), "small — 11px")?;
    fs_s.set_fill(PLAIN_TEXT)?;
    fs_s.set_font_size(11.0)?;

    let fs_m = svg.text(Point::new(215.0, 92.0), "medium — 17px")?;
    fs_m.set_fill(PLAIN_TEXT)?;
    fs_m.set_font_size(17.0)?;

    let fs_l = svg.text(Point::new(215.0, 128.0), "large — 26px")?;
    fs_l.set_fill(PLAIN_TEXT)?;
    fs_l.set_font_size(26.0)?;

    caption(&svg, 300.0, "font-size")?;

    // ── dominant-baseline ─────────────────────────────────────────────────────────
    // Horizontal dashed guide at y=90; three labels share that y but use different
    // DominantBaseline values so the guide passes through a different part of each.
    let hguide = svg.line(Point::new(415.0, 90.0), Point::new(588.0, 90.0))?;
    hguide.set_stroke(GUIDE)?;
    hguide.set_attr("stroke-dasharray", "4 3")?;

    let db_h = svg.text(Point::new(432.0, 90.0), "hanging")?;
    db_h.set_fill(PLAIN_TEXT)?;
    db_h.set_font_size(13.0)?;
    db_h.set_text_anchor(TextAnchor::Middle)?;
    db_h.set_dominant_baseline(DominantBaseline::Hanging)?;

    let db_m = svg.text(Point::new(503.0, 90.0), "middle")?;
    db_m.set_fill(STEELBLUE)?;
    db_m.set_font_size(13.0)?;
    db_m.set_text_anchor(TextAnchor::Middle)?;
    db_m.set_dominant_baseline(DominantBaseline::Middle)?;

    let db_a = svg.text(Point::new(573.0, 90.0), "alphabetic")?;
    db_a.set_fill(CORAL)?;
    db_a.set_font_size(13.0)?;
    db_a.set_text_anchor(TextAnchor::Middle)?;
    db_a.set_dominant_baseline(DominantBaseline::Alphabetic)?;

    caption(&svg, 500.0, "dominant-baseline")?;

    // ── font-family ───────────────────────────────────────────────────────────────
    // The same word rendered in the three CSS generic font families.
    let ff_serif = svg.text(Point::new(618.0, 60.0), "Serif")?;
    ff_serif.set_fill(PLAIN_TEXT)?;
    ff_serif.set_font_size(16.0)?;
    ff_serif.set_font_family("serif")?;

    let ff_sans = svg.text(Point::new(618.0, 91.0), "Sans-serif")?;
    ff_sans.set_fill(STEELBLUE)?;
    ff_sans.set_font_size(16.0)?;
    ff_sans.set_font_family("sans-serif")?;

    let ff_mono = svg.text(Point::new(618.0, 124.0), "Monospace")?;
    ff_mono.set_fill(CORAL)?;
    ff_mono.set_font_size(16.0)?;
    ff_mono.set_font_family("monospace")?;

    caption(&svg, 700.0, "font-family")?;

    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// group (<g>)
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn demo_group() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-group", Size::new(W, H))?;

    // Group A — steelblue block, positioned with translate.
    //
    // `build_batch_into` creates the block and label straight inside the <g> via a detached fragment, so they never
    // touch the root and are not re-parented afterwards — unlike `svg.rect(...)` + `g.append(...)`, which would append
    // each child to the root first and then move it.
    let g1 = svg.group()?;
    svg.build_batch_into(&g1, |b| {
        let block = b.rect(Point::new(0.0, 0.0), Size::new(150.0, 80.0))?;
        block.set_fill(STEELBLUE)?;
        let label = b.text(Point::new(75.0, 47.0), "Group A")?;
        label.set_fill(WHITE)?;
        label.set_attrs([("font-size", "15"), ("text-anchor", "middle")])?;
        Ok(())
    })?;
    g1.set_attr("transform", &format!("translate(40, {})", 25.0 + PAD_Y))?;

    // Dashed connector
    let conn = svg.line(Point::new(190.0, 65.0 + PAD_Y), Point::new(280.0, 65.0 + PAD_Y))?;
    conn.set_stroke(GUIDE)?;
    conn.set_stroke_width(2.0)?;
    conn.set_attr("stroke-dasharray", "5 4")?;

    // Group B — darkorange block, different translate (built the same batched way)
    let g2 = svg.group()?;
    svg.build_batch_into(&g2, |b| {
        let block = b.rect(Point::new(0.0, 0.0), Size::new(150.0, 80.0))?;
        block.set_fill(DARK_ORANGE)?;
        let label = b.text(Point::new(75.0, 47.0), "Group B")?;
        label.set_fill(WHITE)?;
        label.set_attrs([("font-size", "15"), ("text-anchor", "middle")])?;
        Ok(())
    })?;
    g2.set_attr("transform", &format!("translate(280, {})", 25.0 + PAD_Y))?;

    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// AnimationLoop
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn demo_anim() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-anim", Size::new(W, H))?;

    // Pulsing circle — radius oscillates
    let pulse = svg.circle(Point::new(80.0, 55.0 + PAD_Y), 20.0)?;
    pulse.set_fill(MEDIUM_ORCHID)?;
    caption(&svg, 80.0, "radius pulse")?;

    // Travelling circle — cx oscillates horizontally
    let travel = svg.circle(Point::new(400.0, 55.0 + PAD_Y), 14.0)?;
    travel.set_fill(LIGHT_SKY_BLUE)?;
    caption(&svg, 400.0, "horizontal travel")?;

    // Hue-rotating rectangle
    let hue_rect = svg.rect(Point::new(600.0, 10.0 + PAD_Y), Size::new(185.0, 90.0))?;
    caption(&svg, 693.0, "hue rotation")?;

    let anim = AnimationLoop::start_with_frame(move |ts, frame| {
        // r: 10..48 at ~0.7 Hz
        let r = 10.0 + 38.0 * ((ts / 700.0).sin().abs());
        let _ = frame.set_attr_fmt(&pulse, "r", format_args!("{r:.1}"));

        // cx: 300..500 at ~0.6 Hz
        let cx = 400.0 + 100.0 * (ts / 1050.0).sin();
        let _ = frame.set_attr_fmt(&travel, "cx", format_args!("{cx:.1}"));

        // hue: full rotation every 9 s
        let hue = (ts / 25.0) % 360.0;
        let _ = frame.set_fill_fmt(&hue_rect, format_args!("hsl({hue:.0},70%,50%)"));
    })?;

    // The loop must outlive this function. Park it in thread-local state so it keeps running for the page's lifetime;
    // because `AnimationLoop` stops on `Drop`, this stays cancellable, unlike `mem::forget`.
    keep_demo_anim(anim);
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// defs / marker (arrowhead)
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn demo_marker() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-marker", Size::new(W, H))?;

    // Build a <defs> container with a named arrowhead <marker> inside it.
    // build_marker appends to <defs> only when the closure returns Ok, so a partially-built
    // marker is never visible in the DOM if construction fails partway through.
    let defs = svg.defs()?;
    let arrow = defs.build_marker("arrow", |m| {
        m.set_ref_x(10.0)?;
        m.set_ref_y(3.5)?;
        m.set_marker_width(10.0)?;
        m.set_marker_height(7.0)?;
        m.set_orient("auto")?;
        let head = m.polygon(&[Point::new(0.0, 0.0), Point::new(10.0, 3.5), Point::new(0.0, 7.0)])?;
        head.set_fill(ACCENT_BLUE)?;
        Ok(())
    })?;

    // Horizontal
    let l1 = svg.line(Point::new(20.0, 55.0 + PAD_Y), Point::new(240.0, 55.0 + PAD_Y))?;
    l1.set_stroke(ACCENT_BLUE)?;
    l1.set_stroke_width(2.0)?;
    l1.set_marker_end_ref(&arrow)?;
    caption(&svg, 130.0, "marker-end")?;

    // Diagonal — orient="auto" rotates the arrowhead to track the path tangent
    let l2 = svg.line(Point::new(280.0, 20.0 + PAD_Y), Point::new(490.0, 100.0 + PAD_Y))?;
    l2.set_stroke(ACCENT_BLUE)?;
    l2.set_stroke_width(2.0)?;
    l2.set_marker_end_ref(&arrow)?;
    caption(&svg, 385.0, r#"orient="auto""#)?;

    // Thick — same marker reused across all three lines
    let l3 = svg.line(Point::new(530.0, 55.0 + PAD_Y), Point::new(770.0, 55.0 + PAD_Y))?;
    l3.set_stroke(ACCENT_BLUE)?;
    l3.set_stroke_width(4.0)?;
    l3.set_marker_end_ref(&arrow)?;
    caption(&svg, 650.0, "set_marker_end_ref")?;

    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// use — stamp copies of a <defs> shape
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn demo_use() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-use", Size::new(W, H))?;

    // Define a diamond-shaped path once inside <defs>; it is not rendered until referenced.
    svg.build_defs(|d| {
        let gem = d.path("M 0,-28 L 22,0 L 0,28 L -22,0 Z")?;
        gem.set_attr("id", "gem")?;
        gem.set_fill(ACCENT_BLUE)?;
        gem.set_stroke(WHITE)?;
        gem.set_stroke_width(2.0)?;
        Ok(())
    })?;

    // Stamp five independent copies of the same path using <use>.
    // Positioning is done entirely through the transform attribute so that x and y stay at zero
    // and the rotation centres fall exactly on each copy's visual midpoint.
    let cy = PAD_Y + BAND / 2.0;
    let mut buf = String::new();
    for i in 0..5usize {
        let cx = 80.0 + i as f64 * 160.0;
        let angle = i as f64 * 18.0;
        let u = svg.use_node("#gem", Point::origin())?;
        u.set_transform_fmt(&mut buf, format_args!("translate({cx},{cy}) rotate({angle})"))?;
    }

    caption(&svg, W / 2.0, "one <defs> path stamped five times with <use>")?;
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// image — embed a raster or SVG image
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn demo_image() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-image", Size::new(W, H))?;

    // A 60×40 four-quadrant colour grid embedded as a base64 SVG data URI.
    // Base64 avoids having to percent-encode '<', '>' and '#' that appear in raw SVG data URIs.
    // The 3:2 source aspect ratio differs from the 1:1 display boxes below, making the three
    // preserveAspectRatio modes visually distinct.
    const SRC: &str = "data:image/svg+xml;base64,\
        PHN2ZyB4bWxucz0naHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmcnIHdpZHRoPSc2MCcgaGVpZ2h0\
        PSc0MCc+PHJlY3Qgd2lkdGg9JzMwJyBoZWlnaHQ9JzIwJyBmaWxsPSdzdGVlbGJsdWUnLz48cmVj\
        dCB4PSczMCcgd2lkdGg9JzMwJyBoZWlnaHQ9JzIwJyBmaWxsPSdjb3JhbCcvPjxyZWN0IHk9JzIw\
        JyB3aWR0aD0nMzAnIGhlaWdodD0nMjAnIGZpbGw9J2dvbGQnLz48cmVjdCB4PSczMCcgeT0nMjAn\
        IHdpZHRoPSczMCcgaGVpZ2h0PScyMCcgZmlsbD0nbWVkaXVtc2VhZ3JlZW4nLz48L3N2Zz4=";

    // Alternative image: slate-blue with a centred white circle (used in the set_href demo slot).
    const ALT: &str = "data:image/svg+xml;base64,\
        PHN2ZyB4bWxucz0naHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmcnIHdpZHRoPSc2MCcgaGVpZ2h0\
        PSc0MCc+PHJlY3Qgd2lkdGg9JzYwJyBoZWlnaHQ9JzQwJyBmaWxsPSdzbGF0ZWJsdWUnLz48Y2ly\
        Y2xlIGN4PSczMCcgY3k9JzIwJyByPScxNCcgZmlsbD0nd2hpdGUnIG9wYWNpdHk9Jy44NScvPjwv\
        c3ZnPg==";

    // 100×100 px square display boxes; the 3:2 source makes preserveAspectRatio effects clear.
    let img_w = 100.0_f64;
    let img_h = 100.0_f64;
    let y0 = PAD_Y + (BAND - img_h) / 2.0;
    let xs: [f64; 4] = [80.0, 250.0, 420.0, 590.0];

    // Thin guide outline showing each image's bounding box.
    let slot = |x: f64| -> Result<(), Error> {
        let r = svg.rect(Point::new(x, y0), Size::new(img_w, img_h))?;
        r.set_fill(NONE)?;
        r.set_stroke(GUIDE)?;
        r.set_stroke_width(1.0)?;
        Ok(())
    };

    // 1. xMidYMid meet (default) — fits the whole image inside the box, preserving the 3:2 ratio.
    //    Horizontal bars appear because the box is square and the source is wider than tall.
    slot(xs[0])?;
    let i1 = svg.image(SRC, Point::new(xs[0], y0), Size::new(img_w, img_h))?;
    i1.set_attr("preserveAspectRatio", "xMidYMid meet")?;
    caption(&svg, xs[0] + img_w / 2.0, "meet (default)")?;

    // 2. none — stretches to fill the exact box dimensions, squashing the 3:2 source into a square.
    slot(xs[1])?;
    let i2 = svg.image(SRC, Point::new(xs[1], y0), Size::new(img_w, img_h))?;
    i2.set_attr("preserveAspectRatio", "none")?;
    caption(&svg, xs[1] + img_w / 2.0, "none (stretch)")?;

    // 3. xMidYMid slice — scales up to fill the box and clips the sides.
    slot(xs[2])?;
    let i3 = svg.image(SRC, Point::new(xs[2], y0), Size::new(img_w, img_h))?;
    i3.set_attr("preserveAspectRatio", "xMidYMid slice")?;
    caption(&svg, xs[2] + img_w / 2.0, "slice (fill+clip)")?;

    // 4. set_href — the element is created with SRC, then the source is swapped to ALT after creation.
    slot(xs[3])?;
    let i4 = svg.image(SRC, Point::new(xs[3], y0), Size::new(img_w, img_h))?;
    i4.set_href(ALT)?;
    caption(&svg, xs[3] + img_w / 2.0, "set_href swap")?;

    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// linearGradient — horizontal, vertical, diagonal, multi-stop, and gradient stroke
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn demo_linear_gradient() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-linear-gradient", Size::new(W, H))?;

    // All gradient ids must be globally unique in the document, so use a per-demo prefix.
    let defs = svg.build_defs(|d| {
        // 1. Horizontal (default x1/y1/x2/y2): steelblue left → coral right.
        d.build_linear_gradient("demo-lg-h", |g| {
            g.add_stop(0.0, STEELBLUE)?;
            g.add_stop(1.0, CORAL)?;
            Ok(())
        })?;

        // 2. Vertical gradient: set x2=0, y2=1 to rotate the axis 90°.
        d.build_linear_gradient("demo-lg-v", |g| {
            g.add_stop(0.0, GOLDENROD)?;
            g.add_stop(1.0, "midnightblue")?;
            g.set_x2(0.0)?;
            g.set_y2(1.0)?;
            Ok(())
        })?;

        // 3. Diagonal: gradientTransform rotates the gradient vector 45°.
        //    Keeping the default horizontal endpoints and rotating is simpler than computing
        //    trigonometric endpoint coordinates by hand.
        d.build_linear_gradient("demo-lg-d", |g| {
            g.add_stop(0.0, TEAL)?;
            g.add_stop(1.0, MEDIUM_ORCHID)?;
            g.set_gradient_transform("rotate(45, 0.5, 0.5)")?;
            Ok(())
        })?;

        // 4. Multi-stop sunrise spectrum (4 stops).
        d.build_linear_gradient("demo-lg-s", |g| {
            g.add_stop(0.0, "#1a1a2e")?;
            g.add_stop(0.35, DARK_ORANGE)?;
            g.add_stop(0.65, GOLDENROD)?;
            g.add_stop(1.0, "#fffde7")?;
            Ok(())
        })?;

        // 5. Gradient stroke: a thin-to-thick colour sweep applied to stroke, not fill.
        d.build_linear_gradient("demo-lg-stroke", |g| {
            g.add_stop(0.0, MEDIUM_SEA_GREEN)?;
            g.add_stop(1.0, CORAL)?;
            Ok(())
        })?;

        Ok(())
    })?;

    // `defs` is used only to hold the gradients; so give it a "don't care" name to keep cargo happy
    let _ = defs;

    // Shape dimensions — centred vertically in the BAND.
    let rect_h = 90.0_f64;
    let ry = PAD_Y + (BAND - rect_h) / 2.0;
    let rect_w = 130.0_f64;
    let xs: [f64; 5] = [20.0, 175.0, 330.0, 485.0, 640.0];

    // 1. Horizontal gradient fill.
    let r1 = svg.rect(Point::new(xs[0], ry), Size::new(rect_w, rect_h))?;
    r1.set_fill_gradient("demo-lg-h")?;
    caption(&svg, xs[0] + rect_w / 2.0, "horizontal")?;

    // 2. Vertical gradient fill.
    let r2 = svg.rect(Point::new(xs[1], ry), Size::new(rect_w, rect_h))?;
    r2.set_fill_gradient("demo-lg-v")?;
    caption(&svg, xs[1] + rect_w / 2.0, "vertical")?;

    // 3. Diagonal gradient fill.
    let r3 = svg.rect(Point::new(xs[2], ry), Size::new(rect_w, rect_h))?;
    r3.set_fill_gradient("demo-lg-d")?;
    caption(&svg, xs[2] + rect_w / 2.0, "diagonal (rotate 45°)")?;

    // 4. Multi-stop sunrise spectrum.
    let r4 = svg.rect(Point::new(xs[3], ry), Size::new(rect_w, rect_h))?;
    r4.set_fill_gradient("demo-lg-s")?;
    caption(&svg, xs[3] + rect_w / 2.0, "4-stop spectrum")?;

    // 5. Thick stroked path with gradient stroke (fill=none).
    let stroke_y = PAD_Y + BAND / 2.0;
    let path_str = format!(
        "M {:.1} {:.1} C {:.1} {:.1} {:.1} {:.1} {:.1} {:.1}",
        xs[4],
        stroke_y + 35.0,
        xs[4] + 40.0,
        stroke_y - 45.0,
        xs[4] + 90.0,
        stroke_y + 45.0,
        xs[4] + rect_w,
        stroke_y - 35.0,
    );
    let stroke_path = svg.path(&path_str)?;
    stroke_path.set_fill("none")?;
    stroke_path.set_stroke_gradient("demo-lg-stroke")?;
    stroke_path.set_stroke_width(14.0)?;
    stroke_path.set_attr("stroke-linecap", "round")?;
    caption(&svg, xs[4] + rect_w / 2.0, "gradient stroke")?;

    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// radialGradient — centred, off-centre focal, spreadMethod:reflect, and ellipse fill
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn demo_radial_gradient() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-radial-gradient", Size::new(W, H))?;

    svg.build_defs(|d| {
        // 1. Centred glow: white hot core fading to deep transparent blue (default cx/cy/r = 0.5/0.5/0.5).
        d.build_radial_gradient("demo-rg-c", |g| {
            g.add_stop_opacity(0.0, "white", 1.0)?;
            g.add_stop_opacity(0.5, STEELBLUE, 0.9)?;
            g.add_stop_opacity(1.0, "#0d1b2a", 1.0)?;
            Ok(())
        })?;

        // 2. Off-centre focal point: the first-stop colour (gold) originates from the upper-left
        //    while the outer colour (deep red) fills the rest.
        d.build_radial_gradient("demo-rg-f", |g| {
            g.add_stop(0.0, GOLDENROD)?;
            g.add_stop(1.0, "#7b0000")?;
            g.set_fx(0.25)?;
            g.set_fy(0.25)?;
            Ok(())
        })?;

        // 3. spreadMethod=reflect with a tight radius (r=0.25) so the pattern tiles visibly.
        //    Two contrasting stops create concentric rings.
        d.build_radial_gradient("demo-rg-r", |g| {
            g.add_stop(0.0, LIGHT_SKY_BLUE)?;
            g.add_stop(1.0, "#00264d")?;
            g.set_r(0.25)?;
            g.set_spread_method(SpreadMethod::Reflect)?;
            Ok(())
        })?;

        // 4. Ellipse with a compressed-Y radial (gradient in objectBoundingBox so it follows the
        //    ellipse shape automatically).  Three stops give a metallic sheen.
        d.build_radial_gradient("demo-rg-e", |g| {
            g.add_stop(0.0, "white")?;
            g.add_stop(0.4, MEDIUM_SEA_GREEN)?;
            g.add_stop(1.0, "#003d1f")?;
            Ok(())
        })?;

        Ok(())
    })?;

    let mid_y = PAD_Y + BAND / 2.0;

    // 1. Centred radial on a circle.
    let c1 = svg.circle(Point::new(75.0, mid_y), 52.0)?;
    c1.set_fill_gradient("demo-rg-c")?;
    caption(&svg, 75.0, "centred")?;

    // 2. Off-centre focal on a rect.
    let r2 = svg.rect(Point::new(155.0, PAD_Y + 10.0), Size::new(155.0, BAND - 20.0))?;
    r2.set_fill_gradient("demo-rg-f")?;
    caption(&svg, 232.5, "off-centre focal")?;

    // 3. Reflected spread on a circle.
    let c3 = svg.circle(Point::new(425.0, mid_y), 52.0)?;
    c3.set_fill_gradient("demo-rg-r")?;
    caption(&svg, 425.0, "spreadMethod: reflect")?;

    // 4. Metallic sheen on an ellipse.
    let e4 = svg.ellipse(Point::new(640.0, mid_y), Size::new(130.0, 52.0))?;
    e4.set_fill_gradient("demo-rg-e")?;
    caption(&svg, 640.0, "ellipse metallic sheen")?;

    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// clipPath — clip a shape, polygon, and group to illustrate three clip-path use cases
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn demo_clip_path() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-clip-path", Size::new(W, H))?;

    // All clip-path and gradient ids must be globally unique in the document.
    svg.build_defs(|d| {
        // Horizontal linear gradient: steelblue → coral (section 1).
        d.build_linear_gradient("cp-grad-lin", |g| {
            g.add_stop(0.0, STEELBLUE)?;
            g.add_stop(1.0, CORAL)?;
            Ok(())
        })?;
        // Radial gradient: white centre → deep-navy edge (section 2).
        d.build_radial_gradient("cp-grad-rad", |g| {
            g.add_stop(0.0, "white")?;
            g.add_stop(1.0, "#1a237e")?;
            Ok(())
        })?;
        // Clip 1: circle centred at (130, 90).
        d.build_clip_path("cp-circle", |c| {
            c.circle(Point::new(130.0, 90.0), 53.0)?;
            Ok(())
        })?;
        // Clip 2: flat-top hexagon centred at (400, 90), circumradius 55.
        // Vertices: (cx + R·cos(k·60°), cy + R·sin(k·60°)) for k = 0..5.
        d.build_clip_path("cp-hex", |c| {
            c.polygon(&[
                Point::new(455.0, 90.0),
                Point::new(427.5, 137.6),
                Point::new(372.5, 137.6),
                Point::new(345.0, 90.0),
                Point::new(372.5, 42.4),
                Point::new(427.5, 42.4),
            ])?;
            Ok(())
        })?;
        // Clip 3: right-pointing arrow centred at (665, 90).
        // Rectangular body (595..645, y 70..110) plus a triangular head pointing at x = 735.
        d.build_clip_path("cp-arrow", |c| {
            c.path("M 595,70 L 645,70 L 645,50 L 735,90 L 645,130 L 645,110 L 595,110 Z")?;
            Ok(())
        })?;
        Ok(())
    })?;

    // Section 1: gradient rectangle revealed through a circular viewport.
    // The rect's bounding box (77,37 + 106×106) matches the circle's bounding box exactly,
    // so the gradient fills the entire circular aperture from edge to edge.
    let r1 = svg.rect(Point::new(77.0, 37.0), Size::new(106.0, 106.0))?;
    r1.set_fill_gradient("cp-grad-lin")?;
    r1.set_clip_path("cp-circle")?;
    caption(&svg, 130.0, "circle clip")?;

    // Section 2: gradient rectangle revealed through a hexagonal frame.
    let r2 = svg.rect(Point::new(345.0, 42.0), Size::new(110.0, 96.0))?;
    r2.set_fill_gradient("cp-grad-rad")?;
    r2.set_clip_path("cp-hex")?;
    caption(&svg, 400.0, "polygon clip (hexagon)")?;

    // Section 3: three coloured horizontal bands clipped to an arrow shape.
    // build_batch_into writes all three rects directly into the group in one DOM operation.
    let arrow_group = svg.group()?;
    svg.build_batch_into(&arrow_group, |b| {
        b.rect(Point::new(595.0, 50.0), Size::new(140.0, 27.0))?.set_fill(STEELBLUE)?;
        b.rect(Point::new(595.0, 77.0), Size::new(140.0, 26.0))?.set_fill(CORAL)?;
        b.rect(Point::new(595.0, 103.0), Size::new(140.0, 27.0))?.set_fill(GOLD)?;
        Ok(())
    })?;
    arrow_group.set_clip_path("cp-arrow")?;
    caption(&svg, 665.0, "path clip on a group")?;

    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// tspan — multi-line and inline mixed-style text
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn demo_tspan() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-tspan", Size::new(W, H))?;

    // ── multi-line (dy) ───────────────────────────────────────────────────────
    // A <text> with three <tspan> children, each advancing 22px down via dy.
    // The first tspan inherits x/y from the parent; subsequent ones use dy to
    // step to the next line without needing absolute y coordinates.
    const LINE_DY: f64 = 22.0;
    let ml = svg.text(Point::new(50.0, 50.0 + PAD_Y), "")?;
    ml.set_fill(PLAIN_TEXT)?;
    ml.set_font_size(15.0)?;

    ml.tspan("The quick brown fox")?;
    ml.tspan_dy(LINE_DY, "jumps over the")?;
    ml.tspan_dy(LINE_DY, "lazy dog.")?;

    caption(&svg, 200.0, "multi-line (tspan dy)")?;

    // ── inline mixed styles ───────────────────────────────────────────────────
    // A single <text> element whose <tspan> children each override fill and
    // font-size, producing a mixed-style run on one baseline.
    let mx = svg.text(Point::new(420.0, 90.0 + PAD_Y), "")?;

    let w1 = mx.tspan("small ")?;
    w1.set_fill(PLAIN_TEXT)?;
    w1.set_font_size(12.0)?;
    w1.set_dominant_baseline(DominantBaseline::Middle)?;

    let w2 = mx.tspan("MEDIUM ")?;
    w2.set_fill(STEELBLUE)?;
    w2.set_font_size(18.0)?;
    w2.set_dominant_baseline(DominantBaseline::Middle)?;

    let w3 = mx.tspan("LARGE")?;
    w3.set_fill(CORAL)?;
    w3.set_font_size(26.0)?;
    w3.set_dominant_baseline(DominantBaseline::Middle)?;

    caption(&svg, 600.0, "inline mixed styles (tspan)")?;

    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Events — click counter + reset button (two on_click handlers over shared state)
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
//
// Counting and resetting live on two *separate* buttons on purpose.  A "double-click to reset" on the counter itself
// would misbehave: the browser always fires two `click` events before a `dblclick`, so any quick pair of clicks would
// increment twice and then immediately reset to zero.
fn demo_events_click() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-events-click", Size::new(W, H))?;

    // Counter button.  Its colour cycles on every click so repeated presses are visible.
    let btn = svg.rect(Point::new(40.0, 30.0 + PAD_Y), Size::new(150.0, 60.0))?;
    btn.set_fill(STEELBLUE)?;
    btn.set_attrs([("rx", "8"), ("style", "cursor:pointer")])?;

    // The label sits on top of the button; `pointer-events:none` lets clicks fall through to the rect beneath.
    let btn_label = svg.text(Point::new(115.0, 66.0 + PAD_Y), "click me")?;
    btn_label.set_fill(WHITE)?;
    btn_label.set_attrs([("font-size", "16"), ("text-anchor", "middle"), ("style", "pointer-events:none")])?;

    // Reset button — greyed out until there is actually something to reset.
    let reset = svg.rect(Point::new(210.0, 30.0 + PAD_Y), Size::new(110.0, 60.0))?;
    reset.set_fill(RESET_IDLE)?;
    reset.set_attrs([("rx", "8"), ("style", "cursor:pointer")])?;

    let reset_label = svg.text(Point::new(265.0, 66.0 + PAD_Y), "reset")?;
    reset_label.set_fill(WHITE)?;
    reset_label.set_attrs([("font-size", "15"), ("text-anchor", "middle"), ("style", "pointer-events:none")])?;

    let readout = svg.text(Point::new(350.0, 66.0 + PAD_Y), "clicks: 0")?;
    readout.set_fill(TEXT)?;
    readout.set_attr("font-size", "15")?;

    let count = Rc::new(Cell::new(0u32));

    // Counter click → increment.  The closures also capture clones of other demo nodes (cross-captures, no cycle).
    // inc_btn is a self-capture of btn — a harmless cycle because keep_demo_node(btn) below already holds it alive.
    let inc_btn = btn.clone();
    let inc_reset = reset.clone();
    let inc_readout = readout.clone();
    let inc_count = count.clone();
    btn.on_click(move |_| {
        let n = inc_count.get() + 1;
        inc_count.set(n);
        let _ = inc_btn.set_fill(&format!("hsl({},60%,45%)", (n * 40) % 360));
        let _ = inc_reset.set_fill(TOMATO); // reset now has something to do
        inc_readout.set_text(&format!("clicks: {n}"));
    })?;

    // Reset click → zero the count and restore the resting colours.
    let rst_btn = btn.clone();
    let rst_reset = reset.clone();
    let rst_readout = readout.clone();
    let rst_count = count.clone();
    reset.on_click(move |_| {
        rst_count.set(0);
        let _ = rst_btn.set_fill(STEELBLUE);
        let _ = rst_reset.set_fill(RESET_IDLE);
        rst_readout.set_text("clicks: 0");
    })?;

    caption(&svg, 400.0, "two on_click handlers sharing one Rc<Cell> counter")?;
    keep_demo_node(btn);
    keep_demo_node(reset);
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Events — colour wheel (managed on_pointermove drives a second element's fill)
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
//
// A single transparent rect on *top* of everything captures the pointer, and every decoration below it carries
// `pointer-events:none`.  That keeps exactly one element under the pointer at all times, so the cursor never flickers
// as it moves (the earlier crosshair version flickered because the moving guides stole hover from the surface).
fn demo_events_colour() -> Result<(), Error> {
    const CX: f64 = 90.0; // wheel centre (horizontal)
    const CY: f64 = H / 2.0; // wheel centre (vertical) — middle of the canvas
    const R: f64 = H / 2.0 - 13.0; // wheel radius, scaled to the canvas height (keeps a ~13px margin)
    const STEP: f64 = 2.0; // angular width of each wedge, in degrees

    let svg = SvgRoot::create_in("demo-events-colour", Size::new(W, H))?;

    // The wheel is built from thin pie wedges, each filled with its own hue.  Grouping them lets a single
    // `pointer-events:none` on the <g> apply to every wedge at once.
    let wheel = svg.group()?;
    wheel.set_attr("pointer-events", NONE)?;

    // Build all ~180 wedges straight into the <g> through a detached fragment, committed in one DOM operation.
    // Creating each with `svg.path(...)` and then `wheel.append(...)` would instead append every wedge to the live
    // root and immediately move it into the group — a lot of avoidable setup-time DOM churn.
    svg.build_batch_into(&wheel, |b| {
        let mut a: f64 = 0.0;
        while a < 360.0 {
            let (r0, r1) = (a.to_radians(), (a + STEP).to_radians());
            let wedge = b.path(&format!(
                "M {CX} {CY} L {:.2} {:.2} A {R} {R} 0 0 1 {:.2} {:.2} Z",
                CX + R * r0.cos(),
                CY + R * r0.sin(),
                CX + R * r1.cos(),
                CY + R * r1.sin(),
            ))?;
            wedge.set_fill(&format!("hsl({:.0},90%,50%)", a + STEP / 2.0))?;
            a += STEP;
        }
        Ok(())
    })?;

    // A hollow ring that marks the sampled point on the wheel; parked off-canvas until the pointer arrives.
    let marker = svg.circle(Point::new(-20.0, -20.0), 6.0)?;
    marker.set_fill(NONE)?;
    marker.set_stroke(WHITE)?;
    marker.set_stroke_width(2.0)?;
    marker.set_attr("pointer-events", NONE)?;

    // The "second object": its fill follows whatever hue the pointer is over.
    let swatch = svg.rect(Point::new(210.0, 18.0 + PAD_Y), Size::new(250.0, 94.0))?;
    swatch.set_fill(SWATCH_EMPTY)?;
    swatch.set_stroke(GUIDE)?;
    swatch.set_attrs([("rx", "12"), ("pointer-events", NONE)])?;

    let readout = svg.text(Point::new(485.0, 70.0 + PAD_Y), "move over the wheel →")?;
    readout.set_fill(TEXT)?;
    readout.set_attrs([("font-size", "15"), ("pointer-events", NONE)])?;

    caption(
        &svg,
        450.0,
        "Managed on_pointermove over the wheel sets the swatch fill (hue from pointer angle)",
    )?;

    // The pointer-capture surface goes on last so it sits on top of everything above. `touch-action:none` lets a
    // finger-drag sample the wheel instead of scrolling the page, so the pointer handler works for touch and pen too.
    let surface = svg.rect(Point::origin(), Size::new(W, H))?;
    surface.set_fill(TRANSPARENT)?;
    surface.set_attr("style", "cursor:crosshair; touch-action:none")?;

    let mv_marker = marker.clone();
    let mv_swatch = swatch.clone();
    let mv_readout = readout.clone();

    // Managed handlers are `FnMut`, so this per-move handler can *own* its reusable buffers directly — no
    // `Rc<RefCell<...>>`, no runtime borrow on every `pointermove`. `SvgAttrs` formats the attributes; a scratch `String`
    // backs the readout text.
    let mut attrs = SvgAttrs::new();
    let mut text = String::new();
    // Avoids writing cx/cy on every outside-wheel event when the marker is already parked.
    let parked = Cell::new(false);

    surface.on_pointermove(move |e| {
        let (x, y) = (f64::from(e.offset_x()), f64::from(e.offset_y()));
        let (dx, dy) = (x - CX, y - CY);

        if dx * dx + dy * dy <= R * R {
            // Inside the wheel: hue is the pointer's angle about the centre.
            parked.set(false);
            let hue = (dy.atan2(dx).to_degrees() + 360.0) % 360.0;
            let _ = attrs.fmt(&mv_swatch, "fill", format_args!("hsl({hue:.0},90%,50%)"));
            let _ = attrs.fmt(&mv_marker, "cx", format_args!("{x:.1}"));
            let _ = attrs.fmt(&mv_marker, "cy", format_args!("{y:.1}"));
            let _ = mv_readout.set_text_fmt(&mut text, format_args!("hsl({hue:.0},90%,50%)"));
        } else if !parked.get() {
            // Outside the wheel: park the marker once; skip the DOM writes on subsequent outside events.
            parked.set(true);
            let _ = attrs.set(&mv_marker, "cx", "-20");
            let _ = attrs.set(&mv_marker, "cy", "-20");
        }
    })?;

    keep_demo_node(surface);
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Events — modifier keys (on_click) + right-click (on_contextmenu, preventDefault)
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn demo_events_modifiers() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-events-modifiers", Size::new(W, H))?;

    let pad = svg.rect(Point::new(40.0, 25.0 + PAD_Y), Size::new(240.0, 80.0))?;
    pad.set_fill(SLATE_BLUE)?;
    pad.set_attrs([("rx", "8"), ("style", "cursor:pointer")])?;

    let hint = svg.text(Point::new(160.0, 70.0 + PAD_Y), "click me")?;
    hint.set_fill(WHITE)?;
    hint.set_attrs([("font-size", "15"), ("text-anchor", "middle"), ("style", "pointer-events:none")])?;

    let readout = svg.text(Point::new(310.0, 70.0 + PAD_Y), "try: click · shift · ctrl · alt · right-click")?;
    readout.set_fill(TEXT)?;
    readout.set_attr("font-size", "14")?;

    // Left-click → inspect the modifier-key flags carried by the MouseEvent.
    let pad_click = pad.clone();
    let ro_click = readout.clone();
    pad.on_click(move |e| {
        let (label, colour) = if e.shift_key() {
            ("shift + click", TOMATO)
        } else if e.ctrl_key() {
            ("ctrl + click", MEDIUM_SEA_GREEN)
        } else if e.alt_key() {
            ("alt + click", GOLDENROD)
        } else if e.meta_key() {
            ("meta + click", ORCHID)
        } else {
            ("plain click", SLATE_BLUE)
        };
        let _ = pad_click.set_fill(colour);
        ro_click.set_text(&format!("last: {label}"));
    })?;

    // Right-click → suppress the browser context menu and report it.  ('click' never fires for the secondary button,
    // so the contextmenu event is the idiomatic hook for right-clicks.)
    let pad_ctx = pad.clone();
    let ro_ctx = readout.clone();
    pad.on_contextmenu(move |e| {
        e.prevent_default();
        let _ = pad_ctx.set_fill(CRIMSON);
        ro_ctx.set_text("last: right-click (context menu suppressed)");
    })?;

    caption(
        &svg,
        400.0,
        "on_click reads modifier keys · on_contextmenu calls preventDefault()",
    )?;
    keep_demo_node(pad);
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Events — press state (managed mousedown / mouseup / pointerleave)
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn demo_events_press() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-events-press", Size::new(W, H))?;

    let pad = svg.rect(Point::new(60.0, 25.0 + PAD_Y), Size::new(200.0, 80.0))?;
    pad.set_fill(TEAL)?;
    pad.set_attrs([("rx", "8"), ("style", "cursor:pointer")])?;

    let label = svg.text(Point::new(160.0, 70.0 + PAD_Y), "press & hold")?;
    label.set_fill(WHITE)?;
    label.set_attrs([("font-size", "15"), ("text-anchor", "middle"), ("style", "pointer-events:none")])?;

    let readout = svg.text(Point::new(320.0, 70.0 + PAD_Y), "state: idle")?;
    readout.set_fill(TEXT)?;
    readout.set_attr("font-size", "14")?;

    // Closures are `Clone` when everything they capture is `Clone` (SvgNode is), so we can build `press`/`release`
    // once and reuse them across several listeners.
    let press = {
        let pad = pad.clone();
        let readout = readout.clone();
        move |mods: &str| {
            let _ = pad.set_fill(TEAL_PRESSED); // darken while held
            let _ = pad.set_attr("transform", "translate(2,2)");
            readout.set_text(&format!("state: pressed{mods}"));
        }
    };
    let release = {
        let pad = pad.clone();
        let readout = readout.clone();
        move || {
            let _ = pad.set_fill(TEAL);
            let _ = pad.set_attr("transform", "translate(0,0)");
            readout.set_text("state: idle");
        }
    };

    // Only the primary button starts a press, so a plain right-click never engages one. That guard is not enough on
    // its own: on macOS a ctrl+click is reported as a *primary* mousedown (button 0) yet still opens the context
    // menu, which swallows the matching mouseup and would leave the state stuck on "pressed".
    // The `contextmenu` listener below is an OS-agnostic fix, which treats any context-menu trigger as a release.
    // The readout also lists any modifier keys held during the press.
    pad.on_mousedown(move |e| {
        if e.button() != 0 {
            return;
        }

        let mut held = Vec::new();
        if e.shift_key() {
            held.push("shift");
        }
        if e.ctrl_key() {
            held.push("ctrl");
        }
        if e.alt_key() {
            held.push("alt");
        }
        if e.meta_key() {
            held.push("meta");
        }
        let mods = if held.is_empty() {
            String::new()
        } else {
            format!("  ·  {}", held.join(" + "))
        };
        press(&mods);
    })?;

    let release_up = release.clone();
    pad.on_mouseup(move |_| release_up())?;

    // A context menu (right-click, or ctrl+click on macOS) interrupts the gesture consuming the mouseup event, so treat
    // it as a release so the button can never get stuck in the pressed state, whatever the platform.
    let release_ctx = release.clone();
    pad.on_contextmenu(move |_| release_ctx())?;

    // If the pointer leaves while still held, treat it as a release so the button cannot get stuck in the pressed state
    pad.on_pointerleave(move |_| release())?;

    caption(
        &svg,
        400.0,
        "managed mousedown / mouseup / pointerleave · pressed-state tracking · reports held modifier keys",
    )?;
    keep_demo_node(pad);
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Events — pointerenter wrappers on groups
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn demo_events_group() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-events-group", Size::new(W, H))?;

    // Builds a bordered container group holding two child shapes, translated to `x`.  The filled background rect is
    // the group's visible boundary AND its hit area: a <g> has no geometry of its own, so without a filled background
    // the gaps between the children would not count as "inside" the group.  Its fill matches the canvas colour so
    // only the coloured border shows.  Returns the group node.
    let build = |x: f64, border: &str| -> Result<SvgNode, Error> {
        let g = svg.group()?;

        // Build the boundary and children straight into the <g> via a detached fragment, instead of creating them on
        // the root and re-parenting each one with append.
        svg.build_batch_into(&g, |b| {
            let boundary = b.rect(Point::new(0.0, 0.0), Size::new(300.0, 80.0))?;
            boundary.set_fill(CANVAS_BG)?; // == canvas background, so only the stroke is visible
            boundary.set_stroke(border)?;
            boundary.set_stroke_width(2.0)?;
            boundary.set_attr("rx", "8")?;

            let child_a = b.circle(Point::new(75.0, 40.0), 22.0)?;
            child_a.set_fill(LEAF_ORANGE)?;

            let child_b = b.rect(Point::new(160.0, 18.0), Size::new(110.0, 44.0))?;
            child_b.set_fill(LEAF_GREEN)?;
            child_b.set_attr("rx", "4")?;
            Ok(())
        })?;
        g.set_attr("transform", &format!("translate({x}, {})", 26.0 + PAD_Y))?;
        Ok(g)
    };

    // Builds the title + counter labels for a group and returns the (clonable) counter text node.
    let labels = |x: f64, colour: &str, title: &str| -> Result<SvgNode, Error> {
        let t = svg.text(Point::new(x, 18.0 + PAD_Y), title)?;
        t.set_fill(colour)?;
        t.set_attr("font-size", "12")?;
        let count = svg.text(Point::new(x, 124.0 + PAD_Y), "fires: 0")?;
        count.set_fill(TEXT)?;
        count.set_attr("font-size", "14")?;
        Ok(count)
    };

    // group 1 — on_pointerenter. This uses the non-bubbling pointerenter event, so the handler fires once when the
    // pointer enters the group boundary and ignores child-to-child movement inside the group.
    let g1_count = labels(40.0, ACCENT_BLUE, "group 1: on_pointerenter")?;
    let group1 = build(40.0, ACCENT_BLUE)?;
    let c1 = Rc::new(Cell::new(0u32));
    group1.on_pointerenter(move |_| {
        let n = c1.get() + 1;
        c1.set(n);
        g1_count.set_text(&format!("fires: {n}"));
    })?;
    // Managed listeners are removed when their owning SvgNode is dropped, so keep this interactive node alive for the
    // page lifetime.
    keep_demo_node(group1);

    // group 2 — the same wrapper on another group, showing the behaviour is independent of the child shapes.
    let g2_count = labels(440.0, ACCENT_AMBER, "group 2: on_pointerenter")?;
    let group2 = build(440.0, ACCENT_AMBER)?;
    let c2 = Rc::new(Cell::new(0u32));
    group2.on_pointerenter(move |_| {
        let n = c2.get() + 1;
        c2.set(n);
        g2_count.set_text(&format!("fires: {n}"));
    })?;
    // Managed listeners are removed when their owning SvgNode is dropped, so keep this interactive node alive for the
    // page lifetime.
    keep_demo_node(group2);

    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Events — pointer and mouse lifecycle wrappers
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn demo_events_pointer_lifecycle() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-events-pointer", Size::new(W, H))?;

    let target = svg.rect(Point::new(54.0, 24.0 + PAD_Y), Size::new(230.0, 84.0))?;
    target.set_fill(DROP_ZONE_FILL)?;
    target.set_stroke(ACCENT_BLUE)?;
    target.set_stroke_width(2.0)?;
    target.set_attrs([("rx", "10"), ("style", "cursor:crosshair; touch-action:none")])?;

    let title = svg.text(Point::new(169.0, 58.0 + PAD_Y), "pointer target")?;
    title.set_fill(TEXT)?;
    title.set_attrs([
        ("font-size", "15"),
        ("font-weight", "bold"),
        ("text-anchor", "middle"),
        ("style", "pointer-events:none"),
    ])?;

    let readout = svg.text(Point::new(330.0, 58.0 + PAD_Y), "last: none")?;
    readout.set_fill(TEXT)?;
    readout.set_attr("font-size", "14")?;

    let coords = svg.text(Point::new(330.0, 84.0 + PAD_Y), "move inside the target")?;
    coords.set_fill(TEXT_MUTED)?;
    coords.set_attr("font-size", "12")?;

    // Every "last: ..." readout write goes through one shared CachedAttr: a burst of identical labels (a stream of
    // pointermove events) then skips the DOM write after the first. Routing *all* writers through the same cache is
    // what keeps it from going stale when the event type changes.
    let label_cache = Rc::new(RefCell::new(CachedAttr::new()));

    // Discrete transitions go through the shared cache via the module-level `cached_label` helper.
    target.on_pointerover(cached_label(readout.clone(), label_cache.clone(), "pointerover"))?;
    target.on_pointerenter(cached_label(readout.clone(), label_cache.clone(), "pointerenter"))?;
    target.on_pointerdown(cached_label(readout.clone(), label_cache.clone(), "pointerdown"))?;
    target.on_pointerup(cached_label(readout.clone(), label_cache.clone(), "pointerup"))?;
    target.on_pointercancel(cached_label(readout.clone(), label_cache.clone(), "pointercancel"))?;
    target.on_pointerout(cached_label(readout.clone(), label_cache.clone(), "pointerout"))?;
    target.on_pointerleave(cached_label(readout.clone(), label_cache.clone(), "pointerleave"))?;
    target.on_mouseenter(cached_label(readout.clone(), label_cache.clone(), "mouseenter"))?;
    target.on_mouseleave(cached_label(readout.clone(), label_cache.clone(), "mouseleave"))?;
    target.on_dblclick(cached_label(readout.clone(), label_cache.clone(), "dblclick"))?;

    let move_readout = readout.clone();
    let move_coords = coords.clone();
    // The `last: ...` readout is shared with the discrete handlers above, so its cache stays in an `Rc<RefCell<...>>`.
    // The coordinate buffer is used only here, so this `FnMut` handler can simply own it.
    let move_cache = label_cache.clone();
    let mut coords_buf = String::new();
    target.on_pointermove(move |e| {
        // Constant label through the shared cache: no allocation, and the DOM write is skipped on repeat moves.
        let _ = move_cache.borrow_mut().set_text(&move_readout, "last: pointermove");
        // Coordinates change every move: format them through the owned scratch buffer.
        let _ = move_coords.set_text_fmt(
            &mut coords_buf,
            format_args!(
                "x: {}  y: {}  id: {}  type: {}",
                e.offset_x(),
                e.offset_y(),
                e.pointer_id(),
                e.pointer_type(),
            ),
        );
    })?;

    caption(
        &svg,
        400.0,
        "managed pointerover/enter/down/move/up/cancel/out/leave plus mouseenter/leave/dblclick",
    )?;
    keep_demo_node(target);
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Events — keyboard, focus and wheel wrappers
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn demo_events_keyboard_wheel() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-events-keyboard-wheel", Size::new(W, H))?;

    let pad = svg.rect(Point::new(50.0, 24.0 + PAD_Y), Size::new(270.0, 84.0))?;
    pad.set_fill(SLATE_GRAY)?;
    pad.set_stroke(ACCENT_AMBER)?;
    pad.set_stroke_width(2.0)?;
    pad.set_attrs([("rx", "10"), ("tabindex", "0"), ("style", "cursor:pointer; outline:none")])?;

    let label = svg.text(Point::new(185.0, 58.0 + PAD_Y), "click, type, or wheel")?;
    label.set_fill(WHITE)?;
    label.set_attrs([
        ("font-size", "15"),
        ("font-weight", "bold"),
        ("text-anchor", "middle"),
        ("style", "pointer-events:none"),
    ])?;

    let readout = svg.text(Point::new(360.0, 58.0 + PAD_Y), "focus: no · key: — · wheel: 0")?;
    readout.set_fill(TEXT)?;
    readout.set_attr("font-size", "14")?;

    let wheel_total = Rc::new(Cell::new(0i32));

    {
        let readout = readout.clone();
        pad.on_focus(move |_| readout.set_text("focus: yes · key: — · wheel: 0"))?;
    }
    {
        let readout = readout.clone();
        pad.on_blur(move |_| readout.set_text("focus: no · key: — · wheel: 0"))?;
    }
    {
        let readout = readout.clone();
        let wheel_total = wheel_total.clone();
        pad.on_keydown(move |e| {
            readout.set_text(&format!("focus: yes · keydown: {} · wheel: {}", e.key(), wheel_total.get(),));
        })?;
    }
    {
        let readout = readout.clone();
        let wheel_total = wheel_total.clone();
        pad.on_keyup(move |e| {
            readout.set_text(&format!("focus: yes · keyup: {} · wheel: {}", e.key(), wheel_total.get(),));
        })?;
    }
    {
        let readout = readout.clone();
        let wheel_total = wheel_total.clone();
        // Wheel events fire rapidly during a continuous scroll/trackpad gesture, so the closure passed to `on_wheel`
        // will genuinely lie on the hot path.  Therefore, it is beneficial to capture a reusable buffer and format
        // into it rather than allocating a fresh String each tick.
        //
        // The discrete focus/blur/keydown/keyup handlers above deliberately keep the simpler `set_text(&format!(...))`
        // idiom since that coding does not lie on a hot path.
        //
        // Buffering is worthwhile for code lying on the hot path, but is not a blanket rule for every handler.
        let mut buf = String::new();
        pad.on_wheel(move |e| {
            e.prevent_default();
            let delta = if e.delta_y() < 0.0 { 1 } else { -1 };
            let next = wheel_total.get() + delta;
            wheel_total.set(next);
            buf.clear();
            let _ = write!(buf, "focus: yes · key: — · wheel: {next}");
            readout.set_text(&buf);
        })?;
    }

    caption(&svg, 400.0, "managed focus/blur · keydown/keyup · wheel with preventDefault()")?;
    keep_demo_node(pad);
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Events — browser drag/drop, touch and generic Event wrappers
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn demo_events_drag_drop_touch() -> Result<(), Error> {
    const CARD_W: f64 = 130.0;
    const CARD_H: f64 = 58.0;
    const MIN_X: f64 = 34.0;
    const MAX_X: f64 = 600.0;
    const MIN_Y: f64 = 24.0 + PAD_Y;
    const MAX_Y: f64 = 96.0 + PAD_Y;

    // Drop-zone bounds — shared by the rect that draws it and the drop test that decides whether the card stays put.
    const ZONE_X: f64 = 245.0;
    const ZONE_Y: f64 = 24.0 + PAD_Y;
    const ZONE_W: f64 = 220.0;
    const ZONE_H: f64 = 84.0;

    let svg = SvgRoot::create_in("demo-events-drag-drop-touch", Size::new(W, H))?;

    let zone = svg.rect(Point::new(ZONE_X, ZONE_Y), Size::new(ZONE_W, ZONE_H))?;
    zone.set_fill(DROP_ZONE_FILL)?;
    zone.set_stroke(DROP_ZONE_BORDER)?;
    zone.set_stroke_width(2.0)?;
    zone.set_attrs([("rx", "10"), ("stroke-dasharray", "6 4")])?;

    let zone_label = svg.text(Point::new(355.0, 72.0 + PAD_Y), "native drop zone")?;
    zone_label.set_fill(TEXT_MUTED)?;
    zone_label.set_attrs([
        ("font-size", "14"),
        ("text-anchor", "middle"),
        ("style", "pointer-events:none; user-select:none"),
    ])?;

    // The blue rectangle is the hit target for the drag gesture.  A <g> has no geometry of its own, so the group's
    // pointer listeners only fire when one of its children is hittable — hence the card background must NOT opt out of
    // pointer events.  Only the text label below carries `pointer-events:none`.
    let card = svg.group()?;
    svg.build_batch_into(&card, |b| {
        let card_bg = b.rect(Point::origin(), Size::new(CARD_W, CARD_H))?;
        card_bg.set_fill(ACCENT_BLUE)?;
        card_bg.set_attr("rx", "8")?;

        let card_label = b.text(Point::new(CARD_W / 2.0, CARD_H / 2.0 + 5.0), "drag / touch")?;
        card_label.set_fill(INK)?;
        card_label.set_attrs([
            ("font-size", "13"),
            ("font-weight", "bold"),
            ("text-anchor", "middle"),
            ("style", "pointer-events:none; user-select:none"),
        ])?;
        Ok(())
    })?;
    card.set_attrs([("style", "cursor:grab; touch-action:none; user-select:none")])?;

    let start = (50.0, 36.0 + PAD_Y);
    card.set_attr("transform", &format!("translate({:.1}, {:.1})", start.0, start.1))?;

    let readout = svg.text(Point::new(500.0, 48.0 + PAD_Y), "last: none")?;
    readout.set_fill(TEXT)?;
    readout.set_attr("font-size", "14")?;

    // Every "last: ..." readout write (the hot pointermove/touchmove/dragover streams *and* the discrete handlers
    // below) goes through this one shared CachedAttr as the pointer-lifecycle demo does.
    //
    // Repeated attempts to write the same label text do not touch the DOM after the first write. Routing *all* writers
    // through the same cache is what keeps it coherent; partial caching (some writers bypassing it) could lead to it
    // skipping a needed write.
    let label_cache = Rc::new(RefCell::new(CachedAttr::new()));

    let coords = svg.text(Point::new(500.0, 74.0 + PAD_Y), &format!("box: {:.0}, {:.0}", start.0, start.1))?;
    coords.set_fill(TEXT_MUTED)?;
    coords.set_attr("font-size", "12")?;

    let pos = Rc::new(Cell::new(start));
    let last_pointer: Rc<Cell<Option<(i32, i32)>>> = Rc::new(Cell::new(None));

    {
        let listener = card.clone();
        let card = card.clone();
        let readout = readout.clone();
        let label_cache = label_cache.clone();
        let last_pointer = last_pointer.clone();
        listener.on_pointerdown(move |e| {
            e.prevent_default();
            let _ = card.as_element().set_pointer_capture(e.pointer_id());
            last_pointer.set(Some((e.client_x(), e.client_y())));
            let _ = card.set_attr("style", "cursor:grabbing; touch-action:none; user-select:none");
            let _ = label_cache.borrow_mut().set_text(&readout, "last: pointerdown — moving box");
        })?;
    }

    {
        let listener = card.clone();
        let card = card.clone();
        let coords = coords.clone();
        let readout = readout.clone();
        let label_cache = label_cache.clone();
        let pos = pos.clone();
        let last_pointer = last_pointer.clone();
        let mut scratch = String::new();
        listener.on_pointermove(move |e| {
            if let Some((last_x, last_y)) = last_pointer.get() {
                e.prevent_default();
                let dx = f64::from(e.client_x() - last_x);
                let dy = f64::from(e.client_y() - last_y);
                let (x, y) = pos.get();
                let nx = (x + dx).clamp(MIN_X, MAX_X);
                let ny = (y + dy).clamp(MIN_Y, MAX_Y);
                pos.set((nx, ny));
                last_pointer.set(Some((e.client_x(), e.client_y())));
                let _ = card.set_translate(&mut scratch, nx, ny);
                let _ = coords.set_text_fmt(&mut scratch, format_args!("box: {nx:.0}, {ny:.0}"));
                // Constant label through the shared cache: the DOM write is skipped on repeated moves.
                let _ = label_cache.borrow_mut().set_text(&readout, "last: pointermove — moving box");
            }
        })?;
    }

    {
        let listener = card.clone();
        let card = card.clone();
        let readout = readout.clone();
        let label_cache = label_cache.clone();
        let coords = coords.clone();
        let pos = pos.clone();
        let last_pointer = last_pointer.clone();
        let mut scratch = String::new();
        let finish = move |e: web_sys::PointerEvent| {
            e.prevent_default();
            let _ = card.as_element().release_pointer_capture(e.pointer_id());
            last_pointer.set(None);
            let _ = card.set_attr("style", "cursor:grab; touch-action:none; user-select:none");

            // The card only counts as dropped if it is *fully* inside the zone; otherwise it snaps back to its
            // original position.
            let (x, y) = pos.get();
            let fully_inside =
                x >= ZONE_X && x + CARD_W <= ZONE_X + ZONE_W && y >= ZONE_Y && y + CARD_H <= ZONE_Y + ZONE_H;

            if fully_inside {
                let _ = label_cache.borrow_mut().set_text(&readout, "last: pointerup — dropped in zone");
            } else {
                pos.set(start);
                let _ = card.set_translate(&mut scratch, start.0, start.1);
                let _ = coords.set_text_fmt(&mut scratch, format_args!("box: {:.0}, {:.0}", start.0, start.1));
                let _ = label_cache
                    .borrow_mut()
                    .set_text(&readout, "last: pointerup — outside zone, returned to start");
            }
        };
        listener.on_pointerup(finish)?;
    }

    {
        let listener = card.clone();
        let card = card.clone();
        let readout = readout.clone();
        let label_cache = label_cache.clone();
        let last_pointer = last_pointer.clone();
        listener.on_pointercancel(move |e| {
            let _ = card.as_element().release_pointer_capture(e.pointer_id());
            last_pointer.set(None);
            let _ = card.set_attr("style", "cursor:grab; touch-action:none; user-select:none");
            let _ = label_cache.borrow_mut().set_text(&readout, "last: pointercancel");
        })?;
    }

    // The blue card is moved using pointer events because native browser drag/drop reports a DragEvent but does not
    // reposition SVG content for you.  These DragEvent wrappers are still attached so the demo logs any native drag
    // events a browser chooses to emit for the element.
    card.on_dragstart(cached_label(readout.clone(), label_cache.clone(), "dragstart"))?;
    card.on_drag(cached_label(readout.clone(), label_cache.clone(), "drag"))?;
    card.on_dragend(cached_label(readout.clone(), label_cache.clone(), "dragend"))?;
    {
        let readout = readout.clone();
        let label_cache = label_cache.clone();
        card.on_touchstart(move |e| {
            e.prevent_default();
            let _ = label_cache.borrow_mut().set_text(&readout, "last: touchstart");
        })?;
    }
    {
        let readout = readout.clone();
        let label_cache = label_cache.clone();
        card.on_touchmove(move |e| {
            e.prevent_default();
            let _ = label_cache.borrow_mut().set_text(&readout, "last: touchmove");
        })?;
    }
    card.on_touchend(cached_label(readout.clone(), label_cache.clone(), "touchend"))?;
    card.on_touchcancel(cached_label(readout.clone(), label_cache.clone(), "touchcancel"))?;

    zone.on_dragenter(cached_label(readout.clone(), label_cache.clone(), "dragenter"))?;
    zone.on_dragleave(cached_label(readout.clone(), label_cache.clone(), "dragleave"))?;
    {
        let readout = readout.clone();
        let label_cache = label_cache.clone();
        zone.on_dragover(move |e| {
            e.prevent_default();
            let _ = label_cache.borrow_mut().set_text(&readout, "last: dragover (drop enabled)");
        })?;
    }
    {
        let readout = readout.clone();
        let label_cache = label_cache.clone();
        zone.on_drop(move |e| {
            e.prevent_default();
            let _ = label_cache.borrow_mut().set_text(&readout, "last: drop");
        })?;
    }

    // Generic Event wrapper: auxclick is deliberately handled as a plain Event, proving that callers are not forced
    // back to raw Closure management when a typed convenience method is absent.
    card.on_event(
        "auxclick",
        cached_label(readout.clone(), label_cache.clone(), "generic auxclick"),
    )?;

    caption(
        &svg,
        400.0,
        "managed pointer drag moves the box · touch wrappers prevent scrolling · drag/drop wrappers are logged",
    )?;
    keep_demo_node(card);
    keep_demo_node(zone);
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Events — passive wheel, touchstart, and touchmove listeners
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn demo_events_passive() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-events-passive", Size::new(W, H))?;

    let box_y = PAD_Y + 5.0;
    let box_h = 100.0;
    let box_w = 360.0;
    let left_cx = 20.0 + box_w / 2.0; // 200
    let right_cx = 420.0 + box_w / 2.0; // 600

    // --- Passive wheel ---
    let wheel_area = svg.rect(Point::new(20.0, box_y), Size::new(box_w, box_h))?;
    wheel_area.set_fill(SLATE_BLUE)?;
    wheel_area.set_attrs([("rx", "8"), ("style", "cursor:ns-resize")])?;

    let wh_name = svg.text(Point::new(left_cx, box_y + 25.0), "on_wheel_passive")?;
    wh_name.set_fill(WHITE)?;
    wh_name.set_attrs([
        ("font-size", "14"),
        ("font-weight", "bold"),
        ("text-anchor", "middle"),
        ("style", "pointer-events:none"),
    ])?;

    let wh_hint = svg.text(Point::new(left_cx, box_y + 47.0), "wheel here — page scroll is not blocked")?;
    wh_hint.set_fill(WHITE)?;
    wh_hint.set_attrs([("font-size", "11"), ("text-anchor", "middle"), ("style", "pointer-events:none")])?;

    let wh_readout = svg.text(Point::new(left_cx, box_y + 74.0), "delta: 0")?;
    wh_readout.set_fill(WHITE)?;
    wh_readout.set_attrs([("font-size", "15"), ("text-anchor", "middle"), ("style", "pointer-events:none")])?;

    let wh_total = Rc::new(Cell::new(0i32));
    let wh_read = wh_readout.clone();
    let mut wh_buf = String::new();
    wheel_area.on_wheel_passive(move |e| {
        let n = wh_total.get() + if e.delta_y() < 0.0 { 1 } else { -1 };
        wh_total.set(n);
        wh_buf.clear();
        let _ = write!(wh_buf, "delta: {n}");
        wh_read.set_text(&wh_buf);
    })?;

    // --- Passive touch ---
    let touch_area = svg.rect(Point::new(420.0, box_y), Size::new(box_w, box_h))?;
    touch_area.set_fill(TEAL)?;
    touch_area.set_attrs([("rx", "8")])?;

    let tc_name1 = svg.text(Point::new(right_cx, box_y + 20.0), "on_touchstart_passive")?;
    tc_name1.set_fill(WHITE)?;
    tc_name1.set_attrs([
        ("font-size", "13"),
        ("font-weight", "bold"),
        ("text-anchor", "middle"),
        ("style", "pointer-events:none"),
    ])?;

    let tc_name2 = svg.text(Point::new(right_cx, box_y + 38.0), "on_touchmove_passive")?;
    tc_name2.set_fill(WHITE)?;
    tc_name2.set_attrs([
        ("font-size", "13"),
        ("font-weight", "bold"),
        ("text-anchor", "middle"),
        ("style", "pointer-events:none"),
    ])?;

    let tc_hint = svg.text(
        Point::new(right_cx, box_y + 58.0),
        "touch here (mobile) — scroll is not blocked",
    )?;
    tc_hint.set_fill(WHITE)?;
    tc_hint.set_attrs([("font-size", "11"), ("text-anchor", "middle"), ("style", "pointer-events:none")])?;

    let tc_readout = svg.text(Point::new(right_cx, box_y + 80.0), "last: none")?;
    tc_readout.set_fill(WHITE)?;
    tc_readout.set_attrs([("font-size", "15"), ("text-anchor", "middle"), ("style", "pointer-events:none")])?;

    let tc_cache = Rc::new(RefCell::new(CachedAttr::new()));
    touch_area.on_touchstart_passive(cached_label(tc_readout.clone(), tc_cache.clone(), "touchstart"))?;
    touch_area.on_touchmove_passive(cached_label(tc_readout, tc_cache, "touchmove"))?;

    caption(
        &svg,
        400.0,
        "passive listeners do not block browser scroll or touch · prevent_default() inside is silently ignored",
    )?;

    keep_demo_node(wheel_area);
    keep_demo_node(touch_area);
    Ok(())
}
