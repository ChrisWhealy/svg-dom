use crate::{common, helpers::make_svg};
use svg_dom::root::utils::{Matrix2D, Point, Size};
use wasm_bindgen_test::*;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Geometry read-back: bounding_box / total_length / point_at_length / ctm / screen_ctm / bounding_client_rect
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Tolerance for browser-measured geometry comparisons (`bounding_box`, `bounding_client_rect`, `total_length`,
/// `point_at_length`) — these are real layout/paint reads, not pure computation, so sub-pixel slack is expected.
const GEOM_EPS: f64 = 0.5;

/// Tighter tolerance for matrix-component comparisons (`ctm`/`screen_ctm` fields) against values this test suite
/// itself wrote via `set_matrix_precise`/`set_translate`/`set_scale`. The only expected divergence there is the
/// `f32` round-trip through the legacy SVG DOM matrix types (see `src/node/geometry.rs`'s module doc comment) —
/// `f32` carries roughly 7 significant decimal digits, so `1e-4` comfortably separates "expected `f32` noise" from
/// a real mapping bug (e.g. a transposed `b`/`c` or `a`/`d` field) without being so tight it flakes on rounding.
const MATRIX_EPS: f64 = 1e-4;

fn approx(got: f64, expected: f64) -> Result<(), String> {
    common::check(
        (got - expected).abs() < GEOM_EPS,
        &format!("expected approximately {expected}, got {got}"),
    )
}

fn approx_tight(got: f64, expected: f64) -> Result<(), String> {
    common::check(
        (got - expected).abs() < MATRIX_EPS,
        &format!("expected approximately {expected} (tight), got {got}"),
    )
}

/// `bounding_box` reports a known rect's `x`/`y`/`width`/`height` in local coordinates.
#[wasm_bindgen_test]
fn should_report_bounding_box_of_a_known_rect() -> Result<(), String> {
    let rect = make_svg("node-bbox")
        .rect(Point::new(10.0, 10.0), Size::new(80.0, 40.0))
        .map_err(|e| e.to_string())?;
    let bbox = rect.bounding_box().map_err(|e| e.to_string())?;
    approx(bbox.origin.x, 10.0)?;
    approx(bbox.origin.y, 10.0)?;
    approx(bbox.size.width, 80.0)?;
    approx(bbox.size.height, 40.0)
}

/// `total_length` on a `<rect>` matches its perimeter, per SVG's definition of a rect's implicit path.
#[wasm_bindgen_test]
fn should_report_total_length_of_a_known_rect() -> Result<(), String> {
    let rect = make_svg("node-total-length-rect")
        .rect(Point::new(10.0, 10.0), Size::new(80.0, 40.0))
        .map_err(|e| e.to_string())?;
    let length = rect.total_length().ok_or("expected Some(length) for a <rect>")?;
    approx(length, 2.0 * (80.0 + 40.0))
}

/// `total_length` returns `None` for a `<text>` node — `SVGGeometryElement` does not apply to text content.
#[wasm_bindgen_test]
fn should_return_none_for_total_length_on_a_text_element() -> Result<(), String> {
    let text = make_svg("node-total-length-text")
        .text(Point::origin(), "hello")
        .map_err(|e| e.to_string())?;
    common::check_eq(text.total_length(), None)
}

/// `total_length` returns `None` for a `<g>` node — a container has no geometry of its own.
#[wasm_bindgen_test]
fn should_return_none_for_total_length_on_a_group() -> Result<(), String> {
    let group = make_svg("node-total-length-group").group().map_err(|e| e.to_string())?;
    common::check_eq(group.total_length(), None)
}

/// A filter primitive (`<feGaussianBlur>`, returned as a plain `SvgNode` by `SvgFilter::gaussian_blur`) fails every
/// geometry interface check — `SVGGraphicsElement` and `SVGGeometryElement` alike — because filter primitives are
/// non-rendering elements, unlike every shape/text/container factory this crate exposes. This proves the interface
/// gate is reachable through the crate's own API, not only through raw `<text>`/`<g>` nodes checked against
/// `SVGGeometryElement` alone: `bounding_box`, `ctm`, and `screen_ctm` are all gated on `SVGGraphicsElement`, which
/// a filter primitive does not implement either.
#[wasm_bindgen_test]
fn should_fail_every_geometry_interface_check_on_a_filter_primitive() -> Result<(), String> {
    let svg = make_svg("node-geometry-filter-primitive");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("blur-filter").map_err(|e| e.to_string())?;
    let blur = filter.gaussian_blur(4.0).map_err(|e| e.to_string())?;

    common::check(
        blur.bounding_box().is_err(),
        "expected Err: feGaussianBlur is not SVGGraphicsElement",
    )?;
    common::check_eq(blur.ctm(), None)?;
    common::check_eq(blur.screen_ctm(), None)?;
    common::check_eq(blur.total_length(), None)?;
    common::check(
        blur.point_at_length(0.0).is_err(),
        "expected Err: feGaussianBlur is not SVGGeometryElement",
    )
}

/// `point_at_length` returns `Err` for a `<text>` node, the `Result`-returning sibling of `total_length`'s `None`.
#[wasm_bindgen_test]
fn should_return_err_for_point_at_length_on_a_text_element() -> Result<(), String> {
    let text = make_svg("node-point-at-length-text")
        .text(Point::origin(), "hello")
        .map_err(|e| e.to_string())?;
    common::check(text.point_at_length(0.0).is_err(), "expected Err for a <text> node")
}

/// `point_at_length(0.0)` on a `<rect>` reports its starting corner (top-left, per SVG's rect-as-path definition).
#[wasm_bindgen_test]
fn should_report_point_at_length_zero_as_path_start() -> Result<(), String> {
    let rect = make_svg("node-point-at-length-start")
        .rect(Point::new(10.0, 10.0), Size::new(80.0, 40.0))
        .map_err(|e| e.to_string())?;
    let start = rect.point_at_length(0.0).map_err(|e| e.to_string())?;
    approx(start.x, 10.0)?;
    approx(start.y, 10.0)
}

/// A negative `distance` clamps to the path's start point, on a straight open path where start and end are
/// unambiguously distinct (unlike a closed `<rect>`, whose implicit closing `Z` puts its start and end at the same
/// point).
#[wasm_bindgen_test]
fn should_clamp_negative_point_at_length_distance_to_start() -> Result<(), String> {
    let line = make_svg("node-point-at-length-negative")
        .path("M10 10L90 10")
        .map_err(|e| e.to_string())?;
    let point = line.point_at_length(-50.0).map_err(|e| e.to_string())?;
    approx(point.x, 10.0)?;
    approx(point.y, 10.0)
}

/// A `distance` beyond `total_length()` clamps to the path's end point, the same open-path fixture as the negative
/// case above so start and end are unambiguously distinct.
#[wasm_bindgen_test]
fn should_clamp_point_at_length_distance_beyond_total_length_to_end() -> Result<(), String> {
    let line = make_svg("node-point-at-length-beyond-total")
        .path("M10 10L90 10")
        .map_err(|e| e.to_string())?;
    let total = line.total_length().ok_or("expected Some(length) for a <path>")?;
    let point = line.point_at_length(total + 50.0).map_err(|e| e.to_string())?;
    approx(point.x, 90.0)?;
    approx(point.y, 10.0)
}

/// `point_at_length` rejects `NaN` and both infinities without ever crossing into the browser.
#[wasm_bindgen_test]
fn should_reject_non_finite_point_at_length_distance() -> Result<(), String> {
    let rect = make_svg("node-point-at-length-non-finite")
        .rect(Point::new(10.0, 10.0), Size::new(80.0, 40.0))
        .map_err(|e| e.to_string())?;
    common::check(rect.point_at_length(f64::NAN).is_err(), "expected Err for NaN")?;
    common::check(rect.point_at_length(f64::INFINITY).is_err(), "expected Err for +infinity")?;
    common::check(rect.point_at_length(f64::NEG_INFINITY).is_err(), "expected Err for -infinity")
}

/// A finite `distance` far beyond `f32::MAX` (e.g. `f64::MAX`) is saturated to `f32::MAX`/`f32::MIN`, not left to
/// overflow into an actual `f32` infinity — it clamps to the path's end/start exactly like any other out-of-range
/// finite distance, rather than erroring the way an actually-infinite distance does.
#[wasm_bindgen_test]
fn should_saturate_out_of_f32_range_finite_distance_instead_of_erroring() -> Result<(), String> {
    let rect = make_svg("node-point-at-length-saturate")
        .rect(Point::new(10.0, 10.0), Size::new(80.0, 40.0))
        .map_err(|e| e.to_string())?;
    let total = rect.total_length().ok_or("expected Some(length) for a <rect>")?;

    let end = rect.point_at_length(f64::MAX).map_err(|e| e.to_string())?;
    let start_via_min = rect.point_at_length(f64::MIN).map_err(|e| e.to_string())?;
    let start_via_negative = rect.point_at_length(-1.0).map_err(|e| e.to_string())?;

    // f64::MAX saturates to a huge but finite positive f32, which clamps to the path's end, the same as any other
    // distance past `total_length`.
    let expected_end = rect.point_at_length(total).map_err(|e| e.to_string())?;
    approx(end.x, expected_end.x)?;
    approx(end.y, expected_end.y)?;

    // f64::MIN saturates to a huge but finite negative f32, which clamps to the path's start, same as any ordinary
    // negative distance.
    approx(start_via_min.x, start_via_negative.x)?;
    approx(start_via_min.y, start_via_negative.y)
}

/// `ctm` on an untransformed top-level `<rect>` is approximately the identity matrix.
#[wasm_bindgen_test]
fn should_report_identity_ctm_for_untransformed_rect() -> Result<(), String> {
    let rect = make_svg("node-ctm-identity")
        .rect(Point::origin(), Size::new(80.0, 40.0))
        .map_err(|e| e.to_string())?;
    let ctm = rect.ctm().ok_or("expected Some(ctm) for a rendered <rect>")?;
    approx_tight(ctm.h_scale, 1.0)?;
    approx_tight(ctm.v_scale, 1.0)?;
    approx_tight(ctm.h_skew, 0.0)?;
    approx_tight(ctm.v_skew, 0.0)?;
    approx_tight(ctm.h_trans, 0.0)?;
    approx_tight(ctm.v_trans, 0.0)
}

/// `ctm` reflects a `set_translate` round-trip: what was written is what comes back.
#[wasm_bindgen_test]
fn should_reflect_translate_in_ctm() -> Result<(), String> {
    let rect = make_svg("node-ctm-translate")
        .rect(Point::origin(), Size::new(80.0, 40.0))
        .map_err(|e| e.to_string())?;
    let mut buf = String::new();
    rect.set_translate(&mut buf, 25.0, 15.0).map_err(|e| e.to_string())?;
    let ctm = rect.ctm().ok_or("expected Some(ctm) for a rendered <rect>")?;
    approx_tight(ctm.h_trans, 25.0)?;
    approx_tight(ctm.v_trans, 15.0)
}

/// `ctm()` correctly maps all six `SVGMatrix` components back onto `Matrix2D`'s role-named fields. The identity and
/// single-axis-translation fixtures above cannot catch a mapping error involving `b`/`c` (`v_skew`/`h_skew`) or
/// `a`/`d` (`h_scale`/`v_scale`), since those fields are all `0`/`1`/equal-to-each-other there; six distinct values
/// (mirroring the `set_matrix` argument-order fixture) can.
#[wasm_bindgen_test]
fn should_read_back_all_six_distinct_ctm_components() -> Result<(), String> {
    let rect = make_svg("node-ctm-six-components")
        .rect(Point::origin(), Size::new(80.0, 40.0))
        .map_err(|e| e.to_string())?;
    let mut buf = String::new();
    rect.set_matrix_precise(
        &mut buf,
        Matrix2D {
            h_scale: 1.1,
            v_scale: 2.2,
            h_skew: 3.3,
            v_skew: 4.4,
            h_trans: 5.5,
            v_trans: -6.6,
        },
    )
    .map_err(|e| e.to_string())?;

    let ctm = rect.ctm().ok_or("expected Some(ctm) for a rendered <rect>")?;
    approx_tight(ctm.h_scale, 1.1)?;
    approx_tight(ctm.v_scale, 2.2)?;
    approx_tight(ctm.h_skew, 3.3)?;
    approx_tight(ctm.v_skew, 4.4)?;
    approx_tight(ctm.h_trans, 5.5)?;
    approx_tight(ctm.v_trans, -6.6)
}

/// `ctm` accumulates every ancestor transform up to the nearest *viewport* ancestor — here, the root `<svg>` itself,
/// since nothing between the rect and the root establishes a nested viewport. So a rect nested inside a translated
/// `<g>` reports the *combined* chain, not just its own translate.
#[wasm_bindgen_test]
fn should_accumulate_ancestor_transforms_in_ctm_up_to_the_root_viewport() -> Result<(), String> {
    let svg = make_svg("node-ctm-accumulates");
    let group = svg.group().map_err(|e| e.to_string())?;
    let mut buf = String::new();
    group.set_translate(&mut buf, 100.0, 0.0).map_err(|e| e.to_string())?;

    let child = svg.rect(Point::origin(), Size::new(80.0, 40.0)).map_err(|e| e.to_string())?;
    group.append(&child).map_err(|e| e.to_string())?;
    child.set_translate(&mut buf, 0.0, 50.0).map_err(|e| e.to_string())?;

    let ctm = child.ctm().ok_or("expected Some(ctm) for the nested <rect>")?;
    approx_tight(ctm.h_trans, 100.0)?;
    approx_tight(ctm.v_trans, 50.0)
}

/// `ctm` composes ancestor and own transforms in the mathematically correct order (`ctm_child = ctm_parent ·
/// local_child`). Two translations alone (as above) cannot catch a reversed multiplication order, because
/// translation matrices commute — `translate(a) · translate(b) == translate(b) · translate(a)` either way. A parent
/// *scale* combined with a child *translate* does not commute, so this test fails under either transposed order:
/// correct composition places the child's own `translate(10, 0)` in its local space first, then the parent's
/// `scale(2)` maps that point out to `(20, 0)`; the reversed order would instead give `(10, 0)`.
#[wasm_bindgen_test]
fn should_accumulate_non_commuting_ancestor_transform_in_correct_order() -> Result<(), String> {
    let svg = make_svg("node-ctm-non-commuting");
    let group = svg.group().map_err(|e| e.to_string())?;
    let mut buf = String::new();
    group.set_scale(&mut buf, 2.0).map_err(|e| e.to_string())?;

    let child = svg.rect(Point::origin(), Size::new(10.0, 10.0)).map_err(|e| e.to_string())?;
    group.append(&child).map_err(|e| e.to_string())?;
    child.set_translate(&mut buf, 10.0, 0.0).map_err(|e| e.to_string())?;

    let ctm = child.ctm().ok_or("expected Some(ctm) for the nested <rect>")?;
    approx_tight(ctm.h_scale, 2.0)?;
    approx_tight(ctm.v_scale, 2.0)?;
    approx_tight(ctm.h_trans, 20.0)?;
    approx_tight(ctm.v_trans, 0.0)
}

/// `screen_ctm` additionally carries the root `<svg>`'s own position on the page, which `ctm` never reflects (`ctm`
/// stops at the nearest viewport ancestor, i.e. the root `<svg>` itself here). A tall spacer placed immediately
/// before the `<svg>` guarantees a large, known vertical page offset, regardless of whatever earlier tests already
/// left in the document (this suite has no teardown hooks between tests).
#[wasm_bindgen_test]
fn should_reflect_page_position_in_screen_ctm_but_not_ctm() -> Result<(), String> {
    let spacer = common::div("node-ctm-vs-screen-ctm-spacer");
    spacer.set_attribute("style", "height:600px").map_err(|e| format!("{e:?}"))?;

    let rect = make_svg("node-ctm-vs-screen-ctm")
        .rect(Point::origin(), Size::new(80.0, 40.0))
        .map_err(|e| e.to_string())?;

    let ctm = rect.ctm().ok_or("expected Some(ctm) for the rect")?;
    let screen_ctm = rect.screen_ctm().ok_or("expected Some(screen_ctm) for the rect")?;

    // Untransformed, and nothing between it and the root <svg> — ctm carries no page-position information at all.
    approx_tight(ctm.h_trans, 0.0)?;
    approx_tight(ctm.v_trans, 0.0)?;

    // screen_ctm additionally carries the root <svg>'s own position on the page, at least the spacer's height.
    common::check(
        screen_ctm.v_trans - ctm.v_trans > 500.0,
        &format!(
            "expected screen_ctm to include a large page offset beyond ctm, got ctm={ctm:?} screen_ctm={screen_ctm:?}"
        ),
    )
}

/// `bounding_client_rect` on a rendered rect is a real, positive measurement. Exact pixel values are not asserted
/// (they vary with the headless test runner's viewport), and it is not expected to equal `bounding_box`'s numbers —
/// the two report different coordinate spaces (see `Rect`'s own doc comment).
#[wasm_bindgen_test]
fn should_report_bounding_client_rect_as_nonzero_for_rendered_element() -> Result<(), String> {
    let rect = make_svg("node-bounding-client-rect")
        .rect(Point::new(10.0, 10.0), Size::new(80.0, 40.0))
        .map_err(|e| e.to_string())?;
    let client_rect = rect.bounding_client_rect();
    common::check(client_rect.size.width > 0.0, "expected a positive width")?;
    common::check(client_rect.size.height > 0.0, "expected a positive height")
}

/// Turns the documented coordinate-space distinction between `bounding_box` (local, excludes the element's own
/// `transform`) and `bounding_client_rect` (viewport, includes it) into an executable invariant: translating the
/// element leaves `bounding_box` unchanged but moves `bounding_client_rect` by exactly the same amount.
#[wasm_bindgen_test]
fn should_diverge_bounding_box_and_bounding_client_rect_under_transform() -> Result<(), String> {
    let rect = make_svg("node-bbox-vs-client-rect-transform")
        .rect(Point::new(10.0, 10.0), Size::new(80.0, 40.0))
        .map_err(|e| e.to_string())?;

    let bbox_before = rect.bounding_box().map_err(|e| e.to_string())?;
    let client_before = rect.bounding_client_rect();

    let mut buf = String::new();
    rect.set_translate(&mut buf, 300.0, 300.0).map_err(|e| e.to_string())?;

    let bbox_after = rect.bounding_box().map_err(|e| e.to_string())?;
    let client_after = rect.bounding_client_rect();

    // bounding_box excludes the element's own transform — unaffected by the translate.
    approx(bbox_after.origin.x, bbox_before.origin.x)?;
    approx(bbox_after.origin.y, bbox_before.origin.y)?;

    // bounding_client_rect includes it — the on-screen position moves by exactly (300, 300).
    approx(client_after.origin.x - client_before.origin.x, 300.0)?;
    approx(client_after.origin.y - client_before.origin.y, 300.0)
}
