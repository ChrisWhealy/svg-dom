mod common;

use common::*;
use svg_dom::{
    Error, MarkerUnits,
    root::utils::{Point, Size},
};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

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
// SvgMarker construction
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `defs.marker("id")` creates a `<marker>` element with the correct tag name.
#[wasm_bindgen_test]
fn should_create_marker_element() -> Result<(), String> {
    let svg = make_svg("marker-create");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("arrow").map_err(|e| e.to_string())?;
    common::check_eq(marker.as_element().tag_name(), "marker".to_owned())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// The `<marker>` element carries the supplied `id` attribute.
#[wasm_bindgen_test]
fn should_set_marker_id_attribute() -> Result<(), String> {
    let svg = make_svg("marker-id-attr");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("my-arrow").map_err(|e| e.to_string())?;
    common::check_eq(marker.as_element().get_attribute("id"), Some("my-arrow".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `SvgMarker::id()` returns the id string the marker was created with.
#[wasm_bindgen_test]
fn should_expose_marker_id_via_method() -> Result<(), String> {
    let svg = make_svg("marker-id-method");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("dot").map_err(|e| e.to_string())?;
    common::check_eq(marker.id(), "dot")
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// The `<marker>` element is a child of the `<defs>` element.
#[wasm_bindgen_test]
fn should_append_marker_to_defs() -> Result<(), String> {
    let svg = make_svg("marker-defs-parent");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("arrowhead").map_err(|e| e.to_string())?;
    let parent = marker.as_element().parent_element().ok_or("marker has no parent")?;
    common::check_eq(parent.tag_name(), "defs".to_owned())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// SvgMarker attribute setters
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `set_ref_x` writes the `refX` attribute.
#[wasm_bindgen_test]
fn should_set_marker_ref_x() -> Result<(), String> {
    let svg = make_svg("marker-ref-x");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("m").map_err(|e| e.to_string())?;
    marker.set_ref_x(10.0).map_err(|e| e.to_string())?;
    common::check_eq(marker.as_element().get_attribute("refX"), Some("10".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `set_ref_y` writes the `refY` attribute.
#[wasm_bindgen_test]
fn should_set_marker_ref_y() -> Result<(), String> {
    let svg = make_svg("marker-ref-y");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("m").map_err(|e| e.to_string())?;
    marker.set_ref_y(3.5).map_err(|e| e.to_string())?;
    common::check_eq(marker.as_element().get_attribute("refY"), Some("3.5".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `set_marker_width` writes the `markerWidth` attribute.
#[wasm_bindgen_test]
fn should_set_marker_width() -> Result<(), String> {
    let svg = make_svg("marker-width");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("m").map_err(|e| e.to_string())?;
    marker.set_marker_width(10.0).map_err(|e| e.to_string())?;
    common::check_eq(marker.as_element().get_attribute("markerWidth"), Some("10".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `set_marker_height` writes the `markerHeight` attribute.
#[wasm_bindgen_test]
fn should_set_marker_height() -> Result<(), String> {
    let svg = make_svg("marker-height");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("m").map_err(|e| e.to_string())?;
    marker.set_marker_height(7.0).map_err(|e| e.to_string())?;
    common::check_eq(marker.as_element().get_attribute("markerHeight"), Some("7".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `set_orient` writes the `orient` attribute.
#[wasm_bindgen_test]
fn should_set_marker_orient_auto() -> Result<(), String> {
    let svg = make_svg("marker-orient-auto");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("m").map_err(|e| e.to_string())?;
    marker.set_orient("auto").map_err(|e| e.to_string())?;
    common::check_eq(marker.as_element().get_attribute("orient"), Some("auto".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `set_orient` also accepts angle values.
#[wasm_bindgen_test]
fn should_set_marker_orient_angle() -> Result<(), String> {
    let svg = make_svg("marker-orient-angle");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("m").map_err(|e| e.to_string())?;
    marker.set_orient("45deg").map_err(|e| e.to_string())?;
    common::check_eq(marker.as_element().get_attribute("orient"), Some("45deg".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `set_units(StrokeWidth)` writes `markerUnits="strokeWidth"`.
#[wasm_bindgen_test]
fn should_set_marker_units_stroke_width() -> Result<(), String> {
    let svg = make_svg("marker-units-sw");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("m").map_err(|e| e.to_string())?;
    marker.set_units(MarkerUnits::StrokeWidth).map_err(|e| e.to_string())?;
    common::check_eq(marker.as_element().get_attribute("markerUnits"), Some("strokeWidth".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `set_units(UserSpaceOnUse)` writes `markerUnits="userSpaceOnUse"`.
#[wasm_bindgen_test]
fn should_set_marker_units_user_space() -> Result<(), String> {
    let svg = make_svg("marker-units-us");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("m").map_err(|e| e.to_string())?;
    marker.set_units(MarkerUnits::UserSpaceOnUse).map_err(|e| e.to_string())?;
    common::check_eq(marker.as_element().get_attribute("markerUnits"), Some("userSpaceOnUse".into()))
}

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
/// The `points` attribute on a marker polygon is serialised by `write_points` (`"x,y x,y …"` format).
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

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// SvgNode marker reference methods
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `set_marker_end` writes `marker-end="url(#id)"` on the element.
#[wasm_bindgen_test]
fn should_set_marker_end_on_line() -> Result<(), String> {
    let svg = make_svg("marker-end-line");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    defs.marker("arrow-end").map_err(|e| e.to_string())?;
    let line = svg
        .line(Point::new(0.0, 0.0), Point::new(100.0, 0.0))
        .map_err(|e| e.to_string())?;
    line.set_marker_end("arrow-end").map_err(|e| e.to_string())?;
    common::check_eq(line.as_element().get_attribute("marker-end"), Some("url(#arrow-end)".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `set_marker_start` writes `marker-start="url(#id)"` on the element.
#[wasm_bindgen_test]
fn should_set_marker_start_on_path() -> Result<(), String> {
    let svg = make_svg("marker-start-path");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    defs.marker("arrow-start").map_err(|e| e.to_string())?;
    let path = svg.path("M 0 50 L 200 50").map_err(|e| e.to_string())?;
    path.set_marker_start("arrow-start").map_err(|e| e.to_string())?;
    common::check_eq(
        path.as_element().get_attribute("marker-start"),
        Some("url(#arrow-start)".into()),
    )
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `set_marker_mid` writes `marker-mid="url(#id)"` on the element.
#[wasm_bindgen_test]
fn should_set_marker_mid_on_polyline() -> Result<(), String> {
    let svg = make_svg("marker-mid-polyline");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    defs.marker("dot-mid").map_err(|e| e.to_string())?;
    let poly = svg
        .polyline(&[Point::new(0.0, 0.0), Point::new(50.0, 50.0), Point::new(100.0, 0.0)])
        .map_err(|e| e.to_string())?;
    poly.set_marker_mid("dot-mid").map_err(|e| e.to_string())?;
    common::check_eq(poly.as_element().get_attribute("marker-mid"), Some("url(#dot-mid)".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// All three marker attributes can be set independently on the same element.
#[wasm_bindgen_test]
fn should_set_all_three_marker_attrs_on_one_element() -> Result<(), String> {
    let svg = make_svg("marker-all-three");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    defs.marker("start-m").map_err(|e| e.to_string())?;
    defs.marker("mid-m").map_err(|e| e.to_string())?;
    defs.marker("end-m").map_err(|e| e.to_string())?;
    let path = svg.path("M 0 50 L 100 50 L 200 50").map_err(|e| e.to_string())?;
    path.set_marker_start("start-m").map_err(|e| e.to_string())?;
    path.set_marker_mid("mid-m").map_err(|e| e.to_string())?;
    path.set_marker_end("end-m").map_err(|e| e.to_string())?;
    let el = path.as_element();
    common::check_eq(el.get_attribute("marker-start"), Some("url(#start-m)".into()))?;
    common::check_eq(el.get_attribute("marker-mid"), Some("url(#mid-m)".into()))?;
    common::check_eq(el.get_attribute("marker-end"), Some("url(#end-m)".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Full arrowhead assembly
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Assembling a complete arrowhead marker and applying it produces the expected DOM structure and attributes.
#[wasm_bindgen_test]
fn should_assemble_arrowhead_marker() -> Result<(), String> {
    let svg = make_svg("arrowhead-full");
    let defs = svg.defs().map_err(|e| e.to_string())?;

    let marker = defs.marker("arrowhead").map_err(|e| e.to_string())?;
    marker.set_ref_x(10.0).map_err(|e| e.to_string())?;
    marker.set_ref_y(3.5).map_err(|e| e.to_string())?;
    marker.set_marker_width(10.0).map_err(|e| e.to_string())?;
    marker.set_marker_height(7.0).map_err(|e| e.to_string())?;
    marker.set_orient("auto").map_err(|e| e.to_string())?;
    marker
        .polygon(&[Point::new(0.0, 0.0), Point::new(10.0, 3.5), Point::new(0.0, 7.0)])
        .map_err(|e| e.to_string())?;

    let line = svg
        .line(Point::new(20.0, 50.0), Point::new(180.0, 50.0))
        .map_err(|e| e.to_string())?;
    line.set_stroke("black").map_err(|e| e.to_string())?;
    line.set_marker_end("arrowhead").map_err(|e| e.to_string())?;

    let el = marker.as_element();
    common::check_eq(el.get_attribute("id"), Some("arrowhead".into()))?;
    common::check_eq(el.get_attribute("refX"), Some("10".into()))?;
    common::check_eq(el.get_attribute("refY"), Some("3.5".into()))?;
    common::check_eq(el.get_attribute("markerWidth"), Some("10".into()))?;
    common::check_eq(el.get_attribute("markerHeight"), Some("7".into()))?;
    common::check_eq(el.get_attribute("orient"), Some("auto".into()))?;
    common::check_eq(line.as_element().get_attribute("marker-end"), Some("url(#arrowhead)".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// build_defs / build_marker — deferred-append variants
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `build_defs` appends `<defs>` to the SVG root after the closure succeeds.
#[wasm_bindgen_test]
fn should_build_defs_and_append_on_success() -> Result<(), String> {
    let svg = make_svg("build-defs-ok");
    let defs = svg.build_defs(|_| Ok(())).map_err(|e| e.to_string())?;
    // The defs element must now be a child of the SVG.
    let parent = defs
        .as_element()
        .parent_element()
        .ok_or("defs has no parent after build_defs")?;
    common::check_eq(parent.tag_name(), "svg".to_owned())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `build_defs` returns `Err` when the closure does, and the `<defs>` element is NOT appended.
#[wasm_bindgen_test]
fn should_not_append_defs_when_build_closure_fails() -> Result<(), String> {
    use std::cell::RefCell;
    use std::rc::Rc;

    let svg = make_svg("build-defs-err");
    let captured: Rc<RefCell<Option<web_sys::SvgElement>>> = Rc::new(RefCell::new(None));
    let cap2 = captured.clone();

    let result = svg.build_defs(move |defs| {
        *cap2.borrow_mut() = Some(defs.as_element().clone());
        Err(Error::Dom("deliberate failure".into()))
    });

    common::check(result.is_err(), "build_defs must return Err when closure fails")?;

    let borrow = captured.borrow();
    let el = borrow.as_ref().ok_or("closure was never called")?;
    common::check(
        el.parent_element().is_none(),
        "defs must not be appended to SVG when closure fails",
    )
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `build_marker` appends `<marker>` to `<defs>` after the closure succeeds, with all attributes set.
#[wasm_bindgen_test]
fn should_build_marker_and_append_on_success() -> Result<(), String> {
    let svg = make_svg("build-marker-ok");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs
        .build_marker("bm-arrow", |m| {
            m.set_ref_x(10.0)?;
            m.set_ref_y(3.5)?;
            m.set_marker_width(10.0)?;
            m.set_marker_height(7.0)?;
            m.set_orient("auto")?;
            m.polygon(&[Point::new(0.0, 0.0), Point::new(10.0, 3.5), Point::new(0.0, 7.0)])?;
            Ok(())
        })
        .map_err(|e| e.to_string())?;

    // Marker must be inside defs.
    let parent = marker.as_element().parent_element().ok_or("marker has no parent")?;
    common::check_eq(parent.tag_name(), "defs".to_owned())?;

    // All attributes must be present.
    let el = marker.as_element();
    common::check_eq(el.get_attribute("id"), Some("bm-arrow".into()))?;
    common::check_eq(el.get_attribute("refX"), Some("10".into()))?;
    common::check_eq(el.get_attribute("refY"), Some("3.5".into()))?;
    common::check_eq(el.get_attribute("markerWidth"), Some("10".into()))?;
    common::check_eq(el.get_attribute("markerHeight"), Some("7".into()))?;
    common::check_eq(el.get_attribute("orient"), Some("auto".into()))?;

    // Polygon child must be inside the marker.
    common::check_eq(el.child_element_count(), 1_u32)?;
    let child = el.first_element_child().ok_or("marker has no child")?;
    common::check_eq(child.tag_name(), "polygon".to_owned())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `build_marker` returns `Err` when the closure does, and the `<marker>` element is NOT appended to `<defs>`.
#[wasm_bindgen_test]
fn should_not_append_marker_when_build_closure_fails() -> Result<(), String> {
    use std::cell::RefCell;
    use std::rc::Rc;

    let svg = make_svg("build-marker-err");
    let defs = svg.defs().map_err(|e| e.to_string())?;

    let captured: Rc<RefCell<Option<web_sys::SvgElement>>> = Rc::new(RefCell::new(None));
    let cap2 = captured.clone();

    let result = defs.build_marker("bm-err", move |marker| {
        *cap2.borrow_mut() = Some(marker.as_element().clone());
        Err(Error::Dom("deliberate failure".into()))
    });

    common::check(result.is_err(), "build_marker must return Err when closure fails")?;

    let borrow = captured.borrow();
    let el = borrow.as_ref().ok_or("closure was never called")?;
    common::check(
        el.parent_element().is_none(),
        "marker must not be appended to defs when closure fails",
    )
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `build_defs` nesting `build_marker` — the fully built subtree lands in the SVG in one append.
#[wasm_bindgen_test]
fn should_build_nested_defs_and_marker_atomically() -> Result<(), String> {
    let svg = make_svg("build-nested");
    svg.build_defs(|defs| {
        defs.build_marker("nested-arrow", |m| {
            m.set_ref_x(10.0)?;
            m.set_ref_y(3.5)?;
            m.set_orient("auto")?;
            m.polygon(&[Point::new(0.0, 0.0), Point::new(10.0, 3.5), Point::new(0.0, 7.0)])?;
            Ok(())
        })?;
        Ok(())
    })
    .map_err(|e| e.to_string())?;

    // Verify the structure through the live SVG via the common div we can query.
    let doc = web_sys::window().unwrap().document().unwrap();
    let marker_el = doc
        .query_selector("#build-nested svg defs marker")
        .map_err(|_| "query_selector failed".to_owned())?
        .ok_or("marker not found in live DOM")?;

    common::check_eq(marker_el.get_attribute("id"), Some("nested-arrow".into()))?;
    common::check_eq(marker_el.child_element_count(), 1_u32)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Marker ID validation
//
// SvgMarker::set_id — cache-aware rename
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `set_id` updates both the DOM `id` attribute and the cached value returned by `id()`.
#[wasm_bindgen_test]
fn should_update_cache_and_dom_on_set_id() -> Result<(), String> {
    let svg = make_svg("set-id-ok");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let mut marker = defs.marker("original").map_err(|e| e.to_string())?;
    marker.set_id("renamed").map_err(|e| e.to_string())?;
    common::check_eq(marker.id(), "renamed")?;
    common::check_eq(marker.as_element().get_attribute("id"), Some("renamed".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `set_id` with an invalid id returns `InvalidMarkerId` and leaves the marker unchanged.
#[wasm_bindgen_test]
fn should_reject_invalid_id_on_set_id() -> Result<(), String> {
    let svg = make_svg("set-id-invalid");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let mut marker = defs.marker("good-id").map_err(|e| e.to_string())?;
    let result = marker.set_id("url(#bad)");
    common::check(
        matches!(result, Err(Error::InvalidMarkerId(_))),
        "invalid id must return InvalidMarkerId",
    )?;
    // Cache and DOM must be unchanged.
    common::check_eq(marker.id(), "good-id")?;
    common::check_eq(marker.as_element().get_attribute("id"), Some("good-id".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `set_id` with an empty string returns `InvalidMarkerId`.
#[wasm_bindgen_test]
fn should_reject_empty_id_on_set_id() -> Result<(), String> {
    let svg = make_svg("set-id-empty");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let mut marker = defs.marker("initial").map_err(|e| e.to_string())?;
    let result = marker.set_id("");
    common::check(
        matches!(result, Err(Error::InvalidMarkerId(_))),
        "empty id must return InvalidMarkerId",
    )
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Renaming a marker after a reference has been applied leaves the existing attribute pointing at the old id.
/// `set_id` updates the marker's DOM id but cannot retroactively update string snapshots written to other elements.
#[wasm_bindgen_test]
fn should_leave_stale_reference_after_set_id() -> Result<(), String> {
    let svg = make_svg("set-id-stale-ref");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let mut marker = defs.marker("before").map_err(|e| e.to_string())?;
    let line = svg
        .line(Point::new(0.0, 0.0), Point::new(100.0, 0.0))
        .map_err(|e| e.to_string())?;
    line.set_marker_end_ref(&marker).map_err(|e| e.to_string())?;

    // Sanity check: reference points at the original id.
    common::check_eq(line.as_element().get_attribute("marker-end"), Some("url(#before)".into()))?;

    // Rename the marker.
    marker.set_id("after").map_err(|e| e.to_string())?;

    // The marker's own id is updated …
    common::check_eq(marker.as_element().get_attribute("id"), Some("after".into()))?;
    // … but the line's attribute still holds the old snapshot.
    common::check_eq(line.as_element().get_attribute("marker-end"), Some("url(#before)".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// SvgMarker::set_attr / set_attrs / set_attr_display — "id" is reserved
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `set_attr("id", …)` returns `ReservedAttribute` and does not write to the DOM.
#[wasm_bindgen_test]
fn should_reject_id_in_set_attr() -> Result<(), String> {
    let svg = make_svg("reserved-set-attr");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("original").map_err(|e| e.to_string())?;
    common::check(
        matches!(marker.set_attr("id", "new-id"), Err(Error::ReservedAttribute(_))),
        "set_attr(\"id\") must return ReservedAttribute",
    )?;
    // Cached id and DOM attribute must still reflect the original value.
    common::check_eq(marker.id(), "original")?;
    common::check_eq(marker.as_element().get_attribute("id"), Some("original".into()))
}

/// `set_attr("ID", …)` is also rejected (case-insensitive guard).
#[wasm_bindgen_test]
fn should_reject_id_case_insensitive_in_set_attr() -> Result<(), String> {
    let svg = make_svg("reserved-set-attr-case");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("original2").map_err(|e| e.to_string())?;
    common::check(
        matches!(marker.set_attr("ID", "new-id"), Err(Error::ReservedAttribute(_))),
        "set_attr(\"ID\") must also return ReservedAttribute",
    )
}

/// `set_attrs` propagates the `ReservedAttribute` error when `"id"` appears in the iterator.
#[wasm_bindgen_test]
fn should_reject_id_in_set_attrs() -> Result<(), String> {
    let svg = make_svg("reserved-set-attrs");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("original3").map_err(|e| e.to_string())?;
    common::check(
        matches!(
            marker.set_attrs([("viewBox", "0 0 10 7"), ("id", "hijack")]),
            Err(Error::ReservedAttribute(_))
        ),
        "set_attrs must propagate ReservedAttribute for \"id\"",
    )
}

/// `set_attr_display("id", …)` returns `ReservedAttribute`.
#[wasm_bindgen_test]
fn should_reject_id_in_set_attr_display() -> Result<(), String> {
    let svg = make_svg("reserved-set-attr-display");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("original4").map_err(|e| e.to_string())?;
    common::check(
        matches!(marker.set_attr_display("id", "new-id"), Err(Error::ReservedAttribute(_))),
        "set_attr_display(\"id\") must return ReservedAttribute",
    )
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Empty string is rejected as a marker id.
#[wasm_bindgen_test]
fn should_reject_empty_marker_id() -> Result<(), String> {
    let svg = make_svg("val-empty");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    common::check(is_invalid_marker_id(defs.marker("")), "empty id must be rejected")
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Marker id containing a space is rejected.
#[wasm_bindgen_test]
fn should_reject_whitespace_in_marker_id() -> Result<(), String> {
    let svg = make_svg("val-space");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    common::check(is_invalid_marker_id(defs.marker("arrow head")), "space in id must be rejected")?;
    common::check(is_invalid_marker_id(defs.marker("tab\there")), "tab in id must be rejected")
}

/// Marker id containing `#` is rejected.
#[wasm_bindgen_test]
fn should_reject_hash_in_marker_id() -> Result<(), String> {
    let svg = make_svg("val-hash");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    common::check(is_invalid_marker_id(defs.marker("#arrow")), "# in id must be rejected")
}

/// Marker id containing `)` is rejected.
#[wasm_bindgen_test]
fn should_reject_paren_in_marker_id() -> Result<(), String> {
    let svg = make_svg("val-paren");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    common::check(is_invalid_marker_id(defs.marker("arrow)")), ") in id must be rejected")
}

/// Marker id containing `(` is rejected.
#[wasm_bindgen_test]
fn should_reject_open_paren_in_marker_id() -> Result<(), String> {
    let svg = make_svg("val-open-paren");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    common::check(is_invalid_marker_id(defs.marker("arrow(left")), "( in id must be rejected")
}

/// Marker id starting with `url(` is rejected.
#[wasm_bindgen_test]
fn should_reject_url_prefix_in_marker_id() -> Result<(), String> {
    let svg = make_svg("val-url");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    common::check(
        is_invalid_marker_id(defs.marker("url(#arrow)")),
        "id starting with url( must be rejected",
    )
}

/// Characters that are valid in some XML ids but unsafe in a `url(#...)` reference are rejected.
///
/// The allow-list `[A-Za-z_][A-Za-z0-9_-]*` excludes quotes, backslash, semicolon, and control characters.
#[wasm_bindgen_test]
fn should_reject_url_unsafe_chars_in_marker_id() -> Result<(), String> {
    let svg = make_svg("val-unsafe");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    common::check(
        is_invalid_marker_id(defs.marker("arrow\"head")),
        r#"double-quote in id must be rejected"#,
    )?;
    common::check(
        is_invalid_marker_id(defs.marker("arrow'head")),
        "single-quote in id must be rejected",
    )?;
    common::check(
        is_invalid_marker_id(defs.marker("arrow\\head")),
        "backslash in id must be rejected",
    )?;
    common::check(
        is_invalid_marker_id(defs.marker("arrow;head")),
        "semicolon in id must be rejected",
    )?;
    common::check(
        is_invalid_marker_id(defs.marker("arrow\nhead")),
        "newline in id must be rejected",
    )
}

/// An id starting with a digit is rejected (allow-list requires a letter or underscore first).
#[wasm_bindgen_test]
fn should_reject_digit_start_in_marker_id() -> Result<(), String> {
    let svg = make_svg("val-digit-start");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    common::check(is_invalid_marker_id(defs.marker("2arrow")), "digit-first id must be rejected")
}

/// `build_marker` applies the same validation as `marker`.
#[wasm_bindgen_test]
fn should_reject_invalid_id_in_build_marker() -> Result<(), String> {
    let svg = make_svg("val-build");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let result = defs.build_marker("url(#x)", |_| Ok(()));
    common::check(is_invalid_marker_id(result), "build_marker must reject url( prefix")
}

/// A plain alphanumeric id with hyphens is accepted.
#[wasm_bindgen_test]
fn should_accept_valid_marker_id() -> Result<(), String> {
    let svg = make_svg("val-ok");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    defs.marker("arrow-head-2").map_err(|e| e.to_string())?;
    Ok(())
}

/// An id starting with an underscore is accepted.
#[wasm_bindgen_test]
fn should_accept_underscore_start_in_marker_id() -> Result<(), String> {
    let svg = make_svg("val-ok-underscore");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    defs.marker("_arrow").map_err(|e| e.to_string())?;
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// set_marker_{start,mid,end}_ref
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `set_marker_end_ref` writes `marker-end="url(#id)"` using the marker handle directly.
#[wasm_bindgen_test]
fn should_set_marker_end_ref_on_line() -> Result<(), String> {
    let svg = make_svg("ref-end");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("ref-arrow").map_err(|e| e.to_string())?;
    let line = svg
        .line(Point::new(0.0, 0.0), Point::new(100.0, 0.0))
        .map_err(|e| e.to_string())?;
    line.set_marker_end_ref(&marker).map_err(|e| e.to_string())?;
    common::check_eq(line.as_element().get_attribute("marker-end"), Some("url(#ref-arrow)".into()))
}

/// `set_marker_start_ref` writes `marker-start="url(#id)"` using the marker handle directly.
#[wasm_bindgen_test]
fn should_set_marker_start_ref_on_path() -> Result<(), String> {
    let svg = make_svg("ref-start");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("ref-dot").map_err(|e| e.to_string())?;
    let path = svg.path("M 0 50 L 200 50").map_err(|e| e.to_string())?;
    path.set_marker_start_ref(&marker).map_err(|e| e.to_string())?;
    common::check_eq(path.as_element().get_attribute("marker-start"), Some("url(#ref-dot)".into()))
}

/// `set_marker_mid_ref` writes `marker-mid="url(#id)"` using the marker handle directly.
#[wasm_bindgen_test]
fn should_set_marker_mid_ref_on_polyline() -> Result<(), String> {
    let svg = make_svg("ref-mid");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("ref-mid-dot").map_err(|e| e.to_string())?;
    let poly = svg
        .polyline(&[Point::new(0.0, 0.0), Point::new(50.0, 50.0), Point::new(100.0, 0.0)])
        .map_err(|e| e.to_string())?;
    poly.set_marker_mid_ref(&marker).map_err(|e| e.to_string())?;
    common::check_eq(poly.as_element().get_attribute("marker-mid"), Some("url(#ref-mid-dot)".into()))
}

/// All three `_ref` methods can be set on the same element using handles from different markers.
#[wasm_bindgen_test]
fn should_set_all_three_marker_refs_on_one_element() -> Result<(), String> {
    let svg = make_svg("ref-all-three");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let start = defs.marker("rs-start").map_err(|e| e.to_string())?;
    let mid = defs.marker("rs-mid").map_err(|e| e.to_string())?;
    let end = defs.marker("rs-end").map_err(|e| e.to_string())?;
    let path = svg.path("M 0 50 L 100 50 L 200 50").map_err(|e| e.to_string())?;
    path.set_marker_start_ref(&start).map_err(|e| e.to_string())?;
    path.set_marker_mid_ref(&mid).map_err(|e| e.to_string())?;
    path.set_marker_end_ref(&end).map_err(|e| e.to_string())?;
    let el = path.as_element();
    common::check_eq(el.get_attribute("marker-start"), Some("url(#rs-start)".into()))?;
    common::check_eq(el.get_attribute("marker-mid"), Some("url(#rs-mid)".into()))?;
    common::check_eq(el.get_attribute("marker-end"), Some("url(#rs-end)".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// set_marker_{start,mid,end} — validation (no SvgMarker required)
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

fn is_invalid_marker_id_unit(result: Result<(), Error>) -> bool {
    matches!(result, Err(Error::InvalidMarkerId(_)))
}

/// `set_marker_start` rejects a whitespace id and leaves the attribute unset.
#[wasm_bindgen_test]
fn should_reject_whitespace_id_in_set_marker_start() -> Result<(), String> {
    let svg = make_svg("set-start-ws");
    let path = svg.path("M 0 50 L 200 50").map_err(|e| e.to_string())?;
    common::check(
        is_invalid_marker_id_unit(path.set_marker_start("bad id")),
        "set_marker_start: whitespace in id must be rejected",
    )?;
    common::check(
        path.as_element().get_attribute("marker-start").is_none(),
        "set_marker_start: attribute must remain unset after rejection",
    )
}

/// `set_marker_mid` rejects a `url(` prefix and leaves the attribute unset.
#[wasm_bindgen_test]
fn should_reject_url_prefix_in_set_marker_mid() -> Result<(), String> {
    let svg = make_svg("set-mid-url");
    let path = svg.path("M 0 50 L 100 50 L 200 50").map_err(|e| e.to_string())?;
    common::check(
        is_invalid_marker_id_unit(path.set_marker_mid("url(#arrow)")),
        "set_marker_mid: url( prefix must be rejected",
    )?;
    common::check(
        path.as_element().get_attribute("marker-mid").is_none(),
        "set_marker_mid: attribute must remain unset after rejection",
    )
}

/// `set_marker_end` rejects an id containing `(` and leaves the attribute unset.
#[wasm_bindgen_test]
fn should_reject_open_paren_in_set_marker_end() -> Result<(), String> {
    let svg = make_svg("set-end-open-paren");
    let path = svg.path("M 0 50 L 200 50").map_err(|e| e.to_string())?;
    common::check(
        is_invalid_marker_id_unit(path.set_marker_end("arrow(left")),
        "set_marker_end: ( in id must be rejected",
    )?;
    common::check(
        path.as_element().get_attribute("marker-end").is_none(),
        "set_marker_end: attribute must remain unset after rejection",
    )
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Generic attribute surface — SvgMarker
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `SvgMarker::set_attr` writes an arbitrary attribute (motivating example: `viewBox`).
#[wasm_bindgen_test]
fn should_set_attr_on_marker() -> Result<(), String> {
    let svg = make_svg("marker-set-attr");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("sa-m").map_err(|e| e.to_string())?;
    marker.set_attr("viewBox", "0 0 10 7").map_err(|e| e.to_string())?;
    common::check_eq(marker.as_element().get_attribute("viewBox"), Some("0 0 10 7".into()))
}

/// `SvgMarker::set_attrs` sets several attributes in one call.
#[wasm_bindgen_test]
fn should_set_attrs_on_marker() -> Result<(), String> {
    let svg = make_svg("marker-set-attrs");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("sa-m2").map_err(|e| e.to_string())?;
    marker
        .set_attrs([("viewBox", "0 0 10 7"), ("overflow", "visible"), ("class", "arrow")])
        .map_err(|e| e.to_string())?;
    let el = marker.as_element();
    common::check_eq(el.get_attribute("viewBox"), Some("0 0 10 7".into()))?;
    common::check_eq(el.get_attribute("overflow"), Some("visible".into()))?;
    common::check_eq(el.get_attribute("class"), Some("arrow".into()))
}

/// `SvgMarker::set_attr_display` formats a numeric value without an external scratch buffer.
#[wasm_bindgen_test]
fn should_set_attr_display_on_marker() -> Result<(), String> {
    let svg = make_svg("marker-set-attr-display");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("sa-m3").map_err(|e| e.to_string())?;
    marker.set_attr_display("opacity", 0.75_f64).map_err(|e| e.to_string())?;
    common::check_eq(marker.as_element().get_attribute("opacity"), Some("0.75".into()))
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
