use crate::{BAND, H, PAD_Y, W, caption, colours::*};
use svg_dom::{
    Error, SvgRoot,
    root::utils::{Point, Size},
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// symbol — reusable scaled viewport
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-symbol", Size::new(W, H))?;

    // Define a badge icon once inside <defs> with its own 40×40 coordinate space.
    // The viewBox allows the <use> element's width/height to scale the content automatically; the same definition
    // renders at any size with no extra work — this is what separates <symbol> from a plain <g> in <defs>.
    let defs = svg.defs()?;

    defs.build_symbol("badge", |s| {
        s.set_view_box(0.0, 0.0, 40.0, 40.0)?;
        // Outer disc
        s.circle(Point::new(20.0, 20.0), 19.0)?.set_fill(STEELBLUE)?;
        // Inner 5-pointed star: 10-vertex polygon alternating outer (r≈12) and inner (r≈5) vertices
        s.polygon(&[
            Point::new(20.0, 8.0),
            Point::new(22.5, 15.0),
            Point::new(30.0, 15.5),
            Point::new(24.5, 21.0),
            Point::new(27.0, 29.0),
            Point::new(20.0, 24.5),
            Point::new(13.0, 29.0),
            Point::new(15.5, 21.0),
            Point::new(10.0, 15.5),
            Point::new(17.5, 15.0),
        ])?
        .set_fill(WHITE)?;
        Ok(())
    })?;

    // Stamp five copies at increasing sizes.
    // Even spacing: gap = (W − Σsizes) / (n+1).
    let sizes: [f64; 5] = [30.0, 50.0, 70.0, 90.0, 110.0];
    let total: f64 = sizes.iter().sum();
    let gap = (W - total) / (sizes.len() as f64 + 1.0);
    let cy = PAD_Y + BAND / 2.0;
    let mut x = gap;

    for sz in sizes {
        let u = svg.use_node("#badge", Point::new(x, cy - sz / 2.0))?;
        u.set_attr("width", &format!("{sz}"))?;
        u.set_attr("height", &format!("{sz}"))?;
        x += sz + gap;
    }

    caption(
        &svg,
        W / 2.0,
        "one <symbol> (viewBox 40×40) stamped five times at increasing sizes via <use>",
    )?;
    Ok(())
}
