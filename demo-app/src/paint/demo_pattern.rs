use crate::{BAND, H, PAD_Y, W, caption, colours::*};
use svg_dom::{
    Error, SvgRoot,
    root::{
        pattern::PatternUnits,
        utils::{Point, Size},
    },
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// pattern — four named tiles applied as fills
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-pattern", Size::new(W, H))?;

    // Four named patterns, each defined with patternUnits="userSpaceOnUse" so tile dimensions are in the canvas's user
    // coordinate system which is easier to reason about than bounding-box fractions.
    svg.build_defs(|d| {
        // 1. Dot grid: white circles on a steelblue field.
        d.build_pattern("demo-pat-dots", |p| {
            p.set_pattern_units(PatternUnits::UserSpaceOnUse)?;
            p.set_width(18.0)?;
            p.set_height(18.0)?;
            p.rect(Point::new(0.0, 0.0), Size::new(18.0, 18.0))?.set_fill(STEELBLUE)?;
            p.circle(Point::new(9.0, 9.0), 5.0)?.set_fill(WHITE)?;
            Ok(())
        })?;

        // 2. Horizontal stripes: alternating coral and white bands.
        d.build_pattern("demo-pat-stripes", |p| {
            p.set_pattern_units(PatternUnits::UserSpaceOnUse)?;
            p.set_width(30.0)?;
            p.set_height(20.0)?;
            p.rect(Point::new(0.0, 0.0), Size::new(30.0, 10.0))?.set_fill(CORAL)?;
            p.rect(Point::new(0.0, 10.0), Size::new(30.0, 10.0))?.set_fill(WHITE)?;
            Ok(())
        })?;

        // 3. Diagonal stripes: horizontal stripes rotated 45° via patternTransform.
        d.build_pattern("demo-pat-diag", |p| {
            p.set_pattern_units(PatternUnits::UserSpaceOnUse)?;
            p.set_width(20.0)?;
            p.set_height(20.0)?;
            p.set_pattern_transform("rotate(45)")?;
            p.rect(Point::new(0.0, 0.0), Size::new(20.0, 10.0))?.set_fill(MEDIUM_ORCHID)?;
            p.rect(Point::new(0.0, 10.0), Size::new(20.0, 10.0))?.set_fill(WHITE)?;
            Ok(())
        })?;

        // 4. Checkerboard: two 10×10 offset squares in a 20×20 tile.
        d.build_pattern("demo-pat-checker", |p| {
            p.set_pattern_units(PatternUnits::UserSpaceOnUse)?;
            p.set_width(20.0)?;
            p.set_height(20.0)?;
            p.rect(Point::new(0.0, 0.0), Size::new(20.0, 20.0))?.set_fill(TEAL)?;
            p.rect(Point::new(0.0, 0.0), Size::new(10.0, 10.0))?.set_fill(WHITE)?;
            p.rect(Point::new(10.0, 10.0), Size::new(10.0, 10.0))?.set_fill(WHITE)?;
            Ok(())
        })?;

        Ok(())
    })?;

    let rect_h = BAND - 20.0;
    let rect_y = PAD_Y + 10.0;
    let rect_w = 160.0_f64;
    let xs: [f64; 4] = [20.0, 210.0, 400.0, 590.0];

    let r1 = svg.rect(Point::new(xs[0], rect_y), Size::new(rect_w, rect_h))?;
    r1.set_fill_pattern("demo-pat-dots")?;
    caption(&svg, xs[0] + rect_w / 2.0, "dot grid")?;

    let r2 = svg.rect(Point::new(xs[1], rect_y), Size::new(rect_w, rect_h))?;
    r2.set_fill_pattern("demo-pat-stripes")?;
    caption(&svg, xs[1] + rect_w / 2.0, "horizontal stripes")?;

    let r3 = svg.rect(Point::new(xs[2], rect_y), Size::new(rect_w, rect_h))?;
    r3.set_fill_pattern("demo-pat-diag")?;
    caption(&svg, xs[2] + rect_w / 2.0, "diagonal (patternTransform)")?;

    let r4 = svg.rect(Point::new(xs[3], rect_y), Size::new(rect_w, rect_h))?;
    r4.set_fill_pattern("demo-pat-checker")?;
    caption(&svg, xs[3] + rect_w / 2.0, "checkerboard")?;

    Ok(())
}
