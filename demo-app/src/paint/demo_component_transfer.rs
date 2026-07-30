use crate::{BAND, H, PAD_Y, W, caption, colours::*};
use svg_dom::{
    Error, SvgRoot,
    root::{
        filter::{Channel, TransferFunction},
        utils::{Point, Size},
    },
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// feComponentTransfer — same gradient source, three TransferFunction variants across different channels
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-component-transfer", Size::new(W, H))?;

    svg.build_defs(|d| {
        // Same multi-hue gradient technique as the feColorMatrix/feBlend demos: a flat source colour would make
        // the gamma and posterize effects below invisible, since both only become visible where the input colour
        // actually varies across the shape.
        d.build_linear_gradient("component-transfer-source", |g| {
            g.add_stop(0.0, STEELBLUE)?;
            g.add_stop(0.5, GOLD)?;
            g.add_stop(1.0, CRIMSON)?;
            Ok(())
        })?;

        // Gamma-darken all three colour channels identically (exponent > 1.0 darkens midtones; the classic
        // display-gamma value 2.2 is used here purely because it is immediately recognisable, not because it has
        // any special meaning to this crate).
        d.build_filter("component-transfer-gamma", |f| {
            let gamma = TransferFunction::Gamma {
                amplitude: 1.0,
                exponent: 2.2,
                offset: 0.0,
            };
            f.component_transfer(&[
                (Channel::Red, gamma.clone()),
                (Channel::Green, gamma.clone()),
                (Channel::Blue, gamma),
            ])?;
            Ok(())
        })?;

        // Posterise all three colour channels to four discrete steps — the stepping is visible as hard colour
        // bands where the smooth gradient would otherwise blend continuously.
        d.build_filter("component-transfer-discrete", |f| {
            let posterize = TransferFunction::Discrete(vec![0.0, 0.33, 0.66, 1.0]);
            f.component_transfer(&[
                (Channel::Red, posterize.clone()),
                (Channel::Green, posterize.clone()),
                (Channel::Blue, posterize),
            ])?;
            Ok(())
        })?;

        // Fade alpha to 40% via a linear remap, touching only the Alpha channel — colour is untouched, unlike
        // every other filter in this demo. Against this gallery's dark canvas background, the faded rectangle
        // visibly blends toward it.
        d.build_filter("component-transfer-alpha", |f| {
            f.component_transfer(&[(Channel::Alpha, TransferFunction::Linear { slope: 0.4, intercept: 0.0 })])?;
            Ok(())
        })?;

        Ok(())
    })?;

    // No filter region widening is needed here, but that is a property of the specific functions used above, not of
    // feComponentTransfer in general: the gamma/posterize filters only touch RGB, leaving alpha (and so the
    // transparent-vs-opaque silhouette) untouched and the alpha filter's Linear { slope: 0.4, intercept: 0.0 } maps
    // 0.0 to 0.0.
    //
    // A transfer function with f(0) > 0 on Channel::Alpha (e.g. a non-zero intercept/offset, or a Table/Discrete list
    // starting above 0.0) would instead paint every fully-transparent pixel in the primitive subregion — the whole
    // filter region here, since `in` is SourceGraphic — turning it into a rectangular halo.
    let rect_h = BAND - 30.0;
    let rect_y = PAD_Y + 10.0;
    let rect_w = 160.0_f64;
    let xs: [f64; 4] = [20.0, 210.0, 400.0, 590.0];

    let r1 = svg.rect(Point::new(xs[0], rect_y), Size::new(rect_w, rect_h))?;
    r1.set_fill_gradient("component-transfer-source")?;
    caption(&svg, xs[0] + rect_w / 2.0, "original")?;

    let r2 = svg.rect(Point::new(xs[1], rect_y), Size::new(rect_w, rect_h))?;
    r2.set_fill_gradient("component-transfer-source")?;
    r2.set_filter("component-transfer-gamma")?;
    caption(&svg, xs[1] + rect_w / 2.0, "Gamma(2.2)")?;

    let r3 = svg.rect(Point::new(xs[2], rect_y), Size::new(rect_w, rect_h))?;
    r3.set_fill_gradient("component-transfer-source")?;
    r3.set_filter("component-transfer-discrete")?;
    caption(&svg, xs[2] + rect_w / 2.0, "Discrete(4-step)")?;

    let r4 = svg.rect(Point::new(xs[3], rect_y), Size::new(rect_w, rect_h))?;
    r4.set_fill_gradient("component-transfer-source")?;
    r4.set_filter("component-transfer-alpha")?;
    caption(&svg, xs[3] + rect_w / 2.0, "Alpha Linear(0.4)")?;

    Ok(())
}
