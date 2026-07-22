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

/// `href` accepts a same-document fragment reference (`"#id"`), not just an external image URL or a `data:` URI.
/// This is a deliberate, important use case: `<feImage>` can pull in any SVG element already in the document — a
/// `<g>` built or modified at runtime, not only a raster or pre-encoded resource.
#[wasm_bindgen_test]
fn should_accept_internal_reference_as_href() -> Result<(), String> {
    let svg = make_svg("filter-image-internal-ref");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fimi").map_err(|e| e.to_string())?;
    let img = filter.image("#texture").map_err(|e| e.to_string())?;
    check_eq(img.as_element().get_attribute("href"), Some("#texture".into()))
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

/// `crossorigin` (not wrapped by a named parameter either) is likewise reachable via the generic escape hatch —
/// needed for a cross-origin image consumed as `in2` by `displacement_map`, since an untainted CORS check is what
/// keeps that displacement from silently becoming a pass-through.
#[wasm_bindgen_test]
fn should_set_crossorigin_via_generic_escape_hatch() -> Result<(), String> {
    let svg = make_svg("filter-image-crossorigin");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fimc").map_err(|e| e.to_string())?;
    let img = filter.image("https://example.com/map.png").map_err(|e| e.to_string())?;
    img.set_attrs([("crossorigin", "anonymous"), ("result", "displacement-map")])
        .map_err(|e| e.to_string())?;
    check_eq(img.as_element().get_attribute("crossorigin"), Some("anonymous".into()))?;
    check_eq(img.as_element().get_attribute("result"), Some("displacement-map".into()))
}
