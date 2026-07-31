use crate::{H, PAD_Y, W, caption, colours::*};
use svg_dom::{
    Error, SvgRoot,
    root::utils::{Point, Size},
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// polygon / polyline
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
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
