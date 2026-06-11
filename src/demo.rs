//! Interactive element gallery for the browser.
//!
//! Build and serve with:
//! ```sh
//! cargo demo
//! ```
//! which rebuilds the wasm package and serves it via the Actix `demo-server` crate at <http://127.0.0.1:8000/demo/>.
//!
//! The `demo` feature excludes this code from the normal library build.

use std::{cell::Cell, mem, rc::Rc};
use wasm_bindgen::{JsCast, prelude::*};
use web_sys::MouseEvent;

use crate::{AnimationLoop, Error, SvgNode, SvgRoot};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Constants
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
const W: f64 = 800.0;  // SVG width shared across all demo canvases
const H: f64 = 130.0;  // SVG height shared across all demo canvases

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
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Shared helper
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Appends a small grey caption below an element at horizontal centre `cx`.
fn caption(svg: &SvgRoot, cx: f64, text: &str) -> Result<(), Error> {
    let t = svg.text(cx, H - 6.0, text)?;
    t.set_fill("#777")?;
    t.set_attr("font-size", "11")?;
    t.set_attr("text-anchor", "middle")?;
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// rect
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn demo_rect() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-rect", W, H)?;

    // 1. Plain fill
    let r1 = svg.rect(10.0, 10.0, 130.0, 90.0)?;
    r1.set_fill("steelblue")?;
    caption(&svg, 75.0, "fill")?;

    // 2. Stroke-only (no fill)
    let r2 = svg.rect(155.0, 10.0, 130.0, 90.0)?;
    r2.set_fill("none")?;
    r2.set_stroke("coral")?;
    r2.set_stroke_width(3.0)?;
    caption(&svg, 220.0, "stroke")?;

    // 3. Rounded corners via rx attribute
    let r3 = svg.rect(300.0, 10.0, 130.0, 90.0)?;
    r3.set_fill("mediumseagreen")?;
    r3.set_attr("rx", "20")?;
    caption(&svg, 365.0, "rounded (rx)")?;

    // 4. Hover: fill swaps on mouseover / mouseout
    let r4 = svg.rect(445.0, 10.0, 130.0, 90.0)?;
    r4.set_fill("goldenrod")?;
    r4.set_attr("style", "cursor:pointer")?;
    let r4b = r4.clone();
    r4.on_mouseover(move |_| { let _ = r4b.set_fill("gold"); })?;
    let r4c = r4.clone();
    r4.on_mouseout(move |_| { let _ = r4c.set_fill("goldenrod"); })?;
    caption(&svg, 510.0, "hover")?;

    // 5. Click: toggles between two fills
    let r5 = svg.rect(590.0, 10.0, 130.0, 90.0)?;
    r5.set_fill("slategray")?;
    r5.set_attr("style", "cursor:pointer")?;
    let toggled = Rc::new(Cell::new(false));
    let r5b = r5.clone();
    r5.on_click(move |_| {
        let next = !toggled.get();
        toggled.set(next);
        let _ = r5b.set_fill(if next { "coral" } else { "slategray" });
    })?;
    caption(&svg, 655.0, "click (toggle)")?;

    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// circle
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn demo_circle() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-circle", W, H)?;

    // 1. Solid fill
    let c1 = svg.circle(70.0, 57.0, 47.0)?;
    c1.set_fill("tomato")?;
    caption(&svg, 70.0, "fill")?;

    // 2. Stroke-only
    let c2 = svg.circle(210.0, 57.0, 47.0)?;
    c2.set_fill("none")?;
    c2.set_stroke("orchid")?;
    c2.set_stroke_width(4.0)?;
    caption(&svg, 210.0, "stroke")?;

    // 3. Hover: radius grows / shrinks
    let c3 = svg.circle(360.0, 57.0, 35.0)?;
    c3.set_fill("lightskyblue")?;
    c3.set_attr("style", "cursor:pointer")?;
    let c3b = c3.clone();
    c3.on_mouseover(move |_| { let _ = c3b.set_attr("r", "50"); })?;
    let c3c = c3.clone();
    c3.on_mouseout(move |_| { let _ = c3c.set_attr("r", "35"); })?;
    caption(&svg, 360.0, "hover (radius)")?;

    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// line
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn demo_line() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-line", W, H)?;

    // Horizontal
    let l1 = svg.line(10.0, 55.0, 230.0, 55.0)?;
    l1.set_stroke("#aaa")?;
    l1.set_stroke_width(2.0)?;
    caption(&svg, 120.0, "horizontal")?;

    // Diagonal
    let l2 = svg.line(270.0, 10.0, 470.0, 110.0)?;
    l2.set_stroke("coral")?;
    l2.set_stroke_width(2.0)?;
    caption(&svg, 370.0, "diagonal")?;

    // Thick
    let l3 = svg.line(510.0, 55.0, 790.0, 55.0)?;
    l3.set_stroke("goldenrod")?;
    l3.set_stroke_width(18.0)?;
    caption(&svg, 650.0, "thick stroke")?;

    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// path
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn demo_path() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-path", W, H)?;

    // Closed triangle (M / L / Z)
    let tri = svg.path("M 70 10 L 130 110 L 10 110 Z")?;
    tri.set_fill("steelblue")?;
    tri.set_stroke("white")?;
    tri.set_stroke_width(2.0)?;
    caption(&svg, 70.0, "triangle (M L Z)")?;

    // Quadratic Bézier wave (Q)
    let wave = svg.path("M 180 65 Q 245 10 310 65 Q 375 120 440 65")?;
    wave.set_fill("none")?;
    wave.set_stroke("mediumorchid")?;
    wave.set_stroke_width(3.0)?;
    caption(&svg, 310.0, "Bézier wave (Q)")?;

    // Elliptical arc — open semicircle (A)
    let arc = svg.path("M 510 65 A 60 60 0 1 1 630 65")?;
    arc.set_fill("none")?;
    arc.set_stroke("coral")?;
    arc.set_stroke_width(3.0)?;
    caption(&svg, 570.0, "arc (A)")?;

    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// text
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn demo_text() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-text", W, H)?;

    // Small plain text
    let t1 = svg.text(10.0, 60.0, "Plain text — 14px")?;
    t1.set_fill("#d0d0d0")?;
    t1.set_attr("font-size", "14")?;

    // Large bold
    let t2 = svg.text(10.0, 100.0, "Bold — 36px")?;
    t2.set_fill("steelblue")?;
    t2.set_attr("font-size", "36")?;
    t2.set_attr("font-weight", "bold")?;

    // Coloured, medium
    let t3 = svg.text(430.0, 65.0, "Coloured — 22px")?;
    t3.set_fill("coral")?;
    t3.set_attr("font-size", "22")?;

    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// group (<g>)
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn demo_group() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-group", W, H)?;

    // Group A — steelblue block, positioned with translate
    let g1 = svg.group()?;
    let b1 = svg.rect(0.0, 0.0, 150.0, 80.0)?;
    b1.set_fill("steelblue")?;
    let l1 = svg.text(75.0, 47.0, "Group A")?;
    l1.set_fill("white")?;
    l1.set_attr("font-size", "15")?;
    l1.set_attr("text-anchor", "middle")?;
    g1.append(&b1)?;
    g1.append(&l1)?;
    g1.set_attr("transform", "translate(40, 25)")?;

    // Dashed connector
    let conn = svg.line(190.0, 65.0, 280.0, 65.0)?;
    conn.set_stroke("#444")?;
    conn.set_stroke_width(2.0)?;
    conn.set_attr("stroke-dasharray", "5 4")?;

    // Group B — darkorange block, different translate
    let g2 = svg.group()?;
    let b2 = svg.rect(0.0, 0.0, 150.0, 80.0)?;
    b2.set_fill("darkorange")?;
    let l2 = svg.text(75.0, 47.0, "Group B")?;
    l2.set_fill("white")?;
    l2.set_attr("font-size", "15")?;
    l2.set_attr("text-anchor", "middle")?;
    g2.append(&b2)?;
    g2.append(&l2)?;
    g2.set_attr("transform", "translate(280, 25)")?;

    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// AnimationLoop
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn demo_anim() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-anim", W, H)?;

    // Pulsing circle — radius oscillates
    let pulse = svg.circle(80.0, 55.0, 20.0)?;
    pulse.set_fill("mediumorchid")?;
    caption(&svg, 80.0, "radius pulse")?;

    // Travelling circle — cx oscillates horizontally
    let travel = svg.circle(400.0, 55.0, 14.0)?;
    travel.set_fill("lightskyblue")?;
    caption(&svg, 400.0, "horizontal travel")?;

    // Hue-rotating rectangle
    let hue_rect = svg.rect(600.0, 10.0, 185.0, 90.0)?;
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

// `SvgNode` wraps `click`, `mouseover` and `mouseout` directly.  For every other event we drop down to the raw
// `web-sys` element via [`SvgNode::as_element`] and register the listener ourselves.
// The `Closure` is `forget`-ted so that it lives for the page's lifetime — exactly the same leak-on-purpose pattern
// that `demo_anim` uses for its `AnimationLoop`.
// 
// However, in a real application, you would store the `Closure` somewhere with a defined lifetime.
fn on_raw<F: Fn(MouseEvent) + 'static>(node: &SvgNode, event: &str, handler: F) -> Result<(), Error> {
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
    let svg = SvgRoot::create_in("demo-events-click", W, H)?;

    // Counter button.  Its colour cycles on every click so repeated presses are visible.
    let btn = svg.rect(40.0, 30.0, 150.0, 60.0)?;
    btn.set_fill("steelblue")?;
    btn.set_attr("rx", "8")?;
    btn.set_attr("style", "cursor:pointer")?;

    // The label sits on top of the button; `pointer-events:none` lets clicks fall through to the rect beneath.
    let btn_label = svg.text(115.0, 66.0, "click me")?;
    btn_label.set_fill("white")?;
    btn_label.set_attr("font-size", "16")?;
    btn_label.set_attr("text-anchor", "middle")?;
    btn_label.set_attr("style", "pointer-events:none")?;

    // Reset button — greyed out until there is actually something to reset.
    let reset = svg.rect(210.0, 30.0, 110.0, 60.0)?;
    reset.set_fill("#555")?;
    reset.set_attr("rx", "8")?;
    reset.set_attr("style", "cursor:pointer")?;

    let reset_label = svg.text(265.0, 66.0, "reset")?;
    reset_label.set_fill("white")?;
    reset_label.set_attr("font-size", "15")?;
    reset_label.set_attr("text-anchor", "middle")?;
    reset_label.set_attr("style", "pointer-events:none")?;

    let readout = svg.text(350.0, 66.0, "clicks: 0")?;
    readout.set_fill("#c9d1d9")?;
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
        let _ = inc_reset.set_fill("tomato"); // reset now has something to do
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
        let _ = rst_btn.set_fill("steelblue");
        let _ = rst_reset.set_fill("#555");
        rst_readout.as_element().set_text_content(Some("clicks: 0"));
    })?;

    caption(&svg, 400.0, "two on_click handlers sharing one Rc<Cell> counter")?;
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
    const CX: f64 = 90.0; // wheel centre
    const CY: f64 = 65.0;
    const R: f64 = 52.0; // wheel radius
    const STEP: f64 = 2.0; // angular width of each wedge, in degrees

    let svg = SvgRoot::create_in("demo-events-colour", W, H)?;

    // The wheel is built from thin pie wedges, each filled with its own hue.  Grouping them lets a single
    // `pointer-events:none` on the <g> apply to every wedge at once.
    let wheel = svg.group()?;
    wheel.set_attr("pointer-events", "none")?;
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
    let marker = svg.circle(-20.0, -20.0, 6.0)?;
    marker.set_fill("none")?;
    marker.set_stroke("white")?;
    marker.set_stroke_width(2.0)?;
    marker.set_attr("pointer-events", "none")?;

    // The "second object": its fill follows whatever hue the pointer is over.
    let swatch = svg.rect(210.0, 18.0, 250.0, 94.0)?;
    swatch.set_fill("#222")?;
    swatch.set_stroke("#444")?;
    swatch.set_attr("rx", "12")?;
    swatch.set_attr("pointer-events", "none")?;

    let readout = svg.text(485.0, 70.0, "move over the wheel →")?;
    readout.set_fill("#c9d1d9")?;
    readout.set_attr("font-size", "15")?;
    readout.set_attr("pointer-events", "none")?;

    caption(&svg, 335.0, "raw mousemove over the wheel sets the swatch fill (hue from pointer angle)")?;

    // The pointer-capture surface goes on last so it sits on top of everything above.
    let surface = svg.rect(0.0, 0.0, W, H)?;
    surface.set_fill("transparent")?;
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
    let svg = SvgRoot::create_in("demo-events-modifiers", W, H)?;

    let pad = svg.rect(40.0, 25.0, 240.0, 80.0)?;
    pad.set_fill("slateblue")?;
    pad.set_attr("rx", "8")?;
    pad.set_attr("style", "cursor:pointer")?;

    let hint = svg.text(160.0, 70.0, "click me")?;
    hint.set_fill("white")?;
    hint.set_attr("font-size", "15")?;
    hint.set_attr("text-anchor", "middle")?;
    hint.set_attr("style", "pointer-events:none")?;

    let readout = svg.text(310.0, 70.0, "try: click · shift · ctrl · alt · right-click")?;
    readout.set_fill("#c9d1d9")?;
    readout.set_attr("font-size", "14")?;

    // Left-click → inspect the modifier-key flags carried by the MouseEvent.
    let pad_click = pad.clone();
    let ro_click = readout.clone();
    pad.on_click(move |e| {
        let (label, colour) = if e.shift_key() {
            ("shift + click", "tomato")
        } else if e.ctrl_key() {
            ("ctrl + click", "mediumseagreen")
        } else if e.alt_key() {
            ("alt + click", "goldenrod")
        } else if e.meta_key() {
            ("meta + click", "orchid")
        } else {
            ("plain click", "slateblue")
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
        let _ = pad_ctx.set_fill("crimson");
        ro_ctx
            .as_element()
            .set_text_content(Some("last: right-click (context menu suppressed)"));
    })?;

    caption(&svg, 400.0, "on_click reads modifier keys · raw contextmenu calls preventDefault()")?;
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Events — press state (raw mousedown / mouseup / mouseleave)
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn demo_events_press() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-events-press", W, H)?;

    let pad = svg.rect(60.0, 25.0, 200.0, 80.0)?;
    pad.set_fill("teal")?;
    pad.set_attr("rx", "8")?;
    pad.set_attr("style", "cursor:pointer")?;

    let label = svg.text(160.0, 70.0, "press & hold")?;
    label.set_fill("white")?;
    label.set_attr("font-size", "15")?;
    label.set_attr("text-anchor", "middle")?;
    label.set_attr("style", "pointer-events:none")?;

    let readout = svg.text(320.0, 70.0, "state: idle")?;
    readout.set_fill("#c9d1d9")?;
    readout.set_attr("font-size", "14")?;

    // Closures are `Clone` when everything they capture is `Clone` (SvgNode is), so we can build `press`/`release`
    // once and reuse them across several listeners.
    let press = {
        let pad = pad.clone();
        let readout = readout.clone();
        move || {
            let _ = pad.set_fill("#0a3d3d"); // darken while held
            let _ = pad.set_attr("transform", "translate(2,2)");
            readout.as_element().set_text_content(Some("state: pressed"));
        }
    };
    let release = {
        let pad = pad.clone();
        let readout = readout.clone();
        move || {
            let _ = pad.set_fill("teal");
            let _ = pad.set_attr("transform", "translate(0,0)");
            readout.as_element().set_text_content(Some("state: idle"));
        }
    };

    on_raw(&pad, "mousedown", move |_| press())?;
    let release_up = release.clone();
    on_raw(&pad, "mouseup", move |_| release_up())?;
    // If the pointer leaves while still held, treat it as a release so the button cannot stick in the pressed state.
    on_raw(&pad, "mouseleave", move |_| release())?;

    caption(&svg, 400.0, "raw mousedown / mouseup / mouseleave · pressed-state tracking")?;
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Events — bubbling mouseover vs non-bubbling mouseenter on a group
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn demo_events_group() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-events-group", W, H)?;

    // Builds a bordered container group holding two child shapes, translated to `x`.  The filled background rect is
    // the group's visible boundary AND its hit area: a <g> has no geometry of its own, so without a filled background
    // the gaps between the children would not count as "inside" the group.  Its fill matches the canvas colour so
    // only the coloured border shows.  Returns the group node.
    let build = |x: f64, border: &str| -> Result<SvgNode, Error> {
        let g = svg.group()?;

        let boundary = svg.rect(0.0, 0.0, 300.0, 80.0)?;
        boundary.set_fill("#161b22")?; // == canvas background, so only the stroke is visible
        boundary.set_stroke(border)?;
        boundary.set_stroke_width(2.0)?;
        boundary.set_attr("rx", "8")?;

        let child_a = svg.circle(75.0, 40.0, 22.0)?;
        child_a.set_fill("#f0883e")?;
        let child_b = svg.rect(160.0, 18.0, 110.0, 44.0)?;
        child_b.set_fill("#3fb950")?;
        child_b.set_attr("rx", "4")?;

        g.append(&boundary)?;
        g.append(&child_a)?;
        g.append(&child_b)?;
        g.set_attr("transform", &format!("translate({x}, 26)"))?;
        Ok(g)
    };

    // Builds the title + counter labels for a group and returns the (clonable) counter text node.
    let labels = |x: f64, colour: &str, title: &str| -> Result<SvgNode, Error> {
        let t = svg.text(x, 18.0, title)?;
        t.set_fill(colour)?;
        t.set_attr("font-size", "12")?;
        let count = svg.text(x, 124.0, "fires: 0")?;
        count.set_fill("#c9d1d9")?;
        count.set_attr("font-size", "14")?;
        Ok(count)
    };

    // group 1 — on_mouseover.  This event *bubbles*, so the handler on the <g> fires every time the pointer enters a
    // descendant: once for the boundary, then again for each child (and again when crossing back onto the boundary).
    let g1_count = labels(40.0, "#58a6ff", "group 1: on_mouseover (bubbles)")?;
    let group1 = build(40.0, "#58a6ff")?;
    let c1 = Rc::new(Cell::new(0u32));
    group1.on_mouseover(move |_| {
        let n = c1.get() + 1;
        c1.set(n);
        g1_count
            .as_element()
            .set_text_content(Some(&format!("fires: {n}")));
    })?;
    // on_mouseover stores its closure inside the node, so the node must outlive this function for the listener to keep
    // working — leak it for the page's lifetime (just as demo_anim does with its AnimationLoop).
    mem::forget(group1);

    // group 2 — mouseenter.  This event does *not* bubble, so the handler fires exactly once per boundary crossing,
    // no matter how many child shapes the pointer then sweeps over inside the group.
    let g2_count = labels(440.0, "#d29922", "group 2: mouseenter (no bubble)")?;
    let group2 = build(440.0, "#d29922")?;
    let c2 = Rc::new(Cell::new(0u32));
    // mouseenter is not wrapped by SvgNode; on_raw registers it on the element and leaks the closure, so the listener
    // survives without our having to keep the group handle alive.
    on_raw(&group2, "mouseenter", move |_| {
        let n = c2.get() + 1;
        c2.set(n);
        g2_count
            .as_element()
            .set_text_content(Some(&format!("fires: {n}")));
    })?;

    Ok(())
}
