use crate::common::*;
use svg_dom::{EdgeMode, Error};
use wasm_bindgen_test::*;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// convolve_matrix primitive
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `convolve_matrix` appends a `<feConvolveMatrix>` child to the `<filter>` element.
#[wasm_bindgen_test]
fn should_add_convolve_matrix_to_filter() -> Result<(), String> {
    let svg = make_svg("filter-convolve-matrix-child");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fcmc").map_err(|e| e.to_string())?;
    let kernel = [0.0, -1.0, 0.0, -1.0, 5.0, -1.0, 0.0, -1.0, 0.0];
    filter
        .convolve_matrix(3, &kernel, 1.0, EdgeMode::Duplicate, false)
        .map_err(|e| e.to_string())?;
    check_eq(filter.as_element().child_element_count(), 1)
}

/// The appended child has tag name `"feConvolveMatrix"`.
#[wasm_bindgen_test]
fn should_create_fe_convolve_matrix_element() -> Result<(), String> {
    let svg = make_svg("filter-convolve-matrix-tag");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fcmt").map_err(|e| e.to_string())?;
    let kernel = [0.0, -1.0, 0.0, -1.0, 5.0, -1.0, 0.0, -1.0, 0.0];
    let conv = filter
        .convolve_matrix(3, &kernel, 1.0, EdgeMode::Duplicate, false)
        .map_err(|e| e.to_string())?;
    check_eq(conv.as_element().tag_name(), "feConvolveMatrix".to_owned())
}

/// `convolve_matrix` writes `order`, `kernelMatrix`, `divisor`, `edgeMode`, and `preserveAlpha`.
#[wasm_bindgen_test]
fn should_set_convolve_matrix_attrs() -> Result<(), String> {
    let svg = make_svg("filter-convolve-matrix-attrs");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fcma").map_err(|e| e.to_string())?;
    let kernel = [0.0, -1.0, 0.0, -1.0, 5.0, -1.0, 0.0, -1.0, 0.0];
    let conv = filter
        .convolve_matrix(3, &kernel, 1.0, EdgeMode::Wrap, true)
        .map_err(|e| e.to_string())?;
    let el = conv.as_element();
    check_eq(el.get_attribute("order"), Some("3".into()))?;
    check_eq(el.get_attribute("kernelMatrix"), Some("0 -1 0 -1 5 -1 0 -1 0".into()))?;
    check_eq(el.get_attribute("divisor"), Some("1".into()))?;
    check_eq(el.get_attribute("edgeMode"), Some("wrap".into()))?;
    check_eq(el.get_attribute("preserveAlpha"), Some("true".into()))
}

/// `preserve_alpha: false` writes the literal string `"false"`, not an absent attribute.
#[wasm_bindgen_test]
fn should_write_preserve_alpha_false() -> Result<(), String> {
    let svg = make_svg("filter-convolve-matrix-preserve-alpha-false");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fcmpf").map_err(|e| e.to_string())?;
    let kernel = [1.0];
    let conv = filter
        .convolve_matrix(1, &kernel, 1.0, EdgeMode::Duplicate, false)
        .map_err(|e| e.to_string())?;
    check_eq(conv.as_element().get_attribute("preserveAlpha"), Some("false".into()))
}

/// Every `EdgeMode` variant writes its exact SVG keyword.
#[wasm_bindgen_test]
fn should_write_every_edge_mode_keyword() -> Result<(), String> {
    let svg = make_svg("filter-convolve-matrix-edge-modes");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fcmek").map_err(|e| e.to_string())?;
    let kernel = [1.0];
    let cases = [
        (EdgeMode::Duplicate, "duplicate"),
        (EdgeMode::Wrap, "wrap"),
        (EdgeMode::None, "none"),
    ];
    for (edge_mode, expected) in cases {
        let conv = filter
            .convolve_matrix(1, &kernel, 1.0, edge_mode, false)
            .map_err(|e| e.to_string())?;
        check_eq(conv.as_element().get_attribute("edgeMode"), Some(expected.into()))?;
    }
    Ok(())
}

