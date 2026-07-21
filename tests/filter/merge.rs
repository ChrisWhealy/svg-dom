use crate::common::*;
use wasm_bindgen_test::*;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// merge primitive
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `merge` appends a single `<feMerge>` child to the `<filter>` element.
#[wasm_bindgen_test]
fn should_add_merge_to_filter() -> Result<(), String> {
    let svg = make_svg("filter-merge-child");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fmg").map_err(|e| e.to_string())?;
    filter.merge(&["a", "b"]).map_err(|e| e.to_string())?;
    check_eq(filter.as_element().child_element_count(), 1)
}

/// The appended child has tag name `"feMerge"`.
#[wasm_bindgen_test]
fn should_create_fe_merge_element() -> Result<(), String> {
    let svg = make_svg("filter-merge-tag");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fmt").map_err(|e| e.to_string())?;
    let merge = filter.merge(&["a", "b"]).map_err(|e| e.to_string())?;
    check_eq(merge.as_element().tag_name(), "feMerge".to_owned())
}

/// `merge` appends one `<feMergeNode>` per input, in order.
#[wasm_bindgen_test]
fn should_add_one_merge_node_per_input() -> Result<(), String> {
    let svg = make_svg("filter-merge-nodes");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fmn").map_err(|e| e.to_string())?;
    let merge = filter.merge(&["offset-blur", "SourceGraphic"]).map_err(|e| e.to_string())?;
    let el = merge.as_element();
    check_eq(el.child_element_count(), 2)?;
    let first = el.first_element_child().ok_or("missing first feMergeNode")?;
    let second = first.next_element_sibling().ok_or("missing second feMergeNode")?;
    check_eq(first.tag_name(), "feMergeNode".to_owned())?;
    check_eq(first.get_attribute("in"), Some("offset-blur".into()))?;
    check_eq(second.tag_name(), "feMergeNode".to_owned())?;
    check_eq(second.get_attribute("in"), Some("SourceGraphic".into()))
}

/// `merge` with an empty input slice appends a `<feMerge>` with no children.
#[wasm_bindgen_test]
fn should_add_empty_merge_for_no_inputs() -> Result<(), String> {
    let svg = make_svg("filter-merge-empty");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fme").map_err(|e| e.to_string())?;
    let merge = filter.merge(&[]).map_err(|e| e.to_string())?;
    check_eq(merge.as_element().child_element_count(), 0)
}
