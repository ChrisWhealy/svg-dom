use crate::common::*;
use wasm_bindgen_test::*;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// gaussian_blur primitive
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `gaussian_blur` appends a `<feGaussianBlur>` child to the `<filter>` element.
#[wasm_bindgen_test]
fn should_add_gaussian_blur_to_filter() -> Result<(), String> {
    let svg = make_svg("filter-blur-child");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fb").map_err(|e| e.to_string())?;
    filter.gaussian_blur(4.0).map_err(|e| e.to_string())?;
    check_eq(filter.as_element().child_element_count(), 1)
}

/// The appended child has tag name `"feGaussianBlur"`.
#[wasm_bindgen_test]
fn should_create_fe_gaussian_blur_element() -> Result<(), String> {
    let svg = make_svg("filter-blur-tag");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fbt").map_err(|e| e.to_string())?;
    let blur = filter.gaussian_blur(4.0).map_err(|e| e.to_string())?;
    check_eq(blur.as_element().tag_name(), "feGaussianBlur".to_owned())
}

/// `gaussian_blur` writes the `stdDeviation` attribute.
#[wasm_bindgen_test]
fn should_set_std_deviation() -> Result<(), String> {
    let svg = make_svg("filter-blur-std-dev");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fsd").map_err(|e| e.to_string())?;
    let blur = filter.gaussian_blur(6.5).map_err(|e| e.to_string())?;
    check_eq(blur.as_element().get_attribute("stdDeviation"), Some("6.5".into()))
}

/// Multiple primitives can be added to the same filter, in document order.
#[wasm_bindgen_test]
fn should_add_multiple_primitives_in_order() -> Result<(), String> {
    let svg = make_svg("filter-multi");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fm").map_err(|e| e.to_string())?;
    filter.gaussian_blur(2.0).map_err(|e| e.to_string())?;
    filter.gaussian_blur(8.0).map_err(|e| e.to_string())?;
    check_eq(filter.as_element().child_element_count(), 2)
}

/// The generic `SvgNode::set_attr` escape hatch on the returned primitive node covers attributes not yet wrapped by
/// a named parameter, such as `in` and `result`.
#[wasm_bindgen_test]
fn should_set_result_via_generic_escape_hatch() -> Result<(), String> {
    let svg = make_svg("filter-blur-result");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fr").map_err(|e| e.to_string())?;
    let blur = filter.gaussian_blur(4.0).map_err(|e| e.to_string())?;
    blur.set_attr("result", "blurred").map_err(|e| e.to_string())?;
    check_eq(blur.as_element().get_attribute("result"), Some("blurred".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// gaussian_blur_xy primitive
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `gaussian_blur_xy` appends a `<feGaussianBlur>` child to the `<filter>` element.
#[wasm_bindgen_test]
fn should_add_gaussian_blur_xy_to_filter() -> Result<(), String> {
    let svg = make_svg("filter-blur-xy-child");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fbxy").map_err(|e| e.to_string())?;
    filter.gaussian_blur_xy(3.0, 6.5).map_err(|e| e.to_string())?;
    check_eq(filter.as_element().child_element_count(), 1)
}

/// The appended child has tag name `"feGaussianBlur"`, the same element `gaussian_blur` produces.
#[wasm_bindgen_test]
fn should_create_fe_gaussian_blur_element_via_xy() -> Result<(), String> {
    let svg = make_svg("filter-blur-xy-tag");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fbxyt").map_err(|e| e.to_string())?;
    let blur = filter.gaussian_blur_xy(3.0, 6.5).map_err(|e| e.to_string())?;
    check_eq(blur.as_element().tag_name(), "feGaussianBlur".to_owned())
}

/// `gaussian_blur_xy(3.0, 6.5)` writes the two-number `stdDeviation="3 6.5"` form in a single attribute, exactly
/// as the SVG `<number-optional-number>` grammar for `stdDeviation` requires.
#[wasm_bindgen_test]
fn should_set_std_deviation_as_two_numbers() -> Result<(), String> {
    let svg = make_svg("filter-blur-xy-std-dev");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fbxysd").map_err(|e| e.to_string())?;
    let blur = filter.gaussian_blur_xy(3.0, 6.5).map_err(|e| e.to_string())?;
    check_eq(blur.as_element().get_attribute("stdDeviation"), Some("3 6.5".into()))
}

/// Passing `0.0` for one axis blurs only along the other, per the SVG grammar's documented use case.
#[wasm_bindgen_test]
fn should_allow_zero_on_one_axis() -> Result<(), String> {
    let svg = make_svg("filter-blur-xy-zero-axis");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fbxyz").map_err(|e| e.to_string())?;
    let blur = filter.gaussian_blur_xy(0.0, 8.0).map_err(|e| e.to_string())?;
    check_eq(blur.as_element().get_attribute("stdDeviation"), Some("0 8".into()))
}

/// The generic `SvgNode::set_attr` escape hatch works identically on a `gaussian_blur_xy` node as on a
/// `gaussian_blur` one, since both return the same kind of handle around the same element.
#[wasm_bindgen_test]
fn should_set_result_on_gaussian_blur_xy_via_generic_escape_hatch() -> Result<(), String> {
    let svg = make_svg("filter-blur-xy-result");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fbxyr").map_err(|e| e.to_string())?;
    let blur = filter.gaussian_blur_xy(3.0, 6.5).map_err(|e| e.to_string())?;
    blur.set_attr("result", "blurred").map_err(|e| e.to_string())?;
    check_eq(blur.as_element().get_attribute("result"), Some("blurred".into()))
}
