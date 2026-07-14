use std::{cell::Cell, rc::Rc};

use super::colours::*;
use super::{H, PAD_Y, W, caption};
use crate::{
    ArcSize, ArcSweep, EllipticalArc, Error, PathDef, PathDefAbsolute, SvgAttrs, SvgRoot,
    root::utils::{Point, Size},
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// rect
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(super) fn demo_rect() -> Result<(), Error> {
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
pub(super) fn demo_circle() -> Result<(), Error> {
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
pub(super) fn demo_ellipse() -> Result<(), Error> {
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
pub(super) fn demo_line() -> Result<(), Error> {
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
pub(super) fn demo_poly() -> Result<(), Error> {
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
pub(super) fn demo_path() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-path", Size::new(W, H))?;
    // The path data is authored in the BAND; this transform vertically centres each path in the canvas.
    let shift = format!("translate(0,{PAD_Y})");

    // Closed triangle (M / L / Z) — built from typed PathDef segments rather than a hand-written `d` string, so the
    // path data can never be malformed.
    let tri = svg.path_from_defs(&[
        PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(70.0, 10.0))),
        PathDef::Abs(PathDefAbsolute::LineTo(Point::new(130.0, 110.0))),
        PathDef::Abs(PathDefAbsolute::LineTo(Point::new(10.0, 110.0))),
        PathDef::Abs(PathDefAbsolute::ClosePath),
    ])?;
    let mut attrs = SvgAttrs::new();
    tri.attrs(&mut attrs)
        .fill(STEELBLUE)?
        .stroke(WHITE)?
        .stroke_width(2.0)?
        .set("transform", &shift)?;
    caption(&svg, 70.0, "triangle (M L Z)")?;

    // Quadratic Bézier wave (Q)
    let wave = svg.path_from_defs(&[
        PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(180.0, 65.0))),
        PathDef::Abs(PathDefAbsolute::QuadraticBezierTo(
            Point::new(245.0, 10.0),
            Point::new(310.0, 65.0),
        )),
        PathDef::Abs(PathDefAbsolute::QuadraticBezierTo(
            Point::new(375.0, 120.0),
            Point::new(440.0, 65.0),
        )),
    ])?;
    wave.set_fill(NONE)?;
    wave.set_stroke(MEDIUM_ORCHID)?;
    wave.set_stroke_width(3.0)?;
    wave.set_attr("transform", &shift)?;
    caption(&svg, 310.0, "Bézier wave (Q)")?;

    // Elliptical arc — open semicircle (A). ArcSize::Large + ArcSweep::Clockwise picks the same solution as the
    // original hand-written "A 60 60 0 1 1 630 65" (large-arc-flag=1, sweep-flag=1).
    let arc = svg.path_from_defs(&[
        PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(510.0, 65.0))),
        PathDef::Abs(PathDefAbsolute::EllipticalArcTo(EllipticalArc {
            radii: Point::new(60.0, 60.0),
            x_axis_rotation: 0.0,
            size: ArcSize::Large,
            sweep: ArcSweep::Clockwise,
            to: Point::new(630.0, 65.0),
        })),
    ])?;
    arc.set_fill(NONE)?;
    arc.set_stroke(CORAL)?;
    arc.set_stroke_width(3.0)?;
    arc.set_attr("transform", &shift)?;
    caption(&svg, 570.0, "arc (A)")?;

    Ok(())
}
