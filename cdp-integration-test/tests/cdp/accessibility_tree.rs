//! `<title>`/`<desc>` accessible-name/description computation, against the real, browser-computed accessibility
//! tree.
//!
//! Every other test covering `set_title` or `set_desc` (`svg-dom`'s own `tests/accessibility.rs`) proves the DOM
//! structure: that the correct element was created, updated, or removed in the correct place.
//! None of those tests can see the actual, browser-*computed* accessibility tree, since that lives behind
//! Chrome's Accessibility CDP domain, not the DOM `wasm-bindgen-test`'s WebDriver-run tests have access to.
//!
//! This queries `Accessibility.getPartialAXTree` for seven scenarios built by the sibling `cdp-test-fixture` wasm
//! crate, each one independently reported as its own `#[test]`, confirming:
//!
//! 1. A lone `<title>` supplies the accessible name.
//! 2. A `<desc>` alongside it supplies the accessible description.
//! 3. A value in `aria-label` overrides a `<title>` in the accessible name computation.
//! 4. A value in `aria-describedby` overrides a `<desc>` in the description computation.
//! 5. A rejected blank `set_title` leaves the element with no accessible name at all.
//!    This rejection exists to prevent an "apparently nameless object exposed to assistive technology", a case
//!    SVG 2 warns about.
//! 6. A value in `aria-labelledby` overrides *both* `aria-label` and `<title>`, since it has strictly higher
//!    precedence than `aria-label` in accessible-name computation, not just parity with it.
//!    This gets its own scenario, rather than folding into scenario 3, since the API documentation calls the
//!    precedence out explicitly.
//! 7. Visible text wrapped by an `<a>` is exposed as a named link.
//!    `SvgRoot::anchor`'s own rendered-region and nested-link caveats describe the DOM/paint side of `<a>`, but
//!    only the Accessibility CDP domain this file drives can confirm that a real browser assigns it both the
//!    accessible "link" role and computes its name from the linked text content, the same way it would for an
//!    HTML `<a>`.
//!
//! # Why seven `#[test]` functions share one tab
//!
//! [`super::common::new_tab`] already shares the fixture build, static server, and `Browser` process with every
//! other module in this binary.
//! Opening a fresh tab per scenario here would still cost a `Tab::navigate_to`/readiness-wait round trip seven
//! times over, for content that never changes between them, so all seven scenarios additionally share one tab of
//! their own, via a lazily-initialised local `OnceLock`.
//!
//! The tests stay split into seven functions for two reasons:
//! 1. `cargo test` reports each scenario's pass/fail independently, instead of collapsing them into one result.
//! 2. A failing scenario aborts only that function, not every scenario after it.
//!
//! By default, `cargo test` runs test functions in parallel; CDP tab access is not safe under that kind of
//! concurrency, though.
//! `Browser` and `Tab` are `Send + Sync` at the type level, which makes them shareable, but `Tab::find_element`'s
//! `DOM.getDocument` followed by `DOM.querySelector` is not atomic.
//! Two threads racing against the same tab could interleave and hand one of them a `nodeId` from the other's
//! `getDocument` call, which then fails to resolve.
//! The `QUERY_LOCK` mutex below serialises every CDP round trip, so the seven tests still run concurrently as far
//! as the test harness is concerned, but their actual browser interactions never overlap.

use headless_chrome::{Tab, protocol::cdp::Accessibility};
use serde_json::Value;
use std::sync::{Arc, Mutex, OnceLock};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
static TAB: OnceLock<Result<Arc<Tab>, String>> = OnceLock::new();

/// Serialises every CDP round trip against the shared tab — see the module doc comment's concurrency note.
static QUERY_LOCK: Mutex<()> = Mutex::new(());

fn tab() -> Result<&'static Arc<Tab>, String> {
    TAB.get_or_init(|| {
        let tab = super::common::new_tab()?;
        tab.call_method(Accessibility::Enable(None))
            .map_err(|e| format!("Accessibility.enable failed: {e}"))?;
        Ok(tab)
    })
    .as_ref()
    .map_err(String::clone)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Fetches the full computed `AXNode` for the element matching `selector`, via `Accessibility.getPartialAXTree`.
