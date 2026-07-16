mod common;

use common::*;
use svg_dom::{
    Error,
    root::{
        pattern::PatternUnits,
        utils::{Point, Size},
    },
};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// SvgDefs::pattern — eager variant
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `defs.pattern(id)` creates an element with tag name `"pattern"`.
#[wasm_bindgen_test]
fn should_create_pattern_element() -> Result<(), String> {
    let svg = make_svg("pat-create");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let pat = defs.pattern("dots").map_err(|e| e.to_string())?;
    check_eq(pat.as_element().tag_name(), "pattern".to_owned())
}

/// The `<pattern>` element is a child of `<defs>`, not of the root `<svg>`.
#[wasm_bindgen_test]
fn should_append_pattern_to_defs() -> Result<(), String> {
    let svg = make_svg("pat-parent");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let pat = defs.pattern("stripes").map_err(|e| e.to_string())?;
    let parent = pat.as_element().parent_element().ok_or("pattern has no parent")?;
    check_eq(parent.tag_name(), "defs".to_owned())
}

/// `defs.pattern(id)` sets the `id` attribute on the element.
#[wasm_bindgen_test]
fn should_set_pattern_id_attribute() -> Result<(), String> {
    let svg = make_svg("pat-id");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let pat = defs.pattern("my-pat").map_err(|e| e.to_string())?;
    check_eq(pat.as_element().get_attribute("id"), Some("my-pat".into()))
}

