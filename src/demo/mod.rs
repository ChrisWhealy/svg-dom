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

use std::{cell::Cell, mem, rc::Rc};
use wasm_bindgen::{JsCast, prelude::*};
use web_sys::MouseEvent;

use crate::{
    AnimationLoop, Error, SvgNode, SvgRoot,
    root::utils::{Point, Size},
};
use colours::*;

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
    demo_events_drag().map_err(e)?;
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Shared helper
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Appends a small grey caption below an element at horizontal centre `cx`.
fn caption(svg: &SvgRoot, cx: f64, text: &str) -> Result<(), Error> {
    let t = svg.text(Point::new(cx, PAD_Y + BAND - 6.0), text)?;
    t.set_fill(CAPTION)?;
    t.set_attr("font-size", "11")?;
    t.set_attr("text-anchor", "middle")?;
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
    r2.set_fill(NONE)?;
    r2.set_stroke(CORAL)?;
    r2.set_stroke_width(3.0)?;
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
    tri.set_fill(STEELBLUE)?;
    tri.set_stroke(WHITE)?;
    tri.set_stroke_width(2.0)?;
    tri.set_attr("transform", &shift)?;
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
    t2.set_fill(STEELBLUE)?;
    t2.set_attr("font-size", "36")?;
    t2.set_attr("font-weight", "bold")?;

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
    l1.set_attr("font-size", "15")?;
    l1.set_attr("text-anchor", "middle")?;
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
    l2.set_attr("font-size", "15")?;
    l2.set_attr("text-anchor", "middle")?;
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

    let anim = AnimationLoop::start(move |ts| {
        // r: 10..48 at ~0.7 Hz
        let r = 10.0 + 38.0 * ((ts / 700.0).sin().abs());
        let _ = pulse.set_attr("r", &format!("{r:.1}"));

        // cx: 300..500 at ~0.6 Hz
        let cx = 400.0 + 100.0 * (ts / 1050.0).sin();
        let _ = travel.set_attr("cx", &format!("{cx:.1}"));

        // hue: full rotation every 9 s
        let hue = (ts / 25.0) % 360.0;
        let _ = hue_rect.set_fill(&format!("hsl({hue:.0},70%,50%)"));
    })?;

    // The loop must outlive this function — leak it for the page's lifetime.
    mem::forget(anim);
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Event-handling helper
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

// `SvgNode` wraps `click`, `pointerenter` and `pointerleave` directly.  For every other event we drop down to the raw
// `web-sys` element via [`SvgNode::as_element`] and register the listener ourselves.
// The `Closure` is `forget`-ted so that it lives for the page's lifetime — exactly the same leak-on-purpose pattern
// that `demo_anim` uses for its `AnimationLoop`.
//
// However, in a real application, you would store the `Closure` somewhere with a defined lifetime.
fn on_raw<F: Fn(MouseEvent) + 'static>(
    node: &SvgNode,
    event: &str,
    handler: F,
) -> Result<(), Error> {
    let closure = Closure::<dyn Fn(MouseEvent)>::new(handler);
    node.as_element()
        .add_event_listener_with_callback(event, closure.as_ref().unchecked_ref())
        .map_err(|e| Error::Dom(format!("{e:?}")))?;
    closure.forget();
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
    btn.set_attr("rx", "8")?;
    btn.set_attr("style", "cursor:pointer")?;

    // The label sits on top of the button; `pointer-events:none` lets clicks fall through to the rect beneath.
    let btn_label = svg.text(Point::new(115.0, 66.0 + PAD_Y), "click me")?;
    btn_label.set_fill(WHITE)?;
    btn_label.set_attr("font-size", "16")?;
    btn_label.set_attr("text-anchor", "middle")?;
    btn_label.set_attr("style", "pointer-events:none")?;

    // Reset button — greyed out until there is actually something to reset.
    let reset = svg.rect(Point::new(210.0, 30.0 + PAD_Y), Size::new(110.0, 60.0))?;
    reset.set_fill(RESET_IDLE)?;
    reset.set_attr("rx", "8")?;
    reset.set_attr("style", "cursor:pointer")?;

    let reset_label = svg.text(Point::new(265.0, 66.0 + PAD_Y), "reset")?;
    reset_label.set_fill(WHITE)?;
    reset_label.set_attr("font-size", "15")?;
    reset_label.set_attr("text-anchor", "middle")?;
    reset_label.set_attr("style", "pointer-events:none")?;

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
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Events — colour wheel (raw mousemove drives a second element's fill)
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
    swatch.set_attr("rx", "12")?;
    swatch.set_attr("pointer-events", NONE)?;

    let readout = svg.text(Point::new(485.0, 70.0 + PAD_Y), "move over the wheel →")?;
    readout.set_fill(TEXT)?;
    readout.set_attr("font-size", "15")?;
    readout.set_attr("pointer-events", NONE)?;

    caption(
        &svg,
        450.0,
        "Raw mousemove over the wheel sets the swatch fill (hue from pointer angle)",
    )?;

    // The pointer-capture surface goes on last so it sits on top of everything above.
    let surface = svg.rect(Point::origin(), Size::new(W, H))?;
    surface.set_fill(TRANSPARENT)?;
    surface.set_attr("style", "cursor:crosshair")?;

    let mv_marker = marker.clone();
    let mv_swatch = swatch.clone();
    let mv_readout = readout.clone();

    on_raw(&surface, "mousemove", move |e| {
        let (x, y) = (f64::from(e.offset_x()), f64::from(e.offset_y()));
        let (dx, dy) = (x - CX, y - CY);

        if dx * dx + dy * dy <= R * R {
            // Inside the wheel: hue is the pointer's angle about the centre.
            let hue = (dy.atan2(dx).to_degrees() + 360.0) % 360.0;
            let colour = format!("hsl({hue:.0},90%,50%)");
            let _ = mv_swatch.set_fill(&colour);
            let _ = mv_marker.set_attr("cx", &format!("{x:.1}"));
            let _ = mv_marker.set_attr("cy", &format!("{y:.1}"));
            mv_readout.as_element().set_text_content(Some(&colour));
        } else {
            // Outside the wheel: park the marker but leave the last sampled colour on the swatch.
            let _ = mv_marker.set_attr("cx", "-20");
            let _ = mv_marker.set_attr("cy", "-20");
        }
    })?;

    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Events — modifier keys (on_click) + right-click (raw contextmenu, preventDefault)
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn demo_events_modifiers() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-events-modifiers", Size::new(W, H))?;

    let pad = svg.rect(Point::new(40.0, 25.0 + PAD_Y), Size::new(240.0, 80.0))?;
    pad.set_fill(SLATE_BLUE)?;
    pad.set_attr("rx", "8")?;
    pad.set_attr("style", "cursor:pointer")?;

    let hint = svg.text(Point::new(160.0, 70.0 + PAD_Y), "click me")?;
    hint.set_fill(WHITE)?;
    hint.set_attr("font-size", "15")?;
    hint.set_attr("text-anchor", "middle")?;
    hint.set_attr("style", "pointer-events:none")?;

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
    on_raw(&pad, "contextmenu", move |e| {
        e.prevent_default();
        let _ = pad_ctx.set_fill(CRIMSON);
        ro_ctx
            .as_element()
            .set_text_content(Some("last: right-click (context menu suppressed)"));
    })?;

    caption(
        &svg,
        400.0,
        "on_click reads modifier keys · raw contextmenu calls preventDefault()",
    )?;
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Events — press state (raw mousedown / mouseup / pointerleave)
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn demo_events_press() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-events-press", Size::new(W, H))?;

    let pad = svg.rect(Point::new(60.0, 25.0 + PAD_Y), Size::new(200.0, 80.0))?;
    pad.set_fill(TEAL)?;
    pad.set_attr("rx", "8")?;
    pad.set_attr("style", "cursor:pointer")?;

    let label = svg.text(Point::new(160.0, 70.0 + PAD_Y), "press & hold")?;
    label.set_fill(WHITE)?;
    label.set_attr("font-size", "15")?;
    label.set_attr("text-anchor", "middle")?;
    label.set_attr("style", "pointer-events:none")?;

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
    on_raw(&pad, "mousedown", move |e| {
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
    on_raw(&pad, "mouseup", move |_| release_up())?;

    // A context menu (right-click, or ctrl+click on macOS) interrupts the gesture consuming the mouseup event, so treat
    // it as a release so the button can never get stuck in the pressed state, whatever the platform.
    let release_ctx = release.clone();
    on_raw(&pad, "contextmenu", move |_| release_ctx())?;

    // If the pointer leaves while still held, treat it as a release so the button cannot get stuck in the pressed state
    on_raw(&pad, "pointerleave", move |_| release())?;

    caption(
        &svg,
        400.0,
        "raw mousedown / mouseup / pointerleave · pressed-state tracking · reports held modifier keys",
    )?;
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
    // on_pointerenter stores its closure inside the node, so the node must outlive this function for the listener to keep
    // working — leak it for the page's lifetime (just as demo_anim does with its AnimationLoop).
    mem::forget(group1);

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
    // on_pointerenter stores its closure inside the node, so the node must outlive this function for the listener to keep
    // working — leak it for the page's lifetime (just as demo_anim does with its AnimationLoop).
    mem::forget(group2);

    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Events — drag & drop a card within a parent bounding box (raw mousedown / mousemove / mouseup)
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

// A single transparent surface on *top* of everything captures every pointer event and hit-tests the card itself.
// Keeping mousedown, mousemove and mouseup all on the one element means every offsetX/offsetY is read relative to the
// surface, avoiding the cross-element offset mismatch you would get by putting mousedown on the card and mousemove on
// the surface.
fn demo_events_drag() -> Result<(), Error> {
    const BX: f64 = 40.0; // bounding box
    const BW: f64 = 720.0;
    const BH: f64 = 86.0;
    const BY: f64 = (H - BH) / 2.0; // vertically centred in the canvas
    const OW: f64 = 96.0; // draggable card
    const OH: f64 = 50.0;

    let svg = SvgRoot::create_in("demo-events-drag", Size::new(W, H))?;

    // The parent bounding box that constrains the card (a dashed "drop zone").
    let bbox = svg.rect(Point::new(BX, BY), Size::new(BW, BH))?;
    bbox.set_fill(DROP_ZONE_FILL)?;
    bbox.set_stroke(DROP_ZONE_BORDER)?;
    bbox.set_stroke_width(1.5)?;
    bbox.set_attr("rx", "8")?;
    bbox.set_attr("stroke-dasharray", "6 4")?;

    // The draggable card is a group (background + label) moved as a unit via its transform.
    let card_bg = svg.rect(Point::new(0.0, 0.0), Size::new(OW, OH))?;
    card_bg.set_fill(ACCENT_BLUE)?;
    card_bg.set_attr("rx", "8")?;

    let card_label = svg.text(Point::new(OW / 2.0, OH / 2.0 + 5.0), "drag me")?;
    card_label.set_fill(INK)?;
    card_label.set_attr("font-size", "14")?;
    card_label.set_attr("font-weight", "bold")?;
    card_label.set_attr("text-anchor", "middle")?;

    let card = svg.group()?;
    card.append(&card_bg)?;
    card.append(&card_label)?;

    let start = (BX + 16.0, BY + (BH - OH) / 2.0);
    card.set_attr(
        "transform",
        &format!("translate({:.1}, {:.1})", start.0, start.1),
    )?;

    let readout = svg.text(
        Point::new(W - 12.0, BY - 6.0),
        &format!("x: {:.0}  y: {:.0}", start.0, start.1),
    )?;
    readout.set_fill(TEXT_MUTED)?;
    readout.set_attr("font-size", "12")?;
    readout.set_attr("text-anchor", "end")?;

    caption(
        &svg,
        400.0,
        "mousedown on the card, mousemove to drag, mouseup to drop — clamped to the box",
    )?;

    // Capture surface on top; it hit-tests the card and drives the whole gesture.
    let surface = svg.rect(Point::origin(), Size::new(W, H))?;
    surface.set_fill(TRANSPARENT)?;
    surface.set_attr("style", "cursor:default")?;

    // Shared state: the card's current top-left, and Some(grab-offset) while a drag is in progress.
    let pos = Rc::new(Cell::new(start));
    let grab: Rc<Cell<Option<(f64, f64)>>> = Rc::new(Cell::new(None));

    // Top-left bounds that keep the whole card inside the box.
    let max_x = BX + BW - OW;
    let max_y = BY + BH - OH;
    let on_card = |px: f64, py: f64, ox: f64, oy: f64| {
        (ox..ox + OW).contains(&px) && (oy..oy + OH).contains(&py)
    };

    // mousedown → start dragging only if the press landed on the card.
    {
        let pos = pos.clone();
        let grab = grab.clone();
        let card = card.clone();
        let surface_c = surface.clone();

        on_raw(&surface, "mousedown", move |e| {
            let (px, py) = (f64::from(e.offset_x()), f64::from(e.offset_y()));
            let (ox, oy) = pos.get();
            if on_card(px, py, ox, oy) {
                grab.set(Some((px - ox, py - oy)));
                let _ = card.set_attr("opacity", "0.85");
                let _ = surface_c.set_attr("style", "cursor:grabbing");
            }
        })?;
    }

    // mousemove → drag the card (clamped) while held; otherwise just hint the cursor when hovering it.
    {
        let pos = pos.clone();
        let grab = grab.clone();
        let card = card.clone();
        let readout = readout.clone();
        let surface_c = surface.clone();

        on_raw(&surface, "mousemove", move |e| {
            let (px, py) = (f64::from(e.offset_x()), f64::from(e.offset_y()));
            if let Some((gx, gy)) = grab.get() {
                let (nx, ny) = ((px - gx).clamp(BX, max_x), (py - gy).clamp(BY, max_y));
                pos.set((nx, ny));
                let _ = card.set_attr("transform", &format!("translate({nx:.1}, {ny:.1})"));
                readout
                    .as_element()
                    .set_text_content(Some(&format!("x: {nx:.0}  y: {ny:.0}")));
            } else {
                let (ox, oy) = pos.get();
                let cursor = if on_card(px, py, ox, oy) {
                    "cursor:grab"
                } else {
                    "cursor:default"
                };
                let _ = surface_c.set_attr("style", cursor);
            }
        })?;
    }

    // mouseup / pointerleave → drop the card (pointerleave guards against releasing outside the canvas).
    {
        let grab = grab.clone();
        let card = card.clone();
        let surface_c = surface.clone();
        let drop = move || {
            grab.set(None);
            let _ = card.set_attr("opacity", "1");
            let _ = surface_c.set_attr("style", "cursor:default");
        };
        let drop_up = drop.clone();

        on_raw(&surface, "mouseup", move |_| drop_up())?;
        on_raw(&surface, "pointerleave", move |_| drop())?;
    }

    Ok(())
}