/// Shared by every `computed_*` helper below so the CDP round trip (and its locking) exists in one place, whichever
/// of the node's fields a given test actually needs.
fn ax_node(tab: &Tab, selector: &str) -> Result<Accessibility::AXNode, String> {
    // Held for the whole function, not just find_element: GetPartialAXTree also talks to the same session, and a
    // concurrent DOM.getDocument from another test's find_element could otherwise land between these two calls.
    let _guard = QUERY_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);

    let element = tab
        .find_element(selector)
        .map_err(|e| format!("no element matching {selector}: {e}"))?;
    let result = tab
        .call_method(Accessibility::GetPartialAXTree {
            node_id: None,
            backend_node_id: None,
            object_id: Some(element.remote_object_id.clone()),
            fetch_relatives: Some(false),
        })
        .map_err(|e| format!("GetPartialAXTree failed for {selector}: {e}"))?;
    result
        .nodes
        .into_iter()
        .next()
        .ok_or_else(|| format!("no AX node returned for {selector}"))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn ax_value_str(ax_value: &Option<Accessibility::AXValue>) -> Option<String> {
    ax_value
        .as_ref()
        .and_then(|v| v.value.as_ref())
        .and_then(|v: &Value| v.as_str().map(str::to_owned))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Fetches the computed accessible name/description for the element matching `selector`. Returns `(name, description)`,
/// either of which could be `None` when that property is absent from the accessibility tree (e.g. an element with no
/// accessible name at all).
fn computed_name_and_description(tab: &Tab, selector: &str) -> Result<(Option<String>, Option<String>), String> {
    let node = ax_node(tab, selector)?;
    Ok((ax_value_str(&node.name), ax_value_str(&node.description)))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Fetches the computed accessible name/role for the element matching `selector`. Returns `(name, role)`.
fn computed_name_and_role(tab: &Tab, selector: &str) -> Result<(Option<String>, Option<String>), String> {
    let node = ax_node(tab, selector)?;
    Ok((ax_value_str(&node.name), ax_value_str(&node.role)))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Tests
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn title_only_supplies_accessible_name() -> Result<(), String> {
    let (name, _) = computed_name_and_description(tab()?, "#s1")?;
    if name.as_deref() != Some("Save file") {
        return Err(format!("a lone <title> should supply the accessible name, got {name:?}"));
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn desc_supplies_accessible_description() -> Result<(), String> {
    let (name, description) = computed_name_and_description(tab()?, "#s2")?;
    if name.as_deref() != Some("Icon") {
        return Err(format!("expected the accessible name to be \"Icon\", got {name:?}"));
    }
    if description.as_deref() != Some("Writes the current document to disk.") {
        return Err(format!(
            "a <desc> should supply the accessible description, got {description:?}"
        ));
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn aria_label_overrides_title() -> Result<(), String> {
    let (name, _) = computed_name_and_description(tab()?, "#s3")?;
    if name.as_deref() != Some("Override name") {
        return Err(format!(
            "aria-label must take precedence over a <title> child for the accessible name, got {name:?}"
        ));
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn aria_describedby_overrides_desc() -> Result<(), String> {
    let (_, description) = computed_name_and_description(tab()?, "#s4")?;
    if description.as_deref() != Some("Override description") {
        return Err(format!(
            "aria-describedby must take precedence over a <desc> child for the accessible description, got \
             {description:?}"
        ));
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn blank_title_rejection_leaves_no_accessible_name() -> Result<(), String> {
    let (name, _) = computed_name_and_description(tab()?, "#s5")?;
    if !name.is_none_or(|n| n.is_empty()) {
        return Err("an element whose blank set_title was rejected must not have gained an accessible name".to_owned());
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn aria_labelledby_overrides_title_and_aria_label() -> Result<(), String> {
    let (name, _) = computed_name_and_description(tab()?, "#s6")?;
    if name.as_deref() != Some("Labelledby override name") {
        return Err(format!(
            "aria-labelledby must take precedence over both aria-label and a <title> child for the accessible \
             name, got {name:?}"
        ));
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Proves `SvgRoot::anchor` produces something a real browser actually treats as a link, not just an `<a>` tag in
/// the DOM: `svg-dom`'s own tests can see the tag name and the `href` attribute, but only the Accessibility CDP domain
/// can see whether Chrome assigns it the "link" role and computes an accessible name from its linked text content — the
/// two properties exposed to assistive technology. This does not exercise keyboard focus or activation (SVG 2
/// separately defines valid SVG links as focusable); it only proves the accessibility-tree side.
#[test]
fn anchor_with_visible_text_is_a_named_link() -> Result<(), String> {
    let (name, role) = computed_name_and_role(tab()?, "#s7")?;
    if role.as_deref() != Some("link") {
        return Err(format!(
            "an <a> wrapping visible text must be exposed with the accessible \"link\" role, got {role:?}"
        ));
    }
    if name.as_deref() != Some("Read the docs") {
        return Err(format!(
            "the accessible name must come from the linked text content, the same way it would for an HTML <a>, \
             got {name:?}"
        ));
    }
    Ok(())
}