/// `SvgPattern::id()` returns the cached id without a DOM round-trip.
#[wasm_bindgen_test]
fn should_return_cached_pattern_id() -> Result<(), String> {
    let svg = make_svg("pat-cached-id");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let pat = defs.pattern("cached").map_err(|e| e.to_string())?;
    check_eq(pat.id(), "cached")
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// SvgDefs::build_pattern — deferred/transactional variant
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `build_pattern` appends the element to `<defs>` only after the closure returns `Ok`.
#[wasm_bindgen_test]
fn should_build_pattern_and_append_to_defs() -> Result<(), String> {
    let svg = make_svg("pat-build");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let pat = defs
        .build_pattern("built", |p| {
            p.set_pattern_units(PatternUnits::UserSpaceOnUse)?;
            p.set_width(20.0)?;
            p.set_height(20.0)?;
            p.circle(Point::new(10.0, 10.0), 6.0)?;
            Ok(())
        })
        .map_err(|e| e.to_string())?;
    let parent = pat.as_element().parent_element().ok_or("pattern has no parent")?;
    check_eq(parent.tag_name(), "defs".to_owned())
}

/// When `build_pattern`'s closure returns `Err`, the pattern element is not attached to `<defs>`.
#[wasm_bindgen_test]
fn should_not_append_pattern_on_closure_error() -> Result<(), String> {
    let svg = make_svg("pat-build-err");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let result = defs.build_pattern("failing", |_p| Err(Error::Dom("deliberate".into())));
    check(result.is_err(), "expected build_pattern to return Err")?;
    // The <defs> element should have no children since the closure failed.
    let defs_el = defs.as_element();
    check_eq(defs_el.child_element_count(), 0)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Tile dimension and position attributes
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `set_width` writes the `width` attribute.
#[wasm_bindgen_test]
fn should_set_pattern_width() -> Result<(), String> {
    let svg = make_svg("pat-width");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let pat = defs.pattern("pw").map_err(|e| e.to_string())?;
    pat.set_width(30.0).map_err(|e| e.to_string())?;
    check_eq(pat.as_element().get_attribute("width"), Some("30".into()))
}

/// `set_height` writes the `height` attribute.
#[wasm_bindgen_test]
fn should_set_pattern_height() -> Result<(), String> {
    let svg = make_svg("pat-height");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let pat = defs.pattern("ph").map_err(|e| e.to_string())?;
    pat.set_height(20.0).map_err(|e| e.to_string())?;
    check_eq(pat.as_element().get_attribute("height"), Some("20".into()))
}

/// `set_x` writes the `x` attribute.
#[wasm_bindgen_test]
fn should_set_pattern_x() -> Result<(), String> {
    let svg = make_svg("pat-x");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let pat = defs.pattern("px").map_err(|e| e.to_string())?;
    pat.set_x(5.0).map_err(|e| e.to_string())?;
    check_eq(pat.as_element().get_attribute("x"), Some("5".into()))
}

/// `set_y` writes the `y` attribute.
#[wasm_bindgen_test]
fn should_set_pattern_y() -> Result<(), String> {
    let svg = make_svg("pat-y");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let pat = defs.pattern("py").map_err(|e| e.to_string())?;
    pat.set_y(10.0).map_err(|e| e.to_string())?;
    check_eq(pat.as_element().get_attribute("y"), Some("10".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Coordinate space attributes
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `set_pattern_units(UserSpaceOnUse)` writes `patternUnits="userSpaceOnUse"`.
#[wasm_bindgen_test]
fn should_set_pattern_units_user_space() -> Result<(), String> {
    let svg = make_svg("pat-units-user");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let pat = defs.pattern("pu-user").map_err(|e| e.to_string())?;
    pat.set_pattern_units(PatternUnits::UserSpaceOnUse).map_err(|e| e.to_string())?;
    check_eq(pat.as_element().get_attribute("patternUnits"), Some("userSpaceOnUse".into()))
}

/// `set_pattern_units(ObjectBoundingBox)` writes `patternUnits="objectBoundingBox"`.
#[wasm_bindgen_test]
fn should_set_pattern_units_object_bounding_box() -> Result<(), String> {
    let svg = make_svg("pat-units-obb");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let pat = defs.pattern("pu-obb").map_err(|e| e.to_string())?;
    pat.set_pattern_units(PatternUnits::ObjectBoundingBox)
        .map_err(|e| e.to_string())?;
    check_eq(pat.as_element().get_attribute("patternUnits"), Some("objectBoundingBox".into()))
}

/// `set_pattern_content_units` writes the `patternContentUnits` attribute.
#[wasm_bindgen_test]
fn should_set_pattern_content_units() -> Result<(), String> {
    let svg = make_svg("pat-content-units");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let pat = defs.pattern("pcu").map_err(|e| e.to_string())?;
    pat.set_pattern_content_units(PatternUnits::UserSpaceOnUse)
        .map_err(|e| e.to_string())?;
    check_eq(
        pat.as_element().get_attribute("patternContentUnits"),
        Some("userSpaceOnUse".into()),
    )
}

/// `set_pattern_transform` writes the `patternTransform` attribute.
#[wasm_bindgen_test]
fn should_set_pattern_transform() -> Result<(), String> {
    let svg = make_svg("pat-transform");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let pat = defs.pattern("pt").map_err(|e| e.to_string())?;
    pat.set_pattern_transform("rotate(45)").map_err(|e| e.to_string())?;
    check_eq(pat.as_element().get_attribute("patternTransform"), Some("rotate(45)".into()))
}

/// `set_view_box` writes a correctly formatted `viewBox` attribute.
#[wasm_bindgen_test]
fn should_set_view_box() -> Result<(), String> {
    let svg = make_svg("pat-viewbox");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let pat = defs.pattern("pvb").map_err(|e| e.to_string())?;
    pat.set_view_box(0.0, 0.0, 100.0, 50.0).map_err(|e| e.to_string())?;
    check_eq(pat.as_element().get_attribute("viewBox"), Some("0 0 100 50".into()))
}

/// `set_view_box` accepts a `width`/`height` of exactly `0.0` — valid syntax, even though it disables rendering.
#[wasm_bindgen_test]
fn should_accept_zero_width_and_height_view_box() -> Result<(), String> {
    let svg = make_svg("pat-viewbox-zero");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let pat = defs.pattern("pvb-zero").map_err(|e| e.to_string())?;
    pat.set_view_box(0.0, 0.0, 0.0, 0.0).map_err(|e| e.to_string())?;
    check_eq(pat.as_element().get_attribute("viewBox"), Some("0 0 0 0".into()))
}

/// `set_view_box` rejects a negative `width`/`height`, and a non-finite (`NaN`/`±infinity`) component, with
/// `Error::InvalidViewBox` — the same validation `SvgRoot::set_view_box` and `SvgSymbol::set_view_box` share.
#[wasm_bindgen_test]
fn should_reject_invalid_view_box() -> Result<(), String> {
    let svg = make_svg("pat-viewbox-invalid");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let pat = defs.pattern("pvb-invalid").map_err(|e| e.to_string())?;
    check(
        matches!(pat.set_view_box(0.0, 0.0, -100.0, 100.0), Err(Error::InvalidViewBox(_))),
        "expected InvalidViewBox for negative width",
    )?;
    check(
        matches!(pat.set_view_box(0.0, 0.0, 100.0, -100.0), Err(Error::InvalidViewBox(_))),
        "expected InvalidViewBox for negative height",
    )?;
    check(
        matches!(pat.set_view_box(f64::NAN, 0.0, 100.0, 100.0), Err(Error::InvalidViewBox(_))),
        "expected InvalidViewBox for a NaN component",
    )?;
    check(
        matches!(
            pat.set_view_box(0.0, 0.0, f64::INFINITY, 100.0),
            Err(Error::InvalidViewBox(_))
        ),
        "expected InvalidViewBox for an infinite component",
    )
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Shape factories
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Shapes added via factory methods become children of the `<pattern>` element.
#[wasm_bindgen_test]
fn should_add_shapes_to_pattern() -> Result<(), String> {
    let svg = make_svg("pat-shapes");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let pat = defs.pattern("ps").map_err(|e| e.to_string())?;
    pat.circle(Point::new(10.0, 10.0), 6.0).map_err(|e| e.to_string())?;
    pat.rect(Point::new(0.0, 0.0), Size::new(20.0, 20.0))
        .map_err(|e| e.to_string())?;
    check_eq(pat.as_element().child_element_count(), 2)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Id validation
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// An invalid id is rejected with `Error::InvalidPatternId` before any DOM call.
#[wasm_bindgen_test]
fn should_reject_invalid_pattern_id() -> Result<(), String> {
    let svg = make_svg("pat-invalid-id");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let result = defs.pattern("url(#bad)");
    check(
        matches!(result, Err(Error::InvalidPatternId(_))),
        "expected InvalidPatternId error for an id containing '(' and ')'",
    )
}

/// `set_id` updates both the DOM attribute and the cached value.
#[wasm_bindgen_test]
fn should_set_pattern_id_via_set_id() -> Result<(), String> {
    let svg = make_svg("pat-set-id");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let mut pat = defs.pattern("old-id").map_err(|e| e.to_string())?;
    pat.set_id("new-id").map_err(|e| e.to_string())?;
    check_eq(pat.id(), "new-id")?;
    check_eq(pat.as_element().get_attribute("id"), Some("new-id".into()))
}

/// Renaming to a longer id still round-trips correctly (exercises the cached `url(#id)` reference `String`
/// growing past whatever capacity the previous id left it with).
#[wasm_bindgen_test]
fn should_set_pattern_id_longer_than_previous() -> Result<(), String> {
    let svg = make_svg("pat-set-id-longer");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let mut pat = defs.pattern("a").map_err(|e| e.to_string())?;
    pat.set_id("a-much-longer-replacement-id").map_err(|e| e.to_string())?;
    check_eq(pat.id(), "a-much-longer-replacement-id")?;
    check_eq(
        pat.as_element().get_attribute("id"),
        Some("a-much-longer-replacement-id".into()),
    )
}

/// `set_attr("id", ...)` is rejected to protect the cached id.
#[wasm_bindgen_test]
fn should_reject_set_attr_id() -> Result<(), String> {
    let svg = make_svg("pat-reserved-id");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let pat = defs.pattern("safe").map_err(|e| e.to_string())?;
    let result = pat.set_attr("id", "injected");
    check(
        matches!(result, Err(Error::ReservedAttribute("id"))),
        "expected ReservedAttribute error for set_attr(\"id\", ...)",
    )
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// SvgNode paint-server setters
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `set_fill_pattern` writes `fill="url(#id)"`.
#[wasm_bindgen_test]
fn should_set_fill_pattern() -> Result<(), String> {
    let svg = make_svg("pat-fill");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    defs.pattern("my-dots").map_err(|e| e.to_string())?;
    let rect = svg.rect(Point::origin(), Size::new(100.0, 100.0)).map_err(|e| e.to_string())?;
    rect.set_fill_pattern("my-dots").map_err(|e| e.to_string())?;
    check_eq(rect.as_element().get_attribute("fill"), Some("url(#my-dots)".into()))
}

/// `set_stroke_pattern` writes `stroke="url(#id)"`.
#[wasm_bindgen_test]
fn should_set_stroke_pattern() -> Result<(), String> {
    let svg = make_svg("pat-stroke");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    defs.pattern("my-stripes").map_err(|e| e.to_string())?;
    let rect = svg.rect(Point::origin(), Size::new(100.0, 100.0)).map_err(|e| e.to_string())?;
    rect.set_stroke_pattern("my-stripes").map_err(|e| e.to_string())?;
    check_eq(rect.as_element().get_attribute("stroke"), Some("url(#my-stripes)".into()))
}

/// `set_fill_pattern_ref` produces the same result as `set_fill_pattern` using the id from the handle.
#[wasm_bindgen_test]
fn should_set_fill_pattern_ref() -> Result<(), String> {
    let svg = make_svg("pat-fill-ref");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let pat = defs.pattern("ref-fill").map_err(|e| e.to_string())?;
    let rect = svg.rect(Point::origin(), Size::new(100.0, 100.0)).map_err(|e| e.to_string())?;
    rect.set_fill_pattern_ref(&pat).map_err(|e| e.to_string())?;
    check_eq(rect.as_element().get_attribute("fill"), Some("url(#ref-fill)".into()))
}

/// `set_stroke_pattern_ref` produces the same result as `set_stroke_pattern` using the id from the handle.
#[wasm_bindgen_test]
fn should_set_stroke_pattern_ref() -> Result<(), String> {
    let svg = make_svg("pat-stroke-ref");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let pat = defs.pattern("ref-stroke").map_err(|e| e.to_string())?;
    let rect = svg.rect(Point::origin(), Size::new(100.0, 100.0)).map_err(|e| e.to_string())?;
    rect.set_stroke_pattern_ref(&pat).map_err(|e| e.to_string())?;
    check_eq(rect.as_element().get_attribute("stroke"), Some("url(#ref-stroke)".into()))
}

/// `set_fill_pattern` with an invalid id returns `Error::InvalidPatternId`.
#[wasm_bindgen_test]
fn should_reject_invalid_fill_pattern_id() -> Result<(), String> {
    let svg = make_svg("pat-invalid-fill");
    let rect = svg.rect(Point::origin(), Size::new(100.0, 100.0)).map_err(|e| e.to_string())?;
    let result = rect.set_fill_pattern("url(#bad)");
    check(
        matches!(result, Err(Error::InvalidPatternId(_))),
        "expected InvalidPatternId from set_fill_pattern with bad id",
    )
}
