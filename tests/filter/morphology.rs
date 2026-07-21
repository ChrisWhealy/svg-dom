use crate::common::*;
use svg_dom::MorphologyOperator;
use wasm_bindgen_test::*;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// morphology primitive
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `morphology` appends a `<feMorphology>` child to the `<filter>` element.
#[wasm_bindgen_test]
fn should_add_morphology_to_filter() -> Result<(), String> {
    let svg = make_svg("filter-morphology-child");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fmo").map_err(|e| e.to_string())?;
    filter.morphology(3.0, MorphologyOperator::Dilate).map_err(|e| e.to_string())?;
    check_eq(filter.as_element().child_element_count(), 1)
}

/// The appended child has tag name `"feMorphology"`.
#[wasm_bindgen_test]
fn should_create_fe_morphology_element() -> Result<(), String> {
    let svg = make_svg("filter-morphology-tag");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fmot").map_err(|e| e.to_string())?;
    let morph = filter.morphology(3.0, MorphologyOperator::Dilate).map_err(|e| e.to_string())?;
    check_eq(morph.as_element().tag_name(), "feMorphology".to_owned())
}

/// `morphology` writes `operator` and `radius`.
#[wasm_bindgen_test]
fn should_set_operator_and_radius() -> Result<(), String> {
    let svg = make_svg("filter-morphology-attrs");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fmoa").map_err(|e| e.to_string())?;
    let morph = filter.morphology(2.5, MorphologyOperator::Erode).map_err(|e| e.to_string())?;
    let el = morph.as_element();
    check_eq(el.get_attribute("operator"), Some("erode".into()))?;
    check_eq(el.get_attribute("radius"), Some("2.5".into()))
}

/// Every `MorphologyOperator` variant writes its exact SVG keyword.
#[wasm_bindgen_test]
fn should_write_every_morphology_operator_keyword() -> Result<(), String> {
    let svg = make_svg("filter-morphology-operators");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fmok").map_err(|e| e.to_string())?;
    let cases = [(MorphologyOperator::Erode, "erode"), (MorphologyOperator::Dilate, "dilate")];
    for (operator, expected) in cases {
        let morph = filter.morphology(1.0, operator).map_err(|e| e.to_string())?;
        check_eq(morph.as_element().get_attribute("operator"), Some(expected.into()))?;
    }
    Ok(())
}

/// The generic `SvgNode::set_attr` escape hatch on the returned primitive node covers attributes not yet wrapped
/// by a named parameter, such as `in` and `result`.
#[wasm_bindgen_test]
fn should_set_result_via_generic_escape_hatch() -> Result<(), String> {
    let svg = make_svg("filter-morphology-result");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fmor").map_err(|e| e.to_string())?;
    let morph = filter.morphology(3.0, MorphologyOperator::Dilate).map_err(|e| e.to_string())?;
    morph.set_attr("result", "thickened").map_err(|e| e.to_string())?;
    check_eq(morph.as_element().get_attribute("result"), Some("thickened".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// morphology_xy primitive
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `morphology_xy` appends a `<feMorphology>` child to the `<filter>` element.
#[wasm_bindgen_test]
fn should_add_morphology_xy_to_filter() -> Result<(), String> {
    let svg = make_svg("filter-morphology-xy-child");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fmoxy").map_err(|e| e.to_string())?;
    filter
        .morphology_xy(4.0, 1.0, MorphologyOperator::Dilate)
        .map_err(|e| e.to_string())?;
    check_eq(filter.as_element().child_element_count(), 1)
}

/// The appended child has tag name `"feMorphology"`, the same element `morphology` produces.
#[wasm_bindgen_test]
fn should_create_fe_morphology_element_via_xy() -> Result<(), String> {
    let svg = make_svg("filter-morphology-xy-tag");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fmoxyt").map_err(|e| e.to_string())?;
    let morph = filter
        .morphology_xy(4.0, 1.0, MorphologyOperator::Dilate)
        .map_err(|e| e.to_string())?;
    check_eq(morph.as_element().tag_name(), "feMorphology".to_owned())
}

/// `morphology_xy(4.0, 1.0, ...)` writes the two-number `radius="4 1"` form in a single attribute, exactly as the
/// SVG `<number-optional-number>` grammar for `radius` requires.
#[wasm_bindgen_test]
fn should_set_radius_as_two_numbers() -> Result<(), String> {
    let svg = make_svg("filter-morphology-xy-radius");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fmoxyr").map_err(|e| e.to_string())?;
    let morph = filter
        .morphology_xy(4.0, 1.0, MorphologyOperator::Dilate)
        .map_err(|e| e.to_string())?;
    check_eq(morph.as_element().get_attribute("radius"), Some("4 1".into()))
}

/// Passing `0.0` for one axis grows/shrinks only along the other.
#[wasm_bindgen_test]
fn should_allow_zero_on_one_axis() -> Result<(), String> {
    let svg = make_svg("filter-morphology-xy-zero-axis");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fmoxyz").map_err(|e| e.to_string())?;
    let morph = filter
        .morphology_xy(3.0, 0.0, MorphologyOperator::Dilate)
        .map_err(|e| e.to_string())?;
    check_eq(morph.as_element().get_attribute("radius"), Some("3 0".into()))
}

/// The generic `SvgNode::set_attr` escape hatch works identically on a `morphology_xy` node as on a `morphology`
/// one, since both return the same kind of handle around the same element.
#[wasm_bindgen_test]
fn should_set_result_on_morphology_xy_via_generic_escape_hatch() -> Result<(), String> {
    let svg = make_svg("filter-morphology-xy-result");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fmoxyre").map_err(|e| e.to_string())?;
    let morph = filter
        .morphology_xy(4.0, 1.0, MorphologyOperator::Dilate)
        .map_err(|e| e.to_string())?;
    morph.set_attr("result", "widened").map_err(|e| e.to_string())?;
    check_eq(morph.as_element().get_attribute("result"), Some("widened".into()))
}
