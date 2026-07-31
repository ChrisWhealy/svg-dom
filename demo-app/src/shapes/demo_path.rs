use crate::{H, PAD_Y, W, caption, colours::*};
use svg_dom::{
    ArcSize, ArcSweep, EllipticalArc, Error, PathDef, PathDefAbsolute, SvgAttrs, SvgRoot,
    root::utils::{Point, Size},
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// path
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
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
