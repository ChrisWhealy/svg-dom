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
use wasm_bindgen::prelude::*;

use crate::{AnimationLoop, Error, SvgRoot};

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
