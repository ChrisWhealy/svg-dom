use crate::common::*;
use svg_dom::ColorMatrixType;
use wasm_bindgen_test::*;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// color_matrix primitive
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `color_matrix` appends a `<feColorMatrix>` child to the `<filter>` element.
#[wasm_bindgen_test]
fn should_add_color_matrix_to_filter() -> Result<(), String> {
    let svg = make_svg("filter-color-matrix-child");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fcm").map_err(|e| e.to_string())?;
    filter.color_matrix(ColorMatrixType::Saturate(0.0)).map_err(|e| e.to_string())?;
    check_eq(filter.as_element().child_element_count(), 1)
}

/// The appended child has tag name `"feColorMatrix"`.
#[wasm_bindgen_test]
fn should_create_fe_color_matrix_element() -> Result<(), String> {
    let svg = make_svg("filter-color-matrix-tag");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fcmt").map_err(|e| e.to_string())?;
    let cm = filter.color_matrix(ColorMatrixType::Saturate(0.0)).map_err(|e| e.to_string())?;
    check_eq(cm.as_element().tag_name(), "feColorMatrix".to_owned())
}

/// `ColorMatrixType::Saturate` writes `type="saturate"` and `values` as the single number.
#[wasm_bindgen_test]
fn should_set_saturate_type_and_values() -> Result<(), String> {
    let svg = make_svg("filter-color-matrix-saturate");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fcms").map_err(|e| e.to_string())?;
    let cm = filter
        .color_matrix(ColorMatrixType::Saturate(0.25))
        .map_err(|e| e.to_string())?;
    check_eq(cm.as_element().get_attribute("type"), Some("saturate".into()))?;
    check_eq(cm.as_element().get_attribute("values"), Some("0.25".into()))
}

/// `ColorMatrixType::HueRotate` writes `type="hueRotate"` and `values` as the single number (degrees).
#[wasm_bindgen_test]
fn should_set_hue_rotate_type_and_values() -> Result<(), String> {
    let svg = make_svg("filter-color-matrix-hue");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fcmh").map_err(|e| e.to_string())?;
    let cm = filter
        .color_matrix(ColorMatrixType::HueRotate(90.0))
        .map_err(|e| e.to_string())?;
    check_eq(cm.as_element().get_attribute("type"), Some("hueRotate".into()))?;
    check_eq(cm.as_element().get_attribute("values"), Some("90".into()))
}

/// `ColorMatrixType::LuminanceToAlpha` writes `type="luminanceToAlpha"` and omits `values` entirely, since the
/// SVG spec defines `values` as not applicable for this type.
#[wasm_bindgen_test]
fn should_set_luminance_to_alpha_type_and_omit_values() -> Result<(), String> {
    let svg = make_svg("filter-color-matrix-luminance");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fcml").map_err(|e| e.to_string())?;
    let cm = filter
        .color_matrix(ColorMatrixType::LuminanceToAlpha)
        .map_err(|e| e.to_string())?;
    check_eq(cm.as_element().get_attribute("type"), Some("luminanceToAlpha".into()))?;
    check_eq(cm.as_element().get_attribute("values"), None)
}

/// `ColorMatrixType::Matrix` writes `type="matrix"` and `values` as all 20 numbers, space-separated, in order.
#[wasm_bindgen_test]
fn should_set_matrix_type_and_all_twenty_values() -> Result<(), String> {
    let svg = make_svg("filter-color-matrix-matrix");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fcmm").map_err(|e| e.to_string())?;
    #[rustfmt::skip]
    let identity: [f64; 20] = [
        1.0, 0.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 0.0, 1.0, 0.0,
    ];
    let cm = filter
        .color_matrix(ColorMatrixType::Matrix(identity))
        .map_err(|e| e.to_string())?;
    check_eq(cm.as_element().get_attribute("type"), Some("matrix".into()))?;
    check_eq(
        cm.as_element().get_attribute("values"),
        Some("1 0 0 0 0 0 1 0 0 0 0 0 1 0 0 0 0 0 1 0".into()),
    )
}

/// The generic `SvgNode::set_attr` escape hatch works on a `color_matrix` node the same as on every other
/// primitive, for attributes like `in`/`result` not wrapped by a named parameter.
#[wasm_bindgen_test]
fn should_set_result_on_color_matrix_via_generic_escape_hatch() -> Result<(), String> {
    let svg = make_svg("filter-color-matrix-result");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fcmr").map_err(|e| e.to_string())?;
    let cm = filter.color_matrix(ColorMatrixType::Saturate(0.0)).map_err(|e| e.to_string())?;
    cm.set_attr("result", "grey").map_err(|e| e.to_string())?;
    check_eq(cm.as_element().get_attribute("result"), Some("grey".into()))
}
