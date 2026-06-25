#![allow(dead_code)]
// Each test binary uses a different subset of these helpers

// Shared helpers for all browser integration tests.
//
// Each helper that touches the DOM appends elements to the document body.  Tests are
// isolated by using a unique element id per test — there are no teardown hooks, but the
// elements are harmless since the browser page is discarded after the test run.

const SVG_NS: &str = "http://www.w3.org/2000/svg";

fn document() -> web_sys::Document {
    web_sys::window().unwrap().document().unwrap()
}

fn body() -> web_sys::Element {
    // query_selector avoids the HtmlElement feature requirement.
    document().query_selector("body").unwrap().unwrap()
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// DOM fixture helpers
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Creates `<div id="id">` appended to `<body>` and returns the element.
/// Use a unique `id` per test so tests do not interfere with each other.
pub fn div(id: &str) -> web_sys::Element {
    let el = document().create_element("div").unwrap();
    el.set_id(id);
    body().append_child(&el).unwrap();
    el
}

/// Creates `<svg id="id">` (SVG namespace) appended to `<body>` and returns the element.
/// Use this when a test needs an `<svg>` to already exist in the document before
/// calling `SvgRoot::attach`.
pub fn svg(id: &str) -> web_sys::Element {
    let el = document().create_element_ns(Some(SVG_NS), "svg").unwrap();
    el.set_id(id);
    body().append_child(&el).unwrap();
    el
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Assertion helpers
//
// Both functions return Result<(), String> so callers can propagate with `?` rather than
// producing a stack trace on failure.
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Returns `Err` with a descriptive message when `got != expected`.
pub fn check_eq<T: PartialEq + std::fmt::Debug>(got: T, expected: T) -> Result<(), String> {
    if got == expected {
        Ok(())
    } else {
        Err(format!("expected {:?}, got {:?}", expected, got))
    }
}

/// Returns `Err(msg)` when `condition` is `false`.
pub fn check(condition: bool, msg: &str) -> Result<(), String> {
    if condition {
        Ok(())
    } else {
        Err(msg.into())
    }
}
