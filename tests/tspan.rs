mod common;

use common::*;
use svg_dom::root::utils::Point;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// tspan creation
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `tspan` creates an element with tag name `"tspan"`.
#[wasm_bindgen_test]
fn should_create_tspan_element() -> Result<(), String> {
    let svg = make_svg("tspan-tag");
    let txt = svg.text(Point::new(10.0, 20.0), "").map_err(|e| e.to_string())?;
    let span = txt.tspan("hello").map_err(|e| e.to_string())?;
    check_eq(span.as_element().tag_name(), "tspan".to_owned())
}

/// The `<tspan>` is a child of the `<text>` element.
#[wasm_bindgen_test]
fn should_append_tspan_to_text() -> Result<(), String> {
    let svg = make_svg("tspan-parent");
    let txt = svg.text(Point::new(10.0, 20.0), "").map_err(|e| e.to_string())?;
    let span = txt.tspan("content").map_err(|e| e.to_string())?;
    let parent = span
        .as_element()
        .parent_element()
        .ok_or("tspan has no parent")?;
    check_eq(parent.tag_name(), "text".to_owned())
}

/// `tspan` sets the element's text content.
#[wasm_bindgen_test]
fn should_set_tspan_text_content() -> Result<(), String> {
    let svg = make_svg("tspan-content");
    let txt = svg.text(Point::new(10.0, 20.0), "").map_err(|e| e.to_string())?;
    let span = txt.tspan("hello world").map_err(|e| e.to_string())?;
    check_eq(span.as_element().text_content(), Some("hello world".to_owned()))
}

/// Multiple `tspan` calls produce sibling elements in order.
#[wasm_bindgen_test]
fn should_append_multiple_tspans_in_order() -> Result<(), String> {
    let svg = make_svg("tspan-order");
    let txt = svg.text(Point::new(10.0, 20.0), "").map_err(|e| e.to_string())?;
    txt.tspan("first").map_err(|e| e.to_string())?;
    txt.tspan("second").map_err(|e| e.to_string())?;
    txt.tspan("third").map_err(|e| e.to_string())?;
    let children = txt.as_element().child_element_count();
    check_eq(children, 3)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// tspan_dy
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `tspan_dy` creates a `<tspan>` and writes the `dy` attribute.
#[wasm_bindgen_test]
fn should_set_dy_via_tspan_dy() -> Result<(), String> {
    let svg = make_svg("tspan-dy-attr");
    let txt = svg.text(Point::new(10.0, 20.0), "").map_err(|e| e.to_string())?;
    let span = txt.tspan_dy(18.0, "line two").map_err(|e| e.to_string())?;
    check_eq(span.as_element().get_attribute("dy"), Some("18".to_owned()))
}

/// `tspan_dy` also sets the text content.
#[wasm_bindgen_test]
fn should_set_content_via_tspan_dy() -> Result<(), String> {
    let svg = make_svg("tspan-dy-content");
    let txt = svg.text(Point::new(10.0, 20.0), "").map_err(|e| e.to_string())?;
    let span = txt.tspan_dy(18.0, "line two").map_err(|e| e.to_string())?;
    check_eq(span.as_element().text_content(), Some("line two".to_owned()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// set_dy / set_dx
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `set_dy` writes the `dy` attribute as a number.
#[wasm_bindgen_test]
fn should_write_dy_attribute() -> Result<(), String> {
    let svg = make_svg("tspan-set-dy");
    let txt = svg.text(Point::new(10.0, 20.0), "").map_err(|e| e.to_string())?;
    let span = txt.tspan("shifted").map_err(|e| e.to_string())?;
    span.set_dy(24.0).map_err(|e| e.to_string())?;
    check_eq(span.as_element().get_attribute("dy"), Some("24".to_owned()))
}

/// `set_dx` writes the `dx` attribute as a number.
#[wasm_bindgen_test]
fn should_write_dx_attribute() -> Result<(), String> {
    let svg = make_svg("tspan-set-dx");
    let txt = svg.text(Point::new(10.0, 20.0), "").map_err(|e| e.to_string())?;
    let span = txt.tspan("shifted").map_err(|e| e.to_string())?;
    span.set_dx(8.0).map_err(|e| e.to_string())?;
    check_eq(span.as_element().get_attribute("dx"), Some("8".to_owned()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Nesting: tspan inside tspan
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `tspan` can be called on another `<tspan>` to produce nested spans.
#[wasm_bindgen_test]
fn should_nest_tspan_inside_tspan() -> Result<(), String> {
    let svg = make_svg("tspan-nested");
    let txt = svg.text(Point::new(10.0, 20.0), "").map_err(|e| e.to_string())?;
    let outer = txt.tspan("outer ").map_err(|e| e.to_string())?;
    let inner = outer.tspan("inner").map_err(|e| e.to_string())?;
    let parent = inner
        .as_element()
        .parent_element()
        .ok_or("inner tspan has no parent")?;
    check_eq(parent.tag_name(), "tspan".to_owned())
}
