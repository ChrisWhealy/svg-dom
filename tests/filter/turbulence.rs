use crate::common::*;
use svg_dom::TurbulenceType;
use wasm_bindgen_test::*;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// turbulence primitive
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `turbulence` appends a `<feTurbulence>` child to the `<filter>` element.
#[wasm_bindgen_test]
fn should_add_turbulence_to_filter() -> Result<(), String> {
    let svg = make_svg("filter-turbulence-child");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("ft").map_err(|e| e.to_string())?;
    filter
        .turbulence(0.02, 3, 1.0, TurbulenceType::Turbulence)
        .map_err(|e| e.to_string())?;
    check_eq(filter.as_element().child_element_count(), 1)
}

/// The appended child has tag name `"feTurbulence"`.
#[wasm_bindgen_test]
fn should_create_fe_turbulence_element() -> Result<(), String> {
    let svg = make_svg("filter-turbulence-tag");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("ftt").map_err(|e| e.to_string())?;
    let turb = filter
        .turbulence(0.02, 3, 1.0, TurbulenceType::Turbulence)
        .map_err(|e| e.to_string())?;
    check_eq(turb.as_element().tag_name(), "feTurbulence".to_owned())
}

/// `turbulence` writes `type`, `baseFrequency`, `numOctaves`, and `seed`.
#[wasm_bindgen_test]
fn should_set_type_base_frequency_num_octaves_and_seed() -> Result<(), String> {
    let svg = make_svg("filter-turbulence-attrs");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fta").map_err(|e| e.to_string())?;
    let turb = filter
        .turbulence(0.015, 4, 2.5, TurbulenceType::FractalNoise)
        .map_err(|e| e.to_string())?;
    let el = turb.as_element();
    check_eq(el.get_attribute("type"), Some("fractalNoise".into()))?;
    check_eq(el.get_attribute("baseFrequency"), Some("0.015".into()))?;
    check_eq(el.get_attribute("numOctaves"), Some("4".into()))?;
    check_eq(el.get_attribute("seed"), Some("2.5".into()))
}

/// Every `TurbulenceType` variant writes its exact SVG keyword.
#[wasm_bindgen_test]
fn should_write_every_turbulence_type_keyword() -> Result<(), String> {
    let svg = make_svg("filter-turbulence-types");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("ftk").map_err(|e| e.to_string())?;
    let cases = [
        (TurbulenceType::Turbulence, "turbulence"),
        (TurbulenceType::FractalNoise, "fractalNoise"),
    ];
    for (turbulence_type, expected) in cases {
        let turb = filter.turbulence(0.02, 1, 0.0, turbulence_type).map_err(|e| e.to_string())?;
        check_eq(turb.as_element().get_attribute("type"), Some(expected.into()))?;
    }
    Ok(())
}

/// The generic `SvgNode::set_attr` escape hatch on the returned primitive node covers attributes not yet wrapped by
/// a named parameter, such as `result` — needed when the noise is consumed as `in2` (its usual role, feeding
/// `displacement_map`) or referenced by a non-immediately-downstream primitive, not simply because `<feTurbulence>`
/// has no `in`.
#[wasm_bindgen_test]
fn should_set_result_via_generic_escape_hatch() -> Result<(), String> {
    let svg = make_svg("filter-turbulence-result");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("ftr").map_err(|e| e.to_string())?;
    let turb = filter
        .turbulence(0.02, 3, 1.0, TurbulenceType::Turbulence)
        .map_err(|e| e.to_string())?;
    turb.set_attr("result", "noise").map_err(|e| e.to_string())?;
    check_eq(turb.as_element().get_attribute("result"), Some("noise".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// turbulence_xy primitive
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `turbulence_xy` appends a `<feTurbulence>` child to the `<filter>` element.
#[wasm_bindgen_test]
fn should_add_turbulence_xy_to_filter() -> Result<(), String> {
    let svg = make_svg("filter-turbulence-xy-child");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("ftxy").map_err(|e| e.to_string())?;
    filter
        .turbulence_xy(0.05, 0.005, 3, 7.0, TurbulenceType::Turbulence)
        .map_err(|e| e.to_string())?;
    check_eq(filter.as_element().child_element_count(), 1)
}

/// The appended child has tag name `"feTurbulence"`, the same element `turbulence` produces.
#[wasm_bindgen_test]
fn should_create_fe_turbulence_element_via_xy() -> Result<(), String> {
    let svg = make_svg("filter-turbulence-xy-tag");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("ftxyt").map_err(|e| e.to_string())?;
    let turb = filter
        .turbulence_xy(0.05, 0.005, 3, 7.0, TurbulenceType::Turbulence)
        .map_err(|e| e.to_string())?;
    check_eq(turb.as_element().tag_name(), "feTurbulence".to_owned())
}

/// `turbulence_xy(0.05, 0.005, ...)` writes the two-number `baseFrequency="0.05 0.005"` form in a single
/// attribute, exactly as the SVG `<number-optional-number>` grammar for `baseFrequency` requires.
#[wasm_bindgen_test]
fn should_set_base_frequency_as_two_numbers() -> Result<(), String> {
    let svg = make_svg("filter-turbulence-xy-freq");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("ftxyf").map_err(|e| e.to_string())?;
    let turb = filter
        .turbulence_xy(0.05, 0.005, 3, 7.0, TurbulenceType::Turbulence)
        .map_err(|e| e.to_string())?;
    check_eq(turb.as_element().get_attribute("baseFrequency"), Some("0.05 0.005".into()))
}

/// The generic `SvgNode::set_attr` escape hatch works identically on a `turbulence_xy` node as on a `turbulence`
/// one, since both return the same kind of handle around the same element.
#[wasm_bindgen_test]
fn should_set_result_on_turbulence_xy_via_generic_escape_hatch() -> Result<(), String> {
    let svg = make_svg("filter-turbulence-xy-result");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("ftxyr").map_err(|e| e.to_string())?;
    let turb = filter
        .turbulence_xy(0.05, 0.005, 3, 7.0, TurbulenceType::Turbulence)
        .map_err(|e| e.to_string())?;
    turb.set_attr("result", "grain").map_err(|e| e.to_string())?;
    check_eq(turb.as_element().get_attribute("result"), Some("grain".into()))
}
