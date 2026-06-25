use svg_dom::{
    Error, SvgRoot,
    root::utils::{Point, Size},
};
use wasm_bindgen_test::*;

mod common;

wasm_bindgen_test_configure!(run_in_browser);

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// SvgRoot::attach
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Attaching to an `<svg>` that already exists in the document succeeds.
#[wasm_bindgen_test]
fn should_attach_to_existing_svg() -> Result<(), String> {
    common::svg("attach-ok");
    SvgRoot::attach("attach-ok").map_err(|e| e.to_string())?;
    Ok(())
}

/// Attaching to an id that does not exist returns `ElementNotFound` carrying that id.
#[wasm_bindgen_test]
fn should_not_attach_to_element_with_unknown_id() -> Result<(), String> {
    match SvgRoot::attach("no-such-element-xyzzy") {
        Err(Error::ElementNotFound(id)) => common::check_eq(id, "no-such-element-xyzzy".into()),
        Err(e) => Err(format!("wrong error variant: {e:?}")),
        Ok(_) => Err("expected Err, got Ok".into()),
    }
}

/// Attaching to an id that resolves to a non-`<svg>` element returns `CastFailed`.
#[wasm_bindgen_test]
fn should_not_attach_to_non_svg_element() -> Result<(), String> {
    common::div("attach-cast-fail");
    match SvgRoot::attach("attach-cast-fail") {
        Err(Error::CastFailed(_)) => Ok(()),
        Err(e) => Err(format!("wrong error variant: {e:?}")),
        Ok(_) => Err("expected Err, got Ok".into()),
    }
}

/// `attach` reads the initial viewport attributes once and stores them in memory.
#[wasm_bindgen_test]
fn should_cache_existing_svg_viewport_when_attaching() -> Result<(), String> {
    let el = common::svg("attach-cache-viewport");
    el.set_attribute("width", "320").unwrap();
    el.set_attribute("height", "240").unwrap();

    let svg = SvgRoot::attach("attach-cache-viewport").map_err(|e| e.to_string())?;

    common::check_eq(svg.width(), 320.0)?;
    common::check_eq(svg.height(), 240.0)
}

