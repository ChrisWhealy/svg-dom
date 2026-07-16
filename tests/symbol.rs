mod common;

use common::*;
use svg_dom::{
    Error,
    root::utils::{Point, Size},
};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// SvgDefs::symbol — eager variant
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `defs.symbol(id)` creates an element with tag name `"symbol"`.
#[wasm_bindgen_test]
fn should_create_symbol_element() -> Result<(), String> {
    let svg = make_svg("sym-create");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let sym = defs.symbol("icon").map_err(|e| e.to_string())?;
    check_eq(sym.as_element().tag_name(), "symbol".to_owned())
}

/// The `<symbol>` element is a child of `<defs>`, not of the root `<svg>`.
#[wasm_bindgen_test]
fn should_append_symbol_to_defs() -> Result<(), String> {
    let svg = make_svg("sym-parent");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let sym = defs.symbol("badge").map_err(|e| e.to_string())?;
    let parent = sym.as_element().parent_element().ok_or("symbol has no parent")?;
    check_eq(parent.tag_name(), "defs".to_owned())
}

/// `defs.symbol(id)` sets the `id` attribute on the element.
#[wasm_bindgen_test]
fn should_set_symbol_id_attribute() -> Result<(), String> {
    let svg = make_svg("sym-id");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let sym = defs.symbol("my-sym").map_err(|e| e.to_string())?;
    check_eq(sym.as_element().get_attribute("id"), Some("my-sym".into()))
}

