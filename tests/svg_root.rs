use svg_dom::{
    Error, PathDef, PathDefAbsolute, SvgNode, SvgRoot,
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

/// `attach` also accepts `px`-suffixed width/height attributes, which are common in real-world SVG markup.
#[wasm_bindgen_test]
fn should_cache_px_viewport_when_attaching() -> Result<(), String> {
    let el = common::svg("attach-cache-px-viewport");
    el.set_attribute("width", "800px").unwrap();
    el.set_attribute("height", "600px").unwrap();

    let svg = SvgRoot::attach("attach-cache-px-viewport").map_err(|e| e.to_string())?;

    common::check_eq(svg.width(), 800.0)?;
    common::check_eq(svg.height(), 600.0)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
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

/// `set_viewport` with the same size as the cached viewport writes nothing to the DOM (a duplicate resize is a no-op).
#[wasm_bindgen_test]
fn should_skip_viewport_write_when_size_unchanged() -> Result<(), String> {
    common::div("set-viewport-noop");
    let svg = SvgRoot::create_in("set-viewport-noop", Size::new(100.0, 100.0)).map_err(|e| e.to_string())?;
    let el = svg_element("set-viewport-noop");

    // Mutate the width behind the cache's back.
    el.set_attribute("width", "999").unwrap();

    // Same size as the cached viewport → no DOM write, so the external "999" survives.
    svg.set_viewport(Size::new(100.0, 100.0)).map_err(|e| e.to_string())?;
    common::check_eq(el.get_attribute("width"), Some("999".into()))
}

/// `set_viewport` writes only the axis that changed: an unchanged width is not rewritten, a changed height is.
#[wasm_bindgen_test]
fn should_write_only_changed_viewport_axis() -> Result<(), String> {
    common::div("set-viewport-partial");
    let svg = SvgRoot::create_in("set-viewport-partial", Size::new(100.0, 100.0)).map_err(|e| e.to_string())?;
    let el = svg_element("set-viewport-partial");

    el.set_attribute("width", "999").unwrap(); // mutate width behind the cache

    // Width unchanged (100 → 100) so it is not rewritten ("999" survives); height changed (100 → 200) so it is.
    svg.set_viewport(Size::new(100.0, 200.0)).map_err(|e| e.to_string())?;
    common::check_eq(el.get_attribute("width"), Some("999".into()))?;
    common::check_eq(el.get_attribute("height"), Some("200".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// SvgRoot::set_view_box
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `set_view_box` writes the `viewBox` attribute as `"x y width height"`.
#[wasm_bindgen_test]
fn should_write_view_box_attribute() -> Result<(), String> {
    common::div("set-view-box");
    let svg = SvgRoot::create_in("set-view-box", Size::new(800.0, 600.0)).map_err(|e| e.to_string())?;
    svg.set_view_box(0.0, 0.0, 1000.0, 1000.0).map_err(|e| e.to_string())?;
    let el = svg_element("set-view-box");
    common::check_eq(el.get_attribute("viewBox"), Some("0 0 1000 1000".into()))
}

/// `set_view_box` with a non-zero origin writes both the offset and the size.
#[wasm_bindgen_test]
fn should_write_view_box_with_offset_origin() -> Result<(), String> {
    common::div("set-view-box-offset");
    let svg = SvgRoot::create_in("set-view-box-offset", Size::new(800.0, 600.0)).map_err(|e| e.to_string())?;
    svg.set_view_box(-50.0, -25.0, 400.0, 300.0).map_err(|e| e.to_string())?;
    let el = svg_element("set-view-box-offset");
    common::check_eq(el.get_attribute("viewBox"), Some("-50 -25 400 300".into()))
}

/// `set_view_box` does not alter the cached `width`/`height` that `set_viewport` tracks — the two attributes are
/// independent.
#[wasm_bindgen_test]
fn should_leave_viewport_cache_unchanged_by_view_box() -> Result<(), String> {
    common::div("set-view-box-viewport");
    let svg = SvgRoot::create_in("set-view-box-viewport", Size::new(800.0, 600.0)).map_err(|e| e.to_string())?;
    svg.set_view_box(0.0, 0.0, 1000.0, 1000.0).map_err(|e| e.to_string())?;
    common::check_eq(svg.width(), 800.0)?;
    common::check_eq(svg.height(), 600.0)
}

/// `set_view_box` accepts a `width`/`height` of exactly `0.0` — valid syntax, even though it disables rendering.
#[wasm_bindgen_test]
fn should_accept_zero_width_and_height_view_box() -> Result<(), String> {
    common::div("set-view-box-zero");
    let svg = SvgRoot::create_in("set-view-box-zero", Size::new(800.0, 600.0)).map_err(|e| e.to_string())?;
    svg.set_view_box(0.0, 0.0, 0.0, 0.0).map_err(|e| e.to_string())?;
    let el = svg_element("set-view-box-zero");
    common::check_eq(el.get_attribute("viewBox"), Some("0 0 0 0".into()))
}

/// `set_view_box` rejects a negative `width` with `Error::InvalidViewBox`, and writes nothing to the DOM.
#[wasm_bindgen_test]
fn should_reject_negative_view_box_width() -> Result<(), String> {
    common::div("set-view-box-neg-width");
    let svg = SvgRoot::create_in("set-view-box-neg-width", Size::new(800.0, 600.0)).map_err(|e| e.to_string())?;
    let result = svg.set_view_box(0.0, 0.0, -100.0, 100.0);
    common::check(
        matches!(result, Err(Error::InvalidViewBox(_))),
        "expected InvalidViewBox for negative width",
    )?;
    let el = svg_element("set-view-box-neg-width");
    common::check_eq(el.get_attribute("viewBox"), None)
}

/// `set_view_box` rejects a negative `height` with `Error::InvalidViewBox`.
#[wasm_bindgen_test]
fn should_reject_negative_view_box_height() -> Result<(), String> {
    common::div("set-view-box-neg-height");
    let svg = SvgRoot::create_in("set-view-box-neg-height", Size::new(800.0, 600.0)).map_err(|e| e.to_string())?;
    let result = svg.set_view_box(0.0, 0.0, 100.0, -100.0);
    common::check(
        matches!(result, Err(Error::InvalidViewBox(_))),
        "expected InvalidViewBox for negative height",
    )
}

/// `set_view_box` rejects `NaN` in any component with `Error::InvalidViewBox`.
#[wasm_bindgen_test]
fn should_reject_nan_view_box_component() -> Result<(), String> {
    common::div("set-view-box-nan");
    let svg = SvgRoot::create_in("set-view-box-nan", Size::new(800.0, 600.0)).map_err(|e| e.to_string())?;
    let result = svg.set_view_box(f64::NAN, 0.0, 100.0, 100.0);
    common::check(
        matches!(result, Err(Error::InvalidViewBox(_))),
        "expected InvalidViewBox for a NaN component",
    )
}

/// `set_view_box` rejects `+infinity` and `-infinity` (checked independently) in any component with
/// `Error::InvalidViewBox`.
#[wasm_bindgen_test]
fn should_reject_infinite_view_box_component() -> Result<(), String> {
    common::div("set-view-box-inf");
    let svg = SvgRoot::create_in("set-view-box-inf", Size::new(800.0, 600.0)).map_err(|e| e.to_string())?;
    common::check(
        matches!(svg.set_view_box(0.0, 0.0, f64::INFINITY, 100.0), Err(Error::InvalidViewBox(_))),
        "expected InvalidViewBox for a +infinity component",
    )?;
    common::check(
        matches!(
            svg.set_view_box(0.0, 0.0, f64::NEG_INFINITY, 100.0),
            Err(Error::InvalidViewBox(_))
        ),
        "expected InvalidViewBox for a -infinity component",
    )
}

/// `set_view_box` accepts a negative `x`/`y` origin — only `width`/`height` must be non-negative.
#[wasm_bindgen_test]
fn should_accept_negative_view_box_origin() -> Result<(), String> {
    common::div("set-view-box-neg-origin");
    let svg = SvgRoot::create_in("set-view-box-neg-origin", Size::new(800.0, 600.0)).map_err(|e| e.to_string())?;
    common::check(
        svg.set_view_box(-50.0, -50.0, 100.0, 100.0).is_ok(),
        "expected a negative x/y origin to be accepted",
    )
}

/// `set_view_box` writes fractional components verbatim, not just whole numbers.
#[wasm_bindgen_test]
fn should_write_fractional_view_box_values() -> Result<(), String> {
    common::div("set-view-box-fractional");
    let svg = SvgRoot::create_in("set-view-box-fractional", Size::new(800.0, 600.0)).map_err(|e| e.to_string())?;
    svg.set_view_box(-3.25, 12.5, 100.75, 50.125).map_err(|e| e.to_string())?;
    let el = svg_element("set-view-box-fractional");
    common::check_eq(el.get_attribute("viewBox"), Some("-3.25 12.5 100.75 50.125".into()))
}

/// `set_view_box` accepts `width` of exactly `0.0` on its own, with a non-zero `height`.
#[wasm_bindgen_test]
fn should_accept_zero_width_alone() -> Result<(), String> {
    common::div("set-view-box-zero-width");
    let svg = SvgRoot::create_in("set-view-box-zero-width", Size::new(800.0, 600.0)).map_err(|e| e.to_string())?;
    svg.set_view_box(0.0, 0.0, 0.0, 100.0).map_err(|e| e.to_string())?;
    let el = svg_element("set-view-box-zero-width");
    common::check_eq(el.get_attribute("viewBox"), Some("0 0 0 100".into()))
}

/// `set_view_box` accepts `height` of exactly `0.0` on its own, with a non-zero `width`.
#[wasm_bindgen_test]
fn should_accept_zero_height_alone() -> Result<(), String> {
    common::div("set-view-box-zero-height");
    let svg = SvgRoot::create_in("set-view-box-zero-height", Size::new(800.0, 600.0)).map_err(|e| e.to_string())?;
    svg.set_view_box(0.0, 0.0, 100.0, 0.0).map_err(|e| e.to_string())?;
    let el = svg_element("set-view-box-zero-height");
    common::check_eq(el.get_attribute("viewBox"), Some("0 0 100 0".into()))
}

/// Validation happens before anything is written: a `set_view_box` call that fails leaves a previously-set
/// `viewBox` completely untouched, rather than partially overwriting or clearing it.
#[wasm_bindgen_test]
fn should_preserve_previous_view_box_after_failed_validation() -> Result<(), String> {
    common::div("set-view-box-preserve");
    let svg = SvgRoot::create_in("set-view-box-preserve", Size::new(800.0, 600.0)).map_err(|e| e.to_string())?;
    svg.set_view_box(1.0, 2.0, 300.0, 400.0).map_err(|e| e.to_string())?;

    let result = svg.set_view_box(0.0, 0.0, -100.0, 100.0);
    common::check(
        matches!(result, Err(Error::InvalidViewBox(_))),
        "expected the second, invalid call to fail",
    )?;

    let el = svg_element("set-view-box-preserve");
    common::check_eq(el.get_attribute("viewBox"), Some("1 2 300 400".into()))
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

/// `path_from_defs` builds the `d` attribute from typed [`PathDef`] segments, producing compact path syntax (no
/// whitespace the SVG path grammar does not require) without the possibility of creating a malformed path string.
#[wasm_bindgen_test]
fn should_create_path_from_defs_with_correct_d_attribute() -> Result<(), String> {
    common::div("path-from-defs-factory");
    let svg = SvgRoot::create_in("path-from-defs-factory", Size::new(200.0, 200.0)).map_err(|e| e.to_string())?;
    let path = svg
        .path_from_defs(&[
            PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(0.0, 0.0))),
            PathDef::Abs(PathDefAbsolute::LineTo(Point::new(100.0, 100.0))),
        ])
        .map_err(|e| e.to_string())?;
    common::check_eq(path.attr("d"), Some("M0 0L100 100".into()))
}

/// `path_from_defs` rejects a non-empty sequence whose first command is not a `MoveTo` — an SVG user agent renders
/// nothing for such a path, silently, so this is caught before it reaches the DOM.
#[wasm_bindgen_test]
fn should_reject_path_from_defs_not_starting_with_moveto() -> Result<(), String> {
    common::div("path-from-defs-bad-start");
    let svg = SvgRoot::create_in("path-from-defs-bad-start", Size::new(200.0, 200.0)).map_err(|e| e.to_string())?;
    match svg.path_from_defs(&[PathDef::Abs(PathDefAbsolute::LineTo(Point::new(1.0, 1.0)))]) {
        Err(Error::InvalidPathData(_)) => Ok(()),
        Err(e) => Err(format!("wrong error variant: {e:?}")),
        Ok(_) => Err("expected Err, got Ok".into()),
    }
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

/// `anchor` creates an `<a>` child element and writes `href`.
#[wasm_bindgen_test]
fn should_create_a_element() -> Result<(), String> {
    common::div("anchor-factory");
    let svg = SvgRoot::create_in("anchor-factory", Size::new(200.0, 200.0)).map_err(|e| e.to_string())?;
    let link = svg.anchor("https://example.com").map_err(|e| e.to_string())?;
    common::check_eq(link.as_element().tag_name(), "a".to_string())?;
    common::check_eq(link.attr("href"), Some("https://example.com".into()))
}

/// Children appended to an `<a>` become part of the hyperlink, the same way as `group`.
#[wasm_bindgen_test]
fn should_append_children_to_anchor() -> Result<(), String> {
    common::div("anchor-children");
    let svg = SvgRoot::create_in("anchor-children", Size::new(200.0, 200.0)).map_err(|e| e.to_string())?;
    let link = svg.anchor("https://example.com").map_err(|e| e.to_string())?;
    let icon = svg.circle(Point::new(30.0, 30.0), 18.0).map_err(|e| e.to_string())?;
    link.append(&icon).map_err(|e| e.to_string())?;
    common::check_eq(link.as_element().child_element_count(), 1)?;
    common::check_eq(
        link.as_element().first_element_child().map(|c| c.tag_name()),
        Some("circle".to_string()),
    )
}

/// `switch` creates a `<switch>` child element.
#[wasm_bindgen_test]
fn should_create_switch_element() -> Result<(), String> {
    common::div("switch-factory");
    let svg = SvgRoot::create_in("switch-factory", Size::new(200.0, 200.0)).map_err(|e| e.to_string())?;
    let switch = svg.switch().map_err(|e| e.to_string())?;
    common::check_eq(switch.as_element().tag_name(), "switch".to_string())
}

/// Children appended to `switch` keep their own conditional-processing attributes and document order — this crate
/// performs no validation or selection of its own; the browser evaluates them at render time.
#[wasm_bindgen_test]
fn should_append_children_to_switch_in_order() -> Result<(), String> {
    common::div("switch-children");
    let svg = SvgRoot::create_in("switch-children", Size::new(200.0, 200.0)).map_err(|e| e.to_string())?;
    let switch = svg.switch().map_err(|e| e.to_string())?;

    let french = svg.text(Point::new(10.0, 30.0), "Bonjour").map_err(|e| e.to_string())?;
    french.set_attr("systemLanguage", "fr").map_err(|e| e.to_string())?;
    let fallback = svg.text(Point::new(10.0, 30.0), "Hello").map_err(|e| e.to_string())?;

    switch.append(&french).map_err(|e| e.to_string())?;
    switch.append(&fallback).map_err(|e| e.to_string())?;

    common::check_eq(switch.as_element().child_element_count(), 2)?;
    let first = switch.as_element().first_element_child().ok_or("expected a first child")?;
    common::check_eq(first.get_attribute("systemLanguage"), Some("fr".into()))?;
    let second = first.next_element_sibling().ok_or("expected a second child")?;
    common::check_eq(second.get_attribute("systemLanguage"), None)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Batching (build_batch / build_batch_into)
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Returns the `<svg>` root element created inside the container `id`, for asserting on its children directly.
fn svg_element(id: &str) -> web_sys::Element {
    web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id(id)
        .unwrap()
        .first_element_child()
        .unwrap()
}

/// `build_batch` commits every element it creates to the root in one operation.
#[wasm_bindgen_test]
fn should_build_batch_into_root() -> Result<(), String> {
    common::div("batch-root");
    let svg = SvgRoot::create_in("batch-root", Size::new(200.0, 200.0)).map_err(|e| e.to_string())?;
    svg.build_batch(|b| {
        b.rect(Point::origin(), Size::new(20.0, 20.0))?;
        b.rect(Point::new(30.0, 0.0), Size::new(20.0, 20.0))?;
        Ok(())
    })
    .map_err(|e| e.to_string())?;

    common::check_eq(svg_element("batch-root").child_element_count(), 2)
}

/// `build_batch_into` puts the created children directly inside the target group; the root's only child is the group
/// itself (the children never land on the root).
#[wasm_bindgen_test]
fn should_build_batch_into_group() -> Result<(), String> {
    common::div("batch-into");
    let svg = SvgRoot::create_in("batch-into", Size::new(200.0, 200.0)).map_err(|e| e.to_string())?;
    let group = svg.group().map_err(|e| e.to_string())?;

    svg.build_batch_into(&group, |b| {
        b.rect(Point::origin(), Size::new(20.0, 20.0))?;
        b.circle(Point::new(10.0, 10.0), 5.0)?;
        Ok(())
    })
    .map_err(|e| e.to_string())?;

    common::check_eq(group.as_element().child_element_count(), 2)?;
    // The <svg> root has exactly one element child: the <g> (not the batched shapes).
    common::check_eq(svg_element("batch-into").child_element_count(), 1)
}

/// When the build closure errors, the fragment is dropped and nothing is appended to the target.
#[wasm_bindgen_test]
fn should_not_commit_batch_when_closure_errors() -> Result<(), String> {
    common::div("batch-err");
    let svg = SvgRoot::create_in("batch-err", Size::new(200.0, 200.0)).map_err(|e| e.to_string())?;
    let group = svg.group().map_err(|e| e.to_string())?;

    let result = svg.build_batch_into(&group, |b| {
        b.rect(Point::origin(), Size::new(10.0, 10.0))?;
        Err(Error::Dom("deliberate failure".into()))
    });

    common::check(result.is_err(), "the closure error should propagate")?;
    // The batched child lived only in the (now-dropped) fragment, so the group stays empty.
    common::check_eq(group.as_element().child_element_count(), 0)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// SvgRoot / SvgBatch factory parity
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Asserts a node built via `SvgRoot` and one built via `SvgBatch` have the same tag and the same value for each named
/// attribute. Guards against the two factory surfaces drifting apart.
fn assert_parity(
    label: &str,
    root: Result<SvgNode, Error>,
    batch: Result<SvgNode, Error>,
    attrs: &[&str],
) -> Result<(), String> {
    let root = root.map_err(|e| e.to_string())?;
    let batch = batch.map_err(|e| e.to_string())?;

    let (root_tag, batch_tag) = (root.as_element().tag_name(), batch.as_element().tag_name());
    if root_tag != batch_tag {
        return Err(format!("{label}: tag differs (root `{root_tag}` vs batch `{batch_tag}`)"));
    }
    for name in attrs {
        let (r, b) = (root.attr(name), batch.attr(name));
        if r != b {
            return Err(format!("{label}: attr `{name}` differs (root {r:?} vs batch {b:?})"));
        }
    }
    Ok(())
}

/// Every supported element must be creatable through both `SvgRoot` and `SvgBatch`, producing equivalent DOM.
///
/// This also fails to *compile* if a factory is ever added to one path but not the other — the structural guard the
/// shared `SvgFactory` is meant to provide.
#[wasm_bindgen_test]
fn should_create_equivalent_elements_via_root_and_batch() -> Result<(), String> {
    common::div("factory-parity");
    let svg = SvgRoot::create_in("factory-parity", Size::new(200.0, 200.0)).map_err(|e| e.to_string())?;
    let batch = svg.batch();

    let p = Point::new(10.0, 20.0);
    let q = Point::new(110.0, 120.0);
    let s = Size::new(120.0, 60.0);
    let pts = [Point::new(0.0, 0.0), Point::new(20.0, 40.0), Point::new(40.0, 0.0)];

    assert_parity("rect", svg.rect(p, s), batch.rect(p, s), &["x", "y", "width", "height"])?;
    assert_parity("circle", svg.circle(p, 25.0), batch.circle(p, 25.0), &["cx", "cy", "r"])?;
    assert_parity("ellipse", svg.ellipse(p, s), batch.ellipse(p, s), &["cx", "cy", "rx", "ry"])?;
    assert_parity("line", svg.line(p, q), batch.line(p, q), &["x1", "y1", "x2", "y2"])?;
    assert_parity("path", svg.path("M 0 0 L 10 10"), batch.path("M 0 0 L 10 10"), &["d"])?;
    assert_parity("polyline", svg.polyline(&pts), batch.polyline(&pts), &["points"])?;
    assert_parity("polygon", svg.polygon(&pts), batch.polygon(&pts), &["points"])?;
    assert_parity("group", svg.group(), batch.group(), &[])?;
    assert_parity(
        "anchor",
        svg.anchor("https://example.com"),
        batch.anchor("https://example.com"),
        &["href"],
    )?;
    assert_parity("switch", svg.switch(), batch.switch(), &[])?;

    // text also carries text content, which the attribute comparison above does not cover.
    let r_text = svg.text(p, "parity").map_err(|e| e.to_string())?;
    let b_text = batch.text(p, "parity").map_err(|e| e.to_string())?;
    common::check_eq(r_text.attr("x"), b_text.attr("x"))?;
    common::check_eq(r_text.attr("y"), b_text.attr("y"))?;
    common::check_eq(r_text.as_element().text_content(), b_text.as_element().text_content())
}
