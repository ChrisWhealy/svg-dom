use crate::common::*;
use wasm_bindgen_test::*;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// image primitive
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `image` appends a `<feImage>` child to the `<filter>` element.
#[wasm_bindgen_test]
fn should_add_image_to_filter() -> Result<(), String> {
    let svg = make_svg("filter-image-child");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fim").map_err(|e| e.to_string())?;
    filter.image("photo.jpg").map_err(|e| e.to_string())?;
    check_eq(filter.as_element().child_element_count(), 1)
}

/// The appended child has tag name `"feImage"`.
#[wasm_bindgen_test]
fn should_create_fe_image_element() -> Result<(), String> {
    let svg = make_svg("filter-image-tag");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fimt").map_err(|e| e.to_string())?;
    let img = filter.image("photo.jpg").map_err(|e| e.to_string())?;
    check_eq(img.as_element().tag_name(), "feImage".to_owned())
}

/// `image` writes the `href` attribute verbatim.
#[wasm_bindgen_test]
fn should_set_href() -> Result<(), String> {
    let svg = make_svg("filter-image-href");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fimh").map_err(|e| e.to_string())?;
    let img = filter.image("assets/photo.jpg").map_err(|e| e.to_string())?;
    check_eq(img.as_element().get_attribute("href"), Some("assets/photo.jpg".into()))
}

/// The generic `SvgNode::set_attr` escape hatch on the returned primitive node covers attributes not yet wrapped by
/// a named parameter, such as `result` (needed when this primitive's output must be referenced explicitly — as
/// `in2`, by a primitive that is not immediately downstream, or in a branched filter graph; a simple linear chain
/// can consume it implicitly instead) and `preserveAspectRatio`.
#[wasm_bindgen_test]
fn should_set_result_and_preserve_aspect_ratio_via_generic_escape_hatch() -> Result<(), String> {
    let svg = make_svg("filter-image-result");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fimr").map_err(|e| e.to_string())?;
    let img = filter.image("photo.jpg").map_err(|e| e.to_string())?;
    img.set_attrs([("result", "imported"), ("preserveAspectRatio", "none")])
        .map_err(|e| e.to_string())?;
    check_eq(img.as_element().get_attribute("result"), Some("imported".into()))?;
    check_eq(img.as_element().get_attribute("preserveAspectRatio"), Some("none".into()))
}
