use crate::common::*;
use wasm_bindgen_test::*;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// flood primitive
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `flood` appends a `<feFlood>` child to the `<filter>` element.
#[wasm_bindgen_test]
fn should_add_flood_to_filter() -> Result<(), String> {
    let svg = make_svg("filter-flood-child");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("ffl").map_err(|e| e.to_string())?;
    filter.flood("black", 0.5).map_err(|e| e.to_string())?;
    check_eq(filter.as_element().child_element_count(), 1)
}

/// The appended child has tag name `"feFlood"`.
#[wasm_bindgen_test]
fn should_create_fe_flood_element() -> Result<(), String> {
    let svg = make_svg("filter-flood-tag");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fflt").map_err(|e| e.to_string())?;
    let flood = filter.flood("black", 0.5).map_err(|e| e.to_string())?;
    check_eq(flood.as_element().tag_name(), "feFlood".to_owned())
}

/// `flood` writes the `flood-color` and `flood-opacity` attributes.
#[wasm_bindgen_test]
fn should_set_flood_color_and_opacity() -> Result<(), String> {
    let svg = make_svg("filter-flood-attrs");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("ffa").map_err(|e| e.to_string())?;
    let flood = filter.flood("crimson", 0.65).map_err(|e| e.to_string())?;
    check_eq(flood.as_element().get_attribute("flood-color"), Some("crimson".into()))?;
    check_eq(flood.as_element().get_attribute("flood-opacity"), Some("0.65".into()))
}
