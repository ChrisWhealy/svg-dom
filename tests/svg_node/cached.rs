use crate::{common, helpers::make_svg};
use svg_dom::root::utils::{Point, Size};
use wasm_bindgen_test::*;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// CachedAttr
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// The first `CachedAttr::set` writes, since there is no remembered value to compare against.
#[wasm_bindgen_test]
fn should_write_first_cached_value() -> Result<(), String> {
    let rect = make_svg("node-cached-attr-first")
        .rect(Point::origin(), Size::new(50.0, 50.0))
        .map_err(|e| e.to_string())?;
    let mut cache = svg_dom::CachedAttr::new();
    cache.set(&rect, "style", "cursor:grab").map_err(|e| e.to_string())?;
    common::check_eq(rect.attr("style"), Some("cursor:grab".into()))
}

/// When the cached value is unchanged, `set` does not touch the DOM. We prove this by mutating the attribute behind the
/// cache's back: a cached no-op must leave that external value in place rather than rewriting the cached one.
#[wasm_bindgen_test]
fn should_skip_dom_write_when_cached_value_unchanged() -> Result<(), String> {
    let rect = make_svg("node-cached-attr-noop")
        .rect(Point::origin(), Size::new(50.0, 50.0))
        .map_err(|e| e.to_string())?;
    let mut cache = svg_dom::CachedAttr::new();
    cache.set(&rect, "style", "cursor:grab").map_err(|e| e.to_string())?;

    // Change the attribute through a different path; the cache still believes "cursor:grab" is current.
    rect.set_attr("style", "cursor:wait").map_err(|e| e.to_string())?;

    // Same value as cached → no write, so the external "cursor:wait" survives.
    cache.set(&rect, "style", "cursor:grab").map_err(|e| e.to_string())?;
    common::check_eq(rect.attr("style"), Some("cursor:wait".into()))
}

/// A changed value writes through and updates the cache.
#[wasm_bindgen_test]
fn should_write_changed_cached_value() -> Result<(), String> {
    let rect = make_svg("node-cached-attr-change")
        .rect(Point::origin(), Size::new(50.0, 50.0))
        .map_err(|e| e.to_string())?;
    let mut cache = svg_dom::CachedAttr::new();
    cache.set(&rect, "style", "cursor:grab").map_err(|e| e.to_string())?;
    cache.set(&rect, "style", "cursor:grabbing").map_err(|e| e.to_string())?;
    common::check_eq(rect.attr("style"), Some("cursor:grabbing".into()))
}

/// After `invalidate`, the next `set` writes even if the value matches what was last written.
#[wasm_bindgen_test]
fn should_write_after_invalidate() -> Result<(), String> {
    let rect = make_svg("node-cached-attr-invalidate")
        .rect(Point::origin(), Size::new(50.0, 50.0))
        .map_err(|e| e.to_string())?;
    let mut cache = svg_dom::CachedAttr::new();
    cache.set(&rect, "style", "cursor:grab").map_err(|e| e.to_string())?;

    rect.set_attr("style", "cursor:wait").map_err(|e| e.to_string())?;
    cache.invalidate();

    // Cache was invalidated, so this writes through and restores "cursor:grab".
    cache.set(&rect, "style", "cursor:grab").map_err(|e| e.to_string())?;
    common::check_eq(rect.attr("style"), Some("cursor:grab".into()))
}

/// `CachedAttr::set_text` writes the first value to the element's text content.
#[wasm_bindgen_test]
fn should_write_first_cached_text() -> Result<(), String> {
    let label = make_svg("node-cached-text-first")
        .text(Point::new(10.0, 20.0), "")
        .map_err(|e| e.to_string())?;
    let mut cache = svg_dom::CachedAttr::new();
    cache.set_text(&label, "moving").map_err(|e| e.to_string())?;
    common::check_eq(label.as_element().text_content(), Some("moving".into()))
}

/// When the cached text is unchanged, `set_text` does not touch the DOM (proved by a behind-the-cache mutation surviving).
#[wasm_bindgen_test]
fn should_skip_text_write_when_unchanged() -> Result<(), String> {
    let label = make_svg("node-cached-text-noop")
        .text(Point::new(10.0, 20.0), "")
        .map_err(|e| e.to_string())?;
    let mut cache = svg_dom::CachedAttr::new();
    cache.set_text(&label, "moving").map_err(|e| e.to_string())?;

    label.set_text("dropped"); // change behind the cache's back
    cache.set_text(&label, "moving").map_err(|e| e.to_string())?; // same as cached → no write

    common::check_eq(label.as_element().text_content(), Some("dropped".into()))
}

/// A changed value writes through and updates the text cache.
#[wasm_bindgen_test]
fn should_write_changed_cached_text() -> Result<(), String> {
    let label = make_svg("node-cached-text-change")
        .text(Point::new(10.0, 20.0), "")
        .map_err(|e| e.to_string())?;
    let mut cache = svg_dom::CachedAttr::new();
    cache.set_text(&label, "moving").map_err(|e| e.to_string())?;
    cache.set_text(&label, "dropped").map_err(|e| e.to_string())?;
    common::check_eq(label.as_element().text_content(), Some("dropped".into()))
}

/// `CachedAttr::set_text_fmt` formats through a caller-owned scratch buffer, then caches: an unchanged formatted value
/// skips the DOM write (no allocation either), and a changed one writes through.
#[wasm_bindgen_test]
fn should_cache_formatted_text_via_set_text_fmt() -> Result<(), String> {
    let label = make_svg("node-cached-text-fmt")
        .text(Point::new(10.0, 20.0), "")
        .map_err(|e| e.to_string())?;
    let mut cache = svg_dom::CachedAttr::new();
    let mut scratch = String::new();

    cache
        .set_text_fmt(&label, &mut scratch, format_args!("row: {}", 5))
        .map_err(|e| e.to_string())?;
    common::check_eq(label.as_element().text_content(), Some("row: 5".into()))?;

    // Same formatted value → cache skips the write (proved by a behind-the-cache change surviving).
    label.set_text("changed");
    cache
        .set_text_fmt(&label, &mut scratch, format_args!("row: {}", 5))
        .map_err(|e| e.to_string())?;
    common::check_eq(label.as_element().text_content(), Some("changed".into()))?;

    // A different formatted value writes through.
    cache
        .set_text_fmt(&label, &mut scratch, format_args!("row: {}", 6))
        .map_err(|e| e.to_string())?;
    common::check_eq(label.as_element().text_content(), Some("row: 6".into()))
}

/// `CachedAttr::set_fmt` does the same for an attribute: formats through the scratch buffer, then elides the redundant
/// write when the formatted value repeats.
#[wasm_bindgen_test]
fn should_cache_formatted_attribute_via_set_fmt() -> Result<(), String> {
    let rect = make_svg("node-cached-attr-fmt")
        .rect(Point::origin(), Size::new(50.0, 50.0))
        .map_err(|e| e.to_string())?;
    let mut cache = svg_dom::CachedAttr::new();
    let mut scratch = String::new();

    cache
        .set_fmt(&rect, "opacity", &mut scratch, format_args!("{:.1}", 0.5))
        .map_err(|e| e.to_string())?;
    common::check_eq(rect.attr("opacity"), Some("0.5".into()))?;

    // Same value → skipped (a behind-the-cache change survives).
    rect.set_attr("opacity", "0.9").map_err(|e| e.to_string())?;
    cache
        .set_fmt(&rect, "opacity", &mut scratch, format_args!("{:.1}", 0.5))
        .map_err(|e| e.to_string())?;
    common::check_eq(rect.attr("opacity"), Some("0.9".into()))
}
