//! Chrome-DevTools-Protocol (CDP) integration test for `<title>`/`<desc>` accessible-name/description computation.
//!
//! Every other test covering `set_title`/`set_desc` (`svg-dom`'s `tests/accessibility.rs`) proves DOM structure —
//! the right element got created/updated/removed in the right place. None of them can see the actual, browser-
//! computed accessibility tree, since that is not exposed through the DOM at all: it lives behind Chrome's
//! Accessibility CDP domain, which `wasm-bindgen-test`'s WebDriver-run browser tests have no access to.
//!
//! This drives a real Chrome instance via CDP (through the `headless_chrome` crate) and queries
//! `Accessibility.getPartialAXTree` for seven scenarios built by the sibling `a11y-fixture` wasm crate — one
//! independently reported `#[test]` per scenario, confirming:
//!
//! 1. A lone `<title>` supplies the accessible name;
//! 2. A `<desc>` alongside it supplies the accessible description;
//! 3. A value in `aria-label` overrides a `<title>` in name computation;
//! 4. A value in `aria-describedby` overrides a `<desc>` in description computation;
//! 5. A rejected blank `set_title` leaves the element with no accessible name at all — i.e. the rejection actually
//!    prevents the "apparently nameless object exposed to assistive technology" case SVG 2 warns about, not just
//!    the DOM mutation;
//! 6. A value in `aria-labelledby` overrides *both* `aria-label` and `<title>` — it has higher precedence than
//!    `aria-label` in accessible-name computation, not just parity with it, and the API documentation calls this out
//!    explicitly, so it earns its own scenario rather than being folded into scenario 3.
//! 7. An `<a>` wrapping visible text is exposed as a named link — `SvgRoot::anchor`'s rendered-region and
//!    nested-link caveats describe the DOM/paint side of `<a>`, but neither `svg-dom`'s own DOM-structure tests nor
//!    the WebDriver-run browser tests can see whether a real browser actually assigns it the accessible "link" role
//!    and computes its name from the linked text content, the same way it would for an HTML `<a>` — only the
//!    Accessibility CDP domain this file already drives can.
//!
//! # Why six `#[test]` functions share one browser session
//!
//! Building the fixture and launching Chrome are expensive actions requiring wasm-pack compile and browser startup, so
//! all six scenarios share a single fixture build, static server, and Chrome tab via a lazily-initialised `OnceLock`
//! rather than each test repeating this cost individually.
//!
//! Splitting into six functions (rather than one function with six sequential `assert_eq!` calls) matters for two
//! reasons: `cargo test` reports each scenario's pass/fail independently instead of collapsing them into a single
//! result, and an `assert_eq!` failure in one scenario no longer aborts the others before they get a chance to run.
//!
//! `cargo test` runs test functions in parallel by default, though, and CDP tab access is not safe under that kind
//! of concurrency: `Browser`/`Tab` are `Send + Sync` at the type level (built entirely from `Arc`/`Mutex`/`RwLock`
//! internally), which makes them shareable, but `Tab::find_element`'s `DOM.getDocument`-then-`DOM.querySelector`
//! sequence is not atomic — two threads racing it against the same tab can interleave and hand one of them a
//! `nodeId` from the other's `getDocument` call, which then fails to resolve. A `QUERY_LOCK` mutex below serialises
//! every CDP round trip so the six tests still run concurrently as far as the test harness is concerned, but their
//! actual browser interactions never overlap.
//!
//! Lives in its own on-demand workspace member (excluded from the root package's `default-members`, same as
//! `demo-server`) because it pulls in `headless_chrome` and requires a local Chrome/Chromium binary — neither of
//! which the ordinary `cargo test`/`cargo nextest run` workflow should have to pay for. Run explicitly with:
//! `cargo test -p accessibility-tree-test`. That command also runs the sibling `filter_blend_render.rs`, a second,
//! unrelated CDP integration test for `SvgFilter::blend` sharing this crate's `fixture_dir`/`build_fixture`/`serve`/
//! `launch_browser` setup helpers (in `src/lib.rs`) but not its running `Browser`/`Tab` instance or `#[test]`s —
//! see that file's own module doc comment for what it verifies.
//!
//! Run in CI by its own job in `.github/workflows/ci.yml` (`accessibility-tree-test`), using the Chrome installation
//! already present on GitHub's `ubuntu-latest` runner image — being a separate job, its failure does not block the
//! other, unrelated CI jobs from reporting their own results, but it still gates the merge like any other required
//! check.
//!
//! # Why the browser is launched with `sandbox(false)`
//!
//! `Browser::default()` launches with Chrome's own sandbox enabled, which is the right default for browsing untrusted
//! content — but `ubuntu-latest` now resolves to Ubuntu 24.04+, which restricts unprivileged user namespaces via
//! AppArmor, which is what Chrome's sandbox relies on. That makes Chrome's sandbox initialisation fail even for a
//! non-root CI user, unless `--no-sandbox` is passed. Since this test only ever loads a local fixture page this crate
//! builds itself, there is no untrusted content for the sandbox to matter for, so it is disabled unconditionally rather
//! than only in CI — keeping local and CI runs on the same code path.

use std::{
    sync::{Arc, Mutex, OnceLock},
    time::Duration,
};

use accessibility_tree_test::{build_fixture, fixture_dir, launch_browser, serve};
use headless_chrome::{Browser, Tab, protocol::cdp::Accessibility};
use serde_json::Value;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
struct Fixture {
    // Never read after construction, but must outlive every test: dropping the `Browser` closes the Chrome process
    // (and with it every `Tab`, including the one below).
    _browser: Browser,
    tab: Arc<Tab>,
}

