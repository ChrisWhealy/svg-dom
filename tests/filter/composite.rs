use crate::common::*;
use svg_dom::CompositeOperator;
use wasm_bindgen_test::*;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// composite primitive
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `composite` appends a `<feComposite>` child to the `<filter>` element.
#[wasm_bindgen_test]
fn should_add_composite_to_filter() -> Result<(), String> {
    let svg = make_svg("filter-composite-child");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fcp").map_err(|e| e.to_string())?;
    filter.composite("blur", CompositeOperator::In).map_err(|e| e.to_string())?;
    check_eq(filter.as_element().child_element_count(), 1)
}

/// The appended child has tag name `"feComposite"`.
#[wasm_bindgen_test]
fn should_create_fe_composite_element() -> Result<(), String> {
    let svg = make_svg("filter-composite-tag");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fcpt").map_err(|e| e.to_string())?;
    let composite = filter.composite("blur", CompositeOperator::In).map_err(|e| e.to_string())?;
    check_eq(composite.as_element().tag_name(), "feComposite".to_owned())
}

/// `composite` writes the `in2` and `operator` attributes.
#[wasm_bindgen_test]
fn should_set_in2_and_operator() -> Result<(), String> {
    let svg = make_svg("filter-composite-attrs");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fca").map_err(|e| e.to_string())?;
    let composite = filter.composite("blur", CompositeOperator::In).map_err(|e| e.to_string())?;
    check_eq(composite.as_element().get_attribute("in2"), Some("blur".into()))?;
    check_eq(composite.as_element().get_attribute("operator"), Some("in".into()))
}

/// Every `CompositeOperator` variant writes its exact SVG keyword.
#[wasm_bindgen_test]
fn should_write_every_composite_operator_keyword() -> Result<(), String> {
    let svg = make_svg("filter-composite-operators");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fco").map_err(|e| e.to_string())?;
    let cases = [
        (CompositeOperator::Over, "over"),
        (CompositeOperator::In, "in"),
        (CompositeOperator::Out, "out"),
        (CompositeOperator::Atop, "atop"),
        (CompositeOperator::Xor, "xor"),
        (CompositeOperator::Lighter, "lighter"),
        (CompositeOperator::Arithmetic, "arithmetic"),
    ];
    for (operator, expected) in cases {
        let composite = filter.composite("blur", operator).map_err(|e| e.to_string())?;
        check_eq(composite.as_element().get_attribute("operator"), Some(expected.into()))?;
    }
    Ok(())
}
