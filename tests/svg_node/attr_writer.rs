use crate::{common, helpers::make_svg};
use svg_dom::root::utils::{Point, Size};
use wasm_bindgen_test::*;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// SvgAttrs / AttrWriter
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `SvgAttrs` reuses its scratch buffer while setting string, numeric and formatted attributes.
#[wasm_bindgen_test]
fn should_set_attributes_with_reusable_attr_writer() -> Result<(), String> {
    let rect = make_svg("node-svg-attrs")
        .rect(Point::origin(), Size::new(50.0, 50.0))
        .map_err(|e| e.to_string())?;
    let mut attrs = svg_dom::SvgAttrs::with_capacity(64);

    rect.attrs(&mut attrs)
        .fill("steelblue")
        .map_err(|e| e.to_string())?
        .stroke("white")
        .map_err(|e| e.to_string())?
        .stroke_width(2.5)
        .map_err(|e| e.to_string())?
        .fmt("transform", format_args!("translate({}, {})", 10, 20))
        .map_err(|e| e.to_string())?;

    common::check_eq(rect.attr("fill"), Some("steelblue".into()))?;
    common::check_eq(rect.attr("stroke"), Some("white".into()))?;
    common::check_eq(rect.attr("stroke-width"), Some("2.5".into()))?;
    common::check_eq(rect.attr("transform"), Some("translate(10, 20)".into()))?;
    common::check(attrs.capacity() >= 64, "SvgAttrs should retain its scratch allocation")
}

/// `AttrWriter::points` updates a polyline's `points` attribute through the reusable buffer, and the same buffer can be
/// reused for a second update (the latest value wins) — the allocation-light animation path.
#[wasm_bindgen_test]
fn should_update_points_via_reusable_buffer() -> Result<(), String> {
    let svg = make_svg("node-points");
    let poly = svg
        .polyline(&[Point::new(0.0, 0.0), Point::new(10.0, 10.0)])
        .map_err(|e| e.to_string())?;
    common::check_eq(poly.attr("points"), Some("0,0 10,10".into()))?;

    let mut attrs = svg_dom::SvgAttrs::new();
    poly.attrs(&mut attrs)
        .points(&[Point::new(1.0, 2.0), Point::new(3.0, 4.0), Point::new(5.0, 6.0)])
        .map_err(|e| e.to_string())?;
    common::check_eq(poly.attr("points"), Some("1,2 3,4 5,6".into()))?;

    // Reuse the same buffer for another update.
    poly.attrs(&mut attrs)
        .points(&[Point::new(7.0, 8.0)])
        .map_err(|e| e.to_string())?;
    common::check_eq(poly.attr("points"), Some("7,8".into()))
}

/// `AnimationFrame::set_points` writes a polyline's `points` through the frame's reusable buffer (the per-frame
/// counterpart to `SvgAttrs::points`).
#[wasm_bindgen_test]
fn should_update_points_via_animation_frame() -> Result<(), String> {
    let svg = make_svg("node-frame-points");
    let poly = svg.polyline(&[Point::origin()]).map_err(|e| e.to_string())?;

    let mut frame = svg_dom::AnimationFrame::new();
    frame
        .set_points(&poly, &[Point::new(1.0, 2.0), Point::new(3.0, 4.0)])
        .map_err(|e| e.to_string())?;
    common::check_eq(poly.attr("points"), Some("1,2 3,4".into()))?;

    // Reuse across frames: the latest value wins.
    frame.set_points(&poly, &[Point::new(5.0, 6.0)]).map_err(|e| e.to_string())?;
    common::check_eq(poly.attr("points"), Some("5,6".into()))
}

/// `points_fixed` writes each coordinate at the requested fixed precision (rounding), trimming the `points` string;
/// `AnimationFrame::set_points_fixed` produces the same output through the frame buffer.
#[wasm_bindgen_test]
fn should_write_fixed_precision_points() -> Result<(), String> {
    let svg = make_svg("node-points-fixed");
    let poly = svg.polyline(&[Point::origin()]).map_err(|e| e.to_string())?;

    // Via the chainable writer (also exercises SvgAttrs::points_fixed): 1 decimal place, with rounding.
    let mut attrs = svg_dom::SvgAttrs::new();
    poly.attrs(&mut attrs)
        .points_fixed(&[Point::new(1.23456, 2.0), Point::new(3.0, 4.98765)], 1)
        .map_err(|e| e.to_string())?;
    common::check_eq(poly.attr("points"), Some("1.2,2.0 3.0,5.0".into()))?;

    // The AnimationFrame counterpart, at 0 decimals (integer rounding).
    let mut frame = svg_dom::AnimationFrame::new();
    frame
        .set_points_fixed(&poly, &[Point::new(1.6, 2.4)], 0)
        .map_err(|e| e.to_string())?;
    common::check_eq(poly.attr("points"), Some("2,2".into()))
}