static FIXTURE: OnceLock<Fixture> = OnceLock::new();

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Serialises every CDP round trip against the shared tab — see the module doc comment's concurrency note.
static QUERY_LOCK: Mutex<()> = Mutex::new(());

fn fixture() -> &'static Fixture {
    FIXTURE.get_or_init(|| {
        let dir = fixture_dir();
        build_fixture(&dir);
        let port = serve(dir);

        let browser = launch_browser().expect("failed to launch Chrome — is it installed locally?");
        let tab = browser.new_tab().expect("failed to open a new tab");
        tab.navigate_to(&format!("http://127.0.0.1:{port}/index.html"))
            .expect("failed to navigate to fixture page");
        tab.wait_for_element_with_custom_timeout("#fixture-ready", Duration::from_secs(10))
            .expect("fixture did not signal readiness in time");
        tab.call_method(Accessibility::Enable(None))
            .expect("Accessibility.enable failed");

        Fixture { _browser: browser, tab }
    })
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Fetches the full computed `AXNode` for the element matching `selector`, via `Accessibility.getPartialAXTree`.
/// Shared by every `computed_*` helper below so the CDP round trip (and its locking) exists in one place, whichever
/// of the node's fields a given test actually needs.
fn ax_node(tab: &Tab, selector: &str) -> Accessibility::AXNode {
    // Held for the whole function, not just find_element: GetPartialAXTree also talks to the same session, and a
    // concurrent DOM.getDocument from another test's find_element could otherwise land between these two calls.
    let _guard = QUERY_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);

    let element = tab
        .find_element(selector)
        .unwrap_or_else(|e| panic!("no element matching {selector}: {e}"));
    let result = tab
        .call_method(Accessibility::GetPartialAXTree {
            node_id: None,
            backend_node_id: None,
            object_id: Some(element.remote_object_id.clone()),
            fetch_relatives: Some(false),
        })
        .unwrap_or_else(|e| panic!("GetPartialAXTree failed for {selector}: {e}"));
    result
        .nodes
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("no AX node returned for {selector}"))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn ax_value_str(ax_value: &Option<Accessibility::AXValue>) -> Option<String> {
    ax_value
        .as_ref()
        .and_then(|v| v.value.as_ref())
        .and_then(|v: &Value| v.as_str().map(str::to_owned))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Fetches the computed accessible name/description for the element matching `selector`. Returns `(name,
/// description)`, either of which is `None` when that property is absent from the accessibility tree (e.g. an
/// element with no accessible name at all).
fn computed_name_and_description(tab: &Tab, selector: &str) -> (Option<String>, Option<String>) {
    let node = ax_node(tab, selector);
    (ax_value_str(&node.name), ax_value_str(&node.description))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Fetches the computed accessible name/role for the element matching `selector`. Returns `(name, role)`.
fn computed_name_and_role(tab: &Tab, selector: &str) -> (Option<String>, Option<String>) {
    let node = ax_node(tab, selector);
    (ax_value_str(&node.name), ax_value_str(&node.role))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Tests
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn title_only_supplies_accessible_name() {
    let (name, _) = computed_name_and_description(&fixture().tab, "#s1");
    assert_eq!(
        name.as_deref(),
        Some("Save file"),
        "a lone <title> should supply the accessible name"
    );
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn desc_supplies_accessible_description() {
    let (name, description) = computed_name_and_description(&fixture().tab, "#s2");
    assert_eq!(name.as_deref(), Some("Icon"));
    assert_eq!(
        description.as_deref(),
        Some("Writes the current document to disk."),
        "a <desc> should supply the accessible description"
    );
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn aria_label_overrides_title() {
    let (name, _) = computed_name_and_description(&fixture().tab, "#s3");
    assert_eq!(
        name.as_deref(),
        Some("Override name"),
        "aria-label must take precedence over a <title> child for the accessible name"
    );
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn aria_describedby_overrides_desc() {
    let (_, description) = computed_name_and_description(&fixture().tab, "#s4");
    assert_eq!(
        description.as_deref(),
        Some("Override description"),
        "aria-describedby must take precedence over a <desc> child for the accessible description"
    );
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn blank_title_rejection_leaves_no_accessible_name() {
    let (name, _) = computed_name_and_description(&fixture().tab, "#s5");
    assert!(
        name.is_none_or(|n| n.is_empty()),
        "an element whose blank set_title was rejected must not have gained an accessible name"
    );
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn aria_labelledby_overrides_title_and_aria_label() {
    let (name, _) = computed_name_and_description(&fixture().tab, "#s6");
    assert_eq!(
        name.as_deref(),
        Some("Labelledby override name"),
        "aria-labelledby must take precedence over both aria-label and a <title> child for the accessible name"
    );
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Proves `SvgRoot::anchor` produces something a real browser actually treats as a link, not just an `<a>` tag in
/// the DOM: `svg-dom`'s own tests can see the tag name and the `href` attribute, but only the Accessibility CDP
/// domain can see whether Chrome assigns it the "link" role and computes an accessible name from its linked text
/// content — the two properties assistive technology and keyboard navigation actually rely on.
#[test]
fn anchor_with_visible_text_is_a_named_link() {
    let (name, role) = computed_name_and_role(&fixture().tab, "#s7");
    assert_eq!(
        role.as_deref(),
        Some("link"),
        "an <a> wrapping visible text must be exposed with the accessible \"link\" role"
    );
    assert_eq!(
        name.as_deref(),
        Some("Read the docs"),
        "the accessible name must come from the linked text content, the same way it would for an HTML <a>"
    );
}
