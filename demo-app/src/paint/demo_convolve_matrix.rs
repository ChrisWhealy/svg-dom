use crate::{BAND, H, PAD_Y, W, caption, colours::*};
use svg_dom::{
    Error, SvgRoot, TextAnchor,
    root::{
        filter::{ColorMatrixType, EdgeMode},
        utils::{Point, Size},
    },
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// feConvolveMatrix — sharpen, edge-detect, and a desaturated emboss (feConvolveMatrix + feColorMatrix)
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-convolve-matrix", Size::new(W, H))?;

    svg.build_defs(|d| {
        // Classic 3x3 sharpen kernel: values sum to 1.0, so divisor: 1.0 preserves overall brightness and the
        // default bias of 0.0 needs no adjustment.
        d.build_filter("convolve-sharpen", |f| {
            #[rustfmt::skip]
            let kernel = [
                 0.0, -1.0,  0.0,
                -1.0,  5.0, -1.0,
                 0.0, -1.0,  0.0,
            ];
            f.convolve_matrix(3, &kernel, 1.0, EdgeMode::Duplicate, false)?;
            Ok(())
        })?;

        // Classic 3x3 edge-detect kernel: values sum to 0.0, so a flat region convolves to 0.0 (black) — exactly
        // the wanted effect here, since only the glyphs' edges should light up. preserve_alpha: true keeps the
        // text's own alpha coverage unfiltered, so the edges stay sharp rather than eroding into the transparent
        // background.
        d.build_filter("convolve-edge-detect", |f| {
            #[rustfmt::skip]
            let kernel = [
                -1.0, -1.0, -1.0,
                -1.0,  8.0, -1.0,
                -1.0, -1.0, -1.0,
            ];
            f.convolve_matrix(3, &kernel, 1.0, EdgeMode::Duplicate, true)?;
            Ok(())
        })?;

        // 3x3 emboss kernel, also zero-sum: bias: 0.5 (set via the generic escape hatch, since it is not one of
        // convolve_matrix's own parameters) shifts a flat region's response up to mid-grey instead of clamping to
        // black — the standard "classic emboss" look. A final color_matrix(Saturate(0.0)) desaturates the result,
        // since the raw convolution output still carries the source's own (steelblue) hue — the same two-primitive
        // chain as should_build_desaturated_emboss_chain in tests/filter/chains.rs.
        d.build_filter("convolve-emboss", |f| {
            #[rustfmt::skip]
            let kernel = [
                -2.0, -1.0, 0.0,
                -1.0,  1.0, 1.0,
                 0.0,  1.0, 2.0,
            ];
            f.convolve_matrix(3, &kernel, 1.0, EdgeMode::Duplicate, true)?
                .set_attr("bias", "0.5")?;
            f.color_matrix(ColorMatrixType::Saturate(0.0))?;
            Ok(())
        })?;

        Ok(())
    })?;

    let mid_y = PAD_Y + BAND / 2.0;
    let rect_w = 160.0_f64;
    let xs: [f64; 4] = [20.0, 210.0, 400.0, 590.0];

    let original = svg.text(Point::new(xs[0] + rect_w / 2.0, mid_y + 10.0), "KERNEL")?;
    original.set_fill(STEELBLUE)?;
    original.set_font_size(34.0)?;
    original.set_text_anchor(TextAnchor::Middle)?;
    original.set_attr("font-weight", "bold")?;
    caption(&svg, xs[0] + rect_w / 2.0, "original")?;

    let sharpened = svg.text(Point::new(xs[1] + rect_w / 2.0, mid_y + 10.0), "KERNEL")?;
    sharpened.set_fill(STEELBLUE)?;
    sharpened.set_font_size(34.0)?;
    sharpened.set_text_anchor(TextAnchor::Middle)?;
    sharpened.set_attr("font-weight", "bold")?;
    sharpened.set_filter("convolve-sharpen")?;
    caption(&svg, xs[1] + rect_w / 2.0, "sharpen")?;

    let edges = svg.text(Point::new(xs[2] + rect_w / 2.0, mid_y + 10.0), "KERNEL")?;
    edges.set_fill(STEELBLUE)?;
    edges.set_font_size(34.0)?;
    edges.set_text_anchor(TextAnchor::Middle)?;
    edges.set_attr("font-weight", "bold")?;
    edges.set_filter("convolve-edge-detect")?;
    caption(&svg, xs[2] + rect_w / 2.0, "edge-detect")?;

    let embossed = svg.text(Point::new(xs[3] + rect_w / 2.0, mid_y + 10.0), "KERNEL")?;
    embossed.set_fill(STEELBLUE)?;
    embossed.set_font_size(34.0)?;
    embossed.set_text_anchor(TextAnchor::Middle)?;
    embossed.set_attr("font-weight", "bold")?;
    embossed.set_filter("convolve-emboss")?;
    caption(&svg, xs[3] + rect_w / 2.0, "emboss (desaturated)")?;

    Ok(())
}
