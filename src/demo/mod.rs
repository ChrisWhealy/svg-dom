//! Interactive element gallery for the browser.
//!
//! Build and serve with:
//! ```sh
//! cargo demo
//! ```
//! which rebuilds the wasm package and serves it via the Actix `demo-server` crate at <http://127.0.0.1:8000/demo/>.
//!
//! The `demo` feature excludes this code from the normal library build.

mod colours;
mod highlight;

use std::{
    cell::{Cell, RefCell},
    mem,
    rc::Rc,
};
use wasm_bindgen::prelude::*;

use crate::{
    AnimationLoop, Error, SvgAttrs, SvgNode, SvgRoot,
    root::utils::{Point, Size},
};
use colours::*;

thread_local! {
    /// Demo-only owner for interactive nodes whose managed listeners must remain attached after `run_demo` returns.
    ///
    /// The library intentionally removes a DOM listener when the last `SvgNode` handle is dropped.  Browser demos are
    /// long-lived, so they keep listener-owning nodes here instead of leaking individual `Closure`s.
    static LIVE_DEMO_NODES: RefCell<Vec<SvgNode>> = RefCell::new(Vec::new());
}

fn keep_demo_node(node: SvgNode) {
    LIVE_DEMO_NODES.with(|nodes| nodes.borrow_mut().push(node));
}

