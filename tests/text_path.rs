mod common;

use common::*;
use svg_dom::{TextPathMethod, TextPathSide, TextPathSpacing, root::utils::Point};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// text_path creation
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `text_path` creates an element with tag name `"textPath"`.
#[wasm_bindgen_test]
fn should_create_text_path_element() -> Result<(), String> {
    let svg = make_svg("text-path-tag");
    let txt = svg.text(Point::new(10.0, 20.0), "").map_err(|e| e.to_string())?;
    let path_text = txt.text_path("#arc", "hello").map_err(|e| e.to_string())?;
    check_eq(path_text.as_element().tag_name(), "textPath".to_owned())
}

/// The `<textPath>` is a child of the `<text>` element.
#[wasm_bindgen_test]
fn should_append_text_path_to_text() -> Result<(), String> {
    let svg = make_svg("text-path-parent");
    let txt = svg.text(Point::new(10.0, 20.0), "").map_err(|e| e.to_string())?;
    let path_text = txt.text_path("#arc", "content").map_err(|e| e.to_string())?;
    let parent = path_text.as_element().parent_element().ok_or("textPath has no parent")?;
    check_eq(parent.tag_name(), "text".to_owned())
}

/// `text_path` sets the element's text content.
#[wasm_bindgen_test]
fn should_set_text_path_content() -> Result<(), String> {
    let svg = make_svg("text-path-content");
    let txt = svg.text(Point::new(10.0, 20.0), "").map_err(|e| e.to_string())?;
    let path_text = txt.text_path("#arc", "hello world").map_err(|e| e.to_string())?;
    check_eq(path_text.as_element().text_content(), Some("hello world".to_owned()))
}

/// `text_path` writes the `href` attribute to the supplied fragment reference.
#[wasm_bindgen_test]
fn should_set_href_via_text_path() -> Result<(), String> {
    let svg = make_svg("text-path-href");
    let txt = svg.text(Point::new(10.0, 20.0), "").map_err(|e| e.to_string())?;
    let path_text = txt.text_path("#arc", "hello").map_err(|e| e.to_string())?;
    check_eq(path_text.as_element().get_attribute("href"), Some("#arc".to_owned()))
}

/// Multiple `text_path` calls produce sibling elements in order.
#[wasm_bindgen_test]
fn should_append_multiple_text_paths_in_order() -> Result<(), String> {
    let svg = make_svg("text-path-order");
    let txt = svg.text(Point::new(10.0, 20.0), "").map_err(|e| e.to_string())?;
    txt.text_path("#arc-a", "first").map_err(|e| e.to_string())?;
    txt.text_path("#arc-b", "second").map_err(|e| e.to_string())?;
    let children = txt.as_element().child_element_count();
    check_eq(children, 2)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// set_start_offset
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `set_start_offset` writes the `startOffset` attribute as a number.
#[wasm_bindgen_test]
fn should_write_start_offset_attribute() -> Result<(), String> {
    let svg = make_svg("text-path-start-offset");
    let txt = svg.text(Point::new(10.0, 20.0), "").map_err(|e| e.to_string())?;
    let path_text = txt.text_path("#arc", "hello").map_err(|e| e.to_string())?;
    path_text.set_start_offset(42.5).map_err(|e| e.to_string())?;
    check_eq(path_text.as_element().get_attribute("startOffset"), Some("42.5".to_owned()))
}

/// The generic `set_attr` escape hatch accepts a percentage `startOffset` value.
#[wasm_bindgen_test]
fn should_allow_percentage_start_offset_via_set_attr() -> Result<(), String> {
    let svg = make_svg("text-path-start-offset-pct");
    let txt = svg.text(Point::new(10.0, 20.0), "").map_err(|e| e.to_string())?;
    let path_text = txt.text_path("#arc", "hello").map_err(|e| e.to_string())?;
    path_text.set_attr("startOffset", "50%").map_err(|e| e.to_string())?;
    check_eq(path_text.as_element().get_attribute("startOffset"), Some("50%".to_owned()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// set_text_path_method / set_text_path_spacing / set_text_path_side
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `set_text_path_method` writes the `method` attribute.
#[wasm_bindgen_test]
fn should_write_method_attribute() -> Result<(), String> {
    let svg = make_svg("text-path-method");
    let txt = svg.text(Point::new(10.0, 20.0), "").map_err(|e| e.to_string())?;
    let path_text = txt.text_path("#arc", "hello").map_err(|e| e.to_string())?;
    path_text
        .set_text_path_method(TextPathMethod::Stretch)
        .map_err(|e| e.to_string())?;
    check_eq(path_text.as_element().get_attribute("method"), Some("stretch".to_owned()))
}

/// `set_text_path_spacing` writes the `spacing` attribute.
#[wasm_bindgen_test]
fn should_write_spacing_attribute() -> Result<(), String> {
    let svg = make_svg("text-path-spacing");
    let txt = svg.text(Point::new(10.0, 20.0), "").map_err(|e| e.to_string())?;
    let path_text = txt.text_path("#arc", "hello").map_err(|e| e.to_string())?;
    path_text
        .set_text_path_spacing(TextPathSpacing::Exact)
        .map_err(|e| e.to_string())?;
    check_eq(path_text.as_element().get_attribute("spacing"), Some("exact".to_owned()))
}

/// `set_text_path_side` writes the `side` attribute.
#[wasm_bindgen_test]
fn should_write_side_attribute() -> Result<(), String> {
    let svg = make_svg("text-path-side");
    let txt = svg.text(Point::new(10.0, 20.0), "").map_err(|e| e.to_string())?;
    let path_text = txt.text_path("#arc", "hello").map_err(|e| e.to_string())?;
    path_text.set_text_path_side(TextPathSide::Right).map_err(|e| e.to_string())?;
    check_eq(path_text.as_element().get_attribute("side"), Some("right".to_owned()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Styling overrides
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// The usual text-styling helpers work on the `<textPath>` handle, overriding the parent `<text>`.
#[wasm_bindgen_test]
fn should_allow_fill_and_font_size_overrides() -> Result<(), String> {
    let svg = make_svg("text-path-style");
    let txt = svg.text(Point::new(10.0, 20.0), "").map_err(|e| e.to_string())?;
    let path_text = txt.text_path("#arc", "hello").map_err(|e| e.to_string())?;
    path_text.set_fill("coral").map_err(|e| e.to_string())?;
    path_text.set_font_size(18.0).map_err(|e| e.to_string())?;
    check_eq(path_text.as_element().get_attribute("fill"), Some("coral".to_owned()))?;
    check_eq(path_text.as_element().get_attribute("font-size"), Some("18".to_owned()))
}
