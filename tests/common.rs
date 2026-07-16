#![allow(dead_code)]
// Each test binary uses a different subset of these helpers

// Shared helpers for all browser integration tests.
//
// Each helper that touches the DOM appends elements to the document body.  Tests are
// isolated by using a unique element id per test — there are no teardown hooks, but the
// elements are harmless since the browser page is discarded after the test run.
use svg_dom::{Error, SvgRoot, root::utils::Size};

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

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
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
// Non-SVG content fixtures
//
// `SvgNode`'s tree-navigation methods (`first_child`, `next_sibling`, `children`, `query_selector`, ...) all document
// what happens when browser traversal or CSS-selector matching lands on a non-SVG element — the canonical way that
// happens in a real document is HTML content inside a `<foreignObject>`. These helpers build that fixture with plain
// `web_sys` calls, since a `<div>` cannot be represented as an `SvgNode` at all.
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Appends an SVG-namespaced `<foreignObject>` to `target` and returns it.
///
/// `<foreignObject>` is itself a genuine SVG element — it casts to `SvgElement` like any other — but it is the only
/// DOM-valid place for arbitrary HTML content to appear inside an SVG document, so fixtures that need a non-SVG
/// element build it as (a descendant of) one.
pub fn foreign_object(target: &web_sys::Element) -> web_sys::Element {
    let el = document().create_element_ns(Some(SVG_NS), "foreignObject").unwrap();
    target.append_child(&el).unwrap();
    el
}

/// Appends a plain HTML `<div>` (default, non-SVG namespace) to `target` and returns it.
pub fn html_div(target: &web_sys::Element) -> web_sys::Element {
    let el = document().create_element("div").unwrap();
    target.append_child(&el).unwrap();
    el
}

/// Appends a bare SVG-namespaced `<rect>` to `target` and returns it.
///
/// Used where a fixture needs a genuine `SvgElement`-castable sibling inside a `<foreignObject>` but does not need
/// the full `svg-dom` rect factory (geometry, styling, ...) — just something that satisfies `dyn_into::<SvgElement>()`.
pub fn svg_rect(target: &web_sys::Element) -> web_sys::Element {
    let el = document().create_element_ns(Some(SVG_NS), "rect").unwrap();
    target.append_child(&el).unwrap();
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

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Returns `Err(msg)` when `condition` is `false`.
pub fn check(condition: bool, msg: &str) -> Result<(), String> {
    if condition { Ok(()) } else { Err(msg.into()) }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Helper: create an SvgRoot inside a fresh div.
pub fn make_svg(id: &str) -> SvgRoot {
    div(id);
    SvgRoot::create_in(id, Size::new(400.0, 300.0)).unwrap()
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub fn is_invalid_marker_id(result: Result<svg_dom::SvgMarker, Error>) -> bool {
    matches!(result, Err(Error::InvalidMarkerId(_)))
}
