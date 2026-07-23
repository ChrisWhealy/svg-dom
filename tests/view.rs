mod common;

use common::*;
use svg_dom::Error;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// SvgDefs::view — eager variant
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `defs.view(id)` creates an element with tag name `"view"`.
#[wasm_bindgen_test]
fn should_create_view_element() -> Result<(), String> {
    let svg = make_svg("view-create");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let view = defs.view("detail").map_err(|e| e.to_string())?;
    check_eq(view.as_element().tag_name(), "view".to_owned())
}

/// The `<view>` element is a child of `<defs>`, not of the root `<svg>`.
#[wasm_bindgen_test]
fn should_append_view_to_defs() -> Result<(), String> {
    let svg = make_svg("view-parent");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let view = defs.view("detail").map_err(|e| e.to_string())?;
    let parent = view.as_element().parent_element().ok_or("view has no parent")?;
    check_eq(parent.tag_name(), "defs".to_owned())
}

/// `defs.view(id)` sets the `id` attribute on the element.
#[wasm_bindgen_test]
fn should_set_view_id_attribute() -> Result<(), String> {
    let svg = make_svg("view-id");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let view = defs.view("my-view").map_err(|e| e.to_string())?;
    check_eq(view.as_element().get_attribute("id"), Some("my-view".into()))
}

/// `SvgView::id()` returns the cached id without a DOM round-trip.
#[wasm_bindgen_test]
fn should_return_cached_view_id() -> Result<(), String> {
    let svg = make_svg("view-cached-id");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let view = defs.view("cached").map_err(|e| e.to_string())?;
    check_eq(view.id(), "cached")
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// SvgDefs::build_view — deferred/transactional variant
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `build_view` appends the element to `<defs>` only after the closure returns `Ok`.
#[wasm_bindgen_test]
fn should_build_view_and_append_to_defs() -> Result<(), String> {
    let svg = make_svg("view-build");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let view = defs
        .build_view("built", |v| v.set_view_box(0.0, 0.0, 50.0, 50.0))
        .map_err(|e| e.to_string())?;
    let parent = view.as_element().parent_element().ok_or("view has no parent")?;
    check_eq(parent.tag_name(), "defs".to_owned())?;
    check_eq(view.as_element().get_attribute("viewBox"), Some("0 0 50 50".into()))
}

/// When `build_view`'s closure returns `Err`, the view element is not attached to `<defs>`.
#[wasm_bindgen_test]
fn should_not_append_view_on_closure_error() -> Result<(), String> {
    let svg = make_svg("view-build-err");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let result = defs.build_view("failing", |_v| Err(Error::Dom("deliberate".into())));
    check(result.is_err(), "expected build_view to return Err")?;
    let defs_el = defs.as_element();
    check_eq(defs_el.child_element_count(), 0)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Viewport attributes
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `set_view_box` writes a correctly formatted `viewBox` attribute.
#[wasm_bindgen_test]
fn should_set_view_box() -> Result<(), String> {
    let svg = make_svg("view-viewbox");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let view = defs.view("vb").map_err(|e| e.to_string())?;
    view.set_view_box(0.0, 0.0, 100.0, 50.0).map_err(|e| e.to_string())?;
    check_eq(view.as_element().get_attribute("viewBox"), Some("0 0 100 50".into()))
}

/// `set_view_box` rejects a negative `width`/`height`, and a non-finite (`NaN`/`±infinity`) component, with
/// `Error::InvalidViewBox` — the same validation `SvgRoot::set_view_box` and `SvgSymbol::set_view_box` share.
#[wasm_bindgen_test]
fn should_reject_invalid_view_box() -> Result<(), String> {
    let svg = make_svg("view-viewbox-invalid");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let view = defs.view("vb-invalid").map_err(|e| e.to_string())?;
    check(
        matches!(view.set_view_box(0.0, 0.0, -100.0, 100.0), Err(Error::InvalidViewBox(_))),
        "expected InvalidViewBox for negative width",
    )?;
    check(
        matches!(view.set_view_box(f64::NAN, 0.0, 100.0, 100.0), Err(Error::InvalidViewBox(_))),
        "expected InvalidViewBox for a NaN component",
    )
}

/// `set_preserve_aspect_ratio` writes the attribute verbatim.
#[wasm_bindgen_test]
fn should_set_preserve_aspect_ratio() -> Result<(), String> {
    let svg = make_svg("view-par");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let view = defs.view("par").map_err(|e| e.to_string())?;
    view.set_preserve_aspect_ratio("xMidYMid slice").map_err(|e| e.to_string())?;
    check_eq(
        view.as_element().get_attribute("preserveAspectRatio"),
        Some("xMidYMid slice".into()),
    )
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Id validation
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// An invalid id is rejected with `Error::InvalidViewId` before any DOM call.
#[wasm_bindgen_test]
fn should_reject_invalid_view_id() -> Result<(), String> {
    let svg = make_svg("view-invalid-id");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let result = defs.view("url(#bad)");
    check(
        matches!(result, Err(Error::InvalidViewId(_))),
        "expected InvalidViewId error for an id containing '(' and ')'",
    )
}

/// `set_id` updates both the DOM attribute and the cached value.
#[wasm_bindgen_test]
fn should_set_view_id_via_set_id() -> Result<(), String> {
    let svg = make_svg("view-set-id");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let mut view = defs.view("old-id").map_err(|e| e.to_string())?;
    view.set_id("new-id").map_err(|e| e.to_string())?;
    check_eq(view.id(), "new-id")?;
    check_eq(view.as_element().get_attribute("id"), Some("new-id".into()))
}

/// `set_attr("id", ...)` is rejected to protect the cached id.
#[wasm_bindgen_test]
fn should_reject_set_attr_id() -> Result<(), String> {
    let svg = make_svg("view-reserved-id");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let view = defs.view("safe").map_err(|e| e.to_string())?;
    let result = view.set_attr("id", "injected");
    check(
        matches!(result, Err(Error::ReservedAttribute("id"))),
        "expected ReservedAttribute error for set_attr(\"id\", ...)",
    )
}
