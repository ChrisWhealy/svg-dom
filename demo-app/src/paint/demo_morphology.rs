use crate::{BAND, H, PAD_Y, W, caption, colours::*};
use svg_dom::{
    Error, SvgRoot, TextAnchor,
    root::{
        filter::MorphologyOperator,
        utils::{Point, Size},
    },
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// feMorphology — erode/dilate a shape's silhouette, plus dilate + merge for a bold text outline
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-morphology", Size::new(W, H))?;

    svg.build_defs(|d| {
        // A small radius so the eroded/dilated strokes stay legible as text rather than collapsing into gaps or
        // fusing into blobs.
        d.build_filter("morphology-erode", |f| {
            f.morphology(1.2, MorphologyOperator::Erode)?;
            Ok(())
        })?;
        d.build_filter("morphology-dilate", |f| {
            f.morphology(1.2, MorphologyOperator::Dilate)?;
            Ok(())
        })?;

        // The main showcase, and the exact chain from SvgFilter::morphology's doc comment: dilate the source
        // alpha, then merge it underneath the original text so only the grown-outward fringe shows through — a
        // bolder outline without otherwise changing the glyphs' own weight or colour. Dilating can push content
        // beyond the SVG default filter region, so it is widened the same way demo_filter widens it for a wide
        // blur.
        d.build_filter("morphology-outline", |f| {
            super::widen_filter_region(f)?;
            f.morphology(2.0, MorphologyOperator::Dilate)?
                .set_attrs([("in", "SourceAlpha"), ("result", "thickened")])?;
            f.merge(&["thickened", "SourceGraphic"])?;
            Ok(())
        })?;

        Ok(())
    })?;

    let mid_y = PAD_Y + BAND / 2.0;
    let rect_w = 160.0_f64;
    let xs: [f64; 4] = [20.0, 210.0, 400.0, 590.0];

    let original = svg.text(Point::new(xs[0] + rect_w / 2.0, mid_y + 10.0), "MORPH")?;
    original.set_fill(STEELBLUE)?;
    original.set_font_size(34.0)?;
    original.set_text_anchor(TextAnchor::Middle)?;
    original.set_attr("font-weight", "bold")?;
    caption(&svg, xs[0] + rect_w / 2.0, "original")?;

    let eroded = svg.text(Point::new(xs[1] + rect_w / 2.0, mid_y + 10.0), "MORPH")?;
    eroded.set_fill(STEELBLUE)?;
    eroded.set_font_size(34.0)?;
    eroded.set_text_anchor(TextAnchor::Middle)?;
    eroded.set_attr("font-weight", "bold")?;
    eroded.set_filter("morphology-erode")?;
    caption(&svg, xs[1] + rect_w / 2.0, "Erode(1.2)")?;

    let dilated = svg.text(Point::new(xs[2] + rect_w / 2.0, mid_y + 10.0), "MORPH")?;
    dilated.set_fill(STEELBLUE)?;
    dilated.set_font_size(34.0)?;
    dilated.set_text_anchor(TextAnchor::Middle)?;
    dilated.set_attr("font-weight", "bold")?;
    dilated.set_filter("morphology-dilate")?;
    caption(&svg, xs[2] + rect_w / 2.0, "Dilate(1.2)")?;

    let outlined = svg.text(Point::new(xs[3] + rect_w / 2.0, mid_y + 10.0), "MORPH")?;
    outlined.set_fill(STEELBLUE)?;
    outlined.set_font_size(34.0)?;
    outlined.set_text_anchor(TextAnchor::Middle)?;
    outlined.set_attr("font-weight", "bold")?;
    outlined.set_filter("morphology-outline")?;
    caption(&svg, xs[3] + rect_w / 2.0, "bold outline (dilate + merge)")?;

    Ok(())
}
