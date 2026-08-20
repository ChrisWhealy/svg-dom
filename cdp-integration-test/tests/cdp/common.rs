//! Shared browser lifecycle for every scenario module in this binary.
//!
//! Builds the `cdp-test-fixture` wasm package, starts its static server, and launches Chrome exactly once per test
//! run, via a lazily-initialised `OnceLock`.
//! Every scenario module used to pay this cost independently, as its own separate `tests/*.rs` binary.
//! Consolidating them into modules of one binary, sharing this setup, is the same organisational move
//! `tests/filter/`, `tests/svg_node/`, and `tests/defs/` already use in the main `svg-dom` crate.
//!
//! Each module still opens its own [`Tab`] via [`new_tab`], for test isolation.
//! Only the fixture build, static server, and `Browser` process itself are shared.
//! [`accessibility_tree`](super::accessibility_tree) additionally shares one tab across its own seven `#[test]`
//! functions, behind its own `Mutex` — that sharing is local to that module, not part of this file.
//!
//! Every `#[test]` in this binary returns `Result<(), String>`, the same failure-reporting convention the main
//! `svg-dom` crate's own browser tests use (see `docs/testing.md`'s "Failure Reporting" section) — a failure prints
//! its `String` message directly, with no panic and no stack trace. [`new_tab`] reports Chrome-launch failure the
//! same way; `build_fixture`/`serve` (`cdp-integration-test/src/lib.rs`) still panic on a broken environment
//! (missing `wasm-pack`, an unbindable port), the same as they did before this file existed.

use cdp_integration_test::{build_fixture, fixture_dir, launch_browser, serve};
use headless_chrome::{Browser, Tab};
use std::{
    sync::{Arc, OnceLock},
    time::Duration,
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
struct Shared {
    // Never read after construction, but must outlive every test: dropping `Browser` closes the Chrome process,
    // and with it every `Tab` opened from it.
    browser: Browser,
    base_url: String,
}

static SHARED: OnceLock<Result<Shared, String>> = OnceLock::new();

fn shared() -> Result<&'static Shared, String> {
    SHARED
        .get_or_init(|| {
            let dir = fixture_dir();
            build_fixture(&dir);
            let port = serve(dir);
            let browser =
                launch_browser().map_err(|e| format!("failed to launch Chrome — is it installed locally? {e}"))?;
            Ok(Shared {
                browser,
                base_url: format!("http://127.0.0.1:{port}/index.html"),
            })
        })
        .as_ref()
        .map_err(String::clone)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Opens a fresh tab on the shared fixture page and waits for `#fixture-ready` before returning it.
///
/// Building the fixture, starting the server, and launching Chrome happen at most once for the whole binary; only
/// this tab is new.
pub(crate) fn new_tab() -> Result<Arc<Tab>, String> {
    let shared = shared()?;
    let tab = shared.browser.new_tab().map_err(|e| format!("failed to open a new tab: {e}"))?;
    tab.navigate_to(&shared.base_url)
        .map_err(|e| format!("failed to navigate to fixture page: {e}"))?;
    tab.wait_for_element_with_custom_timeout("#fixture-ready", Duration::from_secs(10))
        .map_err(|e| format!("fixture did not signal readiness in time: {e}"))?;
    Ok(tab)
}
