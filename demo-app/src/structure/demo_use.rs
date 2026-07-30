use crate::{BAND, H, PAD_Y, W, caption, colours::*};
use svg_dom::{
    Error, PathDef, PathDefAbsolute, SvgRoot,
    root::utils::{Point, Size},
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// use — stamp copies of a <defs> shape
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-use", Size::new(W, H))?;

    // Define a diamond-shaped path once inside <defs>; it is not rendered until referenced.
    svg.build_defs(|d| {
        let gem = d.path_from_defs(&[
            PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(0.0, -28.0))),
            PathDef::Abs(PathDefAbsolute::LineTo(Point::new(22.0, 0.0))),
            PathDef::Abs(PathDefAbsolute::LineTo(Point::new(0.0, 28.0))),
            PathDef::Abs(PathDefAbsolute::LineTo(Point::new(-22.0, 0.0))),
            PathDef::Abs(PathDefAbsolute::ClosePath),
        ])?;
        gem.set_attr("id", "gem")?;
        gem.set_fill(ACCENT_BLUE)?;
        gem.set_stroke(WHITE)?;
        gem.set_stroke_width(2.0)?;
        Ok(())
    })?;

    // Stamp five independent copies of the same path using <use>.
    // Positioning is done entirely through the transform attribute so that x and y stay at zero
    // and the rotation centres fall exactly on each copy's visual midpoint.
    let cy = PAD_Y + BAND / 2.0;
    let mut buf = String::new();
    for i in 0..5usize {
        let cx = 80.0 + i as f64 * 160.0;
        let angle = i as f64 * 18.0;
        let u = svg.use_node("#gem", Point::origin())?;
        u.set_transform_fmt(&mut buf, format_args!("translate({cx},{cy}) rotate({angle})"))?;
    }

    caption(&svg, W / 2.0, "one <defs> path stamped five times with <use>")?;
    Ok(())
}