/// A `kernel_matrix` whose length does not equal `order * order` is written verbatim, unvalidated: per the SVG
/// spec, this makes `<feConvolveMatrix>` "act as a pass through filter" rather than an error — a defined,
/// well-formed (if inert) rendering outcome the browser handles, not something this crate needs to reject. See
/// `convolve_matrix`'s own doc comment for the full explanation.
#[wasm_bindgen_test]
fn should_serialize_mismatched_kernel_length_unvalidated() -> Result<(), String> {
    let svg = make_svg("filter-convolve-matrix-mismatched-kernel");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fcmmk").map_err(|e| e.to_string())?;
    let kernel = [1.0, 2.0]; // order 3 wants 9 values, not 2
    let conv = filter
        .convolve_matrix(3, &kernel, 1.0, EdgeMode::Duplicate, false)
        .map_err(|e| e.to_string())?;
    check_eq(conv.as_element().get_attribute("kernelMatrix"), Some("1 2".into()))
}

/// The generic `SvgNode::set_attr` escape hatch on the returned primitive node covers `in`, `result`, and every
/// other attribute not wrapped by a named parameter: `bias`, `targetX`, `targetY`, and `kernelUnitLength`.
#[wasm_bindgen_test]
fn should_set_unwrapped_attrs_via_generic_escape_hatch() -> Result<(), String> {
    let svg = make_svg("filter-convolve-matrix-escape-hatch");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fcmeh").map_err(|e| e.to_string())?;
    let kernel = [-2.0, -1.0, 0.0, -1.0, 1.0, 1.0, 0.0, 1.0, 2.0];
    let conv = filter
        .convolve_matrix(3, &kernel, 1.0, EdgeMode::Duplicate, true)
        .map_err(|e| e.to_string())?;
    conv.set_attr("result", "embossed").map_err(|e| e.to_string())?;
    conv.set_attr("bias", "0.5").map_err(|e| e.to_string())?;
    conv.set_attr("targetX", "1").map_err(|e| e.to_string())?;
    conv.set_attr("targetY", "1").map_err(|e| e.to_string())?;
    let el = conv.as_element();
    check_eq(el.get_attribute("result"), Some("embossed".into()))?;
    check_eq(el.get_attribute("bias"), Some("0.5".into()))?;
    check_eq(el.get_attribute("targetX"), Some("1".into()))?;
    check_eq(el.get_attribute("targetY"), Some("1".into()))
}

