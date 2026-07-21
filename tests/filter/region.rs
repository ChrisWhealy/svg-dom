use crate::common::*;
use svg_dom::FilterUnits;
use wasm_bindgen_test::*;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Filter region and coordinate-space attributes
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `set_x` writes the `x` attribute.
#[wasm_bindgen_test]
fn should_set_filter_x() -> Result<(), String> {
    let svg = make_svg("filter-x");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fx").map_err(|e| e.to_string())?;
    filter.set_x(-0.2).map_err(|e| e.to_string())?;
    check_eq(filter.as_element().get_attribute("x"), Some("-0.2".into()))
}

/// `set_y` writes the `y` attribute.
#[wasm_bindgen_test]
fn should_set_filter_y() -> Result<(), String> {
    let svg = make_svg("filter-y");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fy").map_err(|e| e.to_string())?;
    filter.set_y(-0.2).map_err(|e| e.to_string())?;
    check_eq(filter.as_element().get_attribute("y"), Some("-0.2".into()))
}

/// `set_width` writes the `width` attribute.
#[wasm_bindgen_test]
fn should_set_filter_width() -> Result<(), String> {
    let svg = make_svg("filter-width");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fw").map_err(|e| e.to_string())?;
    filter.set_width(1.4).map_err(|e| e.to_string())?;
    check_eq(filter.as_element().get_attribute("width"), Some("1.4".into()))
}

/// `set_height` writes the `height` attribute.
#[wasm_bindgen_test]
fn should_set_filter_height() -> Result<(), String> {
    let svg = make_svg("filter-height");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fh").map_err(|e| e.to_string())?;
    filter.set_height(1.4).map_err(|e| e.to_string())?;
    check_eq(filter.as_element().get_attribute("height"), Some("1.4".into()))
}

/// `set_filter_units(UserSpaceOnUse)` writes `filterUnits="userSpaceOnUse"`.
#[wasm_bindgen_test]
fn should_set_filter_units_user_space() -> Result<(), String> {
    let svg = make_svg("filter-units-user");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fu-user").map_err(|e| e.to_string())?;
    filter
        .set_filter_units(FilterUnits::UserSpaceOnUse)
        .map_err(|e| e.to_string())?;
    check_eq(filter.as_element().get_attribute("filterUnits"), Some("userSpaceOnUse".into()))
}

/// `set_filter_units(ObjectBoundingBox)` writes `filterUnits="objectBoundingBox"`.
#[wasm_bindgen_test]
fn should_set_filter_units_object_bounding_box() -> Result<(), String> {
    let svg = make_svg("filter-units-obb");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fu-obb").map_err(|e| e.to_string())?;
    filter
        .set_filter_units(FilterUnits::ObjectBoundingBox)
        .map_err(|e| e.to_string())?;
    check_eq(
        filter.as_element().get_attribute("filterUnits"),
        Some("objectBoundingBox".into()),
    )
}

/// `set_primitive_units` writes the `primitiveUnits` attribute.
#[wasm_bindgen_test]
fn should_set_primitive_units() -> Result<(), String> {
    let svg = make_svg("filter-primitive-units");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("pu").map_err(|e| e.to_string())?;
    filter
        .set_primitive_units(FilterUnits::ObjectBoundingBox)
        .map_err(|e| e.to_string())?;
    check_eq(
        filter.as_element().get_attribute("primitiveUnits"),
        Some("objectBoundingBox".into()),
    )
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Region attributes with explicit SVG units via the generic escape hatch
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `set_attr` remains available when percentage or other explicit length syntax cannot be represented by the numeric
/// named setters.
#[wasm_bindgen_test]
fn should_set_filter_region_with_explicit_units_via_generic_escape_hatch() -> Result<(), String> {
    let svg = make_svg("filter-region");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("region").map_err(|e| e.to_string())?;
    filter.set_attr("x", "-20%").map_err(|e| e.to_string())?;
    check_eq(filter.as_element().get_attribute("x"), Some("-20%".into()))
}
