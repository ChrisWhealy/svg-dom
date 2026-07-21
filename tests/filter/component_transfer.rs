use crate::common::*;
use svg_dom::{Channel, Error, TransferFunction};
use wasm_bindgen_test::*;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// component_transfer primitive
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `component_transfer` appends a `<feComponentTransfer>` child to the `<filter>` element.
#[wasm_bindgen_test]
fn should_add_component_transfer_to_filter() -> Result<(), String> {
    let svg = make_svg("filter-component-transfer-child");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fct").map_err(|e| e.to_string())?;
    filter
        .component_transfer(&[(Channel::Alpha, TransferFunction::Linear { slope: 0.5, intercept: 0.0 })])
        .map_err(|e| e.to_string())?;
    check_eq(filter.as_element().child_element_count(), 1)
}

/// The appended child has tag name `"feComponentTransfer"`.
#[wasm_bindgen_test]
fn should_create_fe_component_transfer_element() -> Result<(), String> {
    let svg = make_svg("filter-component-transfer-tag");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fctt").map_err(|e| e.to_string())?;
    let ct = filter
        .component_transfer(&[(Channel::Alpha, TransferFunction::Linear { slope: 0.5, intercept: 0.0 })])
        .map_err(|e| e.to_string())?;
    check_eq(ct.as_element().tag_name(), "feComponentTransfer".to_owned())
}

/// `component_transfer` creates exactly one `<feFuncX>` child per entry in `funcs`, in the order given, with a
/// tag name matching the requested `Channel`.
#[wasm_bindgen_test]
fn should_create_one_fe_func_child_per_channel_in_order() -> Result<(), String> {
    let svg = make_svg("filter-component-transfer-children");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fctc").map_err(|e| e.to_string())?;
    let ct = filter
        .component_transfer(&[
            (Channel::Alpha, TransferFunction::Identity),
            (Channel::Red, TransferFunction::Identity),
            (Channel::Green, TransferFunction::Identity),
            (Channel::Blue, TransferFunction::Identity),
        ])
        .map_err(|e| e.to_string())?;
    let el = ct.as_element();
    check_eq(el.child_element_count(), 4)?;
    let children = el.children();
    check_eq(children.item(0).map(|c| c.tag_name()), Some("feFuncA".to_owned()))?;
    check_eq(children.item(1).map(|c| c.tag_name()), Some("feFuncR".to_owned()))?;
    check_eq(children.item(2).map(|c| c.tag_name()), Some("feFuncG".to_owned()))?;
    check_eq(children.item(3).map(|c| c.tag_name()), Some("feFuncB".to_owned()))
}

/// Naming the same `Channel` twice in `funcs` is not deduplicated: both `<feFuncX>` children are created, in the
/// order given. (Per the SVG spec only the last one has any rendered effect, but that is a browser-side rendering
/// rule, not something this crate enforces or hides — see `Channel::Alpha`'s doc comment for the analogous
/// f(0) > 0 caveat.)
#[wasm_bindgen_test]
fn should_create_both_children_for_a_duplicated_channel_in_order() -> Result<(), String> {
    let svg = make_svg("filter-component-transfer-duplicate-channel");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fctdup").map_err(|e| e.to_string())?;
    let ct = filter
        .component_transfer(&[
            (Channel::Red, TransferFunction::Linear { slope: 1.0, intercept: 0.0 }),
            (Channel::Red, TransferFunction::Linear { slope: 0.5, intercept: 0.1 }),
        ])
        .map_err(|e| e.to_string())?;
    let el = ct.as_element();
    check_eq(el.child_element_count(), 2)?;
    let children = el.children();
    let first = children.item(0).ok_or("expected a first feFuncR child")?;
    let second = children.item(1).ok_or("expected a second feFuncR child")?;
    check_eq(first.tag_name(), "feFuncR".to_owned())?;
    check_eq(second.tag_name(), "feFuncR".to_owned())?;
    check_eq(first.get_attribute("slope"), Some("1".into()))?;
    check_eq(first.get_attribute("intercept"), Some("0".into()))?;
    check_eq(second.get_attribute("slope"), Some("0.5".into()))?;
    check_eq(second.get_attribute("intercept"), Some("0.1".into()))
}

