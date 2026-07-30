use crate::{BAND, H, PAD_Y, caption, colours::*};
use svg_dom::{
    Error, SvgRoot, TextAnchor,
    root::{
        filter::{BlendMode, ColorMatrixType, CompositeOperator},
        utils::{Point, Size},
    },
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// feImage — brings external image content into a filter graph, then combines it with feColorMatrix/feBlend
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    // Wider than the shared W: five panels, and two of their captions ("feImage + color_matrix",
    // "feImage + composite + blend") are long enough to collide at the shared layout's usual spacing.
    const FE_IMAGE_W: f64 = 900.0;
    let svg = SvgRoot::create_in("demo-fe-image", Size::new(FE_IMAGE_W, H))?;

    // Same 60×40 four-quadrant colour grid used by the plain <image> demo, so the "original" panel here is
    // recognisably the same source, just placed directly rather than routed through a filter graph.
    const SRC: &str = "data:image/svg+xml;base64,\
        PHN2ZyB4bWxucz0naHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmcnIHdpZHRoPSc2MCcgaGVpZ2h0\
        PSc0MCc+PHJlY3Qgd2lkdGg9JzMwJyBoZWlnaHQ9JzIwJyBmaWxsPSdzdGVlbGJsdWUnLz48cmVj\
        dCB4PSczMCcgd2lkdGg9JzMwJyBoZWlnaHQ9JzIwJyBmaWxsPSdjb3JhbCcvPjxyZWN0IHk9JzIw\
        JyB3aWR0aD0nMzAnIGhlaWdodD0nMjAnIGZpbGw9J2dvbGQnLz48cmVjdCB4PSczMCcgeT0nMjAn\
        IHdpZHRoPSczMCcgaGVpZ2h0PScyMCcgZmlsbD0nbWVkaXVtc2VhZ3JlZW4nLz48L3N2Zz4=";

    // The same four-quadrant grid again, but as a live `<g>` in `<defs>` rather than a data URI, referenced by three
    // of the four feImage panels below via "#fe-image-quadrants". This is the more realistic case `href` supports:
    // feImage can pull in any SVG element already in the document, such as vector content built or modified at runtime,
    // not just an external/data-URI image resource — which a base64-encoded raster or SVG snapshot cannot show.
    //
    // Per the Filter Effects spec, a stand-alone resource (raster or data-URI SVG) renders <image>-style that is scaled
    // to fill feImage's primitive subregion via `preserveAspectRatio`, while a same-document element references the
    // <use>-style instead.
    //
    // A `<g>`, like the one referenced here, has no viewport of its own to resize: `preserveAspectRatio` has no effect
    // on it, and its content renders at its own native geometry, positioned relative to the subregion's own origin.
    // (A referenced `<svg>` or `<symbol>` would establish a viewport and scale like an image resource instead. This is
    // specific to referencing a `<g>`.)
    //
    // So this group is built directly at 120×80 — the exact size of the three `<rect>` panels below — rather than
    // relying on any scaling to make it fit.
    svg.build_defs(|d| {
        let quadrants = d.group()?;
        quadrants.set_attr("id", "fe-image-quadrants")?;
        svg.build_batch_into(&quadrants, |b| {
            b.rect(Point::new(0.0, 0.0), Size::new(60.0, 40.0))?.set_fill(STEELBLUE)?;
            b.rect(Point::new(60.0, 0.0), Size::new(60.0, 40.0))?.set_fill(CORAL)?;
            b.rect(Point::new(0.0, 40.0), Size::new(60.0, 40.0))?.set_fill(GOLD)?;
            b.rect(Point::new(60.0, 40.0), Size::new(60.0, 40.0))?
                .set_fill(MEDIUM_SEA_GREEN)?;
            Ok(())
        })?;

        // feImage alone: the referenced group becomes this filter's entire output, unmodified — no other primitive
        // reads it, so nothing here is any different from placing the same content directly.
        d.build_filter("fe-image-plain", |f| {
            super::exact_filter_region(f)?;
            f.image("#fe-image-quadrants")?;
            Ok(())
        })?;

        // Import the group via feImage, then greyscale it with color_matrix — the exact chain from SvgFilter::image's
        // doc comment. Since feImage does not read from the `in` argument, `color_matrix`'s implicit input (being the
        // filter's second primitive) is `feImage`'s own output, not SourceGraphic. A filtered plain <image> could be
        // greyscaled the same way (it becomes SourceGraphic on its own); this panel only shows that feImage's output
        // composes with a later primitive like any other primitive's output does.
        d.build_filter("fe-image-greyscale", |f| {
            super::exact_filter_region(f)?;
            f.image("#fe-image-quadrants")?;
            f.color_matrix(ColorMatrixType::Saturate(0.0))?;
            Ok(())
        })?;

        // Tint the imported group by blending a flood colour over it. As with the greyscale panel above, a
        // filtered plain <image> could be tinted the same way — feFlood supplies its own second input, so this
        // does not need feImage either; it is still just feImage's output composing with two more primitives.
        d.build_filter("fe-image-tinted", |f| {
            super::exact_filter_region(f)?;
            f.image("#fe-image-quadrants")?.set_attr("result", "photo")?;
            f.flood(GOLDENROD, 1.0)?.set_attr("result", "colour")?;
            f.blend("photo", BlendMode::Multiply)?.set_attr("in", "colour")?;
            Ok(())
        })?;

        // The genuine distinguishing case: combine feImage's output with the *filtered element's own* SourceAlpha
        // and SourceGraphic — something a filtered plain <image> cannot do, since it has no second, independent
        // source to combine with. composite(SourceAlpha, In) clips the imported texture to the filtered text's own
        // glyph shapes; blend(SourceGraphic, Multiply) then composes it back over the text's own fill. The text
        // below is filled white, multiplication's identity colour, so the result is exactly the clipped texture
        // with no tint from the glyphs' own fill.
        d.build_filter("fe-image-texture", |f| {
            // Unlike the three panels above, this one needs its source scaled to an oddly-shaped target (the text's own
            // glyph-run bounding box), which only an image resource's preserveAspectRatio can do.
            //
            // Note that an SVG element reference like "#fe-image-quadrants" renders at its own native size regardless
            // (see the comment above the group's definition), so this panel goes back to the data-URI source. The text's
            // own bounding box is much wider and shorter than the 3:2 source image, so the default preserveAspectRatio
            // ("xMidYMid meet") would letterbox it, leaving most of the glyphs' width uncovered.
            //
            // The use of "none" disables uniform scaling and stretches the image independently on each axis so it
            // exactly fills the region in which it is placed, and, being at least as large as the text's own bounding
            // box, guarantees every glyph is covered before `composite(SourceAlpha, In)` clips it back.
            f.image(SRC)?
                .set_attrs([("result", "texture"), ("preserveAspectRatio", "none")])?;
            f.composite("SourceAlpha", CompositeOperator::In)?
                .set_attrs([("in", "texture"), ("result", "clipped-texture")])?;
            f.blend("clipped-texture", BlendMode::Multiply)?
                .set_attr("in", "SourceGraphic")?;
            Ok(())
        })?;

        Ok(())
    })?;

    // 60×40 is a 3:2 source aspect ratio; sizing each box 120×80 keeps that ratio exactly, so the default
    // preserveAspectRatio ("xMidYMid meet") neither letterboxes nor crops any panel.
    let img_w = 120.0_f64;
    let img_h = 80.0_f64;
    let y0 = PAD_Y + (BAND - img_h) / 2.0;
    let xs: [f64; 5] = [40.0, 215.0, 390.0, 565.0, 740.0];

    svg.image(SRC, Point::new(xs[0], y0), Size::new(img_w, img_h))?;
    caption(&svg, xs[0] + img_w / 2.0, "original <image>")?;

    let plain = svg.rect(Point::new(xs[1], y0), Size::new(img_w, img_h))?;
    plain.set_filter("fe-image-plain")?;
    caption(&svg, xs[1] + img_w / 2.0, "feImage")?;

    let greyscale = svg.rect(Point::new(xs[2], y0), Size::new(img_w, img_h))?;
    greyscale.set_filter("fe-image-greyscale")?;
    caption(&svg, xs[2] + img_w / 2.0, "feImage + color_matrix")?;

    let tinted = svg.rect(Point::new(xs[3], y0), Size::new(img_w, img_h))?;
    tinted.set_filter("fe-image-tinted")?;
    caption(&svg, xs[3] + img_w / 2.0, "feImage + flood + blend")?;

    let textured = svg.text(Point::new(xs[4] + img_w / 2.0, y0 + img_h / 2.0 + 14.0), "feImage")?;
    textured.set_fill("white")?;
    textured.set_font_size(32.0)?;
    textured.set_text_anchor(TextAnchor::Middle)?;
    textured.set_attr("font-weight", "bold")?;
    textured.set_filter("fe-image-texture")?;
    caption(&svg, xs[4] + img_w / 2.0, "feImage + composite + blend")?;

    Ok(())
}
