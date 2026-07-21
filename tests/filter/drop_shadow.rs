use crate::common::*;
use wasm_bindgen_test::*;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// drop_shadow primitive
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `drop_shadow` appends a `<feDropShadow>` child to the `<filter>` element.
#[wasm_bindgen_test]
fn should_add_drop_shadow_to_filter() -> Result<(), String> {
    let svg = make_svg("filter-drop-shadow-child");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fds").map_err(|e| e.to_string())?;
    filter.drop_shadow(4.0, 4.0, 4.0, "black", 0.5).map_err(|e| e.to_string())?;
    check_eq(filter.as_element().child_element_count(), 1)
}

/// The appended child has tag name `"feDropShadow"`.
#[wasm_bindgen_test]
fn should_create_fe_drop_shadow_element() -> Result<(), String> {
    let svg = make_svg("filter-drop-shadow-tag");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fdst").map_err(|e| e.to_string())?;
    let shadow = filter.drop_shadow(4.0, 4.0, 4.0, "black", 0.5).map_err(|e| e.to_string())?;
    check_eq(shadow.as_element().tag_name(), "feDropShadow".to_owned())
}

/// `drop_shadow` writes `stdDeviation`, `dx`, `dy`, `flood-color`, and `flood-opacity`.
#[wasm_bindgen_test]
fn should_set_drop_shadow_attributes() -> Result<(), String> {
    let svg = make_svg("filter-drop-shadow-attrs");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fdsa").map_err(|e| e.to_string())?;
    let shadow = filter.drop_shadow(3.5, 6.0, 8.0, "crimson", 0.65).map_err(|e| e.to_string())?;
    let el = shadow.as_element();
    check_eq(el.get_attribute("stdDeviation"), Some("3.5".into()))?;
    check_eq(el.get_attribute("dx"), Some("6".into()))?;
    check_eq(el.get_attribute("dy"), Some("8".into()))?;
    check_eq(el.get_attribute("flood-color"), Some("crimson".into()))?;
    check_eq(el.get_attribute("flood-opacity"), Some("0.65".into()))
}

/// A `<filter>` containing only `drop_shadow` is already a complete shadow effect — the primitive's own SVG
/// semantics merge the original graphic back on top, so this filter needs exactly one child, not a further
/// `merge` call.
#[wasm_bindgen_test]
fn should_need_no_further_merge_after_drop_shadow() -> Result<(), String> {
    let svg = make_svg("filter-drop-shadow-standalone");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs
        .build_filter("standalone-shadow", |f| {
            f.drop_shadow(4.0, 4.0, 4.0, "black", 0.5)?;
            Ok(())
        })
        .map_err(|e| e.to_string())?;
    check_eq(filter.as_element().child_element_count(), 1)
}

/// The generic `SvgNode::set_attr` escape hatch works on a `drop_shadow` node the same as on every other
/// primitive, for attributes like `in`/`result` not wrapped by a named parameter.
#[wasm_bindgen_test]
fn should_set_result_on_drop_shadow_via_generic_escape_hatch() -> Result<(), String> {
    let svg = make_svg("filter-drop-shadow-result");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fdsr").map_err(|e| e.to_string())?;
    let shadow = filter.drop_shadow(4.0, 4.0, 4.0, "black", 0.5).map_err(|e| e.to_string())?;
    shadow.set_attr("result", "shadow").map_err(|e| e.to_string())?;
    check_eq(shadow.as_element().get_attribute("result"), Some("shadow".into()))
}