/// `width` and `height` use the cached viewport instead of reparsing DOM attributes on every call.
#[wasm_bindgen_test]
fn should_not_reparse_viewport_attributes_after_attach() -> Result<(), String> {
    let el = common::svg("attach-cache-no-reparse");
    el.set_attribute("width", "640").unwrap();
    el.set_attribute("height", "480").unwrap();

    let svg = SvgRoot::attach("attach-cache-no-reparse").map_err(|e| e.to_string())?;
    el.set_attribute("width", "1").unwrap();
    el.set_attribute("height", "2").unwrap();

    common::check_eq(svg.width(), 640.0)?;
    common::check_eq(svg.height(), 480.0)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// SvgRoot::create_in
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Creating inside a parent id that does not exist returns `ElementNotFound`.
#[wasm_bindgen_test]
fn should_not_create_element_as_child_of_unknown_parent() -> Result<(), String> {
    match SvgRoot::create_in("no-such-parent-xyzzy", Size::new(100.0, 100.0)) {
        Err(Error::ElementNotFound(id)) => common::check_eq(id, "no-such-parent-xyzzy".into()),
        Err(e) => Err(format!("wrong error variant: {e:?}")),
        Ok(_) => Err("expected Err, got Ok".into()),
    }
}

/// `create_in` appends exactly one `<svg>` child to the named parent element.
#[wasm_bindgen_test]
fn should_create_single_svg_child_in_parent() -> Result<(), String> {
    common::div("create-in-parent");
    SvgRoot::create_in("create-in-parent", Size::new(400.0, 300.0)).map_err(|e| e.to_string())?;

    let document = web_sys::window().unwrap().document().unwrap();
    let parent = document.get_element_by_id("create-in-parent").unwrap();

    common::check_eq(parent.child_element_count(), 1)?;
    common::check_eq(parent.first_element_child().unwrap().tag_name(), "svg".to_string())
}

/// `create_in` sets the `width` attribute to the requested value.
#[wasm_bindgen_test]
fn should_set_width_when_creating_element() -> Result<(), String> {
    common::div("create-in-width");
    let svg = SvgRoot::create_in("create-in-width", Size::new(640.0, 480.0)).map_err(|e| e.to_string())?;
    common::check_eq(svg.width(), 640.0)
}

/// `create_in` sets the `height` attribute to the requested value.
#[wasm_bindgen_test]
fn should_set_height_when_creating_element() -> Result<(), String> {
    common::div("create-in-height");
    let svg = SvgRoot::create_in("create-in-height", Size::new(640.0, 480.0)).map_err(|e| e.to_string())?;
    common::check_eq(svg.height(), 480.0)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// SvgRoot::width / height
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `width()` returns `0.0` when the `width` attribute is absent.
#[wasm_bindgen_test]
fn should_return_zero_for_missing_width() -> Result<(), String> {
    common::svg("width-absent");
    let svg = SvgRoot::attach("width-absent").map_err(|e| e.to_string())?;
    common::check_eq(svg.width(), 0.0)
}

/// `height()` returns `0.0` when the `height` attribute is absent.
#[wasm_bindgen_test]
fn should_return_zero_for_missing_height() -> Result<(), String> {
    common::svg("height-absent");
    let svg = SvgRoot::attach("height-absent").map_err(|e| e.to_string())?;
    common::check_eq(svg.height(), 0.0)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// SvgRoot::set_viewport
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `set_viewport` updates both `width` and `height` attributes.
#[wasm_bindgen_test]
fn should_update_viewport_width_and_height() -> Result<(), String> {
    common::div("set-viewport");
    let svg = SvgRoot::create_in("set-viewport", Size::new(100.0, 100.0)).map_err(|e| e.to_string())?;
    svg.set_viewport(Size::new(1920.0, 1080.0)).map_err(|e| e.to_string())?;
    common::check_eq(svg.width(), 1920.0)?;
    common::check_eq(svg.height(), 1080.0)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Element factories
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `rect` creates a `<rect>` child with the correct position and size attributes.
#[wasm_bindgen_test]
fn should_create_rect_with_correct_attributes() -> Result<(), String> {
    common::div("rect-factory");
    let svg = SvgRoot::create_in("rect-factory", Size::new(200.0, 200.0)).map_err(|e| e.to_string())?;
    let rect = svg
        .rect(Point::new(10.0, 20.0), Size::new(120.0, 60.0))
        .map_err(|e| e.to_string())?;
    common::check_eq(rect.attr("x"), Some("10".into()))?;
    common::check_eq(rect.attr("y"), Some("20".into()))?;
    common::check_eq(rect.attr("width"), Some("120".into()))?;
    common::check_eq(rect.attr("height"), Some("60".into()))
}

/// `circle` creates a `<circle>` child with the correct centre and radius attributes.
#[wasm_bindgen_test]
fn should_create_circle_with_correct_attributes() -> Result<(), String> {
    common::div("circle-factory");
    let svg = SvgRoot::create_in("circle-factory", Size::new(200.0, 200.0)).map_err(|e| e.to_string())?;
    let circle = svg.circle(Point::new(50.0, 60.0), 25.0).map_err(|e| e.to_string())?;
    common::check_eq(circle.attr("cx"), Some("50".into()))?;
    common::check_eq(circle.attr("cy"), Some("60".into()))?;
    common::check_eq(circle.attr("r"), Some("25".into()))
}

/// `ellipse` creates an `<ellipse>` child with the correct centre and independent radii.
#[wasm_bindgen_test]
fn should_create_ellipse_with_correct_attributes() -> Result<(), String> {
    common::div("ellipse-factory");
    let svg = SvgRoot::create_in("ellipse-factory", Size::new(200.0, 200.0)).map_err(|e| e.to_string())?;
    let ellipse = svg
        .ellipse(Point::new(80.0, 60.0), Size::new(40.0, 25.0))
        .map_err(|e| e.to_string())?;
    common::check_eq(ellipse.attr("cx"), Some("80".into()))?;
    common::check_eq(ellipse.attr("cy"), Some("60".into()))?;
    common::check_eq(ellipse.attr("rx"), Some("40".into()))?;
    common::check_eq(ellipse.attr("ry"), Some("25".into()))
}

/// `polyline` creates a `<polyline>` child whose `points` attribute lists each vertex as `x,y`.
#[wasm_bindgen_test]
fn should_create_polyline_with_points_attribute() -> Result<(), String> {
    common::div("polyline-factory");
    let svg = SvgRoot::create_in("polyline-factory", Size::new(200.0, 200.0)).map_err(|e| e.to_string())?;
    let polyline = svg
        .polyline(&[Point::new(0.0, 0.0), Point::new(20.0, 40.0), Point::new(40.0, 0.0)])
        .map_err(|e| e.to_string())?;
    common::check_eq(polyline.attr("points"), Some("0,0 20,40 40,0".into()))
}

/// `polygon` creates a `<polygon>` child whose `points` attribute lists each vertex as `x,y`.
#[wasm_bindgen_test]
fn should_create_polygon_with_points_attribute() -> Result<(), String> {
    common::div("polygon-factory");
    let svg = SvgRoot::create_in("polygon-factory", Size::new(200.0, 200.0)).map_err(|e| e.to_string())?;
    let polygon = svg
        .polygon(&[Point::new(50.0, 0.0), Point::new(100.0, 80.0), Point::new(0.0, 80.0)])
        .map_err(|e| e.to_string())?;
    common::check_eq(polygon.attr("points"), Some("50,0 100,80 0,80".into()))
}

/// `line` creates a `<line>` child with the correct endpoint attributes.
#[wasm_bindgen_test]
fn should_create_line_with_correct_endpoints() -> Result<(), String> {
    common::div("line-factory");
    let svg = SvgRoot::create_in("line-factory", Size::new(200.0, 200.0)).map_err(|e| e.to_string())?;
    let line = svg
        .line(Point::new(0.0, 10.0), Point::new(100.0, 110.0))
        .map_err(|e| e.to_string())?;
    common::check_eq(line.attr("x1"), Some("0".into()))?;
    common::check_eq(line.attr("y1"), Some("10".into()))?;
    common::check_eq(line.attr("x2"), Some("100".into()))?;
    common::check_eq(line.attr("y2"), Some("110".into()))
}

/// `path` creates a `<path>` child carrying the supplied `d` attribute.
#[wasm_bindgen_test]
fn should_create_path_with_d_attribute() -> Result<(), String> {
    common::div("path-factory");
    let svg = SvgRoot::create_in("path-factory", Size::new(200.0, 200.0)).map_err(|e| e.to_string())?;
    let path = svg.path("M 0 0 L 100 100").map_err(|e| e.to_string())?;
    common::check_eq(path.attr("d"), Some("M 0 0 L 100 100".into()))
}

/// `text` creates a `<text>` child at the correct position with the correct string content.
#[wasm_bindgen_test]
fn should_create_text_with_position_and_content() -> Result<(), String> {
    common::div("text-factory");
    let svg = SvgRoot::create_in("text-factory", Size::new(200.0, 200.0)).map_err(|e| e.to_string())?;
    let text = svg.text(Point::new(15.0, 30.0), "Hello SVG").map_err(|e| e.to_string())?;
    common::check_eq(text.attr("x"), Some("15".into()))?;
    common::check_eq(text.attr("y"), Some("30".into()))?;
    common::check_eq(text.as_element().text_content(), Some("Hello SVG".into()))
}

/// `group` creates a `<g>` child element.
#[wasm_bindgen_test]
fn should_create_g_element() -> Result<(), String> {
    common::div("group-factory");
    let svg = SvgRoot::create_in("group-factory", Size::new(200.0, 200.0)).map_err(|e| e.to_string())?;
    let group = svg.group().map_err(|e| e.to_string())?;
    common::check_eq(group.as_element().tag_name(), "g".to_string())
}
