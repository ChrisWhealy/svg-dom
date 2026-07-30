use crate::colours::*;
use svg_dom::{
    Error, SvgRoot,
    root::utils::{Point, Size},
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// SvgRoot::set_view_box — three mini canvases, identical scene, different viewBox
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    // Draws the same four coloured quadrants (each 100x100, filling a 200x200 area) plus a white landmark dot fixed
    // inside the bottom-right (mediumseagreen) quadrant — identical drawing coordinates every time this is called.
    // Only the set_view_box call made on each SvgRoot after this differs, so any visual difference between the
    // three canvases below comes entirely from the viewBox, not from what was drawn.
    let scene = |svg: &SvgRoot| -> Result<(), Error> {
        svg.rect(Point::new(0.0, 0.0), Size::new(100.0, 100.0))?.set_fill(STEELBLUE)?;
        svg.rect(Point::new(100.0, 0.0), Size::new(100.0, 100.0))?.set_fill(CORAL)?;
        svg.rect(Point::new(0.0, 100.0), Size::new(100.0, 100.0))?.set_fill(GOLD)?;
        svg.rect(Point::new(100.0, 100.0), Size::new(100.0, 100.0))?
            .set_fill(MEDIUM_SEA_GREEN)?;
        let dot = svg.circle(Point::new(170.0, 170.0), 8.0)?;
        dot.set_fill(WHITE)?;
        dot.set_stroke(INK)?;
        dot.set_stroke_width(2.0)?;
        Ok(())
    };
    let panel_size = Size::new(200.0, 200.0);

    // 1. No viewBox: the 200x200 drawing coordinates map directly onto the 200x200 pixel canvas, 1 unit = 1 pixel.
    //    All four quadrants and the landmark dot are visible.
    let svg1 = SvgRoot::create_in("demo-view-box-1", panel_size)?;
    scene(&svg1)?;

    // 2. viewBox(0, 0, 100, 100) mapped onto the same 200x200 pixel canvas magnifies everything 2x. Only the
    //    top-left (steelblue) quadrant is visible, and the landmark dot at (170, 170) falls entirely outside this
    //    viewBox, so it is clipped rather than just shrinking off-screen.
    let svg2 = SvgRoot::create_in("demo-view-box-2", panel_size)?;
    scene(&svg2)?;
    svg2.set_view_box(0.0, 0.0, 100.0, 100.0)?;

    // 3. viewBox(50, 50, 150, 150) is both panned (by the (50, 50) origin) and mildly magnified (150 viewBox units
    //    stretched across the 200px canvas), and — deliberately — its far corner (50+150, 50+150) lands exactly on
    //    the scene's own edge (200, 200), so this crops the outer ring of the other three quadrants without
    //    exposing any blank background beyond the drawn content. The bottom-right quadrant, and the landmark dot
    //    inside it, are now fully back in view.
    let svg3 = SvgRoot::create_in("demo-view-box-3", panel_size)?;
    scene(&svg3)?;
    svg3.set_view_box(50.0, 50.0, 150.0, 150.0)?;

    Ok(())
}
