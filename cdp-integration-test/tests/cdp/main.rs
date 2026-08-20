//! Chrome-DevTools-Protocol (CDP) integration tests for `svg-dom`, in one Cargo test binary.
//!
//! This binary hosts five scenario modules, each verifying browser-computed behaviour that plain DOM inspection
//! (and therefore `wasm-bindgen-test`) cannot see:
//!
//! - [`accessibility_tree`] — accessible-name/description computation, via the Accessibility CDP domain.
//! - [`filter_blend_render`] — `SvgFilter::blend`'s alpha-preserving tint chain, via actual rendered pixels.
//! - [`turbulence_scale_zero_render`] — `SvgFilter::displacement_map`'s `scale` argument at `0.0`, via actual
//!   rendered pixels.
//! - [`lighting_render`] — `demo_lighting.rs`'s own surfaceScale and azimuth sliders, via actual rendered pixels.
//! - [`light_sources_render`] — `demo_light_sources.rs`'s own four sliders, via actual rendered pixels.
//!
//! All five drive the same, shared Chrome instance against the same sibling `cdp-test-fixture` wasm crate (built
//! once, served locally) — see [`common`] for the shared setup and why it now lives in one place.
//! Each module still opens its own tab, so a bug in one scenario's own page state cannot leak into another's.
//!
//! # Why one binary instead of five
//!
//! Every `tests/*.rs` file compiles as its own separate binary with its own process.
//! Before this consolidation, each of the five scenarios above was its own such file: each one built the
//! `cdp-test-fixture` wasm package, started its own static server, and launched its own Chrome instance
//! independently, even though four of the five only ever needed one tab for one `#[test]`.
//! `build_fixture`'s own cross-process lock kept concurrent `wasm-pack build` invocations from corrupting each
//! other, but did nothing to avoid paying for five separate builds and five separate Chrome startups in the first
//! place.
//!
//! That was fairly inconsequential with two or three scenarios.
//! Five scenarios, including the lighting tests that already called out repeated Chrome startup cost in their own
//! module doc comments, made the repeated cost worth removing.
//! `tests/svg_node/`, `tests/defs/`, and `tests/filter/` in the main `svg-dom` crate already establish the
//! pattern this binary now follows: one Cargo-discovered `tests/<name>/main.rs` binary, with one concern per
//! sibling module, indexed here instead of in a `README.md`.
//! [`common::new_tab`](common) is this binary's equivalent of those suites' own `tests/common.rs`.
//!
//! # Why this lives in its own on-demand workspace member
//!
//! This binary pulls in `headless_chrome` and needs a local Chrome/Chromium binary, neither of which the ordinary
//! `cargo test`/`cargo nextest run` workflow should have to pay for.
//! `cdp-integration-test` therefore lives in its own workspace member, excluded from the root package's
//! `default-members`, the same as `demo-server`.
//! Run explicitly with `cargo test -p cdp-integration-test`.
//!
//! # Running in CI
//!
//! Runs as its own job (`cdp-integration-test`) in `.github/workflows/ci.yml`, on every push/PR, using the Chrome
//! installation already present on GitHub's `ubuntu-latest` runner image.
//! Being a separate job means its failure is reported independently, without obscuring or being obscured by the
//! unrelated `wasm-bindgen-test` results, while still gating the merge like any other required check.
//!
//! # Why the browser is launched with `sandbox(false)`
//!
//! `Browser::default()` launches with Chrome's own sandbox enabled, the right default for browsing untrusted
//! content.
//! `ubuntu-latest` now resolves to Ubuntu 24.04+, which restricts unprivileged user namespaces via AppArmor.
//! That breaks Chrome's own sandbox initialisation, even for the runner's non-root user, unless `--no-sandbox` is
//! passed.
//! Since every scenario here only ever loads a local fixture page this crate builds itself, there is no untrusted
//! content for the sandbox to matter for.
//! [`launch_browser`](cdp_integration_test::launch_browser) therefore disables it unconditionally, not just in
//! CI, so local and CI runs stay on the same code path.

mod common;

mod accessibility_tree;
mod filter_blend_render;
mod light_sources_render;
mod lighting_render;
mod turbulence_scale_zero_render;
