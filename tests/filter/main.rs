//! `SvgFilter` integration tests.
//!
//! Promoted from a single `tests/filter.rs` file to this folder once it passed 1100 lines, split into topic files
//! named after — and matching the scope of — their corresponding `src/root/filter/*.rs` module, the same
//! categorisation approach `tests/svg_node/` already uses (one file per concern, indexed here instead of in a
//! `README.md`):
//!
//! - [`construction`] — `SvgDefs::filter`/`build_filter`, the cached id (`id`/`set_id`), and `set_attr("id", ...)`
//!   rejection — mirrors `src/root/filter/mod.rs` and the id-guard part of `src/root/filter/attrs.rs`.
//! - [`apply`] — `SvgNode::set_filter`/`set_filter_ref`/`remove_filter`.
//! - [`region`] — `set_x`/`set_y`/`set_width`/`set_height`/`set_filter_units`/`set_primitive_units`, and the
//!   generic escape hatch for filter-region attributes — mirrors `src/root/filter/region.rs`.
//! - [`gaussian_blur`] — `gaussian_blur`/`gaussian_blur_xy` — mirrors `src/root/filter/primitives/gaussian_blur.rs`.
//! - [`offset`] — `offset` — mirrors `src/root/filter/primitives/offset.rs`.
//! - [`merge`] — `merge` — mirrors `src/root/filter/primitives/merge.rs`.
//! - [`flood`] — `flood` — mirrors `src/root/filter/primitives/flood.rs`.
//! - [`composite`] — `composite` and every `CompositeOperator` keyword — mirrors
//!   `src/root/filter/primitives/composite.rs`.
//! - [`blend`] — `blend` and every `BlendMode` keyword — mirrors `src/root/filter/primitives/blend.rs`.
//! - [`drop_shadow`] — `drop_shadow` — mirrors `src/root/filter/primitives/drop_shadow.rs`.
//! - [`color_matrix`] — `color_matrix` and every `ColorMatrixType` variant — mirrors
//!   `src/root/filter/primitives/color_matrix.rs`.
//! - [`component_transfer`] — `component_transfer`, every `TransferFunction` variant, the `Table`/`Discrete`
//!   single-value edge cases, and duplicate-`Channel` last-wins ordering — mirrors
//!   `src/root/filter/primitives/component_transfer.rs`.
//! - [`turbulence`] — `turbulence`/`turbulence_xy` and every `TurbulenceType` keyword — mirrors
//!   `src/root/filter/primitives/turbulence.rs`.
//! - [`displacement_map`] — `displacement_map` and every `Channel` selector keyword — mirrors
//!   `src/root/filter/primitives/displacement_map.rs`.
//! - [`chains`] — cross-primitive integration tests (the manual blur+offset+merge and blur+flood+composite+
//!   offset+merge drop-shadow chains, the flood+blend+composite tint chain, and the turbulence+displacement_map
//!   noise-distortion chain) — these compose several primitives together, so they don't belong to any single
//!   primitive's own file.
//!
//! `tests/common.rs` is one directory up (shared with every other integration test file), hence the `#[path]`
//! override below rather than a plain `mod common;`.

#[path = "../common.rs"]
mod common;

mod apply;
mod blend;
mod chains;
mod color_matrix;
mod component_transfer;
mod composite;
mod construction;
mod displacement_map;
mod drop_shadow;
mod flood;
mod gaussian_blur;
mod merge;
mod offset;
mod region;
mod turbulence;

use wasm_bindgen_test::wasm_bindgen_test_configure;

wasm_bindgen_test_configure!(run_in_browser);