/// `SvgSymbol::id()` returns the cached id without a DOM round-trip.
#[wasm_bindgen_test]
fn should_return_cached_symbol_id() -> Result<(), String> {
    let svg = make_svg("sym-cached-id");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let sym = defs.symbol("cached").map_err(|e| e.to_string())?;
    check_eq(sym.id(), "cached")
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// SvgDefs::build_symbol — deferred/transactional variant
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `build_symbol` appends the element to `<defs>` only after the closure returns `Ok`.
#[wasm_bindgen_test]
fn should_build_symbol_and_append_to_defs() -> Result<(), String> {
    let svg = make_svg("sym-build");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let sym = defs
        .build_symbol("built", |s| {
            s.set_view_box(0.0, 0.0, 40.0, 40.0)?;
            s.circle(Point::new(20.0, 20.0), 18.0)?;
            Ok(())
        })
        .map_err(|e| e.to_string())?;
    let parent = sym.as_element().parent_element().ok_or("symbol has no parent")?;
    check_eq(parent.tag_name(), "defs".to_owned())
}

/// When `build_symbol`'s closure returns `Err`, the symbol element is not attached to `<defs>`.
#[wasm_bindgen_test]
fn should_not_append_symbol_on_closure_error() -> Result<(), String> {
    let svg = make_svg("sym-build-err");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let result = defs.build_symbol("failing", |_s| Err(Error::Dom("deliberate".into())));
    check(result.is_err(), "expected build_symbol to return Err")?;
    // The <defs> element should have no children since the closure failed.
    let defs_el = defs.as_element();
    check_eq(defs_el.child_element_count(), 0)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Viewport attributes
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `set_view_box` writes a correctly formatted `viewBox` attribute.
#[wasm_bindgen_test]
fn should_set_view_box() -> Result<(), String> {
    let svg = make_svg("sym-viewbox");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let sym = defs.symbol("vb").map_err(|e| e.to_string())?;
    sym.set_view_box(0.0, 0.0, 100.0, 50.0).map_err(|e| e.to_string())?;
    check_eq(sym.as_element().get_attribute("viewBox"), Some("0 0 100 50".into()))
}

/// `set_view_box` accepts a `width`/`height` of exactly `0.0` — valid syntax, even though it disables rendering.
#[wasm_bindgen_test]
fn should_accept_zero_width_and_height_view_box() -> Result<(), String> {
    let svg = make_svg("sym-viewbox-zero");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let sym = defs.symbol("vb-zero").map_err(|e| e.to_string())?;
    sym.set_view_box(0.0, 0.0, 0.0, 0.0).map_err(|e| e.to_string())?;
    check_eq(sym.as_element().get_attribute("viewBox"), Some("0 0 0 0".into()))
}

/// `set_view_box` rejects a negative `width`/`height`, and a non-finite (`NaN`/`±infinity`) component, with
/// `Error::InvalidViewBox` — the same validation `SvgRoot::set_view_box` and `SvgPattern::set_view_box` share.
#[wasm_bindgen_test]
fn should_reject_invalid_view_box() -> Result<(), String> {
    let svg = make_svg("sym-viewbox-invalid");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let sym = defs.symbol("vb-invalid").map_err(|e| e.to_string())?;
    check(
        matches!(sym.set_view_box(0.0, 0.0, -100.0, 100.0), Err(Error::InvalidViewBox(_))),
        "expected InvalidViewBox for negative width",
    )?;
    check(
        matches!(sym.set_view_box(0.0, 0.0, 100.0, -100.0), Err(Error::InvalidViewBox(_))),
        "expected InvalidViewBox for negative height",
    )?;
    check(
        matches!(sym.set_view_box(f64::NAN, 0.0, 100.0, 100.0), Err(Error::InvalidViewBox(_))),
        "expected InvalidViewBox for a NaN component",
    )?;
    check(
        matches!(
            sym.set_view_box(0.0, 0.0, f64::INFINITY, 100.0),
            Err(Error::InvalidViewBox(_))
        ),
        "expected InvalidViewBox for an infinite component",
    )
}

/// `set_preserve_aspect_ratio` writes the attribute verbatim.
#[wasm_bindgen_test]
fn should_set_preserve_aspect_ratio() -> Result<(), String> {
    let svg = make_svg("sym-par");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let sym = defs.symbol("par").map_err(|e| e.to_string())?;
    sym.set_preserve_aspect_ratio("xMidYMid slice").map_err(|e| e.to_string())?;
    check_eq(
        sym.as_element().get_attribute("preserveAspectRatio"),
        Some("xMidYMid slice".into()),
    )
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Shape factories
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Shapes added via factory methods become children of the `<symbol>` element.
#[wasm_bindgen_test]
fn should_add_shapes_to_symbol() -> Result<(), String> {
    let svg = make_svg("sym-shapes");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let sym = defs.symbol("shapes").map_err(|e| e.to_string())?;
    sym.circle(Point::new(20.0, 20.0), 10.0).map_err(|e| e.to_string())?;
    sym.rect(Point::new(0.0, 0.0), Size::new(10.0, 10.0))
        .map_err(|e| e.to_string())?;
    check_eq(sym.as_element().child_element_count(), 2)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Id validation
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// An invalid id is rejected with `Error::InvalidSymbolId` before any DOM call.
#[wasm_bindgen_test]
fn should_reject_invalid_symbol_id() -> Result<(), String> {
    let svg = make_svg("sym-invalid-id");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let result = defs.symbol("url(#bad)");
    check(
        matches!(result, Err(Error::InvalidSymbolId(_))),
        "expected InvalidSymbolId error for an id containing '(' and ')'",
    )
}

/// `set_id` updates both the DOM attribute and the cached value.
#[wasm_bindgen_test]
fn should_set_symbol_id_via_set_id() -> Result<(), String> {
    let svg = make_svg("sym-set-id");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let mut sym = defs.symbol("old-id").map_err(|e| e.to_string())?;
    sym.set_id("new-id").map_err(|e| e.to_string())?;
    check_eq(sym.id(), "new-id")?;
    check_eq(sym.as_element().get_attribute("id"), Some("new-id".into()))
}

/// `set_attr("id", ...)` is rejected to protect the cached id.
#[wasm_bindgen_test]
fn should_reject_set_attr_id() -> Result<(), String> {
    let svg = make_svg("sym-reserved-id");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let sym = defs.symbol("safe").map_err(|e| e.to_string())?;
    let result = sym.set_attr("id", "injected");
    check(
        matches!(result, Err(Error::ReservedAttribute("id"))),
        "expected ReservedAttribute error for set_attr(\"id\", ...)",
    )
}
