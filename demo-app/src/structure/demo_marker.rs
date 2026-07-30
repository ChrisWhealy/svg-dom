use crate::{H, PAD_Y, W, caption, colours::*};
use svg_dom::{
    Error, SvgRoot,
    root::utils::{Point, Size},
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// defs / marker (arrowhead)
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
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
    caption(&svg, 385.0, "orient=\"auto\"")?;

    // Thick — same marker reused across all three lines
    let l3 = svg.line(Point::new(530.0, 55.0 + PAD_Y), Point::new(770.0, 55.0 + PAD_Y))?;
    l3.set_stroke(ACCENT_BLUE)?;
    l3.set_stroke_width(4.0)?;
    l3.set_marker_end_ref(&arrow)?;
    caption(&svg, 650.0, "set_marker_end_ref")?;

    Ok(())
}
