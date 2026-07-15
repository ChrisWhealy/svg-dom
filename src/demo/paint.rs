use super::colours::*;
use super::{BAND, H, PAD_Y, W, caption};
use crate::{
    Error, PathDef, PathDefAbsolute, SvgFilter, SvgRoot, TextAnchor,
    root::{
        filter::ColorMatrixType,
        gradient::SpreadMethod,
        pattern::PatternUnits,
        utils::{Point, Size},
    },
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// linearGradient — horizontal, vertical, diagonal, multi-stop, and gradient stroke
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(super) fn demo_linear_gradient() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-linear-gradient", Size::new(W, H))?;

    // All gradient ids must be globally unique in the document, so use a per-demo prefix.
    let defs = svg.build_defs(|d| {
        // 1. Horizontal (default x1/y1/x2/y2): steelblue left → coral right.
        d.build_linear_gradient("demo-lg-h", |g| {
            g.add_stop(0.0, STEELBLUE)?;
            g.add_stop(1.0, CORAL)?;
            Ok(())
        })?;

        // 2. Vertical gradient: set x2=0, y2=1 to rotate the axis 90°.
        d.build_linear_gradient("demo-lg-v", |g| {
            g.add_stop(0.0, GOLDENROD)?;
            g.add_stop(1.0, "midnightblue")?;
            g.set_x2(0.0)?;
            g.set_y2(1.0)?;
            Ok(())
        })?;

        // 3. Diagonal: gradientTransform rotates the gradient vector 45°.
        //    Keeping the default horizontal endpoints and rotating is simpler than computing
        //    trigonometric endpoint coordinates by hand.
        d.build_linear_gradient("demo-lg-d", |g| {
            g.add_stop(0.0, TEAL)?;
            g.add_stop(1.0, MEDIUM_ORCHID)?;
            g.set_gradient_transform("rotate(45, 0.5, 0.5)")?;
            Ok(())
        })?;

        // 4. Multi-stop sunrise spectrum (4 stops).
        d.build_linear_gradient("demo-lg-s", |g| {
            g.add_stop(0.0, "#1a1a2e")?;
            g.add_stop(0.35, DARK_ORANGE)?;
            g.add_stop(0.65, GOLDENROD)?;
            g.add_stop(1.0, "#fffde7")?;
            Ok(())
        })?;

        // 5. Gradient stroke: a thin-to-thick colour sweep applied to stroke, not fill.
        d.build_linear_gradient("demo-lg-stroke", |g| {
            g.add_stop(0.0, MEDIUM_SEA_GREEN)?;
            g.add_stop(1.0, CORAL)?;
            Ok(())
        })?;

        Ok(())
    })?;

    // `defs` is used only to hold the gradients; so give it a "don't care" name to keep cargo happy
    let _ = defs;

    // Shape dimensions — centred vertically in the BAND.
    let rect_h = 90.0_f64;
    let ry = PAD_Y + (BAND - rect_h) / 2.0;
    let rect_w = 130.0_f64;
    let xs: [f64; 5] = [20.0, 175.0, 330.0, 485.0, 640.0];

    // 1. Horizontal gradient fill.
    let r1 = svg.rect(Point::new(xs[0], ry), Size::new(rect_w, rect_h))?;
    r1.set_fill_gradient("demo-lg-h")?;
    caption(&svg, xs[0] + rect_w / 2.0, "horizontal")?;

    // 2. Vertical gradient fill.
    let r2 = svg.rect(Point::new(xs[1], ry), Size::new(rect_w, rect_h))?;
    r2.set_fill_gradient("demo-lg-v")?;
    caption(&svg, xs[1] + rect_w / 2.0, "vertical")?;

    // 3. Diagonal gradient fill.
    let r3 = svg.rect(Point::new(xs[2], ry), Size::new(rect_w, rect_h))?;
    r3.set_fill_gradient("demo-lg-d")?;
    caption(&svg, xs[2] + rect_w / 2.0, "diagonal (rotate 45°)")?;

    // 4. Multi-stop sunrise spectrum.
    let r4 = svg.rect(Point::new(xs[3], ry), Size::new(rect_w, rect_h))?;
    r4.set_fill_gradient("demo-lg-s")?;
    caption(&svg, xs[3] + rect_w / 2.0, "4-stop spectrum")?;

    // 5. Thick stroked path with gradient stroke (fill=none).
    let stroke_y = PAD_Y + BAND / 2.0;
    let path_str = format!(
        "M {:.1} {:.1} C {:.1} {:.1} {:.1} {:.1} {:.1} {:.1}",
        xs[4],
        stroke_y + 35.0,
        xs[4] + 40.0,
        stroke_y - 45.0,
        xs[4] + 90.0,
        stroke_y + 45.0,
        xs[4] + rect_w,
        stroke_y - 35.0,
    );
    let stroke_path = svg.path(&path_str)?;
    stroke_path.set_fill("none")?;
    stroke_path.set_stroke_gradient("demo-lg-stroke")?;
    stroke_path.set_stroke_width(14.0)?;
    stroke_path.set_attr("stroke-linecap", "round")?;
    caption(&svg, xs[4] + rect_w / 2.0, "gradient stroke")?;

    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// radialGradient — centred, off-centre focal, spreadMethod:reflect, and ellipse fill
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(super) fn demo_radial_gradient() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-radial-gradient", Size::new(W, H))?;

    svg.build_defs(|d| {
        // 1. Centred glow: white hot core fading to deep transparent blue (default cx/cy/r = 0.5/0.5/0.5).
        d.build_radial_gradient("demo-rg-c", |g| {
            g.add_stop_opacity(0.0, "white", 1.0)?;
            g.add_stop_opacity(0.5, STEELBLUE, 0.9)?;
            g.add_stop_opacity(1.0, "#0d1b2a", 1.0)?;
            Ok(())
        })?;

        // 2. Off-centre focal point: the first-stop colour (gold) originates from the upper-left
        //    while the outer colour (deep red) fills the rest.
        d.build_radial_gradient("demo-rg-f", |g| {
            g.add_stop(0.0, GOLDENROD)?;
            g.add_stop(1.0, "#7b0000")?;
            g.set_fx(0.25)?;
            g.set_fy(0.25)?;
            Ok(())
        })?;

        // 3. spreadMethod=reflect with a tight radius (r=0.25) so the pattern tiles visibly.
        //    Two contrasting stops create concentric rings.
        d.build_radial_gradient("demo-rg-r", |g| {
            g.add_stop(0.0, LIGHT_SKY_BLUE)?;
            g.add_stop(1.0, "#00264d")?;
            g.set_r(0.25)?;
            g.set_spread_method(SpreadMethod::Reflect)?;
            Ok(())
        })?;

        // 4. Ellipse with a compressed-Y radial (gradient in objectBoundingBox so it follows the
        //    ellipse shape automatically).  Three stops give a metallic sheen.
        d.build_radial_gradient("demo-rg-e", |g| {
            g.add_stop(0.0, "white")?;
            g.add_stop(0.4, MEDIUM_SEA_GREEN)?;
            g.add_stop(1.0, "#003d1f")?;
            Ok(())
        })?;

        Ok(())
    })?;

    let mid_y = PAD_Y + BAND / 2.0;

    // 1. Centred radial on a circle.
    let c1 = svg.circle(Point::new(75.0, mid_y), 52.0)?;
    c1.set_fill_gradient("demo-rg-c")?;
    caption(&svg, 75.0, "centred")?;

    // 2. Off-centre focal on a rect.
    let r2 = svg.rect(Point::new(155.0, PAD_Y + 10.0), Size::new(155.0, BAND - 20.0))?;
    r2.set_fill_gradient("demo-rg-f")?;
    caption(&svg, 232.5, "off-centre focal")?;

    // 3. Reflected spread on a circle.
    let c3 = svg.circle(Point::new(425.0, mid_y), 52.0)?;
    c3.set_fill_gradient("demo-rg-r")?;
    caption(&svg, 425.0, "spreadMethod: reflect")?;

    // 4. Metallic sheen on an ellipse.
    let e4 = svg.ellipse(Point::new(640.0, mid_y), Size::new(130.0, 52.0))?;
    e4.set_fill_gradient("demo-rg-e")?;
    caption(&svg, 640.0, "ellipse metallic sheen")?;

    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// clipPath — clip a shape, polygon, and group to illustrate three clip-path use cases
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(super) fn demo_clip_path() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-clip-path", Size::new(W, H))?;

    // All clip-path and gradient ids must be globally unique in the document.
    svg.build_defs(|d| {
        // Horizontal linear gradient: steelblue → coral (section 1).
        d.build_linear_gradient("cp-grad-lin", |g| {
            g.add_stop(0.0, STEELBLUE)?;
            g.add_stop(1.0, CORAL)?;
            Ok(())
        })?;
        // Radial gradient: white centre → deep-navy edge (section 2).
        d.build_radial_gradient("cp-grad-rad", |g| {
            g.add_stop(0.0, "white")?;
            g.add_stop(1.0, "#1a237e")?;
            Ok(())
        })?;
        // Clip 1: circle centred at (130, 90).
        d.build_clip_path("cp-circle", |c| {
            c.circle(Point::new(130.0, 90.0), 53.0)?;
            Ok(())
        })?;
        // Clip 2: flat-top hexagon centred at (400, 90), circumradius 55.
        // Vertices: (cx + R·cos(k·60°), cy + R·sin(k·60°)) for k = 0..5.
        d.build_clip_path("cp-hex", |c| {
            c.polygon(&[
                Point::new(455.0, 90.0),
                Point::new(427.5, 137.6),
                Point::new(372.5, 137.6),
                Point::new(345.0, 90.0),
                Point::new(372.5, 42.4),
                Point::new(427.5, 42.4),
            ])?;
            Ok(())
        })?;
        // Clip 3: right-pointing arrow centred at (665, 90).
        // Rectangular body (595..645, y 70..110) plus a triangular head pointing at x = 735.
        d.build_clip_path("cp-arrow", |c| {
            c.path_from_defs(&[
                PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(595.0, 70.0))),
                PathDef::Abs(PathDefAbsolute::LineTo(Point::new(645.0, 70.0))),
                PathDef::Abs(PathDefAbsolute::LineTo(Point::new(645.0, 50.0))),
                PathDef::Abs(PathDefAbsolute::LineTo(Point::new(735.0, 90.0))),
                PathDef::Abs(PathDefAbsolute::LineTo(Point::new(645.0, 130.0))),
                PathDef::Abs(PathDefAbsolute::LineTo(Point::new(645.0, 110.0))),
                PathDef::Abs(PathDefAbsolute::LineTo(Point::new(595.0, 110.0))),
                PathDef::Abs(PathDefAbsolute::ClosePath),
            ])?;
            Ok(())
        })?;
        Ok(())
    })?;

    // Section 1: gradient rectangle revealed through a circular viewport.
    // The rect's bounding box (77,37 + 106×106) matches the circle's bounding box exactly,
    // so the gradient fills the entire circular aperture from edge to edge.
    let r1 = svg.rect(Point::new(77.0, 37.0), Size::new(106.0, 106.0))?;
    r1.set_fill_gradient("cp-grad-lin")?;
    r1.set_clip_path("cp-circle")?;
    caption(&svg, 130.0, "circle clip")?;

    // Section 2: gradient rectangle revealed through a hexagonal frame.
    let r2 = svg.rect(Point::new(345.0, 42.0), Size::new(110.0, 96.0))?;
    r2.set_fill_gradient("cp-grad-rad")?;
    r2.set_clip_path("cp-hex")?;
    caption(&svg, 400.0, "polygon clip (hexagon)")?;

    // Section 3: three coloured horizontal bands clipped to an arrow shape.
    // build_batch_into writes all three rects directly into the group in one DOM operation.
    let arrow_group = svg.group()?;
    svg.build_batch_into(&arrow_group, |b| {
        b.rect(Point::new(595.0, 50.0), Size::new(140.0, 27.0))?.set_fill(STEELBLUE)?;
        b.rect(Point::new(595.0, 77.0), Size::new(140.0, 26.0))?.set_fill(CORAL)?;
        b.rect(Point::new(595.0, 103.0), Size::new(140.0, 27.0))?.set_fill(GOLD)?;
        Ok(())
    })?;
    arrow_group.set_clip_path("cp-arrow")?;
    caption(&svg, 665.0, "path clip on a group")?;

    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// pattern — four named tiles applied as fills
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(super) fn demo_pattern() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-pattern", Size::new(W, H))?;

    // Four named patterns, each defined with patternUnits="userSpaceOnUse" so tile
    // dimensions are in pixel coordinates — easier to reason about than bounding-box fractions.
    svg.build_defs(|d| {
        // 1. Dot grid: white circles on a steelblue field.
        d.build_pattern("demo-pat-dots", |p| {
            p.set_pattern_units(PatternUnits::UserSpaceOnUse)?;
            p.set_width(18.0)?;
            p.set_height(18.0)?;
            p.rect(Point::new(0.0, 0.0), Size::new(18.0, 18.0))?.set_fill(STEELBLUE)?;
            p.circle(Point::new(9.0, 9.0), 5.0)?.set_fill(WHITE)?;
            Ok(())
        })?;

        // 2. Horizontal stripes: alternating coral and white bands.
        d.build_pattern("demo-pat-stripes", |p| {
            p.set_pattern_units(PatternUnits::UserSpaceOnUse)?;
            p.set_width(30.0)?;
            p.set_height(20.0)?;
            p.rect(Point::new(0.0, 0.0), Size::new(30.0, 10.0))?.set_fill(CORAL)?;
            p.rect(Point::new(0.0, 10.0), Size::new(30.0, 10.0))?.set_fill(WHITE)?;
            Ok(())
        })?;

        // 3. Diagonal stripes: horizontal stripes rotated 45° via patternTransform.
        d.build_pattern("demo-pat-diag", |p| {
            p.set_pattern_units(PatternUnits::UserSpaceOnUse)?;
            p.set_width(20.0)?;
            p.set_height(20.0)?;
            p.set_pattern_transform("rotate(45)")?;
            p.rect(Point::new(0.0, 0.0), Size::new(20.0, 10.0))?.set_fill(MEDIUM_ORCHID)?;
            p.rect(Point::new(0.0, 10.0), Size::new(20.0, 10.0))?.set_fill(WHITE)?;
            Ok(())
        })?;

        // 4. Checkerboard: two 10×10 offset squares in a 20×20 tile.
        d.build_pattern("demo-pat-checker", |p| {
            p.set_pattern_units(PatternUnits::UserSpaceOnUse)?;
            p.set_width(20.0)?;
            p.set_height(20.0)?;
            p.rect(Point::new(0.0, 0.0), Size::new(20.0, 20.0))?.set_fill(TEAL)?;
            p.rect(Point::new(0.0, 0.0), Size::new(10.0, 10.0))?.set_fill(WHITE)?;
            p.rect(Point::new(10.0, 10.0), Size::new(10.0, 10.0))?.set_fill(WHITE)?;
            Ok(())
        })?;

        Ok(())
    })?;

    let rect_h = BAND - 20.0;
    let rect_y = PAD_Y + 10.0;
    let rect_w = 160.0_f64;
    let xs: [f64; 4] = [20.0, 210.0, 400.0, 590.0];

    let r1 = svg.rect(Point::new(xs[0], rect_y), Size::new(rect_w, rect_h))?;
    r1.set_fill_pattern("demo-pat-dots")?;
    caption(&svg, xs[0] + rect_w / 2.0, "dot grid")?;

    let r2 = svg.rect(Point::new(xs[1], rect_y), Size::new(rect_w, rect_h))?;
    r2.set_fill_pattern("demo-pat-stripes")?;
    caption(&svg, xs[1] + rect_w / 2.0, "horizontal stripes")?;

    let r3 = svg.rect(Point::new(xs[2], rect_y), Size::new(rect_w, rect_h))?;
    r3.set_fill_pattern("demo-pat-diag")?;
    caption(&svg, xs[2] + rect_w / 2.0, "diagonal (patternTransform)")?;

    let r4 = svg.rect(Point::new(xs[3], rect_y), Size::new(rect_w, rect_h))?;
    r4.set_fill_pattern("demo-pat-checker")?;
    caption(&svg, xs[3] + rect_w / 2.0, "checkerboard")?;

    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// filter — feGaussianBlur at increasing stdDeviation, plus a tinted drop shadow via