/// A channel not named in `funcs` gets no `<feFuncX>` child at all.
#[wasm_bindgen_test]
fn should_omit_fe_func_child_for_unmentioned_channel() -> Result<(), String> {
    let svg = make_svg("filter-component-transfer-omit");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fcto").map_err(|e| e.to_string())?;
    let ct = filter
        .component_transfer(&[(Channel::Alpha, TransferFunction::Identity)])
        .map_err(|e| e.to_string())?;
    check_eq(ct.as_element().child_element_count(), 1)
}

/// `TransferFunction::Identity` writes `type="identity"` and no other attribute.
#[wasm_bindgen_test]
fn should_set_identity_type_and_no_other_attribute() -> Result<(), String> {
    let svg = make_svg("filter-component-transfer-identity");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fcti").map_err(|e| e.to_string())?;
    let ct = filter
        .component_transfer(&[(Channel::Alpha, TransferFunction::Identity)])
        .map_err(|e| e.to_string())?;
    let func = ct.as_element().first_element_child().ok_or("expected a <feFuncA> child")?;
    check_eq(func.get_attribute("type"), Some("identity".into()))?;
    check_eq(func.get_attribute("tableValues"), None)?;
    check_eq(func.get_attribute("slope"), None)
}

/// `TransferFunction::Table` writes `type="table"` and `tableValues` as the space-separated values, in order.
#[wasm_bindgen_test]
fn should_set_table_type_and_values() -> Result<(), String> {
    let svg = make_svg("filter-component-transfer-table");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fctt2").map_err(|e| e.to_string())?;
    let ct = filter
        .component_transfer(&[(Channel::Red, TransferFunction::Table(vec![0.0, 0.5, 1.0]))])
        .map_err(|e| e.to_string())?;
    let func = ct.as_element().first_element_child().ok_or("expected a <feFuncR> child")?;
    check_eq(func.get_attribute("type"), Some("table".into()))?;
    check_eq(func.get_attribute("tableValues"), Some("0 0.5 1".into()))
}

/// `TransferFunction::Table` with an empty list writes `type="table"` and an empty `tableValues` — valid SVG syntax
/// (per spec, equivalent to identity), not a crate-level error.
#[wasm_bindgen_test]
fn should_accept_empty_table_values() -> Result<(), String> {
    let svg = make_svg("filter-component-transfer-table-empty");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fcte").map_err(|e| e.to_string())?;
    let ct = filter
        .component_transfer(&[(Channel::Red, TransferFunction::Table(vec![]))])
        .map_err(|e| e.to_string())?;
    let func = ct.as_element().first_element_child().ok_or("expected a <feFuncR> child")?;
    check_eq(func.get_attribute("type"), Some("table".into()))?;
    check_eq(func.get_attribute("tableValues"), Some("".into()))
}

/// A `TransferFunction::Table` with exactly one value has no defined SVG semantics (the `n+1`-values-describe-`n`-
/// regions formula leaves zero regions to interpolate across) and is rejected before reaching the DOM.
#[wasm_bindgen_test]
fn should_reject_single_value_table() -> Result<(), String> {
    let svg = make_svg("filter-component-transfer-table-single");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fcts1").map_err(|e| e.to_string())?;
    let result = filter.component_transfer(&[(Channel::Red, TransferFunction::Table(vec![0.5]))]);
    check(
        matches!(result, Err(Error::InvalidTransferTable)),
        "expected InvalidTransferTable error for a single-value Table",
    )
}

/// Two equal values is the documented, portable way to write a constant `Table` transfer function.
#[wasm_bindgen_test]
fn should_accept_two_equal_table_values_as_constant_workaround() -> Result<(), String> {
    let svg = make_svg("filter-component-transfer-table-constant");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fctcv").map_err(|e| e.to_string())?;
    let ct = filter
        .component_transfer(&[(Channel::Red, TransferFunction::Table(vec![0.5, 0.5]))])
        .map_err(|e| e.to_string())?;
    let func = ct.as_element().first_element_child().ok_or("expected a <feFuncR> child")?;
    check_eq(func.get_attribute("tableValues"), Some("0.5 0.5".into()))
}

