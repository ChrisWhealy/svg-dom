mod common;

use common::*;
use svg_dom::{
    Error,
    root::{
        mask::{MaskType, MaskUnits},
        utils::{Point, Size},
    },
};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// SvgDefs::mask — eager variant
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `defs.mask(id)` creates an element with tag name `"mask"`.
#[wasm_bindgen_test]
fn should_create_mask_element() -> Result<(), String> {
    let svg = make_svg("mask-create");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let mask = defs.mask("fade").map_err(|e| e.to_string())?;
    check_eq(mask.as_element().tag_name(), "mask".to_owned())
}

/// The `<mask>` element is a child of `<defs>`, not of the root `<svg>`.
#[wasm_bindgen_test]
fn should_append_mask_to_defs() -> Result<(), String> {
    let svg = make_svg("mask-parent");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let mask = defs.mask("fade-p").map_err(|e| e.to_string())?;
    let parent = mask.as_element().parent_element().ok_or("mask has no parent")?;
    check_eq(parent.tag_name(), "defs".to_owned())
}

/// `defs.mask(id)` sets the `id` attribute on the element.
#[wasm_bindgen_test]
fn should_set_mask_id_attribute() -> Result<(), String> {
    let svg = make_svg("mask-id");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let mask = defs.mask("my-mask").map_err(|e| e.to_string())?;
    check_eq(mask.as_element().get_attribute("id"), Some("my-mask".into()))
}

/// `SvgMask::id()` returns the cached id without a DOM round-trip.
#[wasm_bindgen_test]
fn should_return_cached_mask_id() -> Result<(), String> {
    let svg = make_svg("mask-cached-id");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let mask = defs.mask("cached").map_err(|e| e.to_string())?;
    check_eq(mask.id(), "cached")
}

/// An invalid id is rejected with `Error::InvalidMaskId` before any DOM call.
#[wasm_bindgen_test]
fn should_reject_invalid_mask_id() -> Result<(), String> {
    let svg = make_svg("mask-invalid-id");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let result = defs.mask("url(#bad)");
    check(
        matches!(result, Err(Error::InvalidMaskId(_))),
        "expected InvalidMaskId error for an id containing '(' and ')'",
    )
}

/// `set_id` updates both the DOM attribute and the cached value.
#[wasm_bindgen_test]
fn should_set_mask_id_via_set_id() -> Result<(), String> {
    let svg = make_svg("mask-set-id");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let mut mask = defs.mask("old-id").map_err(|e| e.to_string())?;
    mask.set_id("new-id").map_err(|e| e.to_string())?;
    check_eq(mask.id(), "new-id")?;
    check_eq(mask.as_element().get_attribute("id"), Some("new-id".into()))
}

/// `set_id` with an invalid id returns `Error::InvalidMaskId` and leaves the cached id unchanged.
#[wasm_bindgen_test]
fn should_reject_invalid_id_on_set_id() -> Result<(), String> {
    let svg = make_svg("mask-set-id-invalid");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let mut mask = defs.mask("still-valid").map_err(|e| e.to_string())?;
    let result = mask.set_id("url(#bad)");
    check(
        matches!(result, Err(Error::InvalidMaskId(_))),
        "expected InvalidMaskId error from set_id with bad id",
    )?;
    check_eq(mask.id(), "still-valid")
}

