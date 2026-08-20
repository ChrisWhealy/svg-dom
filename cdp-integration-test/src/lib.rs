//! Shared setup helpers for `cdp-integration-test`'s Chrome-DevTools-Protocol integration tests.
//!
//! The tests themselves live in one Cargo integration-test binary, `tests/cdp/`, with one scenario per sibling
//! module.
//! See [`tests/cdp/main.rs`](https://github.com/ChrisWhealy/svg-dom/blob/main/cdp-integration-test/tests/cdp/main.rs)'s
//! own module doc comment for the scenario list, why they now share one binary, why this crate lives in its own
//! on-demand workspace member, how it runs in CI, and why the browser is launched with `sandbox(false)`.
//!
//! The functions below build the `cdp-test-fixture` wasm package, serve it, and launch Chrome.
//! `tests/cdp/common.rs` calls them exactly once per test run, via a lazily-initialised `OnceLock`, and hands
//! every scenario module its own [`Tab`](headless_chrome::Tab) from the one shared [`Browser`] instance.

use fd_lock::RwLock;
use headless_chrome::{Browser, LaunchOptions, browser::default_executable};
use std::{fs::OpenOptions, path::PathBuf, process::Command, thread};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// The path to the sibling `cdp-test-fixture` wasm crate, relative to this crate's own manifest directory.
pub fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("cdp-integration-test must live inside the svg-dom workspace")
        .join("cdp-test-fixture")
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Rebuilds the `cdp-test-fixture` wasm package so `serve`'s output is current.
///
/// `tests/cdp/common.rs` calls this once per test run, from behind its own `OnceLock`, so within this crate's own
/// `cdp` binary only one call is ever in flight. The exclusive cross-process file lock taken below still guards
/// against a second, independent `cargo test`/`cargo nextest run` invocation racing this one: two `wasm-pack build`
/// invocations running in the same `dir` at once would corrupt each other's intermediate state (wasm-bindgen's
/// generated files, wasm-opt's temp output).
pub fn build_fixture(dir: &PathBuf) {
    let lock_path = dir.join(".build_fixture.lock");
    let lock_file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&lock_path)
        .expect("could not open build_fixture lock file");
    let mut lock = RwLock::new(lock_file);
    let _guard = lock.write().expect("could not acquire build_fixture lock");

    let status = Command::new("wasm-pack")
        .current_dir(dir)
        .args(["build", "--target", "web"])
        .status()
        .expect("could not run wasm-pack — is it installed and on PATH?");
    assert!(status.success(), "wasm-pack build failed for cdp-test-fixture");
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
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

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Launches Chrome with its sandbox disabled.
/// See `tests/cdp/main.rs`'s `# Why the browser is launched with sandbox(false)` doc section for an explanation.
pub fn launch_browser() -> Result<Browser, Box<dyn std::error::Error>> {
    let path = default_executable().map_err(|e| format!("could not locate a Chrome/Chromium binary: {e}"))?;
    let launch_options = LaunchOptions::default_builder().path(Some(path)).sandbox(false).build()?;
    Ok(Browser::new(launch_options)?)
}
