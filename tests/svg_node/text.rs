use crate::{common, helpers::make_svg};
use svg_dom::root::utils::Point;
use wasm_bindgen_test::*;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// set_text_fmt / set_text_display
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `set_text_fmt` formats `args` through the scratch buffer and writes the result as text content.
#[wasm_bindgen_test]
fn should_write_text_content_via_set_text_fmt() -> Result<(), String> {
    let label = make_svg("node-set-text-fmt")
        .text(Point::new(10.0, 20.0), "")
        .map_err(|e| e.to_string())?;
    let mut buf = String::new();
    let (x, y) = (12.0, 34.0);
    label
        .set_text_fmt(&mut buf, format_args!("box: {x:.0}, {y:.0}"))
        .map_err(|e| e.to_string())?;
    common::check_eq(label.as_element().text_content(), Some("box: 12, 34".into()))
}

/// The same scratch buffer can be reused across `set_text_fmt` calls and the latest value wins.
#[wasm_bindgen_test]
fn should_reuse_buffer_across_set_text_fmt_calls() -> Result<(), String> {
    let label = make_svg("node-set-text-fmt-reuse")
        .text(Point::new(10.0, 20.0), "")
        .map_err(|e| e.to_string())?;
    let mut buf = String::new();
    label
        .set_text_fmt(&mut buf, format_args!("box: {}, {}", 1, 2))
        .map_err(|e| e.to_string())?;
    label
        .set_text_fmt(&mut buf, format_args!("box: {}, {}", 30, 40))
        .map_err(|e| e.to_string())?;
    common::check_eq(label.as_element().text_content(), Some("box: 30, 40".into()))
}

/// `set_text_display` writes a single displayable value as text content.
#[wasm_bindgen_test]
fn should_write_display_value_via_set_text_display() -> Result<(), String> {
    let label = make_svg("node-set-text-display")
        .text(Point::new(10.0, 20.0), "")
        .map_err(|e| e.to_string())?;
    let mut buf = String::new();
    label.set_text_display(&mut buf, 42).map_err(|e| e.to_string())?;
    common::check_eq(label.as_element().text_content(), Some("42".into()))
}
