use crate::{BAND, H, PAD_Y, W, caption, colours::*};
use svg_dom::{
    Error, SvgRoot,
    root::{
        filter::{Channel, TurbulenceType},
        utils::{Point, Size},
    },
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// feTurbulence — raw Perlin noise for both TurbulenceType variants, plus feDisplacementMap warping a circle's
// edge into an organic, hand-drawn outline
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-turbulence", Size::new(W, H))?;

    svg.build_defs(|d| {
        // feTurbulence reads no `in` at all: it fabricates its own noise image from nothing, so applying it with no
        // other primitive replaces the referencing element's content with a rectangular patch of noise covering the
        // whole filter region, not clipped to the element's own shape (see SvgFilter::turbulence's doc comment).
        //
        // These two filters exist purely to show the raw noise pattern each TurbulenceType produces, so a plain rect
        // (already rectangular) is used rather than a circle, which would look broken — the fill colour on the rects
        // themselves is irrelevant, since the noise output covers all of it.
        d.build_filter("turbulence-fractal", |f| {
            f.turbulence(0.015, 4, 3.0, TurbulenceType::FractalNoise)?;
            Ok(())
        })?;
        d.build_filter("turbulence-marbled", |f| {
            f.turbulence(0.015, 4, 3.0, TurbulenceType::Turbulence)?;
            Ok(())
        })?;

        // The main showcase: feTurbulence -> feDisplacementMap warps a circle's edge into a hand-drawn, organic outline
        // instead of a perfect geometric one — the standard pairing these two primitives exist for. Red/Green (rather
        // than Alpha/Alpha) give the displacement two independent axes: feTurbulence generates each colour channel
        // separately, so dx and dy are no longer forced equal at every pixel the way they would be reading the same
        // channel for both selectors, which would confine every displacement vector to a single diagonal. The
        // displaced sampling can read pixels just outside the circle's own bounding box, so the region is widened the
        // same way demo_filter widens it for a wide blur.
        d.build_filter("turbulence-displace", |f| {
            super::widen_filter_region(f)?;
            f.turbulence(0.02, 3, 5.0, TurbulenceType::FractalNoise)?
                .set_attr("result", "noise")?;
            f.displacement_map("noise", 24.0, Channel::Red, Channel::Green)?
                .set_attr("in", "SourceGraphic")?;
            Ok(())
        })?;

        Ok(())
    })?;

    let mid_y = PAD_Y + BAND / 2.0;
    let rect_w = 160.0_f64;
    let rect_h = BAND - 30.0;
    let rect_y = PAD_Y + 10.0;
    let xs: [f64; 4] = [20.0, 210.0, 400.0, 590.0];

    let original = svg.circle(Point::new(xs[0] + rect_w / 2.0, mid_y), 50.0)?;
    original.set_fill(STEELBLUE)?;
    caption(&svg, xs[0] + rect_w / 2.0, "original")?;

    let r1 = svg.rect(Point::new(xs[1], rect_y), Size::new(rect_w, rect_h))?;
    r1.set_filter("turbulence-fractal")?;
    caption(&svg, xs[1] + rect_w / 2.0, "FractalNoise")?;

    let r2 = svg.rect(Point::new(xs[2], rect_y), Size::new(rect_w, rect_h))?;
    r2.set_filter("turbulence-marbled")?;
    caption(&svg, xs[2] + rect_w / 2.0, "Turbulence")?;

    let distorted = svg.circle(Point::new(xs[3] + rect_w / 2.0, mid_y), 50.0)?;
    distorted.set_fill(STEELBLUE)?;
    distorted.set_filter("turbulence-displace")?;
    caption(&svg, xs[3] + rect_w / 2.0, "organic edge (feDisplacementMap)")?;

    Ok(())
}
