use crate::{H, PAD_Y, W, caption, colours::*};
use svg_dom::{
    Error, SvgRoot,
    root::utils::{Point, Size},
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// line
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
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
