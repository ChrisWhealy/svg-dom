use crate::common::*;
use svg_dom::BlendMode;
use wasm_bindgen_test::*;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// blend primitive
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `blend` appends a `<feBlend>` child to the `<filter>` element.
#[wasm_bindgen_test]
fn should_add_blend_to_filter() -> Result<(), String> {
    let svg = make_svg("filter-blend-child");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fbl").map_err(|e| e.to_string())?;
    filter.blend("colour", BlendMode::Multiply).map_err(|e| e.to_string())?;
    check_eq(filter.as_element().child_element_count(), 1)
}

/// The appended child has tag name `"feBlend"`.
#[wasm_bindgen_test]
fn should_create_fe_blend_element() -> Result<(), String> {
    let svg = make_svg("filter-blend-tag");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fblt").map_err(|e| e.to_string())?;
    let blend = filter.blend("colour", BlendMode::Multiply).map_err(|e| e.to_string())?;
    check_eq(blend.as_element().tag_name(), "feBlend".to_owned())
}

/// `blend` writes the `in2` and `mode` attributes.
#[wasm_bindgen_test]
fn should_set_in2_and_mode() -> Result<(), String> {
    let svg = make_svg("filter-blend-attrs");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fbla").map_err(|e| e.to_string())?;
    let blend = filter.blend("colour", BlendMode::Screen).map_err(|e| e.to_string())?;
    check_eq(blend.as_element().get_attribute("in2"), Some("colour".into()))?;
    check_eq(blend.as_element().get_attribute("mode"), Some("screen".into()))
}

/// Every `BlendMode` variant writes its exact SVG/CSS keyword.
#[wasm_bindgen_test]
fn should_write_every_blend_mode_keyword() -> Result<(), String> {
    let svg = make_svg("filter-blend-modes");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fblm").map_err(|e| e.to_string())?;
    let cases = [
        (BlendMode::Normal, "normal"),
        (BlendMode::Multiply, "multiply"),
        (BlendMode::Screen, "screen"),
        (BlendMode::Darken, "darken"),
        (BlendMode::Lighten, "lighten"),
        (BlendMode::Overlay, "overlay"),
        (BlendMode::ColorDodge, "color-dodge"),
        (BlendMode::ColorBurn, "color-burn"),
        (BlendMode::HardLight, "hard-light"),
        (BlendMode::SoftLight, "soft-light"),
        (BlendMode::Difference, "difference"),
        (BlendMode::Exclusion, "exclusion"),
        (BlendMode::Hue, "hue"),
        (BlendMode::Saturation, "saturation"),
        (BlendMode::Color, "color"),
        (BlendMode::Luminosity, "luminosity"),
    ];
    for (mode, expected) in cases {
        let blend = filter.blend("colour", mode).map_err(|e| e.to_string())?;
        check_eq(blend.as_element().get_attribute("mode"), Some(expected.into()))?;
    }
    Ok(())
}

/// The generic `SvgNode::set_attr` escape hatch works on a `blend` node the same as on every other primitive, for
/// attributes like `in`/`result` not wrapped by a named parameter.
#[wasm_bindgen_test]
fn should_set_result_on_blend_via_generic_escape_hatch() -> Result<(), String> {
    let svg = make_svg("filter-blend-result");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fblr").map_err(|e| e.to_string())?;
    let blend = filter.blend("colour", BlendMode::Multiply).map_err(|e| e.to_string())?;
    blend.set_attr("result", "tinted").map_err(|e| e.to_string())?;
    check_eq(blend.as_element().get_attribute("result"), Some("tinted".into()))
}
