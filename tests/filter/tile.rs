use crate::common::*;
use wasm_bindgen_test::*;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// tile primitive
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `tile` appends a `<feTile>` child to the `<filter>` element.
#[wasm_bindgen_test]
fn should_add_tile_to_filter() -> Result<(), String> {
    let svg = make_svg("filter-tile-child");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("ft").map_err(|e| e.to_string())?;
    filter.tile().map_err(|e| e.to_string())?;
    check_eq(filter.as_element().child_element_count(), 1)
}

/// The appended child has tag name `"feTile"`.
#[wasm_bindgen_test]
fn should_create_fe_tile_element() -> Result<(), String> {
    let svg = make_svg("filter-tile-tag");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("ftt").map_err(|e| e.to_string())?;
    let tile = filter.tile().map_err(|e| e.to_string())?;
    check_eq(tile.as_element().tag_name(), "feTile".to_owned())
}

/// `tile` writes no attributes of its own — everything (`in`, `result`, `x`/`y`/`width`/`height`) is reachable only
/// through the generic `SvgNode::set_attr`/`set_attrs` escape hatch.
#[wasm_bindgen_test]
fn should_set_no_attributes_by_default() -> Result<(), String> {
    let svg = make_svg("filter-tile-no-attrs");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("ftn").map_err(|e| e.to_string())?;
    let tile = filter.tile().map_err(|e| e.to_string())?;
    check_eq(tile.as_element().get_attribute("in"), None)?;
    check_eq(tile.as_element().get_attribute("result"), None)?;
    check_eq(tile.as_element().get_attribute("x"), None)
}

/// The generic `SvgNode::set_attr`/`set_attrs` escape hatch covers `in`, `result`, and the primitive's own
/// `x`/`y`/`width`/`height` subregion — the narrowed rectangle that becomes the repeated tile.
#[wasm_bindgen_test]
fn should_set_subregion_via_generic_escape_hatch() -> Result<(), String> {
    let svg = make_svg("filter-tile-subregion");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fts").map_err(|e| e.to_string())?;
    let tile = filter.tile().map_err(|e| e.to_string())?;
    tile.set_attrs([
        ("in", "noise"),
        ("x", "0"),
        ("y", "0"),
        ("width", "20"),
        ("height", "20"),
        ("result", "tiled"),
    ])
    .map_err(|e| e.to_string())?;
    check_eq(tile.as_element().get_attribute("in"), Some("noise".into()))?;
    check_eq(tile.as_element().get_attribute("x"), Some("0".into()))?;
    check_eq(tile.as_element().get_attribute("y"), Some("0".into()))?;
    check_eq(tile.as_element().get_attribute("width"), Some("20".into()))?;
    check_eq(tile.as_element().get_attribute("height"), Some("20".into()))?;
    check_eq(tile.as_element().get_attribute("result"), Some("tiled".into()))
}
