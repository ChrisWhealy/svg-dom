mod common;

use common::*;
use svg_dom::Error;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

// This file covers construction and id handling only — the part of SvgClipPath touched by the
// cached-id-allocation-reuse change in set_id. Broader coverage (shape factories, set_units) is a separate,
// pre-existing gap, not attempted here.

/// `defs.clip_path(id)` creates an element with tag name `"clipPath"`.
#[wasm_bindgen_test]
fn should_create_clip_path_element() -> Result<(), String> {
    let svg = make_svg("cp-create");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let clip = defs.clip_path("cp").map_err(|e| e.to_string())?;
    check_eq(clip.as_element().tag_name(), "clipPath".to_owned())
}

/// The `<clipPath>` element is a child of `<defs>`, not of the root `<svg>`.
#[wasm_bindgen_test]
fn should_append_clip_path_to_defs() -> Result<(), String> {
    let svg = make_svg("cp-parent");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let clip = defs.clip_path("cp-p").map_err(|e| e.to_string())?;
    let parent = clip.as_element().parent_element().ok_or("clipPath has no parent")?;
    check_eq(parent.tag_name(), "defs".to_owned())
}

/// `SvgClipPath::id()` returns the cached id without a DOM round-trip.
#[wasm_bindgen_test]
fn should_return_cached_clip_path_id() -> Result<(), String> {
    let svg = make_svg("cp-cached-id");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let clip = defs.clip_path("cached").map_err(|e| e.to_string())?;
    check_eq(clip.id(), "cached")
}

/// An invalid id is rejected with `Error::InvalidClipPathId` before any DOM call.
#[wasm_bindgen_test]
fn should_reject_invalid_clip_path_id() -> Result<(), String> {
    let svg = make_svg("cp-invalid-id");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let result = defs.clip_path("url(#bad)");
    check(
        matches!(result, Err(Error::InvalidClipPathId(_))),
        "expected InvalidClipPathId error for an id containing '(' and ')'",
    )
}

/// `set_id` updates both the DOM attribute and the cached value.
#[wasm_bindgen_test]
fn should_set_clip_path_id_via_set_id() -> Result<(), String> {
    let svg = make_svg("cp-set-id");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let mut clip = defs.clip_path("old-id").map_err(|e| e.to_string())?;
    clip.set_id("new-id").map_err(|e| e.to_string())?;
    check_eq(clip.id(), "new-id")?;
    check_eq(clip.as_element().get_attribute("id"), Some("new-id".into()))
}

/// `set_id` with an invalid id returns `Error::InvalidClipPathId` and leaves the cached id unchanged.
#[wasm_bindgen_test]
fn should_reject_invalid_id_on_set_id() -> Result<(), String> {
    let svg = make_svg("cp-set-id-invalid");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let mut clip = defs.clip_path("still-valid").map_err(|e| e.to_string())?;
    let result = clip.set_id("url(#bad)");
    check(
        matches!(result, Err(Error::InvalidClipPathId(_))),
        "expected InvalidClipPathId error from set_id with bad id",
    )?;
    check_eq(clip.id(), "still-valid")
}

/// Renaming to a longer id still round-trips correctly (exercises the id `String` growing past whatever
/// capacity the previous id left it with).
#[wasm_bindgen_test]
fn should_set_clip_path_id_longer_than_previous() -> Result<(), String> {
    let svg = make_svg("cp-set-id-longer");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let mut clip = defs.clip_path("a").map_err(|e| e.to_string())?;
    clip.set_id("a-much-longer-replacement-id").map_err(|e| e.to_string())?;
    check_eq(clip.id(), "a-much-longer-replacement-id")?;
    check_eq(
        clip.as_element().get_attribute("id"),
        Some("a-much-longer-replacement-id".into()),
    )
}
