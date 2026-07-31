use crate::{H, PAD_Y, W, caption, colours::*};
use svg_dom::{
    Error, SvgRoot,
    root::utils::{Point, Size},
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// circle
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
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