/// `AttrWriter::d_from_defs` updates a path's `d` attribute through the reusable buffer, and the same buffer can be
/// reused for a second update (the latest value wins) — the allocation-light counterpart to `SvgNode::set_d_from_defs`.
#[wasm_bindgen_test]
fn should_update_d_via_reusable_buffer() -> Result<(), String> {
    use svg_dom::{PathDef, PathDefAbsolute};

    let svg = make_svg("node-d-from-defs");
    let path = svg.path("M 0 0 L 1 1").map_err(|e| e.to_string())?;

    let mut attrs = svg_dom::SvgAttrs::new();
    path.attrs(&mut attrs)
        .d_from_defs(&[
            PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(0.0, 0.0))),
            PathDef::Abs(PathDefAbsolute::LineTo(Point::new(100.0, 100.0))),
        ])
        .map_err(|e| e.to_string())?;
    common::check_eq(path.attr("d"), Some("M0 0L100 100".into()))?;

    // Reuse the same buffer for another update.
    path.attrs(&mut attrs)
        .d_from_defs(&[PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(5.0, 5.0)))])
        .map_err(|e| e.to_string())?;
    common::check_eq(path.attr("d"), Some("M5 5".into()))
}

/// `AnimationFrame::set_d_from_defs` writes a path's `d` attribute through the frame's reusable buffer (the
/// per-frame counterpart to `SvgAttrs::d_from_defs`).
#[wasm_bindgen_test]
fn should_update_d_via_animation_frame() -> Result<(), String> {
    use svg_dom::{PathDef, PathDefAbsolute};

    let svg = make_svg("node-frame-d-from-defs");
    let path = svg.path("M 0 0 L 1 1").map_err(|e| e.to_string())?;

    let mut frame = svg_dom::AnimationFrame::new();
    frame
        .set_d_from_defs(
            &path,
            &[
                PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(0.0, 0.0))),
                PathDef::Abs(PathDefAbsolute::LineTo(Point::new(20.0, 20.0))),
            ],
        )
        .map_err(|e| e.to_string())?;
    common::check_eq(path.attr("d"), Some("M0 0L20 20".into()))?;

    // Reuse across frames: the latest value wins.
    frame
        .set_d_from_defs(&path, &[PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(9.0, 9.0)))])
        .map_err(|e| e.to_string())?;
    common::check_eq(path.attr("d"), Some("M9 9".into()))
}

/// `SvgRoot::path_from_defs` (and therefore every `path_from_defs` factory sibling) now writes `d` through the
/// factory's own retained `SvgAttrs` buffer rather than allocating a fresh `String` per call — this exercises that
/// path directly, distinct from the `SvgNode::set_d_from_defs` update path covered above.
#[wasm_bindgen_test]
fn should_create_path_from_defs_reusing_factory_buffer() -> Result<(), String> {
    use svg_dom::{PathDef, PathDefAbsolute};

    let svg = make_svg("node-path-from-defs-factory");
    let first = svg
        .path_from_defs(&[
            PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(0.0, 0.0))),
            PathDef::Abs(PathDefAbsolute::LineTo(Point::new(1.0, 1.0))),
        ])
        .map_err(|e| e.to_string())?;
    let second = svg
        .path_from_defs(&[PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(2.0, 2.0)))])
        .map_err(|e| e.to_string())?;

    common::check_eq(first.attr("d"), Some("M0 0L1 1".into()))?;
    common::check_eq(second.attr("d"), Some("M2 2".into()))
}

/// `AttrWriter::d_from_defs_fixed` writes each coordinate at the requested fixed precision (rounding), trimming the
/// `d` string; `AnimationFrame::set_d_from_defs_fixed` produces the same output through the frame buffer.
/// The elliptical-arc flags are never rounded, since the SVG `flag` grammar production is a single `"0"`/`"1"`
/// digit, not a decimal number.
#[wasm_bindgen_test]
fn should_write_fixed_precision_d_from_defs() -> Result<(), String> {
    use svg_dom::{ArcSize, ArcSweep, EllipticalArc, PathDef, PathDefAbsolute};

    let svg = make_svg("node-d-from-defs-fixed");
    let path = svg.path("M 0 0").map_err(|e| e.to_string())?;

    // Via the chainable writer (also exercises SvgAttrs::d_from_defs_fixed): 1 decimal place, with rounding.
    let mut attrs = svg_dom::SvgAttrs::new();
    path.attrs(&mut attrs)
        .d_from_defs_fixed(
            &[
                PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(1.23456, 2.0))),
                PathDef::Abs(PathDefAbsolute::EllipticalArcTo(EllipticalArc {
                    radii: Point::new(1.0 / 3.0, 1.0 / 3.0),
                    x_axis_rotation: 0.0,
                    size: ArcSize::Large,
                    sweep: ArcSweep::Clockwise,
                    to: Point::new(3.0, 4.98765),
                })),
            ],
            1,
        )
        .map_err(|e| e.to_string())?;
    common::check_eq(path.attr("d"), Some("M1.2 2.0A0.3 0.3 0.0 1 1 3.0 5.0".into()))?;

    // The AnimationFrame counterpart, at 0 decimals (integer rounding); flags still un-rounded.
    let mut frame = svg_dom::AnimationFrame::new();
    frame
        .set_d_from_defs_fixed(
            &path,
            &[
                PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(0.0, 0.0))),
                PathDef::Abs(PathDefAbsolute::EllipticalArcTo(EllipticalArc {
                    radii: Point::new(1.6, 2.4),
                    x_axis_rotation: 0.0,
                    size: ArcSize::Small,
                    sweep: ArcSweep::CounterClockwise,
                    to: Point::new(1.6, 2.4),
                })),
            ],
            0,
        )
        .map_err(|e| e.to_string())?;
    common::check_eq(path.attr("d"), Some("M0 0A2 2 0 0 0 2 2".into()))
}
