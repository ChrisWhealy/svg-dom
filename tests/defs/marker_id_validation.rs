use crate::common::{self, *};
use svg_dom::{Error, root::utils::Point};
use wasm_bindgen_test::*;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Marker ID validation
//
// SvgMarker::set_id — cache-aware rename
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `set_id` updates both the DOM `id` attribute and the cached value returned by `id()`.
#[wasm_bindgen_test]
fn should_update_cache_and_dom_on_set_id() -> Result<(), String> {
    let svg = make_svg("set-id-ok");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let mut marker = defs.marker("original").map_err(|e| e.to_string())?;
    marker.set_id("renamed").map_err(|e| e.to_string())?;
    common::check_eq(marker.id(), "renamed")?;
    common::check_eq(marker.as_element().get_attribute("id"), Some("renamed".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Renaming to a longer id still round-trips correctly (exercises the cached `url(#id)` reference `String`
/// growing past whatever capacity the previous id left it with).
#[wasm_bindgen_test]
fn should_set_marker_id_longer_than_previous() -> Result<(), String> {
    let svg = make_svg("set-id-longer");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let mut marker = defs.marker("a").map_err(|e| e.to_string())?;
    marker.set_id("a-much-longer-replacement-id").map_err(|e| e.to_string())?;
    common::check_eq(marker.id(), "a-much-longer-replacement-id")?;
    common::check_eq(
        marker.as_element().get_attribute("id"),
        Some("a-much-longer-replacement-id".into()),
    )
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `set_id` with an invalid id returns `InvalidMarkerId` and leaves the marker unchanged.
#[wasm_bindgen_test]
fn should_reject_invalid_id_on_set_id() -> Result<(), String> {
    let svg = make_svg("set-id-invalid");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let mut marker = defs.marker("good-id").map_err(|e| e.to_string())?;
    let result = marker.set_id("url(#bad)");
    common::check(
        matches!(result, Err(Error::InvalidMarkerId(_))),
        "invalid id must return InvalidMarkerId",
    )?;
    // Cache and DOM must be unchanged.
    common::check_eq(marker.id(), "good-id")?;
    common::check_eq(marker.as_element().get_attribute("id"), Some("good-id".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `set_id` with an empty string returns `InvalidMarkerId`.
#[wasm_bindgen_test]
fn should_reject_empty_id_on_set_id() -> Result<(), String> {
    let svg = make_svg("set-id-empty");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let mut marker = defs.marker("initial").map_err(|e| e.to_string())?;
    let result = marker.set_id("");
    common::check(
        matches!(result, Err(Error::InvalidMarkerId(_))),
        "empty id must return InvalidMarkerId",
    )
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Renaming a marker after a reference has been applied leaves the existing attribute pointing at the old id.
/// `set_id` updates the marker's DOM id but cannot retroactively update string snapshots written to other elements.
#[wasm_bindgen_test]
fn should_leave_stale_reference_after_set_id() -> Result<(), String> {
    let svg = make_svg("set-id-stale-ref");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let mut marker = defs.marker("before").map_err(|e| e.to_string())?;
    let line = svg
        .line(Point::new(0.0, 0.0), Point::new(100.0, 0.0))
        .map_err(|e| e.to_string())?;
    line.set_marker_end_ref(&marker).map_err(|e| e.to_string())?;

    // Sanity check: reference points at the original id.
    common::check_eq(line.as_element().get_attribute("marker-end"), Some("url(#before)".into()))?;

    // Rename the marker.
    marker.set_id("after").map_err(|e| e.to_string())?;

    // The marker's own id is updated...
    common::check_eq(marker.as_element().get_attribute("id"), Some("after".into()))?;
    // ...but the line's attribute still holds the old snapshot.
    common::check_eq(line.as_element().get_attribute("marker-end"), Some("url(#before)".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// SvgMarker::set_attr / set_attrs / set_attr_display — "id" is reserved
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `set_attr("id", ...)` returns `ReservedAttribute` and does not write to the DOM.
#[wasm_bindgen_test]
fn should_reject_id_in_set_attr() -> Result<(), String> {
    let svg = make_svg("reserved-set-attr");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("original").map_err(|e| e.to_string())?;
    common::check(
        matches!(marker.set_attr("id", "new-id"), Err(Error::ReservedAttribute(_))),
        "set_attr(\"id\") must return ReservedAttribute",
    )?;
    // Cached id and DOM attribute must still reflect the original value.
    common::check_eq(marker.id(), "original")?;
    common::check_eq(marker.as_element().get_attribute("id"), Some("original".into()))
}

/// `set_attr("ID", ...)` is also rejected (case-insensitive guard).
#[wasm_bindgen_test]
fn should_reject_id_case_insensitive_in_set_attr() -> Result<(), String> {
    let svg = make_svg("reserved-set-attr-case");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("original2").map_err(|e| e.to_string())?;
    common::check(
        matches!(marker.set_attr("ID", "new-id"), Err(Error::ReservedAttribute(_))),
        "set_attr(\"ID\") must also return ReservedAttribute",
    )
}

/// `set_attrs` propagates the `ReservedAttribute` error when `"id"` appears in the iterator.
#[wasm_bindgen_test]
fn should_reject_id_in_set_attrs() -> Result<(), String> {
    let svg = make_svg("reserved-set-attrs");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("original3").map_err(|e| e.to_string())?;
    common::check(
        matches!(
            marker.set_attrs([("viewBox", "0 0 10 7"), ("id", "hijack")]),
            Err(Error::ReservedAttribute(_))
        ),
        "set_attrs must propagate ReservedAttribute for \"id\"",
    )
}

/// `set_attr_display("id", ...)` returns `ReservedAttribute`.
#[wasm_bindgen_test]
fn should_reject_id_in_set_attr_display() -> Result<(), String> {
    let svg = make_svg("reserved-set-attr-display");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs.marker("original4").map_err(|e| e.to_string())?;
    common::check(
        matches!(marker.set_attr_display("id", "new-id"), Err(Error::ReservedAttribute(_))),
        "set_attr_display(\"id\") must return ReservedAttribute",
    )
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Empty string is rejected as a marker id.
#[wasm_bindgen_test]
fn should_reject_empty_marker_id() -> Result<(), String> {
    let svg = make_svg("val-empty");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    common::check(is_invalid_marker_id(defs.marker("")), "empty id must be rejected")
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Marker id containing a space is rejected.
#[wasm_bindgen_test]
fn should_reject_whitespace_in_marker_id() -> Result<(), String> {
    let svg = make_svg("val-space");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    common::check(is_invalid_marker_id(defs.marker("arrow head")), "space in id must be rejected")?;
    common::check(is_invalid_marker_id(defs.marker("tab\there")), "tab in id must be rejected")
}

/// Marker id containing `#` is rejected.
#[wasm_bindgen_test]
fn should_reject_hash_in_marker_id() -> Result<(), String> {
    let svg = make_svg("val-hash");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    common::check(is_invalid_marker_id(defs.marker("#arrow")), "# in id must be rejected")
}

/// Marker id containing `)` is rejected.
#[wasm_bindgen_test]
fn should_reject_paren_in_marker_id() -> Result<(), String> {
    let svg = make_svg("val-paren");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    common::check(is_invalid_marker_id(defs.marker("arrow)")), ") in id must be rejected")
}

/// Marker id containing `(` is rejected.
#[wasm_bindgen_test]
fn should_reject_open_paren_in_marker_id() -> Result<(), String> {
    let svg = make_svg("val-open-paren");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    common::check(is_invalid_marker_id(defs.marker("arrow(left")), "( in id must be rejected")
}

/// Marker id starting with `url(` is rejected.
#[wasm_bindgen_test]
fn should_reject_url_prefix_in_marker_id() -> Result<(), String> {
    let svg = make_svg("val-url");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    common::check(
        is_invalid_marker_id(defs.marker("url(#arrow)")),
        "id starting with url( must be rejected",
    )
}

/// Characters that are valid in some XML ids but unsafe in a `url(#...)` reference are rejected.
///
/// The allow-list `[A-Za-z_][A-Za-z0-9_-]*` excludes quotes, backslash, semicolon, and control characters.
#[wasm_bindgen_test]
fn should_reject_url_unsafe_chars_in_marker_id() -> Result<(), String> {
    let svg = make_svg("val-unsafe");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    common::check(
        is_invalid_marker_id(defs.marker("arrow\"head")),
        r#"double-quote in id must be rejected"#,
    )?;
    common::check(
        is_invalid_marker_id(defs.marker("arrow'head")),
        "single-quote in id must be rejected",
    )?;
    common::check(
        is_invalid_marker_id(defs.marker("arrow\\head")),
        "backslash in id must be rejected",
    )?;
    common::check(
        is_invalid_marker_id(defs.marker("arrow;head")),
        "semicolon in id must be rejected",
    )?;
    common::check(
        is_invalid_marker_id(defs.marker("arrow\nhead")),
        "newline in id must be rejected",
    )
}

/// An id starting with a digit is rejected (allow-list requires a letter or underscore first).
#[wasm_bindgen_test]
fn should_reject_digit_start_in_marker_id() -> Result<(), String> {
    let svg = make_svg("val-digit-start");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    common::check(is_invalid_marker_id(defs.marker("2arrow")), "digit-first id must be rejected")
}

/// `build_marker` applies the same validation as `marker`.
#[wasm_bindgen_test]
fn should_reject_invalid_id_in_build_marker() -> Result<(), String> {
    let svg = make_svg("val-build");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let result = defs.build_marker("url(#x)", |_| Ok(()));
    common::check(is_invalid_marker_id(result), "build_marker must reject url( prefix")
}

/// A plain alphanumeric id with hyphens is accepted.
#[wasm_bindgen_test]
fn should_accept_valid_marker_id() -> Result<(), String> {
    let svg = make_svg("val-ok");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    defs.marker("arrow-head-2").map_err(|e| e.to_string())?;
    Ok(())
}

/// An id starting with an underscore is accepted.
#[wasm_bindgen_test]
fn should_accept_underscore_start_in_marker_id() -> Result<(), String> {
    let svg = make_svg("val-ok-underscore");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    defs.marker("_arrow").map_err(|e| e.to_string())?;
    Ok(())
}
