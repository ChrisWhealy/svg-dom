use crate::common::{self, *};
use svg_dom::{Error, root::utils::Point};
use wasm_bindgen_test::*;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// SvgNode marker reference methods
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `set_marker_end` writes `marker-end="url(#id)"` on the element.
#[wasm_bindgen_test]
fn should_set_marker_end_on_line() -> Result<(), String> {
    let svg = make_svg("marker-end-line");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    defs.marker("arrow-end").map_err(|e| e.to_string())?;
    let line = svg
        .line(Point::new(0.0, 0.0), Point::new(100.0, 0.0))
        .map_err(|e| e.to_string())?;
    line.set_marker_end("arrow-end").map_err(|e| e.to_string())?;
    common::check_eq(line.as_element().get_attribute("marker-end"), Some("url(#arrow-end)".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `set_marker_start` writes `marker-start="url(#id)"` on the element.
#[wasm_bindgen_test]
fn should_set_marker_start_on_path() -> Result<(), String> {
    let svg = make_svg("marker-start-path");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    defs.marker("arrow-start").map_err(|e| e.to_string())?;
    let path = svg.path("M 0 50 L 200 50").map_err(|e| e.to_string())?;
    path.set_marker_start("arrow-start").map_err(|e| e.to_string())?;
    common::check_eq(
        path.as_element().get_attribute("marker-start"),
        Some("url(#arrow-start)".into()),
    )
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `set_marker_mid` writes `marker-mid="url(#id)"` on the element.
#[wasm_bindgen_test]
fn should_set_marker_mid_on_polyline() -> Result<(), String> {
    let svg = make_svg("marker-mid-polyline");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    defs.marker("dot-mid").map_err(|e| e.to_string())?;
    let poly = svg
        .polyline(&[Point::new(0.0, 0.0), Point::new(50.0, 50.0), Point::new(100.0, 0.0)])
        .map_err(|e| e.to_string())?;
    poly.set_marker_mid("dot-mid").map_err(|e| e.to_string())?;
    common::check_eq(poly.as_element().get_attribute("marker-mid"), Some("url(#dot-mid)".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// All three marker attributes can be set independently on the same element.
#[wasm_bindgen_test]
fn should_set_all_three_marker_attrs_on_one_element() -> Result<(), String> {
    let svg = make_svg("marker-all-three");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    defs.marker("start-m").map_err(|e| e.to_string())?;
    defs.marker("mid-m").map_err(|e| e.to_string())?;
    defs.marker("end-m").map_err(|e| e.to_string())?;
    let path = svg.path("M 0 50 L 100 50 L 200 50").map_err(|e| e.to_string())?;
    path.set_marker_start("start-m").map_err(|e| e.to_string())?;
    path.set_marker_mid("mid-m").map_err(|e| e.to_string())?;
    path.set_marker_end("end-m").map_err(|e| e.to_string())?;
    let el = path.as_element();
    common::check_eq(el.get_attribute("marker-start"), Some("url(#start-m)".into()))?;
    common::check_eq(el.get_attribute("marker-mid"), Some("url(#mid-m)".into()))?;
    common::check_eq(el.get_attribute("marker-end"), Some("url(#end-m)".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Full arrowhead assembly
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Assembling a complete arrowhead marker and applying it produces the expected DOM structure and attributes.
#[wasm_bindgen_test]
fn should_assemble_arrowhead_marker() -> Result<(), String> {
    let svg = make_svg("arrowhead-full");
    let defs = svg.defs().map_err(|e| e.to_string())?;

    let marker = defs.marker("arrowhead").map_err(|e| e.to_string())?;
    marker.set_ref_x(10.0).map_err(|e| e.to_string())?;
    marker.set_ref_y(3.5).map_err(|e| e.to_string())?;
    marker.set_marker_width(10.0).map_err(|e| e.to_string())?;
    marker.set_marker_height(7.0).map_err(|e| e.to_string())?;
    marker.set_orient("auto").map_err(|e| e.to_string())?;
    marker
        .polygon(&[Point::new(0.0, 0.0), Point::new(10.0, 3.5), Point::new(0.0, 7.0)])
        .map_err(|e| e.to_string())?;

    let line = svg
        .line(Point::new(20.0, 50.0), Point::new(180.0, 50.0))
        .map_err(|e| e.to_string())?;
    line.set_stroke("black").map_err(|e| e.to_string())?;
    line.set_marker_end("arrowhead").map_err(|e| e.to_string())?;

    let el = marker.as_element();
    common::check_eq(el.get_attribute("id"), Some("arrowhead".into()))?;
    common::check_eq(el.get_attribute("refX"), Some("10".into()))?;
    common::check_eq(el.get_attribute("refY"), Some("3.5".into()))?;
    common::check_eq(el.get_attribute("markerWidth"), Some("10".into()))?;
    common::check_eq(el.get_attribute("markerHeight"), Some("7".into()))?;
    common::check_eq(el.get_attribute("orient"), Some("auto".into()))?;
    common::check_eq(line.as_element().get_attribute("marker-end"), Some("url(#arrowhead)".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// set_marker_{start,mid,end}_ref
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `set_marker_end_ref` writes `marker-end="url(#id)"` using the marker handle directly.
#[wasm_bindgen_test]
fn should_set_marker_end_ref_on_line() -> Result<(), String> {
    let svg = make_svg("ref-end");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("ref-arrow").map_err(|e| e.to_string())?;
    let line = svg
        .line(Point::new(0.0, 0.0), Point::new(100.0, 0.0))
        .map_err(|e| e.to_string())?;
    line.set_marker_end_ref(&marker).map_err(|e| e.to_string())?;
    common::check_eq(line.as_element().get_attribute("marker-end"), Some("url(#ref-arrow)".into()))
}

/// `set_marker_start_ref` writes `marker-start="url(#id)"` using the marker handle directly.
#[wasm_bindgen_test]
fn should_set_marker_start_ref_on_path() -> Result<(), String> {
    let svg = make_svg("ref-start");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("ref-dot").map_err(|e| e.to_string())?;
    let path = svg.path("M 0 50 L 200 50").map_err(|e| e.to_string())?;
    path.set_marker_start_ref(&marker).map_err(|e| e.to_string())?;
    common::check_eq(path.as_element().get_attribute("marker-start"), Some("url(#ref-dot)".into()))
}

/// `set_marker_mid_ref` writes `marker-mid="url(#id)"` using the marker handle directly.
#[wasm_bindgen_test]
fn should_set_marker_mid_ref_on_polyline() -> Result<(), String> {
    let svg = make_svg("ref-mid");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("ref-mid-dot").map_err(|e| e.to_string())?;
    let poly = svg
        .polyline(&[Point::new(0.0, 0.0), Point::new(50.0, 50.0), Point::new(100.0, 0.0)])
        .map_err(|e| e.to_string())?;
    poly.set_marker_mid_ref(&marker).map_err(|e| e.to_string())?;
    common::check_eq(poly.as_element().get_attribute("marker-mid"), Some("url(#ref-mid-dot)".into()))
}

/// All three `_ref` methods can be set on the same element using handles from different markers.
#[wasm_bindgen_test]
fn should_set_all_three_marker_refs_on_one_element() -> Result<(), String> {
    let svg = make_svg("ref-all-three");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let start = defs.marker("rs-start").map_err(|e| e.to_string())?;
    let mid = defs.marker("rs-mid").map_err(|e| e.to_string())?;
    let end = defs.marker("rs-end").map_err(|e| e.to_string())?;
    let path = svg.path("M 0 50 L 100 50 L 200 50").map_err(|e| e.to_string())?;
    path.set_marker_start_ref(&start).map_err(|e| e.to_string())?;
    path.set_marker_mid_ref(&mid).map_err(|e| e.to_string())?;
    path.set_marker_end_ref(&end).map_err(|e| e.to_string())?;
    let el = path.as_element();
    common::check_eq(el.get_attribute("marker-start"), Some("url(#rs-start)".into()))?;
    common::check_eq(el.get_attribute("marker-mid"), Some("url(#rs-mid)".into()))?;
    common::check_eq(el.get_attribute("marker-end"), Some("url(#rs-end)".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// set_marker_{start,mid,end} — validation (no SvgMarker required)
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

fn is_invalid_marker_id_unit(result: Result<(), Error>) -> bool {
    matches!(result, Err(Error::InvalidMarkerId(_)))
}

/// `set_marker_start` rejects a whitespace id and leaves the attribute unset.
#[wasm_bindgen_test]
fn should_reject_whitespace_id_in_set_marker_start() -> Result<(), String> {
    let svg = make_svg("set-start-ws");
    let path = svg.path("M 0 50 L 200 50").map_err(|e| e.to_string())?;
    common::check(
        is_invalid_marker_id_unit(path.set_marker_start("bad id")),
        "set_marker_start: whitespace in id must be rejected",
    )?;
    common::check(
        path.as_element().get_attribute("marker-start").is_none(),
        "set_marker_start: attribute must remain unset after rejection",
    )
}

/// `set_marker_mid` rejects a `url(` prefix and leaves the attribute unset.
#[wasm_bindgen_test]
fn should_reject_url_prefix_in_set_marker_mid() -> Result<(), String> {
    let svg = make_svg("set-mid-url");
    let path = svg.path("M 0 50 L 100 50 L 200 50").map_err(|e| e.to_string())?;
    common::check(
        is_invalid_marker_id_unit(path.set_marker_mid("url(#arrow)")),
        "set_marker_mid: url( prefix must be rejected",
    )?;
    common::check(
        path.as_element().get_attribute("marker-mid").is_none(),
        "set_marker_mid: attribute must remain unset after rejection",
    )
}

/// `set_marker_end` rejects an id containing `(` and leaves the attribute unset.
#[wasm_bindgen_test]
fn should_reject_open_paren_in_set_marker_end() -> Result<(), String> {
    let svg = make_svg("set-end-open-paren");
    let path = svg.path("M 0 50 L 200 50").map_err(|e| e.to_string())?;
    common::check(
        is_invalid_marker_id_unit(path.set_marker_end("arrow(left")),
        "set_marker_end: ( in id must be rejected",
    )?;
    common::check(
        path.as_element().get_attribute("marker-end").is_none(),
        "set_marker_end: attribute must remain unset after rejection",
    )
}
