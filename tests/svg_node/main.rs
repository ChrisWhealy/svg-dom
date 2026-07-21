//! `SvgNode` integration tests.
//!
//! Promoted from a single `tests/svg_node.rs` file to this folder once it passed 2000 lines, split into topic files
//! named after — and matching the scope of — their corresponding `src/node/*.rs` module, the same categorisation
//! approach `docs/design_notes/` already uses (one file per concern, indexed here instead of in a `README.md`):
//!
//! - [`attrs`] — `set_attr`/`attr`/`remove_attr`/`set_attrs`/`set_attr_if_changed`/`set_attr_display`, the
//!   `add_class`/`remove_class`/`toggle_class`/`has_class`/`set_class_enabled` classList surface, and the
//!   convenience setters (`set_fill`, `set_stroke`, `set_stroke_width`, `set_d`) — mirrors `src/node/attrs.rs`.
//! - [`cached`] — `CachedAttr`'s redundant-write elision for both attributes and text — mirrors `src/node/cached.rs`.
//! - [`text`] — `set_text_fmt`/`set_text_display` — mirrors the relevant part of `src/node/text.rs`.
//! - [`transform`] — `set_translate`/`set_rotate`/`set_rotate_about`/`set_scale`/`set_scale_xy`/
//!   `set_translate_scale`/`set_matrix`/`set_matrix_precise`/`set_transform_fmt` — mirrors `src/node/transform.rs`.
//! - [`tree`] — clone semantics, `append`/`remove`/`insert_before`/`clear`/`replace_with`, `parent`/`first_child`/
//!   `last_child`/`next_sibling`/`previous_sibling`/`children`, `query_selector`/`query_selector_all`, and non-SVG
//!   (`<foreignObject>`) content handling — mirrors `src/node/tree.rs`.
//! - [`events`] — every managed event wrapper, one-shot variants, multiple handlers per node, listener lifetime
//!   through clones, explicit `clear_listeners`/`remove_listeners`, and the self-referential-listener
//!   strong-cycle-vs-`WeakSvgNode` case — mirrors `src/node/event.rs` and `src/node/listeners/`.
//! - [`attr_writer`] — the generic `SvgAttrs`/`AttrWriter` scratch-buffer API — mirrors `src/root/attrs/mod.rs`.
//! - [`geometry`] — `bounding_box`/`total_length`/`point_at_length`/`ctm`/`screen_ctm`/`bounding_client_rect` —
//!   mirrors `src/node/geometry.rs`.
//!
//! [`helpers`] holds the two pieces of test-local setup every other file needs: a `make_svg` that sizes the canvas
//! `200×200` (distinct from `common::make_svg`'s `400×300`) and the synthetic-event `dispatch`/`dispatch_element`
//! pair `events.rs` uses to fire every event type under test.
//!
//! `tests/common.rs` is one directory up (shared with every other integration test file), hence the `#[path]`
//! override below rather than a plain `mod common;`.

#[path = "../common.rs"]
mod common;

mod helpers;

mod attr_writer;
mod attrs;
mod cached;
mod events;
mod geometry;
mod text;
mod transform;
mod tree;

use wasm_bindgen_test::wasm_bindgen_test_configure;

wasm_bindgen_test_configure!(run_in_browser);