/// Unlike a `kernel_matrix` length mismatch or a zero `divisor` (both spec-defined fallbacks, see
/// `should_serialize_mismatched_kernel_length_unvalidated` above), an `order` of `0` has no defined SVG fallback, so
/// `convolve_matrix` rejects it before creating any element.
#[wasm_bindgen_test]
fn should_reject_zero_order() -> Result<(), String> {
    let svg = make_svg("filter-convolve-matrix-zero-order");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fcmzo").map_err(|e| e.to_string())?;
    let kernel: [f64; 0] = [];
    let result = filter.convolve_matrix(0, &kernel, 1.0, EdgeMode::Duplicate, false);
    check(
        matches!(result, Err(Error::InvalidConvolveMatrixOrder(_))),
        "expected InvalidConvolveMatrixOrder error for order: 0",
    )?;
    check_eq(filter.as_element().child_element_count(), 0)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// convolve_matrix_xy primitive
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `convolve_matrix_xy` appends a `<feConvolveMatrix>` child to the `<filter>` element.
#[wasm_bindgen_test]
fn should_add_convolve_matrix_xy_to_filter() -> Result<(), String> {
    let svg = make_svg("filter-convolve-matrix-xy-child");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fcmxy").map_err(|e| e.to_string())?;
    let kernel = [1.0, 1.0, 1.0];
    filter
        .convolve_matrix_xy(3, 1, &kernel, 3.0, EdgeMode::Duplicate, false)
        .map_err(|e| e.to_string())?;
    check_eq(filter.as_element().child_element_count(), 1)
}

/// The appended child has tag name `"feConvolveMatrix"`, the same element `convolve_matrix` produces.
#[wasm_bindgen_test]
fn should_create_fe_convolve_matrix_element_via_xy() -> Result<(), String> {
    let svg = make_svg("filter-convolve-matrix-xy-tag");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fcmxyt").map_err(|e| e.to_string())?;
    let kernel = [1.0, 1.0, 1.0];
    let conv = filter
        .convolve_matrix_xy(3, 1, &kernel, 3.0, EdgeMode::Duplicate, false)
        .map_err(|e| e.to_string())?;
    check_eq(conv.as_element().tag_name(), "feConvolveMatrix".to_owned())
}

/// `convolve_matrix_xy(3, 1, ...)` writes the two-number `order="3 1"` form in a single attribute, exactly as the
/// SVG `<number-optional-number>` grammar for `order` requires.
#[wasm_bindgen_test]
fn should_set_order_as_two_numbers() -> Result<(), String> {
    let svg = make_svg("filter-convolve-matrix-xy-order");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fcmxyo").map_err(|e| e.to_string())?;
    let kernel = [1.0, 1.0, 1.0];
    let conv = filter
        .convolve_matrix_xy(3, 1, &kernel, 3.0, EdgeMode::Duplicate, false)
        .map_err(|e| e.to_string())?;
    check_eq(conv.as_element().get_attribute("order"), Some("3 1".into()))
}

/// The generic `SvgNode::set_attr` escape hatch works identically on a `convolve_matrix_xy` node as on a
/// `convolve_matrix` one, since both return the same kind of handle around the same element.
#[wasm_bindgen_test]
fn should_set_result_on_convolve_matrix_xy_via_generic_escape_hatch() -> Result<(), String> {
    let svg = make_svg("filter-convolve-matrix-xy-result");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fcmxyr").map_err(|e| e.to_string())?;
    let kernel = [1.0, 1.0, 1.0];
    let conv = filter
        .convolve_matrix_xy(3, 1, &kernel, 3.0, EdgeMode::Duplicate, false)
        .map_err(|e| e.to_string())?;
    conv.set_attr("result", "streaked").map_err(|e| e.to_string())?;
    check_eq(conv.as_element().get_attribute("result"), Some("streaked".into()))
}

/// `convolve_matrix_xy` rejects `order_x: 0` before creating any element, the same as `convolve_matrix` does for a
/// zero `order`.
#[wasm_bindgen_test]
fn should_reject_zero_order_x() -> Result<(), String> {
    let svg = make_svg("filter-convolve-matrix-xy-zero-order-x");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fcmxyzx").map_err(|e| e.to_string())?;
    let kernel: [f64; 0] = [];
    let result = filter.convolve_matrix_xy(0, 3, &kernel, 1.0, EdgeMode::Duplicate, false);
    check(
        matches!(result, Err(Error::InvalidConvolveMatrixOrder(_))),
        "expected InvalidConvolveMatrixOrder error for order_x: 0",
    )?;
    check_eq(filter.as_element().child_element_count(), 0)
}

/// `convolve_matrix_xy` rejects `order_y: 0` before creating any element, the same as for `order_x`.
#[wasm_bindgen_test]
fn should_reject_zero_order_y() -> Result<(), String> {
    let svg = make_svg("filter-convolve-matrix-xy-zero-order-y");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fcmxyzy").map_err(|e| e.to_string())?;
    let kernel: [f64; 0] = [];
    let result = filter.convolve_matrix_xy(3, 0, &kernel, 1.0, EdgeMode::Duplicate, false);
    check(
        matches!(result, Err(Error::InvalidConvolveMatrixOrder(_))),
        "expected InvalidConvolveMatrixOrder error for order_y: 0",
    )?;
    check_eq(filter.as_element().child_element_count(), 0)
}
