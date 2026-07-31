use crate::{H, PAD_Y, W, caption, colours::*};
use svg_dom::{
    Error, SvgRoot,
    root::utils::{Point, Size},
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// ellipse
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
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
    // The hover ellipse (90 x 55) fully contains the resting one (60 x 35), so the boundary only ever moves
    // *outward* under the pointer. A hover effect that instead shrank a radius would pull the edge back past a
    // stationary pointer — re-triggering pointerleave, then pointerenter as it grew again — and the ellipse would
    // flicker between states.
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