/// Renaming to a longer id still round-trips correctly (exercises the id `String` growing past whatever
/// capacity the previous id left it with).
#[wasm_bindgen_test]
fn should_set_mask_id_longer_than_previous() -> Result<(), String> {
    let svg = make_svg("mask-set-id-longer");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let mut mask = defs.mask("a").map_err(|e| e.to_string())?;
    mask.set_id("a-much-longer-replacement-id").map_err(|e| e.to_string())?;
    check_eq(mask.id(), "a-much-longer-replacement-id")?;
    check_eq(
        mask.as_element().get_attribute("id"),
        Some("a-much-longer-replacement-id".into()),
    )
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// SvgDefs::build_mask — deferred/transactional variant
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `build_mask` appends the element to `<defs>` only after the closure returns `Ok`.
#[wasm_bindgen_test]
fn should_build_mask_and_append_to_defs() -> Result<(), String> {
    let svg = make_svg("mask-build");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let mask = defs
        .build_mask("built", |m| {
            m.rect(Point::origin(), Size::new(50.0, 50.0))?;
            Ok(())
        })
        .map_err(|e| e.to_string())?;
    let parent = mask.as_element().parent_element().ok_or("mask has no parent")?;
    check_eq(parent.tag_name(), "defs".to_owned())
}

/// When `build_mask`'s closure returns `Err`, the mask element is not attached to `<defs>`.
#[wasm_bindgen_test]
fn should_not_append_mask_on_closure_error() -> Result<(), String> {
    let svg = make_svg("mask-build-err");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let result = defs.build_mask("failing", |_m| Err(Error::Dom("deliberate".into())));
    check(result.is_err(), "expected build_mask to return Err")?;
    // The <defs> element should have no children since the closure failed.
    let defs_el = defs.as_element();
    check_eq(defs_el.child_element_count(), 0)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Mask region attributes
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `set_x` writes the `x` attribute.
#[wasm_bindgen_test]
fn should_set_mask_x() -> Result<(), String> {
    let svg = make_svg("mask-x");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let mask = defs.mask("mx").map_err(|e| e.to_string())?;
    mask.set_x(-0.5).map_err(|e| e.to_string())?;
    check_eq(mask.as_element().get_attribute("x"), Some("-0.5".into()))
}

/// `set_y` writes the `y` attribute.
#[wasm_bindgen_test]
fn should_set_mask_y() -> Result<(), String> {
    let svg = make_svg("mask-y");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let mask = defs.mask("my").map_err(|e| e.to_string())?;
    mask.set_y(-0.5).map_err(|e| e.to_string())?;
    check_eq(mask.as_element().get_attribute("y"), Some("-0.5".into()))
}

/// `set_width` writes the `width` attribute.
#[wasm_bindgen_test]
fn should_set_mask_width() -> Result<(), String> {
    let svg = make_svg("mask-width");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let mask = defs.mask("mw").map_err(|e| e.to_string())?;
    mask.set_width(2.0).map_err(|e| e.to_string())?;
    check_eq(mask.as_element().get_attribute("width"), Some("2".into()))
}

/// `set_height` writes the `height` attribute.
#[wasm_bindgen_test]
fn should_set_mask_height() -> Result<(), String> {
    let svg = make_svg("mask-height");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let mask = defs.mask("mh").map_err(|e| e.to_string())?;
    mask.set_height(2.0).map_err(|e| e.to_string())?;
    check_eq(mask.as_element().get_attribute("height"), Some("2".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Coordinate space and mask-type attributes
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `set_mask_units(UserSpaceOnUse)` writes `maskUnits="userSpaceOnUse"`.
#[wasm_bindgen_test]
fn should_set_mask_units_user_space() -> Result<(), String> {
    let svg = make_svg("mask-units-user");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let mask = defs.mask("mu-user").map_err(|e| e.to_string())?;
    mask.set_mask_units(MaskUnits::UserSpaceOnUse).map_err(|e| e.to_string())?;
    check_eq(mask.as_element().get_attribute("maskUnits"), Some("userSpaceOnUse".into()))
}

/// `set_mask_units(ObjectBoundingBox)` writes `maskUnits="objectBoundingBox"`.
#[wasm_bindgen_test]
fn should_set_mask_units_object_bounding_box() -> Result<(), String> {
    let svg = make_svg("mask-units-obb");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let mask = defs.mask("mu-obb").map_err(|e| e.to_string())?;
    mask.set_mask_units(MaskUnits::ObjectBoundingBox).map_err(|e| e.to_string())?;
    check_eq(mask.as_element().get_attribute("maskUnits"), Some("objectBoundingBox".into()))
}

/// `set_mask_content_units` writes the `maskContentUnits` attribute.
#[wasm_bindgen_test]
fn should_set_mask_content_units() -> Result<(), String> {
    let svg = make_svg("mask-content-units");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let mask = defs.mask("mcu").map_err(|e| e.to_string())?;
    mask.set_mask_content_units(MaskUnits::ObjectBoundingBox)
        .map_err(|e| e.to_string())?;
    check_eq(
        mask.as_element().get_attribute("maskContentUnits"),
        Some("objectBoundingBox".into()),
    )
}

/// `set_mask_type(Luminance)` writes `mask-type="luminance"`.
#[wasm_bindgen_test]
fn should_set_mask_type_luminance() -> Result<(), String> {
    let svg = make_svg("mask-type-luminance");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let mask = defs.mask("mt-lum").map_err(|e| e.to_string())?;
    mask.set_mask_type(MaskType::Luminance).map_err(|e| e.to_string())?;
    check_eq(mask.as_element().get_attribute("mask-type"), Some("luminance".into()))
}

/// `set_mask_type(Alpha)` writes `mask-type="alpha"`.
#[wasm_bindgen_test]
fn should_set_mask_type_alpha() -> Result<(), String> {
    let svg = make_svg("mask-type-alpha");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let mask = defs.mask("mt-alpha").map_err(|e| e.to_string())?;
    mask.set_mask_type(MaskType::Alpha).map_err(|e| e.to_string())?;
    check_eq(mask.as_element().get_attribute("mask-type"), Some("alpha".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Mask shape factories
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `SvgMask::rect` appends a `<rect>` child inside the `<mask>`.
#[wasm_bindgen_test]
fn should_add_rect_shape_to_mask() -> Result<(), String> {
    let svg = make_svg("mask-shape-rect");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let mask = defs.mask("shape-rect").map_err(|e| e.to_string())?;
    mask.rect(Point::origin(), Size::new(10.0, 10.0)).map_err(|e| e.to_string())?;
    check_eq(mask.as_element().child_element_count(), 1)
}

/// `SvgMask::circle` appends a `<circle>` child inside the `<mask>`.
#[wasm_bindgen_test]
fn should_add_circle_shape_to_mask() -> Result<(), String> {
    let svg = make_svg("mask-shape-circle");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let mask = defs.mask("shape-circle").map_err(|e| e.to_string())?;
    mask.circle(Point::new(5.0, 5.0), 5.0).map_err(|e| e.to_string())?;
    check_eq(mask.as_element().child_element_count(), 1)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// SvgNode::set_mask / set_mask_ref / remove_mask — applying a mask to a referencing element
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `set_mask` writes `mask="url(#id)"`.
#[wasm_bindgen_test]
fn should_set_mask_attribute() -> Result<(), String> {
    let svg = make_svg("mask-apply");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    defs.mask("my-fade").map_err(|e| e.to_string())?;
    let rect = svg.rect(Point::origin(), Size::new(100.0, 100.0)).map_err(|e| e.to_string())?;
    rect.set_mask("my-fade").map_err(|e| e.to_string())?;
    check_eq(rect.as_element().get_attribute("mask"), Some("url(#my-fade)".into()))
}

/// `set_mask_ref` produces the same result as `set_mask` using the id from the handle.
#[wasm_bindgen_test]
fn should_set_mask_ref() -> Result<(), String> {
    let svg = make_svg("mask-apply-ref");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let mask = defs.mask("ref-fade").map_err(|e| e.to_string())?;
    let rect = svg.rect(Point::origin(), Size::new(100.0, 100.0)).map_err(|e| e.to_string())?;
    rect.set_mask_ref(&mask).map_err(|e| e.to_string())?;
    check_eq(rect.as_element().get_attribute("mask"), Some("url(#ref-fade)".into()))
}

/// `set_mask` with an invalid id returns `Error::InvalidMaskId`.
#[wasm_bindgen_test]
fn should_reject_invalid_set_mask_id() -> Result<(), String> {
    let svg = make_svg("mask-invalid-apply");
    let rect = svg.rect(Point::origin(), Size::new(100.0, 100.0)).map_err(|e| e.to_string())?;
    let result = rect.set_mask("url(#bad)");
    check(
        matches!(result, Err(Error::InvalidMaskId(_))),
        "expected InvalidMaskId from set_mask with bad id",
    )
}

/// `remove_mask` removes the `mask` attribute.
#[wasm_bindgen_test]
fn should_remove_mask_attribute() -> Result<(), String> {
    let svg = make_svg("mask-remove");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let mask = defs.mask("removable").map_err(|e| e.to_string())?;
    let rect = svg.rect(Point::origin(), Size::new(100.0, 100.0)).map_err(|e| e.to_string())?;
    rect.set_mask_ref(&mask).map_err(|e| e.to_string())?;
    rect.remove_mask().map_err(|e| e.to_string())?;
    check_eq(rect.as_element().get_attribute("mask"), None)
}

/// End-to-end cached-reference invariant: renaming a mask via `set_id` and then applying it with `set_mask_ref`
/// writes the *new* id, not the one the mask was originally constructed with — proving `url_ref` is kept in sync
/// rather than snapshotted once at construction.
#[wasm_bindgen_test]
fn should_reflect_renamed_mask_id_in_set_mask_ref() -> Result<(), String> {
    let svg = make_svg("mask-rename-then-ref");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let mut mask = defs.mask("old").map_err(|e| e.to_string())?;
    mask.set_id("new").map_err(|e| e.to_string())?;
    let rect = svg.rect(Point::origin(), Size::new(50.0, 50.0)).map_err(|e| e.to_string())?;
    rect.set_mask_ref(&mask).map_err(|e| e.to_string())?;
    check_eq(rect.as_element().get_attribute("mask"), Some("url(#new)".into()))
}

/// `set_attr("ID", ...)` (case-insensitive) and `set_attr_display("id", ...)` are both rejected to protect the
/// cached id, and neither one is allowed to desynchronise the cache from the DOM before returning the error.
#[wasm_bindgen_test]
fn should_reject_reserved_id_attribute_case_insensitively_and_leave_cache_unchanged() -> Result<(), String> {
    let svg = make_svg("mask-reserved-id");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let mask = defs.mask("safe").map_err(|e| e.to_string())?;

    let result = mask.set_attr("ID", "injected");
    check(
        matches!(result, Err(Error::ReservedAttribute("id"))),
        "expected ReservedAttribute error for set_attr(\"ID\", ...)",
    )?;

    let result = mask.set_attr_display("id", "injected");
    check(
        matches!(result, Err(Error::ReservedAttribute("id"))),
        "expected ReservedAttribute error for set_attr_display(\"id\", ...)",
    )?;

    check_eq(mask.id(), "safe")?;
    check_eq(mask.as_element().get_attribute("id"), Some("safe".into()))
}
