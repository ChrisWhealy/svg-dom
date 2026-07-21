use crate::common::*;
use svg_dom::{
    Error,
    root::utils::{Point, Size},
};
use wasm_bindgen_test::*;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// SvgNode::set_filter / set_filter_ref / remove_filter
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `set_filter` writes `filter="url(#id)"`.
#[wasm_bindgen_test]
fn should_set_filter_attribute() -> Result<(), String> {
    let svg = make_svg("filter-apply");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    defs.filter("my-blur").map_err(|e| e.to_string())?;
    let rect = svg.rect(Point::origin(), Size::new(100.0, 100.0)).map_err(|e| e.to_string())?;
    rect.set_filter("my-blur").map_err(|e| e.to_string())?;
    check_eq(rect.as_element().get_attribute("filter"), Some("url(#my-blur)".into()))
}

/// `set_filter_ref` produces the same result as `set_filter` using the id from the handle.
#[wasm_bindgen_test]
fn should_set_filter_ref() -> Result<(), String> {
    let svg = make_svg("filter-apply-ref");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("ref-blur").map_err(|e| e.to_string())?;
    let rect = svg.rect(Point::origin(), Size::new(100.0, 100.0)).map_err(|e| e.to_string())?;
    rect.set_filter_ref(&filter).map_err(|e| e.to_string())?;
    check_eq(rect.as_element().get_attribute("filter"), Some("url(#ref-blur)".into()))
}

/// `set_filter` with an invalid id returns `Error::InvalidFilterId`.
#[wasm_bindgen_test]
fn should_reject_invalid_set_filter_id() -> Result<(), String> {
    let svg = make_svg("filter-invalid-apply");
    let rect = svg.rect(Point::origin(), Size::new(100.0, 100.0)).map_err(|e| e.to_string())?;
    let result = rect.set_filter("url(#bad)");
    check(
        matches!(result, Err(Error::InvalidFilterId(_))),
        "expected InvalidFilterId from set_filter with bad id",
    )
}

/// `remove_filter` removes the `filter` attribute.
#[wasm_bindgen_test]
fn should_remove_filter_attribute() -> Result<(), String> {
    let svg = make_svg("filter-remove");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("removable").map_err(|e| e.to_string())?;
    let rect = svg.rect(Point::origin(), Size::new(100.0, 100.0)).map_err(|e| e.to_string())?;
    rect.set_filter_ref(&filter).map_err(|e| e.to_string())?;
    rect.remove_filter().map_err(|e| e.to_string())?;
    check_eq(rect.as_element().get_attribute("filter"), None)
}
