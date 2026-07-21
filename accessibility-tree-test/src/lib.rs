//! Shared setup helpers for `accessibility-tree-test`'s Chrome-DevTools-Protocol integration tests.
//!
//! This crate hosts two integration test files under `tests/`, each verifying browser-computed behaviour that
//! plain DOM inspection (and therefore `wasm-bindgen-test`) cannot see:
//!
//! - `accessibility_tree.rs` — accessible-name/description computation, via the Accessibility CDP domain.
//! - `filter_blend_render.rs` — `SvgFilter::blend`'s alpha-preserving tint chain, via actual rendered pixels.
//!
//! Both drive a real Chrome instance against the same sibling `a11y-fixture` wasm crate (built once, served
//! locally), so the functions below — building the fixture, serving it, and launching Chrome — are shared here
//! rather than duplicated per test file. Each test file still builds and launches its own instance of the fixture
//! and Chrome, since cargo compiles each file under `tests/` as a separate binary with its own process; there is no
//! way to share the running `Browser`/`Tab` *instance* across files, only the setup code that creates one.
//!
//! See `accessibility_tree.rs`'s module doc comment for why this crate lives in its own on-demand workspace member,
//! how it runs in CI, and why the browser is launched with `sandbox(false)` — that reasoning applies equally to
//! `filter_blend_render.rs` and is not repeated here.

use std::{path::PathBuf, process::Command, thread};

use headless_chrome::{Browser, LaunchOptions, browser::default_executable};

/// The path to the sibling `a11y-fixture` wasm crate, relative to this crate's own manifest directory.
pub fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("accessibility-tree-test must live inside the svg-dom workspace")
        .join("a11y-fixture")
}

/// Rebuilds the `a11y-fixture` wasm package so `serve`'s output is current.
pub fn build_fixture(dir: &PathBuf) {
    let status = Command::new("wasm-pack")
        .current_dir(dir)
        .args(["build", "--target", "web"])
        .status()
        .expect("could not run wasm-pack — is it installed and on PATH?");
    assert!(status.success(), "wasm-pack build failed for a11y-fixture");
}

/// Serves `dir` on an OS-assigned local port and returns that port. The server runs for the lifetime of the test
/// process on a background thread; there is no shutdown hook, but the process exits when the test does.
pub fn serve(dir: PathBuf) -> u16 {
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

/// Launches Chrome with its sandbox disabled — see `accessibility_tree.rs`'s
/// `# Why the browser is launched with sandbox(false)` doc section for why.
pub fn launch_browser() -> Result<Browser, Box<dyn std::error::Error>> {
    let path = default_executable().map_err(|e| format!("could not locate a Chrome/Chromium binary: {e}"))?;
    let launch_options = LaunchOptions::default_builder().path(Some(path)).sandbox(false).build()?;
    Ok(Browser::new(launch_options)?)
}
