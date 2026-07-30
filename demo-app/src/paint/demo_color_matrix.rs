use crate::{BAND, H, PAD_Y, W, caption, colours::*};
use svg_dom::{
    Error, SvgRoot,
    root::{
        filter::ColorMatrixType,
        utils::{Point, Size},
    },
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// feColorMatrix — original, greyscale (Saturate), hue-rotated, and sepia (custom Matrix)
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-color-matrix", Size::new(W, H))?;

    svg.build_defs(|d| {
        // A multi-hue gradient source: a single flat colour would not show saturation or hue-rotation changing
        // anything, since both operate on hue/chroma that a flat fill barely has.
        d.build_linear_gradient("cm-source", |g| {
            g.add_stop(0.0, STEELBLUE)?;
            g.add_stop(0.5, GOLD)?;
            g.add_stop(1.0, CRIMSON)?;
            Ok(())
        })?;

        d.build_filter("cm-filter-greyscale", |f| {
            f.color_matrix(ColorMatrixType::Saturate(0.0))?;
            Ok(())
        })?;

        d.build_filter("cm-filter-hue", |f| {
            f.color_matrix(ColorMatrixType::HueRotate(180.0))?;
            Ok(())
        })?;

        // Classic "sepia tone" colour matrix: well-known fixed coefficients (not derived from anything else in
        // this crate), included to demonstrate the fully custom Matrix variant alongside the two named ones.
        #[rustfmt::skip]
        let sepia: [f64; 20] = [
            0.393, 0.769, 0.189, 0.0, 0.0,
            0.349, 0.686, 0.168, 0.0, 0.0,
            0.272, 0.534, 0.131, 0.0, 0.0,
            0.0,   0.0,   0.0,   1.0, 0.0,
        ];
        d.build_filter("cm-filter-sepia", |f| {
            f.color_matrix(ColorMatrixType::Matrix(sepia))?;
            Ok(())
        })?;

        Ok(())
    })?;

    // feColorMatrix transforms colour in place; unlike feGaussianBlur/feOffset it never spreads pixels beyond the
    // source's own bounding box, so (unlike the demo_filter panel) no filter region needs widening here.
    let rect_h = BAND - 30.0;
    let rect_y = PAD_Y + 10.0;
    let rect_w = 160.0_f64;
    let xs: [f64; 4] = [20.0, 210.0, 400.0, 590.0];

    let r1 = svg.rect(Point::new(xs[0], rect_y), Size::new(rect_w, rect_h))?;
    r1.set_fill_gradient("cm-source")?;
    caption(&svg, xs[0] + rect_w / 2.0, "original")?;

    let r2 = svg.rect(Point::new(xs[1], rect_y), Size::new(rect_w, rect_h))?;
    r2.set_fill_gradient("cm-source")?;
    r2.set_filter("cm-filter-greyscale")?;
    caption(&svg, xs[1] + rect_w / 2.0, "Saturate(0.0)")?;

    let r3 = svg.rect(Point::new(xs[2], rect_y), Size::new(rect_w, rect_h))?;
    r3.set_fill_gradient("cm-source")?;
    r3.set_filter("cm-filter-hue")?;
    caption(&svg, xs[2] + rect_w / 2.0, "HueRotate(180)")?;

    let r4 = svg.rect(Point::new(xs[3], rect_y), Size::new(rect_w, rect_h))?;
    r4.set_fill_gradient("cm-source")?;
    r4.set_filter("cm-filter-sepia")?;
    caption(&svg, xs[3] + rect_w / 2.0, "Matrix(sepia)")?;

    Ok(())
}
