mod common;

use common::*;
use svg_dom::{
    Error,
    root::utils::{Point, Size},
};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

// This file covers only the SvgNode id-reference setters (set_fill_gradient/set_stroke_gradient and their
// set_*_linear_gradient/set_*_radial_gradient handle-based siblings) — the part of the gradient API touched by
// the set_url_ref refactor. Broader gradient coverage (stops, axis/focal coordinates, spreadMethod,
// gradientUnits/gradientTransform) is a separate, pre-existing gap, not attempted here.

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Construction sanity
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `defs.linear_gradient(id)` creates an element with tag name `"linearGradient"`.
#[wasm_bindgen_test]
fn should_create_linear_gradient_element() -> Result<(), String> {
    let svg = make_svg("grad-lin-create");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let grad = defs.linear_gradient("lg").map_err(|e| e.to_string())?;
    check_eq(grad.as_element().tag_name(), "linearGradient".to_owned())
}

/// `defs.radial_gradient(id)` creates an element with tag name `"radialGradient"`.
#[wasm_bindgen_test]
fn should_create_radial_gradient_element() -> Result<(), String> {
    let svg = make_svg("grad-rad-create");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let grad = defs.radial_gradient("rg").map_err(|e| e.to_string())?;
    check_eq(grad.as_element().tag_name(), "radialGradient".to_owned())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// SvgNode::set_fill_gradient / set_stroke_gradient — bare id
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `set_fill_gradient` writes `fill="url(#id)"`.
#[wasm_bindgen_test]
fn should_set_fill_gradient() -> Result<(), String> {
    let svg = make_svg("grad-fill");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    defs.linear_gradient("g-fill").map_err(|e| e.to_string())?;
    let rect = svg.rect(Point::origin(), Size::new(100.0, 100.0)).map_err(|e| e.to_string())?;
    rect.set_fill_gradient("g-fill").map_err(|e| e.to_string())?;
    check_eq(rect.as_element().get_attribute("fill"), Some("url(#g-fill)".into()))
}

/// `set_stroke_gradient` writes `stroke="url(#id)"`.
#[wasm_bindgen_test]
fn should_set_stroke_gradient() -> Result<(), String> {
    let svg = make_svg("grad-stroke");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    defs.linear_gradient("g-stroke").map_err(|e| e.to_string())?;
    let rect = svg.rect(Point::origin(), Size::new(100.0, 100.0)).map_err(|e| e.to_string())?;
    rect.set_stroke_gradient("g-stroke").map_err(|e| e.to_string())?;
    check_eq(rect.as_element().get_attribute("stroke"), Some("url(#g-stroke)".into()))
}

/// `set_fill_gradient` with an invalid id returns `Error::InvalidGradientId`.
#[wasm_bindgen_test]
fn should_reject_invalid_fill_gradient_id() -> Result<(), String> {
    let svg = make_svg("grad-invalid-fill");
    let rect = svg.rect(Point::origin(), Size::new(100.0, 100.0)).map_err(|e| e.to_string())?;
    let result = rect.set_fill_gradient("url(#bad)");
    check(
        matches!(result, Err(Error::InvalidGradientId(_))),
        "expected InvalidGradientId from set_fill_gradient with bad id",
    )
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// SvgNode::set_fill_linear_gradient / set_stroke_linear_gradient — handle-based
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `set_fill_linear_gradient` produces the same result as `set_fill_gradient` using the id from the handle.
#[wasm_bindgen_test]
fn should_set_fill_linear_gradient() -> Result<(), String> {
    let svg = make_svg("grad-fill-lin-ref");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let grad = defs.linear_gradient("g-fill-lin-ref").map_err(|e| e.to_string())?;
    let rect = svg.rect(Point::origin(), Size::new(100.0, 100.0)).map_err(|e| e.to_string())?;
    rect.set_fill_linear_gradient(&grad).map_err(|e| e.to_string())?;
    check_eq(rect.as_element().get_attribute("fill"), Some("url(#g-fill-lin-ref)".into()))
}

/// `set_stroke_linear_gradient` produces the same result as `set_stroke_gradient` using the id from the handle.
#[wasm_bindgen_test]
fn should_set_stroke_linear_gradient() -> Result<(), String> {
    let svg = make_svg("grad-stroke-lin-ref");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let grad = defs.linear_gradient("g-stroke-lin-ref").map_err(|e| e.to_string())?;
    let rect = svg.rect(Point::origin(), Size::new(100.0, 100.0)).map_err(|e| e.to_string())?;
    rect.set_stroke_linear_gradient(&grad).map_err(|e| e.to_string())?;
    check_eq(rect.as_element().get_attribute("stroke"), Some("url(#g-stroke-lin-ref)".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// SvgNode::set_fill_radial_gradient / set_stroke_radial_gradient — handle-based
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `set_fill_radial_gradient` produces the same result as `set_fill_gradient` using the id from the handle.
#[wasm_bindgen_test]
fn should_set_fill_radial_gradient() -> Result<(), String> {
    let svg = make_svg("grad-fill-rad-ref");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let grad = defs.radial_gradient("g-fill-rad-ref").map_err(|e| e.to_string())?;
    let circle = svg.circle(Point::new(50.0, 50.0), 40.0).map_err(|e| e.to_string())?;
    circle.set_fill_radial_gradient(&grad).map_err(|e| e.to_string())?;
    check_eq(circle.as_element().get_attribute("fill"), Some("url(#g-fill-rad-ref)".into()))
}

/// `set_stroke_radial_gradient` produces the same result as `set_stroke_gradient` using the id from the handle.
#[wasm_bindgen_test]
fn should_set_stroke_radial_gradient() -> Result<(), String> {
    let svg = make_svg("grad-stroke-rad-ref");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let grad = defs.radial_gradient("g-stroke-rad-ref").map_err(|e| e.to_string())?;
    let circle = svg.circle(Point::new(50.0, 50.0), 40.0).map_err(|e| e.to_string())?;
    circle.set_stroke_radial_gradient(&grad).map_err(|e| e.to_string())?;
    check_eq(circle.as_element().get_attribute("stroke"), Some("url(#g-stroke-rad-ref)".into()))
}
