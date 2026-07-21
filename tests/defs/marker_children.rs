use crate::common::{self, *};
use svg_dom::root::utils::{Point, Size};
use wasm_bindgen_test::*;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Child shapes inside a marker
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// A `<polygon>` created via `marker.polygon(points)` is a child of the `<marker>` element.
#[wasm_bindgen_test]
fn should_append_polygon_to_marker() -> Result<(), String> {
    let svg = make_svg("marker-polygon-parent");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("m").map_err(|e| e.to_string())?;
    let pts = [Point::new(0.0, 0.0), Point::new(10.0, 3.5), Point::new(0.0, 7.0)];
    let poly = marker.polygon(&pts).map_err(|e| e.to_string())?;
    let parent = poly.as_element().parent_element().ok_or("polygon has no parent")?;
    common::check_eq(parent.tag_name(), "marker".to_owned())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// The `points` attribute on a marker polygon is serialised by `write_points` (`"x,y x,y ..."` format).
#[wasm_bindgen_test]
fn should_set_polygon_points_in_marker() -> Result<(), String> {
    let svg = make_svg("marker-polygon-points");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("m").map_err(|e| e.to_string())?;
    let pts = [Point::new(0.0, 0.0), Point::new(10.0, 3.5), Point::new(0.0, 7.0)];
    let poly = marker.polygon(&pts).map_err(|e| e.to_string())?;
    common::check_eq(poly.as_element().get_attribute("points"), Some("0,0 10,3.5 0,7".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// A `<circle>` created inside a marker has the correct attributes.
#[wasm_bindgen_test]
fn should_create_circle_inside_marker() -> Result<(), String> {
    let svg = make_svg("marker-circle");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("dot").map_err(|e| e.to_string())?;
    let circle = marker.circle(Point::new(5.0, 5.0), 4.0).map_err(|e| e.to_string())?;
    let el = circle.as_element();
    common::check_eq(el.get_attribute("cx"), Some("5".into()))?;
    common::check_eq(el.get_attribute("cy"), Some("5".into()))?;
    common::check_eq(el.get_attribute("r"), Some("4".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// A `<path>` created inside a marker carries the `d` attribute.
#[wasm_bindgen_test]
fn should_create_path_inside_marker() -> Result<(), String> {
    let svg = make_svg("marker-path");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("m").map_err(|e| e.to_string())?;
    let path = marker.path("M0,0 L10,5 L0,10 Z").map_err(|e| e.to_string())?;
    common::check_eq(path.as_element().get_attribute("d"), Some("M0,0 L10,5 L0,10 Z".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// SvgMarker factory coverage — every shape factory creates the correct element inside <marker>
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `marker.rect` creates a `<rect>` child with correct attributes whose parent is `<marker>`.
#[wasm_bindgen_test]
fn should_create_rect_in_marker() -> Result<(), String> {
    let svg = make_svg("marker-rect");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("mr").map_err(|e| e.to_string())?;
    let node = marker
        .rect(Point::new(1.0, 2.0), Size::new(8.0, 6.0))
        .map_err(|e| e.to_string())?;
    let el = node.as_element();
    common::check_eq(el.tag_name(), "rect".to_owned())?;
    common::check_eq(el.get_attribute("x"), Some("1".into()))?;
    common::check_eq(el.get_attribute("y"), Some("2".into()))?;
    common::check_eq(el.get_attribute("width"), Some("8".into()))?;
    common::check_eq(el.get_attribute("height"), Some("6".into()))?;
    common::check_eq(el.parent_element().map(|p| p.tag_name()), Some("marker".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `marker.ellipse` creates an `<ellipse>` child with correct attributes whose parent is `<marker>`.
#[wasm_bindgen_test]
fn should_create_ellipse_in_marker() -> Result<(), String> {
    let svg = make_svg("marker-ellipse");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("me").map_err(|e| e.to_string())?;
    let node = marker
        .ellipse(Point::new(5.0, 5.0), Size::new(4.0, 3.0))
        .map_err(|e| e.to_string())?;
    let el = node.as_element();
    common::check_eq(el.tag_name(), "ellipse".to_owned())?;
    common::check_eq(el.get_attribute("cx"), Some("5".into()))?;
    common::check_eq(el.get_attribute("cy"), Some("5".into()))?;
    common::check_eq(el.get_attribute("rx"), Some("4".into()))?;
    common::check_eq(el.get_attribute("ry"), Some("3".into()))?;
    common::check_eq(el.parent_element().map(|p| p.tag_name()), Some("marker".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `marker.line` creates a `<line>` child with correct endpoint attributes whose parent is `<marker>`.
#[wasm_bindgen_test]
fn should_create_line_in_marker() -> Result<(), String> {
    let svg = make_svg("marker-line");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("ml").map_err(|e| e.to_string())?;
    let node = marker
        .line(Point::new(0.0, 5.0), Point::new(10.0, 5.0))
        .map_err(|e| e.to_string())?;
    let el = node.as_element();
    common::check_eq(el.tag_name(), "line".to_owned())?;
    common::check_eq(el.get_attribute("x1"), Some("0".into()))?;
    common::check_eq(el.get_attribute("y1"), Some("5".into()))?;
    common::check_eq(el.get_attribute("x2"), Some("10".into()))?;
    common::check_eq(el.get_attribute("y2"), Some("5".into()))?;
    common::check_eq(el.parent_element().map(|p| p.tag_name()), Some("marker".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `marker.polyline` creates a `<polyline>` child with a `points` attribute whose parent is `<marker>`.
#[wasm_bindgen_test]
fn should_create_polyline_in_marker() -> Result<(), String> {
    let svg = make_svg("marker-polyline");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("mpl").map_err(|e| e.to_string())?;
    let pts = [Point::new(0.0, 0.0), Point::new(5.0, 10.0), Point::new(10.0, 0.0)];
    let node = marker.polyline(&pts).map_err(|e| e.to_string())?;
    let el = node.as_element();
    common::check_eq(el.tag_name(), "polyline".to_owned())?;
    common::check_eq(el.get_attribute("points"), Some("0,0 5,10 10,0".into()))?;
    common::check_eq(el.parent_element().map(|p| p.tag_name()), Some("marker".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `marker.polygon(&[Point])` creates a `<polygon>` child with a `points` attribute whose parent is `<marker>`.
#[wasm_bindgen_test]
fn should_create_polygon_slice_in_marker() -> Result<(), String> {
    let svg = make_svg("marker-polygon-slice");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("mpg").map_err(|e| e.to_string())?;
    let pts = [Point::new(0.0, 0.0), Point::new(10.0, 5.0), Point::new(0.0, 10.0)];
    let node = marker.polygon(&pts).map_err(|e| e.to_string())?;
    let el = node.as_element();
    common::check_eq(el.tag_name(), "polygon".to_owned())?;
    common::check_eq(el.get_attribute("points"), Some("0,0 10,5 0,10".into()))?;
    common::check_eq(el.parent_element().map(|p| p.tag_name()), Some("marker".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `marker.group` creates a `<g>` child whose parent is `<marker>`.
#[wasm_bindgen_test]
fn should_create_group_in_marker() -> Result<(), String> {
    let svg = make_svg("marker-group");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("mg").map_err(|e| e.to_string())?;
    let node = marker.group().map_err(|e| e.to_string())?;
    let el = node.as_element();
    common::check_eq(el.tag_name(), "g".to_owned())?;
    common::check_eq(el.parent_element().map(|p| p.tag_name()), Some("marker".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `marker.text` creates a `<text>` child with correct position attributes whose parent is `<marker>`.
#[wasm_bindgen_test]
fn should_create_text_in_marker() -> Result<(), String> {
    let svg = make_svg("marker-text");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("mt").map_err(|e| e.to_string())?;
    let node = marker.text(Point::new(2.0, 5.0), "A").map_err(|e| e.to_string())?;
    let el = node.as_element();
    common::check_eq(el.tag_name(), "text".to_owned())?;
    common::check_eq(el.get_attribute("x"), Some("2".into()))?;
    common::check_eq(el.get_attribute("y"), Some("5".into()))?;
    common::check_eq(el.parent_element().map(|p| p.tag_name()), Some("marker".into()))
}
