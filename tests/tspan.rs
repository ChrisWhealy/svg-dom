mod common;

use common::*;
use svg_dom::root::utils::Point;
use wasm_bindgen::JsCast;
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
    let parent = span.as_element().parent_element().ok_or("tspan has no parent")?;
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
    let parent = inner.as_element().parent_element().ok_or("inner tspan has no parent")?;
    check_eq(parent.tag_name(), "tspan".to_owned())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// tspan_line
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `tspan_line` creates an element with tag name `"tspan"`.
#[wasm_bindgen_test]
fn should_create_tspan_line_element() -> Result<(), String> {
    let svg = make_svg("tspan-line-tag");
    let txt = svg.text(Point::new(20.0, 50.0), "").map_err(|e| e.to_string())?;
    let span = txt.tspan_line(20.0, 20.0, "Line").map_err(|e| e.to_string())?;
    check_eq(span.as_element().tag_name(), "tspan".to_owned())
}

/// `tspan_line` appends the span to its parent element.
#[wasm_bindgen_test]
fn should_append_tspan_line_to_text() -> Result<(), String> {
    let svg = make_svg("tspan-line-parent");
    let txt = svg.text(Point::new(20.0, 50.0), "").map_err(|e| e.to_string())?;
    let span = txt.tspan_line(20.0, 20.0, "Line").map_err(|e| e.to_string())?;
    let parent = span.as_element().parent_element().ok_or("tspan_line has no parent")?;
    check_eq(parent.tag_name(), "text".to_owned())
}

/// `tspan_line` sets the text content.
#[wasm_bindgen_test]
fn should_set_content_via_tspan_line() -> Result<(), String> {
    let svg = make_svg("tspan-line-content");
    let txt = svg.text(Point::new(20.0, 50.0), "").map_err(|e| e.to_string())?;
    let span = txt.tspan_line(20.0, 20.0, "hello").map_err(|e| e.to_string())?;
    check_eq(span.as_element().text_content(), Some("hello".to_owned()))
}

/// `tspan_line` writes the `x` attribute.
#[wasm_bindgen_test]
fn should_set_x_via_tspan_line() -> Result<(), String> {
    let svg = make_svg("tspan-line-x");
    let txt = svg.text(Point::new(20.0, 50.0), "").map_err(|e| e.to_string())?;
    let span = txt.tspan_line(35.0, 20.0, "Line").map_err(|e| e.to_string())?;
    check_eq(span.as_element().get_attribute("x"), Some("35".to_owned()))
}

/// `tspan_line` writes the `dy` attribute.
#[wasm_bindgen_test]
fn should_set_dy_via_tspan_line() -> Result<(), String> {
    let svg = make_svg("tspan-line-dy");
    let txt = svg.text(Point::new(20.0, 50.0), "").map_err(|e| e.to_string())?;
    let span = txt.tspan_line(20.0, 18.0, "Line").map_err(|e| e.to_string())?;
    check_eq(span.as_element().get_attribute("dy"), Some("18".to_owned()))
}

/// Both `x` and `dy` are set on the same element.
#[wasm_bindgen_test]
fn should_set_both_x_and_dy_via_tspan_line() -> Result<(), String> {
    let svg = make_svg("tspan-line-both");
    let txt = svg.text(Point::new(20.0, 50.0), "").map_err(|e| e.to_string())?;
    let span = txt.tspan_line(20.0, 22.0, "Line").map_err(|e| e.to_string())?;
    check_eq(span.as_element().get_attribute("x"), Some("20".to_owned()))?;
    check_eq(span.as_element().get_attribute("dy"), Some("22".to_owned()))
}

/// `tspan_line` resets the rendered horizontal start position to the requested `x` coordinate,
/// regardless of the advance width of earlier spans in the same `<text>`.
///
/// This is the key correctness property that distinguishes `tspan_line` from `tspan_dy`:
/// a `dy`-only span starts where the previous glyph run ended, producing a staircase;
/// `tspan_line` starts at the same absolute `x` on every line.
#[wasm_bindgen_test]
fn should_reset_horizontal_position_to_x_for_tspan_line() -> Result<(), String> {
    let svg = make_svg("tspan-line-pos");
    let txt = svg.text(Point::new(30.0, 60.0), "").map_err(|e| e.to_string())?;
    txt.set_font_size(16.0).map_err(|e| e.to_string())?;

    txt.tspan("A wide first span").map_err(|e| e.to_string())?;
    let span2 = txt.tspan_line(30.0, 20.0, "B").map_err(|e| e.to_string())?;

    // getStartPositionOfChar(0) returns the rendered x of the first glyph — must equal the absolute
    // x we supplied (30), not the end-of-first-span position.
    let text_el = span2
        .as_element()
        .dyn_ref::<web_sys::SvgTextContentElement>()
        .ok_or("expected SVGTextContentElement on tspan")?;
    let pos = text_el
        .get_start_position_of_char(0)
        .map_err(|e| format!("getStartPositionOfChar failed: {e:?}"))?;

    let actual_x = pos.x() as f64;
    if (actual_x - 30.0).abs() > 0.5 {
        return Err(format!(
            "expected rendered x ≈ 30, got {actual_x:.2} — tspan_line did not reset horizontal position"
        ));
    }
    Ok(())
}
