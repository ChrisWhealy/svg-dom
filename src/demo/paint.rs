use crate::{Error, SvgRoot, root::{gradient::SpreadMethod, utils::{Point, Size}}};
use super::colours::*;
use super::{W, H, BAND, PAD_Y, caption};

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
            c.path("M 595,70 L 645,70 L 645,50 L 735,90 L 645,130 L 645,110 L 595,110 Z")?;
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
