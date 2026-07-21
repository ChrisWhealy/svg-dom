use crate::common::*;
use wasm_bindgen_test::*;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// offset primitive
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `offset` appends a `<feOffset>` child to the `<filter>` element.
#[wasm_bindgen_test]
fn should_add_offset_to_filter() -> Result<(), String> {
    let svg = make_svg("filter-offset-child");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fo").map_err(|e| e.to_string())?;
    filter.offset(4.0, 4.0).map_err(|e| e.to_string())?;
    check_eq(filter.as_element().child_element_count(), 1)
}

/// The appended child has tag name `"feOffset"`.
#[wasm_bindgen_test]
fn should_create_fe_offset_element() -> Result<(), String> {
    let svg = make_svg("filter-offset-tag");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fot").map_err(|e| e.to_string())?;
    let offset = filter.offset(4.0, 4.0).map_err(|e| e.to_string())?;
    check_eq(offset.as_element().tag_name(), "feOffset".to_owned())
}

/// `offset` writes the `dx` and `dy` attributes.
#[wasm_bindgen_test]
fn should_set_dx_dy() -> Result<(), String> {
    let svg = make_svg("filter-offset-dxdy");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fdd").map_err(|e| e.to_string())?;
    let offset = filter.offset(3.5, -2.0).map_err(|e| e.to_string())?;
    check_eq(offset.as_element().get_attribute("dx"), Some("3.5".into()))?;
    check_eq(offset.as_element().get_attribute("dy"), Some("-2".into()))
}
