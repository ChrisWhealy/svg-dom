use crate::{BAND, H, PAD_Y, W, caption, colours::*};
use svg_dom::{
    Error, SvgRoot,
    root::{
        filter::{BlendMode, CompositeOperator},
        utils::{Point, Size},
    },
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// feBlend — same gradient source, flooded with the same orange tint, across three BlendMode variants
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-blend", Size::new(W, H))?;

    svg.build_defs(|d| {
        // Same multi-hue gradient technique as the feColorMatrix demo: a flat source colour would make Multiply,
        // Screen, and Difference each collapse to a single flat result, hiding how differently they actually treat
        // colour.
        d.build_linear_gradient("blend-source", |g| {
            g.add_stop(0.0, STEELBLUE)?;
            g.add_stop(0.5, GOLD)?;
            g.add_stop(1.0, CRIMSON)?;
            Ok(())
        })?;

        // One filter per mode: flood the same tint colour, blend it over the source, then composite the blended
        // result back `In` SourceGraphic. That final step is not optional: `flood` paints its colour opaquely
        // across the *entire* filter region — a rectangle, unrelated to this circle's own round shape — and
        // feBlend's result alpha is the union of its two inputs' alpha. Without clipping back to the source's own
        // alpha coverage, the opaque flood would leak straight through the fully transparent corners of the
        // circle's bounding box, visibly staining them with the flood colour. Using circles here rather than
        // rectangles is deliberate, for exactly this reason: a rectangle has no transparency in its own bounding
        // box for a leaking flood to show through, which would make this mistake invisible. See
        // SvgFilter::blend's own doc comment for the full explanation.
        for (id, mode) in [
            ("blend-filter-multiply", BlendMode::Multiply),
            ("blend-filter-screen", BlendMode::Screen),
            ("blend-filter-difference", BlendMode::Difference),
        ] {
            d.build_filter(id, |f| {
                f.flood(LEAF_ORANGE, 1.0)?.set_attr("result", "tint")?;
                f.blend("tint", mode)?
                    .set_attrs([("in", "SourceGraphic"), ("result", "tinted")])?;
                f.composite("SourceGraphic", CompositeOperator::In)?.set_attr("in", "tinted")?;
                Ok(())
            })?;
        }
        Ok(())
    })?;

    // The final composite(In) step above clips every filter's result back to the source circle's own rendered
    // pixels, so — unlike feGaussianBlur/feOffset, which genuinely spread pixels beyond the source's own shape —
    // no filter region widening is needed here.
    let mid_y = PAD_Y + BAND / 2.0;
    let radius = (BAND - 30.0) / 2.0;
    let xs: [f64; 4] = [100.0, 300.0, 500.0, 700.0];

    let c1 = svg.circle(Point::new(xs[0], mid_y), radius)?;
    c1.set_fill_gradient("blend-source")?;
    caption(&svg, xs[0], "original")?;

    let c2 = svg.circle(Point::new(xs[1], mid_y), radius)?;
    c2.set_fill_gradient("blend-source")?;
    c2.set_filter("blend-filter-multiply")?;
    caption(&svg, xs[1], "Multiply")?;

    let c3 = svg.circle(Point::new(xs[2], mid_y), radius)?;
    c3.set_fill_gradient("blend-source")?;
    c3.set_filter("blend-filter-screen")?;
    caption(&svg, xs[2], "Screen")?;

    let c4 = svg.circle(Point::new(xs[3], mid_y), radius)?;
    c4.set_fill_gradient("blend-source")?;
    c4.set_filter("blend-filter-difference")?;
    caption(&svg, xs[3], "Difference")?;

    Ok(())
}