// feGaussianBlur -> feFlood -> feComposite -> feOffset -> feMerge, applied via set_filter
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Widens a filter's region from the SVG default (-10%/-10%/120%/120% of the referencing element's bounding box)
// to -50%/-50%/200%/200%, via the typed set_x/set_y/set_width/set_height setters rather than the generic
// set_attrs escape hatch. Shared by every build_filter closure below, all of which need the same wider region to
// avoid visibly clipping their blur or offset shadow.
fn widen_filter_region(f: &SvgFilter) -> Result<(), Error> {
    f.set_x(-0.5)?;
    f.set_y(-0.5)?;
    f.set_width(2.0)?;
    f.set_height(2.0)?;
    Ok(())
}

pub(super) fn demo_filter() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-filter", Size::new(W, H))?;

    // Four blur-only filters at increasing stdDeviation, plus one tinted-drop-shadow filter using the single
    // feDropShadow primitive (the browser-native shorthand for feGaussianBlur -> feFlood -> feComposite ->
    // feOffset -> feMerge; see SvgFilter::drop_shadow's doc comment and design_notes.md for the full chain this
    // collapses). The SVG default filter region (-10%/-10%/120%/120% of the referencing element's bounding box)
    // is too tight for the widest blur and the offset shadow, so every filter here widens its region via the
    // typed set_x/set_y/set_width/set_height setters to avoid visibly clipping the edge.
    svg.build_defs(|d| {
        d.build_filter("demo-filter-0", |f| {
            widen_filter_region(f)?;
            f.gaussian_blur(0.0)?;
            Ok(())
        })?;
        d.build_filter("demo-filter-3", |f| {
            widen_filter_region(f)?;
            f.gaussian_blur(3.0)?;
            Ok(())
        })?;
        d.build_filter("demo-filter-6", |f| {
            widen_filter_region(f)?;
            f.gaussian_blur(6.0)?;
            Ok(())
        })?;
        d.build_filter("demo-filter-12", |f| {
            widen_filter_region(f)?;
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
            widen_filter_region(f)?;
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

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// feColorMatrix — original, greyscale (Saturate), hue-rotated, and sepia (custom Matrix)
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(super) fn demo_color_matrix() -> Result<(), Error> {
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
