use crate::{H, W, caption, colours::*};
use svg_dom::{
    Error, SvgRoot, TextAnchor,
    root::{
        mask::MaskType,
        utils::{Point, Size},
    },
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// mask — luminance gradient fade, a text-shaped hole, and an alpha mask on a group
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-mask", Size::new(W, H))?;

    // All mask and gradient ids must be globally unique in the document.
    svg.build_defs(|d| {
        // Opaque white-to-black linear gradient: under the default MaskType::Luminance, opaque white reveals fully
        // and opaque black hides fully, so this fades the referencing rect smoothly from opaque to invisible.
        // (Luminance mode also factors in alpha, not just colour — this gradient just never varies it.)
        d.build_linear_gradient("mk-fade-grad", |g| {
            g.add_stop(0.0, WHITE)?;
            g.add_stop(1.0, "black")?;
            Ok(())
        })?;
        // Mask 1: the gradient painted across the same region as the rect it will mask.
        d.build_mask("mk-fade", |m| {
            m.rect(Point::new(77.0, 37.0), Size::new(106.0, 106.0))?
                .set_fill_gradient("mk-fade-grad")?;
            Ok(())
        })?;
        // Mask 2: a solid black backing (hides everything) with white text on top (reveals only the glyphs) —
        // the standard "cut a hole shaped like this text" technique.
        d.build_mask("mk-text", |m| {
            m.rect(Point::new(345.0, 42.0), Size::new(110.0, 96.0))?.set_fill("black")?;
            let hole = m.text(Point::new(400.0, 100.0), "SVG")?;
            hole.set_fill(WHITE)?;
            hole.set_font_size(40.0)?;
            hole.set_attr("font-weight", "bold")?;
            hole.set_text_anchor(TextAnchor::Middle)?;
            Ok(())
        })?;
        // Mask 3: MaskType::Alpha — a white circle whose fill-opacity (not colour) controls how much of the
        // group beneath shows through, contrasted with a fully-opaque neighbour.
        d.build_mask("mk-alpha", |m| {
            m.set_mask_type(MaskType::Alpha)?;
            let dim = m.circle(Point::new(628.0, 90.0), 45.0)?;
            dim.set_fill(WHITE)?;
            dim.set_attr("fill-opacity", "0.35")?;
            let full = m.circle(Point::new(702.0, 90.0), 45.0)?;
            full.set_fill(WHITE)?;
            Ok(())
        })?;
        Ok(())
    })?;

    // Section 1: gradient rect faded from opaque to invisible against the canvas background.
    let r1 = svg.rect(Point::new(77.0, 37.0), Size::new(106.0, 106.0))?;
    r1.set_fill(STEELBLUE)?;
    r1.set_mask("mk-fade")?;
    caption(&svg, 130.0, "luminance gradient fade")?;

    // Section 2: a gold rect, visible only through the "SVG" text-shaped hole in mk-text.
    let r2 = svg.rect(Point::new(345.0, 42.0), Size::new(110.0, 96.0))?;
    r2.set_fill(GOLD)?;
    r2.set_mask("mk-text")?;
    caption(&svg, 400.0, "text mask (cut a hole)")?;

    // Section 3: two coral circles, one dimmed and one at full strength, revealed through mk-alpha —
    // demonstrating that alpha (not colour/luminance) controls the reveal in MaskType::Alpha.
    let group = svg.group()?;
    svg.build_batch_into(&group, |b| {
        b.circle(Point::new(628.0, 90.0), 45.0)?.set_fill(CORAL)?;
        b.circle(Point::new(702.0, 90.0), 45.0)?.set_fill(CORAL)?;
        Ok(())
    })?;
    group.set_mask("mk-alpha")?;
    caption(&svg, 665.0, "alpha mask on a group")?;

    Ok(())
}
