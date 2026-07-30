use crate::{BAND, H, PAD_Y, W, caption, colours::*};
use svg_dom::{
    Error, SvgRoot, TextAnchor,
    root::utils::{Point, Size},
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// filter — feGaussianBlur at increasing stdDeviation, plus a tinted drop shadow via
// feGaussianBlur -> feFlood -> feComposite -> feOffset -> feMerge, applied via set_filter
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-filter", Size::new(W, H))?;

    // Four blur-only filters at increasing stdDeviation, plus one tinted-drop-shadow filter using the single
    // feDropShadow primitive (the browser-native shorthand for feGaussianBlur -> feFlood -> feComposite ->
    // feOffset -> feMerge; see SvgFilter::drop_shadow's doc comment and docs/design_notes/filters.md for the full chain this
    // collapses). The SVG default filter region (-10%/-10%/120%/120% of the referencing element's bounding box)
    // is too tight for the widest blur and the offset shadow, so every filter here widens its region via the
    // typed set_x/set_y/set_width/set_height setters to avoid visibly clipping the edge.
    svg.build_defs(|d| {
        d.build_filter("demo-filter-0", |f| {
            super::widen_filter_region(f)?;
            f.gaussian_blur(0.0)?;
            Ok(())
        })?;
        d.build_filter("demo-filter-3", |f| {
            super::widen_filter_region(f)?;
            f.gaussian_blur(3.0)?;
            Ok(())
        })?;
        d.build_filter("demo-filter-6", |f| {
            super::widen_filter_region(f)?;
            f.gaussian_blur(6.0)?;
            Ok(())
        })?;
        d.build_filter("demo-filter-12", |f| {
            super::widen_filter_region(f)?;
            f.gaussian_blur(12.0)?;
            Ok(())
        })?;

        // True tinted drop shadow via the feDropShadow shorthand: one primitive call blurs the source alpha,
        // floods a colour into the blurred mask, offsets it, and merges it underneath the original — no separate
        // merge() call needed, since feDropShadow's result already has the original graphic composited on top.
        // A plain black shadow would be nearly invisible against this dark canvas background, so the flood colour
        // is a saturated one instead, which also demonstrates that the shadow's colour is independently
        // controllable, not just a blurred copy of the source graphic's own fill.
        d.build_filter("demo-filter-shadow", |f| {
            super::widen_filter_region(f)?;
            f.drop_shadow(4.0, 6.0, 6.0, CRIMSON, 0.85)?;
            Ok(())
        })?;
        Ok(())
    })?;

    let mid_y = PAD_Y + BAND / 2.0;

    // Four smaller blur-only circles, left-aligned, leaving the right two-thirds of the canvas free for the
    // drop-shadow banner text below. Captions are just the stdDeviation value — the figcaption below the canvas
    // spells out what they mean — since the full "stdDeviation: N" label is too wide for this tighter spacing.
    let xs: [f64; 4] = [55.0, 150.0, 245.0, 340.0];

    let c1 = svg.circle(Point::new(xs[0], mid_y), 30.0)?;
    c1.set_fill(STEELBLUE)?;
    c1.set_filter("demo-filter-0")?;
    caption(&svg, xs[0], "stdDeviation 0")?;

    let c2 = svg.circle(Point::new(xs[1], mid_y), 30.0)?;
    c2.set_fill(STEELBLUE)?;
    c2.set_filter("demo-filter-3")?;
    caption(&svg, xs[1], "3")?;

    let c3 = svg.circle(Point::new(xs[2], mid_y), 30.0)?;
    c3.set_fill(STEELBLUE)?;
    c3.set_filter("demo-filter-6")?;
    caption(&svg, xs[2], "6")?;

    let c4 = svg.circle(Point::new(xs[3], mid_y), 30.0)?;
    c4.set_fill(STEELBLUE)?;
    c4.set_filter("demo-filter-12")?;
    caption(&svg, xs[3], "12")?;

    // Drop-shadow banner text: the feDropShadow filter applied to real text content rather than a plain shape,
    // the effect's most common real-world use. White fill with a narrow dark grey border keeps the glyphs legible
    // against the dark canvas background; independently of that, the shadow's own colour comes from
    // feDropShadow's flood-color, not from the text's fill, so it stays the same crimson regardless of what
    // colour the banner itself is set to.
    let banner_x = 590.0;
    let banner = svg.text(Point::new(banner_x, mid_y + 12.0), "DROP SHADOW")?;
    banner.set_fill(WHITE)?;
    banner.set_stroke("#333333")?;
    banner.set_stroke_width(1.0)?;
    banner.set_font_size(42.0)?;
    banner.set_text_anchor(TextAnchor::Middle)?;
    banner.set_attr("font-weight", "bold")?;
    banner.set_filter("demo-filter-shadow")?;
    caption(&svg, banner_x, "feDropShadow")?;

    Ok(())
}
