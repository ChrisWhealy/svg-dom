use super::colours::*;
use super::{BAND, H, PAD_Y, W, caption};
use crate::{
    Error, PathDef, PathDefAbsolute, SvgFilter, SvgRoot, TextAnchor,
    root::{
        filter::{
            BlendMode, Channel, ColorMatrixType, CompositeOperator, MorphologyOperator, TransferFunction,
            TurbulenceType,
        },
        gradient::SpreadMethod,
        mask::MaskType,
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
// mask — luminance gradient fade, a text-shaped hole, and an alpha mask on a group
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(super) fn demo_mask() -> Result<(), Error> {
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

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// pattern — four named tiles applied as fills
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(super) fn demo_pattern() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-pattern", Size::new(W, H))?;

    // Four named patterns, each defined with patternUnits="userSpaceOnUse" so tile dimensions are in the canvas's user
    // coordinate system which is easier to reason about than bounding-box fractions.
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
    // feOffset -> feMerge; see SvgFilter::drop_shadow's doc comment and docs/design_notes/filters.md for the full chain this
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

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// feBlend — same gradient source, flooded with the same orange tint, across three BlendMode variants
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(super) fn demo_blend() -> Result<(), Error> {
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

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// feComponentTransfer — same gradient source, three TransferFunction variants across different channels
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(super) fn demo_component_transfer() -> Result<(), Error> {
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

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// feTurbulence — raw Perlin noise for both TurbulenceType variants, plus feDisplacementMap warping a circle's
// edge into an organic, hand-drawn outline
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(super) fn demo_turbulence() -> Result<(), Error> {
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
            widen_filter_region(f)?;
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

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// feMorphology — erode/dilate a shape's silhouette, plus dilate + merge for a bold text outline
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(super) fn demo_morphology() -> Result<(), Error> {
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
            widen_filter_region(f)?;
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
