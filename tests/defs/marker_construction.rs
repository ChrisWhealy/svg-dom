use crate::common::{self, *};
use svg_dom::{Error, MarkerUnits};
use wasm_bindgen_test::*;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// SvgMarker construction
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `defs.marker("id")` creates a `<marker>` element with the correct tag name.
#[wasm_bindgen_test]
fn should_create_marker_element() -> Result<(), String> {
    let svg = make_svg("marker-create");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("arrow").map_err(|e| e.to_string())?;
    common::check_eq(marker.as_element().tag_name(), "marker".to_owned())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// The `<marker>` element carries the supplied `id` attribute.
#[wasm_bindgen_test]
fn should_set_marker_id_attribute() -> Result<(), String> {
    let svg = make_svg("marker-id-attr");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("my-arrow").map_err(|e| e.to_string())?;
    common::check_eq(marker.as_element().get_attribute("id"), Some("my-arrow".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `SvgMarker::id()` returns the id string the marker was created with.
#[wasm_bindgen_test]
fn should_expose_marker_id_via_method() -> Result<(), String> {
    let svg = make_svg("marker-id-method");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("dot").map_err(|e| e.to_string())?;
    common::check_eq(marker.id(), "dot")
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// The `<marker>` element is a child of the `<defs>` element.
#[wasm_bindgen_test]
fn should_append_marker_to_defs() -> Result<(), String> {
    let svg = make_svg("marker-defs-parent");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("arrowhead").map_err(|e| e.to_string())?;
    let parent = marker.as_element().parent_element().ok_or("marker has no parent")?;
    common::check_eq(parent.tag_name(), "defs".to_owned())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// SvgMarker attribute setters
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `set_ref_x` writes the `refX` attribute.
#[wasm_bindgen_test]
fn should_set_marker_ref_x() -> Result<(), String> {
    let svg = make_svg("marker-ref-x");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("m").map_err(|e| e.to_string())?;
    marker.set_ref_x(10.0).map_err(|e| e.to_string())?;
    common::check_eq(marker.as_element().get_attribute("refX"), Some("10".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `set_ref_y` writes the `refY` attribute.
#[wasm_bindgen_test]
fn should_set_marker_ref_y() -> Result<(), String> {
    let svg = make_svg("marker-ref-y");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("m").map_err(|e| e.to_string())?;
    marker.set_ref_y(3.5).map_err(|e| e.to_string())?;
    common::check_eq(marker.as_element().get_attribute("refY"), Some("3.5".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `set_marker_width` writes the `markerWidth` attribute.
#[wasm_bindgen_test]
fn should_set_marker_width() -> Result<(), String> {
    let svg = make_svg("marker-width");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("m").map_err(|e| e.to_string())?;
    marker.set_marker_width(10.0).map_err(|e| e.to_string())?;
    common::check_eq(marker.as_element().get_attribute("markerWidth"), Some("10".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `set_marker_height` writes the `markerHeight` attribute.
#[wasm_bindgen_test]
fn should_set_marker_height() -> Result<(), String> {
    let svg = make_svg("marker-height");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("m").map_err(|e| e.to_string())?;
    marker.set_marker_height(7.0).map_err(|e| e.to_string())?;
    common::check_eq(marker.as_element().get_attribute("markerHeight"), Some("7".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `set_orient` writes the `orient` attribute.
#[wasm_bindgen_test]
fn should_set_marker_orient_auto() -> Result<(), String> {
    let svg = make_svg("marker-orient-auto");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("m").map_err(|e| e.to_string())?;
    marker.set_orient("auto").map_err(|e| e.to_string())?;
    common::check_eq(marker.as_element().get_attribute("orient"), Some("auto".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `set_orient` also accepts angle values.
#[wasm_bindgen_test]
fn should_set_marker_orient_angle() -> Result<(), String> {
    let svg = make_svg("marker-orient-angle");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("m").map_err(|e| e.to_string())?;
    marker.set_orient("45deg").map_err(|e| e.to_string())?;
    common::check_eq(marker.as_element().get_attribute("orient"), Some("45deg".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `set_units(StrokeWidth)` writes `markerUnits="strokeWidth"`.
#[wasm_bindgen_test]
fn should_set_marker_units_stroke_width() -> Result<(), String> {
    let svg = make_svg("marker-units-sw");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("m").map_err(|e| e.to_string())?;
    marker.set_units(MarkerUnits::StrokeWidth).map_err(|e| e.to_string())?;
    common::check_eq(marker.as_element().get_attribute("markerUnits"), Some("strokeWidth".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `set_units(UserSpaceOnUse)` writes `markerUnits="userSpaceOnUse"`.
#[wasm_bindgen_test]
fn should_set_marker_units_user_space() -> Result<(), String> {
    let svg = make_svg("marker-units-us");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("m").map_err(|e| e.to_string())?;
    marker.set_units(MarkerUnits::UserSpaceOnUse).map_err(|e| e.to_string())?;
    common::check_eq(marker.as_element().get_attribute("markerUnits"), Some("userSpaceOnUse".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// SvgMarker::set_view_box
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `set_view_box` writes a correctly formatted `viewBox` attribute.
#[wasm_bindgen_test]
fn should_set_marker_view_box() -> Result<(), String> {
    let svg = make_svg("marker-viewbox");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("mvb").map_err(|e| e.to_string())?;
    marker.set_view_box(0.0, 0.0, 10.0, 10.0).map_err(|e| e.to_string())?;
    common::check_eq(marker.as_element().get_attribute("viewBox"), Some("0 0 10 10".into()))
}

/// `set_view_box` writes fractional components verbatim, not just whole numbers.
#[wasm_bindgen_test]
fn should_write_fractional_marker_view_box_values() -> Result<(), String> {
    let svg = make_svg("marker-viewbox-fractional");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("mvb-fractional").map_err(|e| e.to_string())?;
    marker.set_view_box(-3.25, 12.5, 100.75, 50.125).map_err(|e| e.to_string())?;
    common::check_eq(
        marker.as_element().get_attribute("viewBox"),
        Some("-3.25 12.5 100.75 50.125".into()),
    )
}

/// `set_view_box` accepts a negative `x`/`y` origin — only `width`/`height` must be non-negative.
#[wasm_bindgen_test]
fn should_accept_negative_marker_view_box_origin() -> Result<(), String> {
    let svg = make_svg("marker-viewbox-neg-origin");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("mvb-neg-origin").map_err(|e| e.to_string())?;
    common::check(
        marker.set_view_box(-5.0, -5.0, 10.0, 10.0).is_ok(),
        "expected a negative x/y origin to be accepted",
    )
}

/// `set_view_box` accepts `width` or `height` of exactly `0.0` alone, with the other dimension non-zero — valid
/// syntax, even though it disables rendering.
#[wasm_bindgen_test]
fn should_accept_zero_width_or_height_marker_view_box() -> Result<(), String> {
    let svg = make_svg("marker-viewbox-zero-alone");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("mvb-zero-alone").map_err(|e| e.to_string())?;
    marker.set_view_box(0.0, 0.0, 0.0, 10.0).map_err(|e| e.to_string())?;
    common::check_eq(marker.as_element().get_attribute("viewBox"), Some("0 0 0 10".into()))?;
    marker.set_view_box(0.0, 0.0, 10.0, 0.0).map_err(|e| e.to_string())?;
    common::check_eq(marker.as_element().get_attribute("viewBox"), Some("0 0 10 0".into()))
}

/// `set_view_box` rejects a negative `width`/`height`, and a non-finite (`NaN`/`±infinity`) component, with
/// `Error::InvalidViewBox` — the same validation `SvgRoot`/`SvgSymbol`/`SvgPattern`'s `set_view_box` share.
#[wasm_bindgen_test]
fn should_reject_invalid_marker_view_box() -> Result<(), String> {
    let svg = make_svg("marker-viewbox-invalid");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("mvb-invalid").map_err(|e| e.to_string())?;
    common::check(
        matches!(marker.set_view_box(0.0, 0.0, -10.0, 10.0), Err(Error::InvalidViewBox(_))),
        "expected InvalidViewBox for negative width",
    )?;
    common::check(
        matches!(marker.set_view_box(0.0, 0.0, 10.0, -10.0), Err(Error::InvalidViewBox(_))),
        "expected InvalidViewBox for negative height",
    )?;
    common::check(
        matches!(marker.set_view_box(f64::NAN, 0.0, 10.0, 10.0), Err(Error::InvalidViewBox(_))),
        "expected InvalidViewBox for a NaN component",
    )?;
    common::check(
        matches!(
            marker.set_view_box(0.0, 0.0, f64::INFINITY, 10.0),
            Err(Error::InvalidViewBox(_))
        ),
        "expected InvalidViewBox for a +infinity component",
    )?;
    common::check(
        matches!(
            marker.set_view_box(0.0, 0.0, f64::NEG_INFINITY, 10.0),
            Err(Error::InvalidViewBox(_))
        ),
        "expected InvalidViewBox for a -infinity component",
    )
}

/// Validation happens before anything is written: a `set_view_box` call that fails leaves a previously-set
/// `viewBox` completely untouched.
#[wasm_bindgen_test]
fn should_preserve_previous_marker_view_box_after_failed_validation() -> Result<(), String> {
    let svg = make_svg("marker-viewbox-preserve");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("mvb-preserve").map_err(|e| e.to_string())?;
    marker.set_view_box(1.0, 2.0, 30.0, 40.0).map_err(|e| e.to_string())?;
    common::check(
        matches!(marker.set_view_box(0.0, 0.0, -10.0, 10.0), Err(Error::InvalidViewBox(_))),
        "expected the second, invalid call to fail",
    )?;
    common::check_eq(marker.as_element().get_attribute("viewBox"), Some("1 2 30 40".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Generic attribute surface — SvgMarker
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `SvgMarker::set_attr` writes an arbitrary attribute (motivating example: `viewBox`).
#[wasm_bindgen_test]
fn should_set_attr_on_marker() -> Result<(), String> {
    let svg = make_svg("marker-set-attr");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("sa-m").map_err(|e| e.to_string())?;
    marker.set_attr("viewBox", "0 0 10 7").map_err(|e| e.to_string())?;
    common::check_eq(marker.as_element().get_attribute("viewBox"), Some("0 0 10 7".into()))
}

/// `SvgMarker::set_attrs` sets several attributes in one call.
#[wasm_bindgen_test]
fn should_set_attrs_on_marker() -> Result<(), String> {
    let svg = make_svg("marker-set-attrs");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("sa-m2").map_err(|e| e.to_string())?;
    marker
        .set_attrs([("viewBox", "0 0 10 7"), ("overflow", "visible"), ("class", "arrow")])
        .map_err(|e| e.to_string())?;
    let el = marker.as_element();
    common::check_eq(el.get_attribute("viewBox"), Some("0 0 10 7".into()))?;
    common::check_eq(el.get_attribute("overflow"), Some("visible".into()))?;
    common::check_eq(el.get_attribute("class"), Some("arrow".into()))
}

/// `SvgMarker::set_attr_display` formats a numeric value without an external scratch buffer.
#[wasm_bindgen_test]
fn should_set_attr_display_on_marker() -> Result<(), String> {
    let svg = make_svg("marker-set-attr-display");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("sa-m3").map_err(|e| e.to_string())?;
    marker.set_attr_display("opacity", 0.75_f64).map_err(|e| e.to_string())?;
    common::check_eq(marker.as_element().get_attribute("opacity"), Some("0.75".into()))
}