fn event_label<E>(node: SvgNode, name: &'static str) -> impl Fn(E) + 'static {
    move |_| node.set_text(&format!("last: {name}"))
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
    demo_line().map_err(e)?;
    demo_path().map_err(e)?;
    demo_text().map_err(e)?;
    demo_group().map_err(e)?;
    demo_anim().map_err(e)?;

    // Event-handling gallery
    demo_events_click().map_err(e)?;
    demo_events_colour().map_err(e)?;
    demo_events_modifiers().map_err(e)?;
    demo_events_press().map_err(e)?;
    demo_events_group().map_err(e)?;
    demo_events_pointer_lifecycle().map_err(e)?;
    demo_events_keyboard_wheel().map_err(e)?;
    demo_events_drag_drop_touch().map_err(e)?;

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
    ("panel-line", "demo_line"),
    ("panel-path", "demo_path"),
    ("panel-text", "demo_text"),
    ("panel-group", "demo_group"),
    ("panel-anim", "demo_anim"),
    ("panel-events-click", "demo_events_click"),
    ("panel-events-colour", "demo_events_colour"),
    ("panel-events-modifiers", "demo_events_modifiers"),
    ("panel-events-press", "demo_events_press"),
    ("panel-events-group", "demo_events_group"),
    ("panel-events-pointer", "demo_events_pointer_lifecycle"),
    ("panel-events-keyboard-wheel", "demo_events_keyboard_wheel"),
    ("panel-events-drag-drop-touch", "demo_events_drag_drop_touch"),
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

/// Builds `<details class="source"><summary>…</summary><pre><code>…</code></pre></details>` and appends it to the
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
    r2.attrs(&mut attrs)
        .fill(NONE)?
        .stroke(CORAL)?
        .stroke_width(3.0)?;
    caption(&svg, 220.0, "stroke")?;

    // 3. Rounded corners via rx attribute
    let r3 = svg.rect(Point::new(300.0, 10.0 + PAD_Y), Size::new(130.0, 90.0))?;
    r3.set_fill(MEDIUM_SEA_GREEN)?;
    r3.set_attr("rx", "20")?;
    caption(&svg, 365.0, "rounded (rx)")?;

    // 4. Hover: fill swaps on pointerenter / pointerleave
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
// line
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn demo_line() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-line", Size::new(W, H))?;

    // Horizontal
    let l1 = svg.line(
        Point::new(10.0, 55.0 + PAD_Y),
        Point::new(230.0, 55.0 + PAD_Y),
    )?;
    l1.set_stroke(WIRE)?;
    l1.set_stroke_width(2.0)?;
    caption(&svg, 120.0, "horizontal")?;

    // Diagonal
    let l2 = svg.line(
        Point::new(270.0, 10.0 + PAD_Y),
        Point::new(470.0, 110.0 + PAD_Y),
    )?;
    l2.set_stroke(CORAL)?;
    l2.set_stroke_width(2.0)?;
    caption(&svg, 370.0, "diagonal")?;

    // Thick
    let l3 = svg.line(
        Point::new(510.0, 55.0 + PAD_Y),
        Point::new(790.0, 55.0 + PAD_Y),
    )?;
    l3.set_stroke(GOLDENROD)?;
    l3.set_stroke_width(18.0)?;
    caption(&svg, 650.0, "thick stroke")?;

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

    // Small plain text
    let t1 = svg.text(Point::new(10.0, 60.0 + PAD_Y), "Plain text — 14px")?;
    t1.set_fill(PLAIN_TEXT)?;
    t1.set_attr("font-size", "14")?;

    // Large bold
    let t2 = svg.text(Point::new(10.0, 100.0 + PAD_Y), "Bold — 36px")?;
    let mut attrs = SvgAttrs::new();
    t2.attrs(&mut attrs)
        .fill(STEELBLUE)?
        .apply([("font-size", "36"), ("font-weight", "bold")])?;

    // Coloured, medium
    let t3 = svg.text(Point::new(430.0, 65.0 + PAD_Y), "Coloured — 22px")?;
    t3.set_fill(CORAL)?;
    t3.set_attr("font-size", "22")?;

    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// group (<g>)
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn demo_group() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-group", Size::new(W, H))?;

    // Group A — steelblue block, positioned with translate
    let g1 = svg.group()?;
    let b1 = svg.rect(Point::new(0.0, 0.0), Size::new(150.0, 80.0))?;
    b1.set_fill(STEELBLUE)?;
    let l1 = svg.text(Point::new(75.0, 47.0), "Group A")?;
    l1.set_fill(WHITE)?;
    l1.set_attrs([("font-size", "15"), ("text-anchor", "middle")])?;
    g1.append(&b1)?;
    g1.append(&l1)?;
    g1.set_attr("transform", &format!("translate(40, {})", 25.0 + PAD_Y))?;

    // Dashed connector
    let conn = svg.line(
        Point::new(190.0, 65.0 + PAD_Y),
        Point::new(280.0, 65.0 + PAD_Y),
    )?;
    conn.set_stroke(GUIDE)?;
    conn.set_stroke_width(2.0)?;
    conn.set_attr("stroke-dasharray", "5 4")?;

    // Group B — darkorange block, different translate
    let g2 = svg.group()?;
    let b2 = svg.rect(Point::new(0.0, 0.0), Size::new(150.0, 80.0))?;
    b2.set_fill(DARK_ORANGE)?;
    let l2 = svg.text(Point::new(75.0, 47.0), "Group B")?;
    l2.set_fill(WHITE)?;
    l2.set_attrs([("font-size", "15"), ("text-anchor", "middle")])?;
    g2.append(&b2)?;
    g2.append(&l2)?;
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

    // The loop must outlive this function — leak it for the page's lifetime.
    mem::forget(anim);
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
    btn_label.set_attrs([
        ("font-size", "16"),
        ("text-anchor", "middle"),
        ("style", "pointer-events:none"),
    ])?;

    // Reset button — greyed out until there is actually something to reset.
    let reset = svg.rect(Point::new(210.0, 30.0 + PAD_Y), Size::new(110.0, 60.0))?;
    reset.set_fill(RESET_IDLE)?;
    reset.set_attrs([("rx", "8"), ("style", "cursor:pointer")])?;

    let reset_label = svg.text(Point::new(265.0, 66.0 + PAD_Y), "reset")?;
    reset_label.set_fill(WHITE)?;
    reset_label.set_attrs([
        ("font-size", "15"),
        ("text-anchor", "middle"),
        ("style", "pointer-events:none"),
    ])?;

    let readout = svg.text(Point::new(350.0, 66.0 + PAD_Y), "clicks: 0")?;
    readout.set_fill(TEXT)?;
    readout.set_attr("font-size", "15")?;

    let count = Rc::new(Cell::new(0u32));

    // Counter click → increment.  Each closure captures a clone of its own node, which is what keeps that node (and
    // therefore its listener) alive after this function returns.
    let inc_btn = btn.clone();
    let inc_reset = reset.clone();
    let inc_readout = readout.clone();
    let inc_count = count.clone();
    btn.on_click(move |_| {
        let n = inc_count.get() + 1;
        inc_count.set(n);
        let _ = inc_btn.set_fill(&format!("hsl({},60%,45%)", (n * 40) % 360));
        let _ = inc_reset.set_fill(TOMATO); // reset now has something to do
        inc_readout
            .as_element()
            .set_text_content(Some(&format!("clicks: {n}")));
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
        rst_readout.as_element().set_text_content(Some("clicks: 0"));
    })?;

    caption(
        &svg,
        400.0,
        "two on_click handlers sharing one Rc<Cell> counter",
    )?;
    keep_demo_node(btn);
    keep_demo_node(reset);
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Events — colour wheel (managed on_mousemove drives a second element's fill)
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
    let mut a: f64 = 0.0;

    while a < 360.0 {
        let (r0, r1) = (a.to_radians(), (a + STEP).to_radians());
        let wedge = svg.path(&format!(
            "M {CX} {CY} L {:.2} {:.2} A {R} {R} 0 0 1 {:.2} {:.2} Z",
            CX + R * r0.cos(),
            CY + R * r0.sin(),
            CX + R * r1.cos(),
            CY + R * r1.sin(),
        ))?;
        wedge.set_fill(&format!("hsl({:.0},90%,50%)", a + STEP / 2.0))?;
        wheel.append(&wedge)?;
        a += STEP;
    }

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
        "Managed on_mousemove over the wheel sets the swatch fill (hue from pointer angle)",
    )?;

    // The pointer-capture surface goes on last so it sits on top of everything above.
    let surface = svg.rect(Point::origin(), Size::new(W, H))?;
    surface.set_fill(TRANSPARENT)?;
    surface.set_attr("style", "cursor:crosshair")?;

    let mv_marker = marker.clone();
    let mv_swatch = swatch.clone();
    let mv_readout = readout.clone();

    surface.on_mousemove(move |e| {
        let (x, y) = (f64::from(e.offset_x()), f64::from(e.offset_y()));
        let (dx, dy) = (x - CX, y - CY);

        if dx * dx + dy * dy <= R * R {
            // Inside the wheel: hue is the pointer's angle about the centre.
            let hue = (dy.atan2(dx).to_degrees() + 360.0) % 360.0;
            let colour = format!("hsl({hue:.0},90%,50%)");
            let _ = mv_swatch.set_fill(&colour);
            let _ = mv_marker.set_attrs([
                ("cx", format!("{x:.1}")),
                ("cy", format!("{y:.1}")),
            ]);
            mv_readout.as_element().set_text_content(Some(&colour));
        } else {
            // Outside the wheel: park the marker but leave the last sampled colour on the swatch.
            let _ = mv_marker.set_attrs([("cx", "-20"), ("cy", "-20")]);
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
    hint.set_attrs([
        ("font-size", "15"),
        ("text-anchor", "middle"),
        ("style", "pointer-events:none"),
    ])?;

    let readout = svg.text(
        Point::new(310.0, 70.0 + PAD_Y),
        "try: click · shift · ctrl · alt · right-click",
    )?;
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
        ro_click
            .as_element()
            .set_text_content(Some(&format!("last: {label}")));
    })?;

    // Right-click → suppress the browser context menu and report it.  ('click' never fires for the secondary button,
    // so the contextmenu event is the idiomatic hook for right-clicks.)
    let pad_ctx = pad.clone();
    let ro_ctx = readout.clone();
    pad.on_contextmenu(move |e| {
        e.prevent_default();
        let _ = pad_ctx.set_fill(CRIMSON);
        ro_ctx
            .as_element()
            .set_text_content(Some("last: right-click (context menu suppressed)"));
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
    label.set_attrs([
        ("font-size", "15"),
        ("text-anchor", "middle"),
        ("style", "pointer-events:none"),
    ])?;

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
            readout
                .as_element()
                .set_text_content(Some(&format!("state: pressed{mods}")));
        }
    };
    let release = {
        let pad = pad.clone();
        let readout = readout.clone();
        move || {
            let _ = pad.set_fill(TEAL);
            let _ = pad.set_attr("transform", "translate(0,0)");
            readout.as_element().set_text_content(Some("state: idle"));
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

        let boundary = svg.rect(Point::new(0.0, 0.0), Size::new(300.0, 80.0))?;
        boundary.set_fill(CANVAS_BG)?; // == canvas background, so only the stroke is visible
        boundary.set_stroke(border)?;
        boundary.set_stroke_width(2.0)?;
        boundary.set_attr("rx", "8")?;

        let child_a = svg.circle(Point::new(75.0, 40.0), 22.0)?;
        child_a.set_fill(LEAF_ORANGE)?;
        let child_b = svg.rect(Point::new(160.0, 18.0), Size::new(110.0, 44.0))?;
        child_b.set_fill(LEAF_GREEN)?;
        child_b.set_attr("rx", "4")?;

        g.append(&boundary)?;
        g.append(&child_a)?;
        g.append(&child_b)?;
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
        g1_count
            .as_element()
            .set_text_content(Some(&format!("fires: {n}")));
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
        g2_count
            .as_element()
            .set_text_content(Some(&format!("fires: {n}")));
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

    target.on_pointerover(event_label(readout.clone(), "pointerover"))?;
    target.on_pointerenter(event_label(readout.clone(), "pointerenter"))?;
    target.on_pointerdown(event_label(readout.clone(), "pointerdown"))?;
    target.on_pointerup(event_label(readout.clone(), "pointerup"))?;
    target.on_pointercancel(event_label(readout.clone(), "pointercancel"))?;
    target.on_pointerout(event_label(readout.clone(), "pointerout"))?;
    target.on_pointerleave(event_label(readout.clone(), "pointerleave"))?;
    target.on_mouseenter(event_label(readout.clone(), "mouseenter"))?;
    target.on_mouseleave(event_label(readout.clone(), "mouseleave"))?;
    target.on_dblclick(event_label(readout.clone(), "dblclick"))?;

    let move_readout = readout.clone();
    let move_coords = coords.clone();
    target.on_pointermove(move |e| {
        move_readout.set_text("last: pointermove");
        move_coords.set_text(&format!(
            "x: {}  y: {}  id: {}  type: {}",
            e.offset_x(),
            e.offset_y(),
            e.pointer_id(),
            e.pointer_type(),
        ));
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
    pad.set_attrs([
        ("rx", "10"),
        ("tabindex", "0"),
        ("style", "cursor:pointer; outline:none"),
    ])?;

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
            readout.set_text(&format!(
                "focus: yes · keydown: {} · wheel: {}",
                e.key(),
                wheel_total.get(),
            ));
        })?;
    }
    {
        let readout = readout.clone();
        let wheel_total = wheel_total.clone();
        pad.on_keyup(move |e| {
            readout.set_text(&format!(
                "focus: yes · keyup: {} · wheel: {}",
                e.key(),
                wheel_total.get(),
            ));
        })?;
    }
    {
        let readout = readout.clone();
        let wheel_total = wheel_total.clone();
        pad.on_wheel(move |e| {
            e.prevent_default();
            let delta = if e.delta_y() < 0.0 { 1 } else { -1 };
            let next = wheel_total.get() + delta;
            wheel_total.set(next);
            readout.set_text(&format!("focus: yes · key: — · wheel: {next}"));
        })?;
    }

    caption(
        &svg,
        400.0,
        "managed focus/blur · keydown/keyup · wheel with preventDefault()",
    )?;
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
    let card_bg = svg.rect(Point::origin(), Size::new(CARD_W, CARD_H))?;
    card_bg.set_fill(ACCENT_BLUE)?;
    card_bg.set_attr("rx", "8")?;

    let card_label = svg.text(Point::new(CARD_W / 2.0, CARD_H / 2.0 + 5.0), "drag / touch")?;
    card_label.set_fill(INK)?;
    card_label.set_attrs([
        ("font-size", "13"),
        ("font-weight", "bold"),
        ("text-anchor", "middle"),
        ("style", "pointer-events:none; user-select:none"),
    ])?;

    let card = svg.group()?;
    card.append(&card_bg)?;
    card.append(&card_label)?;
    card.set_attrs([("style", "cursor:grab; touch-action:none; user-select:none")])?;

    let start = (50.0, 36.0 + PAD_Y);
    card.set_attr(
        "transform",
        &format!("translate({:.1}, {:.1})", start.0, start.1),
    )?;

    let readout = svg.text(Point::new(500.0, 48.0 + PAD_Y), "last: none")?;
    readout.set_fill(TEXT)?;
    readout.set_attr("font-size", "14")?;

    let coords = svg.text(
        Point::new(500.0, 74.0 + PAD_Y),
        &format!("box: {:.0}, {:.0}", start.0, start.1),
    )?;
    coords.set_fill(TEXT_MUTED)?;
    coords.set_attr("font-size", "12")?;

    let pos = Rc::new(Cell::new(start));
    let last_pointer: Rc<Cell<Option<(i32, i32)>>> = Rc::new(Cell::new(None));

    // One scratch buffer reused across every pointermove and the snap-back on pointerup — for both the card's transform
    // and the coordinate readout text — so dragging the card allocates no fresh String per event. It lives outside the
    // closures and is shared by clone.
    let scratch = Rc::new(RefCell::new(String::new()));

    {
        let listener = card.clone();
        let card = card.clone();
        let readout = readout.clone();
        let last_pointer = last_pointer.clone();
        listener.on_pointerdown(move |e| {
            e.prevent_default();
            let _ = card.as_element().set_pointer_capture(e.pointer_id());
            last_pointer.set(Some((e.client_x(), e.client_y())));
            let _ = card.set_attr("style", "cursor:grabbing; touch-action:none; user-select:none");
            readout.set_text("last: pointerdown — moving box");
        })?;
    }

    {
        let listener = card.clone();
        let card = card.clone();
        let coords = coords.clone();
        let readout = readout.clone();
        let pos = pos.clone();
        let last_pointer = last_pointer.clone();
        let scratch = scratch.clone();
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
                let _ = card.set_translate(&mut scratch.borrow_mut(), nx, ny);
                let _ = coords.set_text_fmt(&mut scratch.borrow_mut(), format_args!("box: {nx:.0}, {ny:.0}"));
                readout.set_text("last: pointermove — moving box");
            }
        })?;
    }

    {
        let listener = card.clone();
        let card = card.clone();
        let readout = readout.clone();
        let coords = coords.clone();
        let pos = pos.clone();
        let last_pointer = last_pointer.clone();
        let scratch = scratch.clone();
        let finish = move |e: web_sys::PointerEvent| {
            e.prevent_default();
            let _ = card.as_element().release_pointer_capture(e.pointer_id());
            last_pointer.set(None);
            let _ = card.set_attr("style", "cursor:grab; touch-action:none; user-select:none");

            // The card only counts as dropped if it is *fully* inside the zone; otherwise it snaps back to its
            // original position.
            let (x, y) = pos.get();
            let fully_inside = x >= ZONE_X
                && x + CARD_W <= ZONE_X + ZONE_W
                && y >= ZONE_Y
                && y + CARD_H <= ZONE_Y + ZONE_H;

            if fully_inside {
                readout.set_text("last: pointerup — dropped in zone");
            } else {
                pos.set(start);
                let _ = card.set_translate(&mut scratch.borrow_mut(), start.0, start.1);
                let _ = coords.set_text_fmt(&mut scratch.borrow_mut(), format_args!("box: {:.0}, {:.0}", start.0, start.1));
                readout.set_text("last: pointerup — outside zone, returned to start");
            }
        };
        listener.on_pointerup(finish)?;
    }

    {
        let listener = card.clone();
        let card = card.clone();
        let readout = readout.clone();
        let last_pointer = last_pointer.clone();
        listener.on_pointercancel(move |e| {
            let _ = card.as_element().release_pointer_capture(e.pointer_id());
            last_pointer.set(None);
            let _ = card.set_attr("style", "cursor:grab; touch-action:none; user-select:none");
            readout.set_text("last: pointercancel");
        })?;
    }

    // The blue card is moved using pointer events because native browser drag/drop reports a DragEvent but does not
    // reposition SVG content for you.  These DragEvent wrappers are still attached so the demo logs any native drag
    // events a browser chooses to emit for the element.
    card.on_dragstart(event_label(readout.clone(), "dragstart"))?;
    card.on_drag(event_label(readout.clone(), "drag"))?;
    card.on_dragend(event_label(readout.clone(), "dragend"))?;
    {
        let readout = readout.clone();
        card.on_touchstart(move |e| {
            e.prevent_default();
            readout.set_text("last: touchstart");
        })?;
    }
    {
        let readout = readout.clone();
        card.on_touchmove(move |e| {
            e.prevent_default();
            readout.set_text("last: touchmove");
        })?;
    }
    card.on_touchend(event_label(readout.clone(), "touchend"))?;
    card.on_touchcancel(event_label(readout.clone(), "touchcancel"))?;

    zone.on_dragenter(event_label(readout.clone(), "dragenter"))?;
    zone.on_dragleave(event_label(readout.clone(), "dragleave"))?;
    {
        let readout = readout.clone();
        zone.on_dragover(move |e| {
            e.prevent_default();
            readout.set_text("last: dragover (drop enabled)");
        })?;
    }
    {
        let readout = readout.clone();
        zone.on_drop(move |e| {
            e.prevent_default();
            readout.set_text("last: drop");
        })?;
    }

    // Generic Event wrapper: auxclick is deliberately handled as a plain Event, proving that callers are not forced
    // back to raw Closure management when a typed convenience method is absent.
    card.on_event("auxclick", event_label(readout.clone(), "generic auxclick"))?;

    caption(
        &svg,
        400.0,
        "managed pointer drag moves the box · touch wrappers prevent scrolling · drag/drop wrappers are logged",
    )?;
    keep_demo_node(card);
    keep_demo_node(zone);
    Ok(())
}
