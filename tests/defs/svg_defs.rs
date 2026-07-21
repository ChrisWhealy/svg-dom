use crate::common::{self, *};
use svg_dom::root::utils::{Point, Size};
use wasm_bindgen_test::*;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// SvgDefs construction
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `defs()` creates a `<defs>` element with the correct SVG tag name.
#[wasm_bindgen_test]
fn should_create_defs_element() -> Result<(), String> {
    let svg = make_svg("defs-create");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    common::check_eq(defs.as_element().tag_name(), "defs".to_owned())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// The `<defs>` element is a child of the root `<svg>`.
#[wasm_bindgen_test]
fn should_append_defs_to_svg_root() -> Result<(), String> {
    let svg = make_svg("defs-parent");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let parent = defs.as_element().parent_element().ok_or("defs has no parent")?;
    // The parent tag is "svg".
    common::check_eq(parent.tag_name(), "svg".to_owned())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Shapes created inside `<defs>` are children of the `<defs>` element, not the root `<svg>`.
#[wasm_bindgen_test]
fn should_append_rect_to_defs_not_svg() -> Result<(), String> {
    let svg = make_svg("defs-rect-parent");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let rect = defs.rect(Point::origin(), Size::new(10.0, 10.0)).map_err(|e| e.to_string())?;
    let parent = rect.as_element().parent_element().ok_or("rect has no parent")?;
    common::check_eq(parent.tag_name(), "defs".to_owned())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// A `<rect>` created inside `<defs>` has correct geometric attributes.
#[wasm_bindgen_test]
fn should_create_rect_inside_defs_with_correct_attrs() -> Result<(), String> {
    let svg = make_svg("defs-rect-attrs");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let rect = defs
        .rect(Point::new(5.0, 10.0), Size::new(20.0, 30.0))
        .map_err(|e| e.to_string())?;
    let el = rect.as_element();
    common::check_eq(el.get_attribute("x"), Some("5".into()))?;
    common::check_eq(el.get_attribute("y"), Some("10".into()))?;
    common::check_eq(el.get_attribute("width"), Some("20".into()))?;
    common::check_eq(el.get_attribute("height"), Some("30".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// SvgDefs factory coverage — every shape factory creates the correct element inside <defs>
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `defs.circle` creates a `<circle>` child with correct attributes whose parent is `<defs>`.
#[wasm_bindgen_test]
fn should_create_circle_in_defs() -> Result<(), String> {
    let svg = make_svg("defs-circle");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let node = defs.circle(Point::new(5.0, 8.0), 4.0).map_err(|e| e.to_string())?;
    let el = node.as_element();
    common::check_eq(el.tag_name(), "circle".to_owned())?;
    common::check_eq(el.get_attribute("cx"), Some("5".into()))?;
    common::check_eq(el.get_attribute("cy"), Some("8".into()))?;
    common::check_eq(el.get_attribute("r"), Some("4".into()))?;
    common::check_eq(el.parent_element().map(|p| p.tag_name()), Some("defs".into()))
}

/// `defs.ellipse` creates an `<ellipse>` child with correct attributes whose parent is `<defs>`.
#[wasm_bindgen_test]
fn should_create_ellipse_in_defs() -> Result<(), String> {
    let svg = make_svg("defs-ellipse");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let node = defs
        .ellipse(Point::new(50.0, 40.0), Size::new(30.0, 20.0))
        .map_err(|e| e.to_string())?;
    let el = node.as_element();
    common::check_eq(el.tag_name(), "ellipse".to_owned())?;
    common::check_eq(el.get_attribute("cx"), Some("50".into()))?;
    common::check_eq(el.get_attribute("cy"), Some("40".into()))?;
    common::check_eq(el.get_attribute("rx"), Some("30".into()))?;
    common::check_eq(el.get_attribute("ry"), Some("20".into()))?;
    common::check_eq(el.parent_element().map(|p| p.tag_name()), Some("defs".into()))
}

/// `defs.line` creates a `<line>` child with correct endpoint attributes whose parent is `<defs>`.
#[wasm_bindgen_test]
fn should_create_line_in_defs() -> Result<(), String> {
    let svg = make_svg("defs-line");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let node = defs
        .line(Point::new(0.0, 0.0), Point::new(20.0, 30.0))
        .map_err(|e| e.to_string())?;
    let el = node.as_element();
    common::check_eq(el.tag_name(), "line".to_owned())?;
    common::check_eq(el.get_attribute("x1"), Some("0".into()))?;
    common::check_eq(el.get_attribute("y1"), Some("0".into()))?;
    common::check_eq(el.get_attribute("x2"), Some("20".into()))?;
    common::check_eq(el.get_attribute("y2"), Some("30".into()))?;
    common::check_eq(el.parent_element().map(|p| p.tag_name()), Some("defs".into()))
}

/// `defs.path` creates a `<path>` child with the correct `d` attribute whose parent is `<defs>`.
#[wasm_bindgen_test]
fn should_create_path_in_defs() -> Result<(), String> {
    let svg = make_svg("defs-path");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let node = defs.path("M 0 0 L 100 0").map_err(|e| e.to_string())?;
    let el = node.as_element();
    common::check_eq(el.tag_name(), "path".to_owned())?;
    common::check_eq(el.get_attribute("d"), Some("M 0 0 L 100 0".into()))?;
    common::check_eq(el.parent_element().map(|p| p.tag_name()), Some("defs".into()))
}

/// `defs.polyline` creates a `<polyline>` child with a `points` attribute whose parent is `<defs>`.
#[wasm_bindgen_test]
fn should_create_polyline_in_defs() -> Result<(), String> {
    let svg = make_svg("defs-polyline");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let pts = [Point::new(0.0, 0.0), Point::new(20.0, 40.0), Point::new(40.0, 0.0)];
    let node = defs.polyline(&pts).map_err(|e| e.to_string())?;
    let el = node.as_element();
    common::check_eq(el.tag_name(), "polyline".to_owned())?;
    common::check_eq(el.get_attribute("points"), Some("0,0 20,40 40,0".into()))?;
    common::check_eq(el.parent_element().map(|p| p.tag_name()), Some("defs".into()))
}

/// `defs.polygon` creates a `<polygon>` child with a `points` attribute whose parent is `<defs>`.
#[wasm_bindgen_test]
fn should_create_polygon_in_defs() -> Result<(), String> {
    let svg = make_svg("defs-polygon");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let pts = [Point::new(0.0, 0.0), Point::new(10.0, 20.0), Point::new(20.0, 0.0)];
    let node = defs.polygon(&pts).map_err(|e| e.to_string())?;
    let el = node.as_element();
    common::check_eq(el.tag_name(), "polygon".to_owned())?;
    common::check_eq(el.get_attribute("points"), Some("0,0 10,20 20,0".into()))?;
    common::check_eq(el.parent_element().map(|p| p.tag_name()), Some("defs".into()))
}

/// `defs.text` creates a `<text>` child with correct position and content whose parent is `<defs>`.
#[wasm_bindgen_test]
fn should_create_text_in_defs() -> Result<(), String> {
    let svg = make_svg("defs-text");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let node = defs.text(Point::new(10.0, 20.0), "hello").map_err(|e| e.to_string())?;
    let el = node.as_element();
    common::check_eq(el.tag_name(), "text".to_owned())?;
    common::check_eq(el.get_attribute("x"), Some("10".into()))?;
    common::check_eq(el.get_attribute("y"), Some("20".into()))?;
    common::check_eq(el.text_content(), Some("hello".into()))?;
    common::check_eq(el.parent_element().map(|p| p.tag_name()), Some("defs".into()))
}

/// `defs.group` creates a `<g>` child whose parent is `<defs>`.
#[wasm_bindgen_test]
fn should_create_group_in_defs() -> Result<(), String> {
    let svg = make_svg("defs-group");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let node = defs.group().map_err(|e| e.to_string())?;
    let el = node.as_element();
    common::check_eq(el.tag_name(), "g".to_owned())?;
    common::check_eq(el.parent_element().map(|p| p.tag_name()), Some("defs".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Generic attribute surface — SvgDefs
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `SvgDefs::set_attr` writes an arbitrary attribute on the `<defs>` element.
#[wasm_bindgen_test]
fn should_set_attr_on_defs() -> Result<(), String> {
    let svg = make_svg("defs-set-attr");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    defs.set_attr("class", "shared-assets").map_err(|e| e.to_string())?;
    common::check_eq(defs.as_element().get_attribute("class"), Some("shared-assets".into()))
}

/// `SvgDefs::set_attrs` sets several attributes in one call.
#[wasm_bindgen_test]
fn should_set_attrs_on_defs() -> Result<(), String> {
    let svg = make_svg("defs-set-attrs");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    defs.set_attrs([("class", "assets"), ("style", "display:none")])
        .map_err(|e| e.to_string())?;
    let el = defs.as_element();
    common::check_eq(el.get_attribute("class"), Some("assets".into()))?;
    common::check_eq(el.get_attribute("style"), Some("display:none".into()))
}

/// `SvgDefs::set_attr_display` formats a numeric value through the internal scratch buffer.
#[wasm_bindgen_test]
fn should_set_attr_display_on_defs() -> Result<(), String> {
    let svg = make_svg("defs-set-attr-display");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    defs.set_attr_display("data-count", 42_u32).map_err(|e| e.to_string())?;
    common::check_eq(defs.as_element().get_attribute("data-count"), Some("42".into()))
}
