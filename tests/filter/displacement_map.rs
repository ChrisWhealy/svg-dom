use crate::common::*;
use svg_dom::Channel;
use wasm_bindgen_test::*;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// displacement_map primitive
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `displacement_map` appends a `<feDisplacementMap>` child to the `<filter>` element.
#[wasm_bindgen_test]
fn should_add_displacement_map_to_filter() -> Result<(), String> {
    let svg = make_svg("filter-displacement-map-child");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fdm").map_err(|e| e.to_string())?;
    filter
        .displacement_map("noise", 20.0, Channel::Alpha, Channel::Alpha)
        .map_err(|e| e.to_string())?;
    check_eq(filter.as_element().child_element_count(), 1)
}

/// The appended child has tag name `"feDisplacementMap"`.
#[wasm_bindgen_test]
fn should_create_fe_displacement_map_element() -> Result<(), String> {
    let svg = make_svg("filter-displacement-map-tag");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fdmt").map_err(|e| e.to_string())?;
    let dm = filter
        .displacement_map("noise", 20.0, Channel::Alpha, Channel::Alpha)
        .map_err(|e| e.to_string())?;
    check_eq(dm.as_element().tag_name(), "feDisplacementMap".to_owned())
}

/// `displacement_map` writes `in2`, `scale`, `xChannelSelector`, and `yChannelSelector`.
#[wasm_bindgen_test]
fn should_set_in2_scale_and_channel_selectors() -> Result<(), String> {
    let svg = make_svg("filter-displacement-map-attrs");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fdma").map_err(|e| e.to_string())?;
    let dm = filter
        .displacement_map("noise", 24.0, Channel::Red, Channel::Green)
        .map_err(|e| e.to_string())?;
    let el = dm.as_element();
    check_eq(el.get_attribute("in2"), Some("noise".into()))?;
    check_eq(el.get_attribute("scale"), Some("24".into()))?;
    check_eq(el.get_attribute("xChannelSelector"), Some("R".into()))?;
    check_eq(el.get_attribute("yChannelSelector"), Some("G".into()))
}

/// Every `Channel` variant writes its exact single-letter SVG keyword for `xChannelSelector`.
#[wasm_bindgen_test]
fn should_write_every_channel_selector_keyword() -> Result<(), String> {
    let svg = make_svg("filter-displacement-map-selectors");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fdms").map_err(|e| e.to_string())?;
    let cases = [
        (Channel::Red, "R"),
        (Channel::Green, "G"),
        (Channel::Blue, "B"),
        (Channel::Alpha, "A"),
    ];
    for (channel, expected) in cases {
        let dm = filter
            .displacement_map("noise", 10.0, channel, channel)
            .map_err(|e| e.to_string())?;
        check_eq(dm.as_element().get_attribute("xChannelSelector"), Some(expected.into()))?;
        check_eq(dm.as_element().get_attribute("yChannelSelector"), Some(expected.into()))?;
    }
    Ok(())
}

/// The generic `SvgNode::set_attr` escape hatch works on a `displacement_map` node the same as on every other
/// primitive, for attributes like `in`/`result` not wrapped by a named parameter.
#[wasm_bindgen_test]
fn should_set_result_on_displacement_map_via_generic_escape_hatch() -> Result<(), String> {
    let svg = make_svg("filter-displacement-map-result");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fdmr").map_err(|e| e.to_string())?;
    let dm = filter
        .displacement_map("noise", 20.0, Channel::Alpha, Channel::Alpha)
        .map_err(|e| e.to_string())?;
    dm.set_attr("in", "SourceGraphic").map_err(|e| e.to_string())?;
    check_eq(dm.as_element().get_attribute("in"), Some("SourceGraphic".into()))
}
