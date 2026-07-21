use crate::common::*;
use svg_dom::Error;
use wasm_bindgen_test::*;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// SvgDefs::filter — eager variant
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `defs.filter(id)` creates an element with tag name `"filter"`.
#[wasm_bindgen_test]
fn should_create_filter_element() -> Result<(), String> {
    let svg = make_svg("filter-create");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("blur").map_err(|e| e.to_string())?;
    check_eq(filter.as_element().tag_name(), "filter".to_owned())
}

/// The `<filter>` element is a child of `<defs>`, not of the root `<svg>`.
#[wasm_bindgen_test]
fn should_append_filter_to_defs() -> Result<(), String> {
    let svg = make_svg("filter-parent");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("soft").map_err(|e| e.to_string())?;
    let parent = filter.as_element().parent_element().ok_or("filter has no parent")?;
    check_eq(parent.tag_name(), "defs".to_owned())
}

/// `defs.filter(id)` sets the `id` attribute on the element.
#[wasm_bindgen_test]
fn should_set_filter_id_attribute() -> Result<(), String> {
    let svg = make_svg("filter-id");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("my-filter").map_err(|e| e.to_string())?;
    check_eq(filter.as_element().get_attribute("id"), Some("my-filter".into()))
}

/// `SvgFilter::id()` returns the cached id without a DOM round-trip.
#[wasm_bindgen_test]
fn should_return_cached_filter_id() -> Result<(), String> {
    let svg = make_svg("filter-cached-id");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("cached").map_err(|e| e.to_string())?;
    check_eq(filter.id(), "cached")
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// SvgDefs::build_filter — deferred/transactional variant
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `build_filter` appends the element to `<defs>` only after the closure returns `Ok`.
#[wasm_bindgen_test]
fn should_build_filter_and_append_to_defs() -> Result<(), String> {
    let svg = make_svg("filter-build");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs
        .build_filter("built", |f| {
            f.gaussian_blur(3.0)?;
            Ok(())
        })
        .map_err(|e| e.to_string())?;
    let parent = filter.as_element().parent_element().ok_or("filter has no parent")?;
    check_eq(parent.tag_name(), "defs".to_owned())
}

/// When `build_filter`'s closure returns `Err`, the filter element is not attached to `<defs>`.
#[wasm_bindgen_test]
fn should_not_append_filter_on_closure_error() -> Result<(), String> {
    let svg = make_svg("filter-build-err");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let result = defs.build_filter("failing", |_f| Err(Error::Dom("deliberate".into())));
    check(result.is_err(), "expected build_filter to return Err")?;
    // The <defs> element should have no children since the closure failed.
    let defs_el = defs.as_element();
    check_eq(defs_el.child_element_count(), 0)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Id validation
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// An invalid id is rejected with `Error::InvalidFilterId` before any DOM call.
#[wasm_bindgen_test]
fn should_reject_invalid_filter_id() -> Result<(), String> {
    let svg = make_svg("filter-invalid-id");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let result = defs.filter("url(#bad)");
    check(
        matches!(result, Err(Error::InvalidFilterId(_))),
        "expected InvalidFilterId error for an id containing '(' and ')'",
    )
}

/// `set_id` updates both the DOM attribute and the cached value.
#[wasm_bindgen_test]
fn should_set_filter_id_via_set_id() -> Result<(), String> {
    let svg = make_svg("filter-set-id");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let mut filter = defs.filter("old-id").map_err(|e| e.to_string())?;
    filter.set_id("new-id").map_err(|e| e.to_string())?;
    check_eq(filter.id(), "new-id")?;
    check_eq(filter.as_element().get_attribute("id"), Some("new-id".into()))
}

/// Renaming to a longer id still round-trips correctly (exercises the cached `url(#id)` reference `String`
/// growing past whatever capacity the previous id left it with).
#[wasm_bindgen_test]
fn should_set_filter_id_longer_than_previous() -> Result<(), String> {
    let svg = make_svg("filter-set-id-longer");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let mut filter = defs.filter("a").map_err(|e| e.to_string())?;
    filter.set_id("a-much-longer-replacement-id").map_err(|e| e.to_string())?;
    check_eq(filter.id(), "a-much-longer-replacement-id")?;
    check_eq(
        filter.as_element().get_attribute("id"),
        Some("a-much-longer-replacement-id".into()),
    )
}

/// `set_attr("id", ...)` is rejected to protect the cached id.
#[wasm_bindgen_test]
fn should_reject_set_attr_id() -> Result<(), String> {
    let svg = make_svg("filter-reserved-id");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("safe").map_err(|e| e.to_string())?;
    let result = filter.set_attr("id", "injected");
    check(
        matches!(result, Err(Error::ReservedAttribute("id"))),
        "expected ReservedAttribute error for set_attr(\"id\", ...)",
    )
}
