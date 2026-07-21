//! `SvgDefs`/`SvgMarker` integration tests.
//!
//! Promoted from a single `tests/defs.rs` file to this folder once it passed 1000 lines, split into topic files the
//! same way `docs/design_notes/` is split — one file per concern, indexed here instead of in a `README.md`.
//!
//! - [`svg_defs`] — `<defs>` construction, every shape factory inside it, and its generic attribute surface.
//! - [`marker_construction`] — `<marker>` construction, its own attribute setters (`refX`/`refY`/`markerWidth`/...),
//!   `set_view_box`, and its generic attribute surface.
//! - [`marker_children`] — every shape factory available inside a `<marker>`.
//! - [`marker_refs`] — `SvgNode`'s `set_marker_start`/`mid`/`end` (and `_ref` variants), their id validation, and
//!   the full arrowhead assembly integration test.
//! - [`deferred_append`] — `build_defs`/`build_marker`'s deferred-append-on-success behaviour, including the
//!   nested case.
//! - [`marker_id_validation`] — `SvgMarker::set_id`'s cache-aware rename, `set_attr("id", ...)` rejection, and the
//!   marker-id character/format allow-list.
//!
//! `tests/common.rs` is one directory up (shared with every other integration test file), hence the `#[path]`
//! override below rather than a plain `mod common;`.

#[path = "../common.rs"]
mod common;

mod deferred_append;
mod marker_children;
mod marker_construction;
mod marker_id_validation;
mod marker_refs;
mod svg_defs;

use wasm_bindgen_test::wasm_bindgen_test_configure;

wasm_bindgen_test_configure!(run_in_browser);
