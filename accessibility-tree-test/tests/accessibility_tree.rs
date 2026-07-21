//! Chrome-DevTools-Protocol integration test for `<title>`/`<desc>` accessible-name/description computation.
//!
//! Every other test covering `set_title`/`set_desc` (`svg-dom`'s `tests/accessibility.rs`) proves DOM structure —
//! the right element got created/updated/removed in the right place. None of them can see the actual, browser-
//! computed accessibility tree, since that is not exposed through the DOM at all: it lives behind Chrome's
//! Accessibility CDP domain, which `wasm-bindgen-test`'s WebDriver-run browser tests have no access to.
//!
//! This test drives a real, real Chrome instance via CDP (through the `headless_chrome` crate) and queries
//! `Accessibility.getPartialAXTree` for five scenarios built by the sibling `a11y-fixture` wasm crate, confirming:
//!
//! 1. a lone `<title>` supplies the accessible name;
//! 2. a `<desc>` alongside it supplies the accessible description;
//! 3. `aria-label` overrides a `<title>` in name computation;
//! 4. `aria-describedby` overrides a `<desc>` in description computation;
//! 5. a rejected blank `set_title` leaves the element with no accessible name at all — i.e. the rejection actually
//!    prevents the "apparently nameless object exposed to assistive technology" case SVG 2 warns about, not just
//!    the DOM mutation.
//!
//! Lives in its own on-demand workspace member (excluded from the root package's `default-members`, same as
//! `demo-server`) because it pulls in `headless_chrome` and requires a local Chrome/Chromium binary — neither of
//! which the ordinary `cargo test`/`cargo nextest run` workflow should have to pay for. Run explicitly with:
//! `cargo test -p accessibility-tree-test`.

use std::{path::PathBuf, process::Command, thread, time::Duration};

use headless_chrome::{Browser, protocol::cdp::Accessibility};
use serde_json::Value;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("accessibility-tree-test must live inside the svg-dom workspace")
        .join("a11y-fixture")
}

fn build_fixture(dir: &PathBuf) {
    let status = Command::new("wasm-pack")
        .current_dir(dir)
        .args(["build", "--target", "web"])
        .status()
        .expect("could not run wasm-pack — is it installed and on PATH?");
    assert!(status.success(), "wasm-pack build failed for a11y-fixture");
}

/// Serves `dir` on an OS-assigned local port and returns that port. The server runs for the lifetime of the test
/// process on a background thread; there is no shutdown hook, but the process exits when the test does.
fn serve(dir: PathBuf) -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("failed to bind ephemeral port");
    let port = listener.local_addr().expect("no local addr").port();
    let server = tiny_http::Server::from_listener(listener, None).expect("failed to start static file server");

    thread::spawn(move || {
        for request in server.incoming_requests() {
            let mut path = request.url().trim_start_matches('/').to_owned();
            if path.is_empty() {
                path = "index.html".to_owned();
            }
            let file_path = dir.join(&path);
            let response_result = match std::fs::read(&file_path) {
                Ok(bytes) => {
                    let content_type = if path.ends_with(".wasm") {
                        "application/wasm"
                    } else if path.ends_with(".js") {
                        "text/javascript"
                    } else {
                        "text/html"
                    };
                    let header = tiny_http::Header::from_bytes(b"Content-Type".as_slice(), content_type.as_bytes())
                        .expect("valid header");
                    request.respond(tiny_http::Response::from_data(bytes).with_header(header))
                },
                Err(_) => request.respond(tiny_http::Response::from_string("not found").with_status_code(404)),
            };
            let _ = response_result;
        }
    });

    port
}

/// Fetches the computed accessible name/description for the element matching `selector`, via
/// `Accessibility.getPartialAXTree`. Returns `(name, description)`, either of which is `None` when that property
/// is absent from the accessibility tree (e.g. an element with no accessible name at all).
fn computed_name_and_description(tab: &headless_chrome::Tab, selector: &str) -> (Option<String>, Option<String>) {
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
    let node = result
        .nodes
        .first()
        .unwrap_or_else(|| panic!("no AX node returned for {selector}"));

    let extract = |ax_value: &Option<Accessibility::AXValue>| -> Option<String> {
        ax_value
            .as_ref()
            .and_then(|v| v.value.as_ref())
            .and_then(|v: &Value| v.as_str().map(str::to_owned))
    };

    (extract(&node.name), extract(&node.description))
}

#[test]
fn accessible_name_and_description_computation() {
    let dir = fixture_dir();
    build_fixture(&dir);
    let port = serve(dir);

    let browser = Browser::default().expect("failed to launch Chrome — is it installed locally?");
    let tab = browser.new_tab().expect("failed to open a new tab");
    tab.navigate_to(&format!("http://127.0.0.1:{port}/index.html"))
        .expect("failed to navigate to fixture page");
    tab.wait_for_element_with_custom_timeout("#fixture-ready", Duration::from_secs(10))
        .expect("fixture did not signal readiness in time");

    tab.call_method(Accessibility::Enable(None))
        .expect("Accessibility.enable failed");

    // 1. title-only naming.
    let (name, _) = computed_name_and_description(&tab, "#s1");
    assert_eq!(
        name.as_deref(),
        Some("Save file"),
        "a lone <title> should supply the accessible name"
    );

    // 2. description exposure alongside a name.
    let (name, description) = computed_name_and_description(&tab, "#s2");
    assert_eq!(name.as_deref(), Some("Icon"));
    assert_eq!(
        description.as_deref(),
        Some("Writes the current document to disk."),
        "a <desc> should supply the accessible description"
    );

    // 3. aria-label overrides <title> in accessible-name computation.
    let (name, _) = computed_name_and_description(&tab, "#s3");
    assert_eq!(
        name.as_deref(),
        Some("Override name"),
        "aria-label must take precedence over a <title> child for the accessible name"
    );

    // 4. aria-describedby overrides <desc> in accessible-description computation.
    let (_, description) = computed_name_and_description(&tab, "#s4");
    assert_eq!(
        description.as_deref(),
        Some("Override description"),
        "aria-describedby must take precedence over a <desc> child for the accessible description"
    );

    // 5. a rejected blank set_title must not leave the element with any accessible name.
    let (name, _) = computed_name_and_description(&tab, "#s5");
    assert!(
        name.is_none_or(|n| n.is_empty()),
        "an element whose blank set_title was rejected must not have gained an accessible name"
    );
}