/// Unlike `Table`, `TransferFunction::Discrete` with a single value is well-defined by the SVG "discrete" stepping
/// formula (every input maps to the one entry), so it is accepted rather than rejected.
#[wasm_bindgen_test]
fn should_accept_single_value_discrete() -> Result<(), String> {
    let svg = make_svg("filter-component-transfer-discrete-single");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fcds1").map_err(|e| e.to_string())?;
    let ct = filter
        .component_transfer(&[(Channel::Red, TransferFunction::Discrete(vec![0.5]))])
        .map_err(|e| e.to_string())?;
    let func = ct.as_element().first_element_child().ok_or("expected a <feFuncR> child")?;
    check_eq(func.get_attribute("tableValues"), Some("0.5".into()))
}

/// `TransferFunction::Discrete` writes `type="discrete"` and `tableValues` as the space-separated values.
#[wasm_bindgen_test]
fn should_set_discrete_type_and_values() -> Result<(), String> {
    let svg = make_svg("filter-component-transfer-discrete");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fctd").map_err(|e| e.to_string())?;
    let ct = filter
        .component_transfer(&[(Channel::Green, TransferFunction::Discrete(vec![0.0, 0.25, 0.5, 0.75, 1.0]))])
        .map_err(|e| e.to_string())?;
    let func = ct.as_element().first_element_child().ok_or("expected a <feFuncG> child")?;
    check_eq(func.get_attribute("type"), Some("discrete".into()))?;
    check_eq(func.get_attribute("tableValues"), Some("0 0.25 0.5 0.75 1".into()))
}

/// `TransferFunction::Linear` writes `type="linear"`, `slope`, and `intercept`.
#[wasm_bindgen_test]
fn should_set_linear_type_slope_and_intercept() -> Result<(), String> {
    let svg = make_svg("filter-component-transfer-linear");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fctl").map_err(|e| e.to_string())?;
    let ct = filter
        .component_transfer(&[(Channel::Alpha, TransferFunction::Linear { slope: 0.6, intercept: 0.1 })])
        .map_err(|e| e.to_string())?;
    let func = ct.as_element().first_element_child().ok_or("expected a <feFuncA> child")?;
    check_eq(func.get_attribute("type"), Some("linear".into()))?;
    check_eq(func.get_attribute("slope"), Some("0.6".into()))?;
    check_eq(func.get_attribute("intercept"), Some("0.1".into()))
}

/// `TransferFunction::Gamma` writes `type="gamma"`, `amplitude`, `exponent`, and `offset`.
#[wasm_bindgen_test]
fn should_set_gamma_type_amplitude_exponent_and_offset() -> Result<(), String> {
    let svg = make_svg("filter-component-transfer-gamma");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fctg").map_err(|e| e.to_string())?;
    let ct = filter
        .component_transfer(&[(
            Channel::Red,
            TransferFunction::Gamma {
                amplitude: 1.0,
                exponent: 0.45,
                offset: 0.0,
            },
        )])
        .map_err(|e| e.to_string())?;
    let func = ct.as_element().first_element_child().ok_or("expected a <feFuncR> child")?;
    check_eq(func.get_attribute("type"), Some("gamma".into()))?;
    check_eq(func.get_attribute("amplitude"), Some("1".into()))?;
    check_eq(func.get_attribute("exponent"), Some("0.45".into()))?;
    check_eq(func.get_attribute("offset"), Some("0".into()))
}

/// The generic `SvgNode::set_attr` escape hatch works on a `component_transfer` node the same as on every other
/// primitive, for attributes like `in`/`result` not wrapped by a named parameter.
#[wasm_bindgen_test]
fn should_set_result_on_component_transfer_via_generic_escape_hatch() -> Result<(), String> {
    let svg = make_svg("filter-component-transfer-result");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fctr").map_err(|e| e.to_string())?;
    let ct = filter
        .component_transfer(&[(Channel::Alpha, TransferFunction::Identity)])
        .map_err(|e| e.to_string())?;
    ct.set_attr("result", "faded").map_err(|e| e.to_string())?;
    check_eq(ct.as_element().get_attribute("result"), Some("faded".into()))
}
