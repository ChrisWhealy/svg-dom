mod common;

use common::*;
use svg_dom::{
    Error,
    root::utils::{Point, Size},
};
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// SvgDefs::view — eager variant
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `defs.view(id)` creates an element with tag name `"view"`.
#[wasm_bindgen_test]
fn should_create_view_element() -> Result<(), String> {
    let svg = make_svg("view-create");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let view = defs.view("detail").map_err(|e| e.to_string())?;
    check_eq(view.as_element().tag_name(), "view".to_owned())
}

/// The `<view>` element is a child of `<defs>`, not of the root `<svg>`.
#[wasm_bindgen_test]
fn should_append_view_to_defs() -> Result<(), String> {
    let svg = make_svg("view-parent");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let view = defs.view("detail").map_err(|e| e.to_string())?;
    let parent = view.as_element().parent_element().ok_or("view has no parent")?;
    check_eq(parent.tag_name(), "defs".to_owned())
}

/// `defs.view(id)` sets the `id` attribute on the element.
#[wasm_bindgen_test]
fn should_set_view_id_attribute() -> Result<(), String> {
    let svg = make_svg("view-id");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let view = defs.view("my-view").map_err(|e| e.to_string())?;
    check_eq(view.as_element().get_attribute("id"), Some("my-view".into()))
}

/// `SvgView::id()` returns the cached id without a DOM round-trip.
#[wasm_bindgen_test]
fn should_return_cached_view_id() -> Result<(), String> {
    let svg = make_svg("view-cached-id");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let view = defs.view("cached").map_err(|e| e.to_string())?;
    check_eq(view.id(), "cached")
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// SvgDefs::build_view — deferred/transactional variant
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `build_view` appends the element to `<defs>` only after the closure returns `Ok`.
#[wasm_bindgen_test]
fn should_build_view_and_append_to_defs() -> Result<(), String> {
    let svg = make_svg("view-build");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let view = defs
        .build_view("built", |v| v.set_view_box(0.0, 0.0, 50.0, 50.0))
        .map_err(|e| e.to_string())?;
    let parent = view.as_element().parent_element().ok_or("view has no parent")?;
    check_eq(parent.tag_name(), "defs".to_owned())?;
    check_eq(view.as_element().get_attribute("viewBox"), Some("0 0 50 50".into()))
}

/// When `build_view`'s closure returns `Err`, the view element is not attached to `<defs>`.
#[wasm_bindgen_test]
fn should_not_append_view_on_closure_error() -> Result<(), String> {
    let svg = make_svg("view-build-err");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let result = defs.build_view("failing", |_v| Err(Error::Dom("deliberate".into())));
    check(result.is_err(), "expected build_view to return Err")?;
    let defs_el = defs.as_element();
    check_eq(defs_el.child_element_count(), 0)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Viewport attributes
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `set_view_box` writes a correctly formatted `viewBox` attribute.
#[wasm_bindgen_test]
fn should_set_view_box() -> Result<(), String> {
    let svg = make_svg("view-viewbox");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let view = defs.view("vb").map_err(|e| e.to_string())?;
    view.set_view_box(0.0, 0.0, 100.0, 50.0).map_err(|e| e.to_string())?;
    check_eq(view.as_element().get_attribute("viewBox"), Some("0 0 100 50".into()))
}

/// `set_view_box` rejects a negative `width`/`height`, and a non-finite (`NaN`/`±infinity`) component, with
/// `Error::InvalidViewBox` — the same validation `SvgRoot::set_view_box` and `SvgSymbol::set_view_box` share.
#[wasm_bindgen_test]
fn should_reject_invalid_view_box() -> Result<(), String> {
    let svg = make_svg("view-viewbox-invalid");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let view = defs.view("vb-invalid").map_err(|e| e.to_string())?;
    check(
        matches!(view.set_view_box(0.0, 0.0, -100.0, 100.0), Err(Error::InvalidViewBox(_))),
        "expected InvalidViewBox for negative width",
    )?;
    check(
        matches!(view.set_view_box(f64::NAN, 0.0, 100.0, 100.0), Err(Error::InvalidViewBox(_))),
        "expected InvalidViewBox for a NaN component",
    )
}

/// `set_preserve_aspect_ratio` writes the attribute verbatim.
#[wasm_bindgen_test]
fn should_set_preserve_aspect_ratio() -> Result<(), String> {
    let svg = make_svg("view-par");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let view = defs.view("par").map_err(|e| e.to_string())?;
    view.set_preserve_aspect_ratio("xMidYMid slice").map_err(|e| e.to_string())?;
    check_eq(
        view.as_element().get_attribute("preserveAspectRatio"),
        Some("xMidYMid slice".into()),
    )
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Id validation
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// An invalid id is rejected with `Error::InvalidViewId` before any DOM call.
#[wasm_bindgen_test]
fn should_reject_invalid_view_id() -> Result<(), String> {
    let svg = make_svg("view-invalid-id");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let result = defs.view("url(#bad)");
    check(
        matches!(result, Err(Error::InvalidViewId(_))),
        "expected InvalidViewId error for an id containing '(' and ')'",
    )
}

/// `set_id` updates both the DOM attribute and the cached value.
#[wasm_bindgen_test]
fn should_set_view_id_via_set_id() -> Result<(), String> {
    let svg = make_svg("view-set-id");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let mut view = defs.view("old-id").map_err(|e| e.to_string())?;
    view.set_id("new-id").map_err(|e| e.to_string())?;
    check_eq(view.id(), "new-id")?;
    check_eq(view.as_element().get_attribute("id"), Some("new-id".into()))
}

/// `set_attr("id", ...)` is rejected to protect the cached id.
#[wasm_bindgen_test]
fn should_reject_set_attr_id() -> Result<(), String> {
    let svg = make_svg("view-reserved-id");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let view = defs.view("safe").map_err(|e| e.to_string())?;
    let result = view.set_attr("id", "injected");
    check(
        matches!(result, Err(Error::ReservedAttribute("id"))),
        "expected ReservedAttribute error for set_attr(\"id\", ...)",
    )
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Fragment-navigation behaviour
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Registers a one-shot listener for the next `"load"` or `"error"` event on `image`, returning a future that
/// resolves on `"load"` or rejects (with a message, rather than hanging forever) on `"error"`.
///
/// Must be called *before* the `href` change expected to trigger the event: image loading is queued as a browser
/// task, so a listener registered only after that change risks losing the race if the load completes first.
///
/// `"load"` and `"error"` are mutually exclusive terminal outcomes of the same load attempt, so exactly one of these
/// two listeners fires per call. Both are registered against one shared `AbortController`, and whichever fires calls
/// `abort()` first — via `AddEventListenerOptions::set_signal`, this detaches *both* listeners at once, including the
/// one that never fired. (`{ once: true }` alone would not do this: it only detaches the listener that actually
/// fired, leaving its counterpart — for the event that never happened — attached for the rest of the element's
/// lifetime, since the test loads the same element more than once.)
fn next_load_event(image: &web_sys::SvgImageElement) -> wasm_bindgen_futures::JsFuture {
    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        let controller = web_sys::AbortController::new().unwrap();
        let options = web_sys::AddEventListenerOptions::new();
        options.set_signal(&controller.signal());

        let abort_on_load = controller.clone();
        let on_load = Closure::once_into_js(move || {
            abort_on_load.abort();
            resolve.call0(&JsValue::NULL).unwrap();
        });
        image
            .add_event_listener_with_callback_and_add_event_listener_options(
                "load",
                on_load.as_ref().unchecked_ref(),
                &options,
            )
            .unwrap();

        let on_error = Closure::once_into_js(move || {
            controller.abort();
            reject
                .call1(&JsValue::NULL, &JsValue::from_str("image failed to load"))
                .unwrap();
        });
        image
            .add_event_listener_with_callback_and_add_event_listener_options(
                "error",
                on_error.as_ref().unchecked_ref(),
                &options,
            )
            .unwrap();
    });
    wasm_bindgen_futures::JsFuture::from(promise)
}

/// Draws `image`'s current content to a fresh offscreen canvas and returns the RGBA pixel at `(x, y)`.
///
/// `data:` URIs do not taint the canvas, so `get_image_data` can read the pixels straight back.
fn sample_pixel(image: &web_sys::SvgImageElement, x: f64, y: f64) -> [u8; 4] {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas: web_sys::HtmlCanvasElement = document.create_element("canvas").unwrap().dyn_into().unwrap();
    canvas.set_width(200);
    canvas.set_height(200);
    let ctx: web_sys::CanvasRenderingContext2d = canvas.get_context("2d").unwrap().unwrap().dyn_into().unwrap();
    ctx.draw_image_with_svg_image_element(image, 0.0, 0.0).unwrap();
    let data = ctx.get_image_data(x, y, 1.0, 1.0).unwrap().data().0;
    [data[0], data[1], data[2], data[3]]
}

/// Loads `data_uri` into a fresh `<image>` hosted under a `<div id="host_id">`, first without and then with a
/// `#detail` fragment, and asserts that the *rendered* viewport actually changes: the unnavigated baseline must show
/// two different quadrant colours, and after navigating to `#detail` both sample points must land in the same
/// (bottom-right) quadrant. `data_uri` must reference a 200×200 `viewBox="0 0 200 200"` SVG document with a
/// `<view id="detail" viewBox="100 100 100 100"/>` and four distinctly-coloured 100×100 quadrant rects — the shape
/// both fragment-navigation tests below build, one by hand and one through `svg-dom`'s own API.
///
/// `host_id` must be unique per caller: `wasm-bindgen-test` runs every test in a file against the same page, so a
/// shared id would make a second call's `SvgRoot::create_in` attach to the first call's leftover `<div>` instead of
/// its own.
async fn assert_view_fragment_switches_viewport(host_id: &str, data_uri: &str) -> Result<(), String> {
    let svg = make_svg(host_id);
    // Created with an empty `href` (no request, no load/error event) so the very first real load — like the second
    // one below — has its listener registered before the `href` that triggers it is set.
    let img_node = svg
        .image("", Point::new(0.0, 0.0), Size::new(200.0, 200.0))
        .map_err(|e| e.to_string())?;
    let img_element: web_sys::SvgImageElement = img_node
        .as_element()
        .dyn_ref::<web_sys::SvgImageElement>()
        .ok_or("expected an SVGImageElement")?
        .clone();

    let load = next_load_event(&img_element);
    img_node.set_href(data_uri).map_err(|e| e.to_string())?;
    load.await.map_err(|e| format!("image failed to load: {e:?}"))?;

    // Baseline: the whole 2x2 grid is visible, so opposite corners land in different quadrants.
    let before_top_left = sample_pixel(&img_element, 20.0, 20.0);
    let before_bottom_right = sample_pixel(&img_element, 180.0, 180.0);
    check(
        before_top_left != before_bottom_right,
        "expected the baseline (unnavigated) render to show two different quadrants",
    )?;

    // Navigate to the named view. Re-setting `href` on an already-loaded resource re-triggers loading; the listener
    // is (re-)registered first, exactly as above.
    let load = next_load_event(&img_element);
    img_node.set_href(&format!("{data_uri}#detail")).map_err(|e| e.to_string())?;
    load.await.map_err(|e| format!("image failed to load: {e:?}"))?;

    // The view's `viewBox` ("100 100 100 100") selects only the bottom-right quadrant, so both sample points now
    // land inside it and read the same colour — proof that the *rendered* viewport changed, not just an attribute.
    let after_top_left = sample_pixel(&img_element, 20.0, 20.0);
    let after_bottom_right = sample_pixel(&img_element, 180.0, 180.0);
    check_eq(after_top_left, after_bottom_right)?;
    check_eq(after_top_left, before_bottom_right)
}

/// Fragment navigation to `#viewId` is the entire reason `<view>` exists: it swaps the referenced resource's
/// effective `viewBox`, changing what is actually rendered — not just a DOM attribute somewhere.
///
/// SVG 2 activates this behaviour only when the SVG resource itself is the document being navigated — a standalone
/// SVG document opened directly, or a genuinely external reference into one (`<img src="...#viewId">`, an SVG
/// `<image>`, a hyperlink). It does *not* extend to an inline `<svg>` embedded in an HTML page responding to a
/// same-page anchor click (confirmed by hand while building `demo/view-demo.svg`) — which rules out a same-document
/// link here too, since every test in this suite (this one included) hosts its `SvgRoot` inside an HTML test page,
/// not a standalone SVG document. A self-contained `data:image/svg+xml;base64,...` URI is a genuine external
/// reference as far as the browser's resource-loading and fragment-navigation machinery is concerned, so it
/// exercises the real mechanism without needing a static test fixture, a standalone top-level SVG document, or
/// test-server support.
///
/// This proves the browser mechanism itself works; it does not prove `svg-dom`'s own factories produce markup that
/// exercises it — [`should_switch_rendered_viewport_for_svg_dom_generated_markup`] closes that gap using the exact
/// same assertions against crate-generated, then serialized, markup instead of this hand-written string.
#[wasm_bindgen_test]
async fn should_switch_rendered_viewport_when_navigating_to_view_fragment() -> Result<(), String> {
    // Hand-written markup, not built through svg-dom's own factories: it stands in for an already-exported SVG
    // file (xmlns and all) — one of the two cases <view>'s fragment effect actually applies to (see the doc comment
    // above); the other, a standalone top-level SVG document, isn't reachable from this HTML-hosted test harness.
    const FIXTURE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200" viewBox="0 0 200 200">
        <view id="detail" viewBox="100 100 100 100"/>
        <rect width="100" height="100" fill="rgb(10,20,220)"/>
        <rect x="100" width="100" height="100" fill="rgb(230,90,10)"/>
        <rect y="100" width="100" height="100" fill="rgb(230,210,10)"/>
        <rect x="100" y="100" width="100" height="100" fill="rgb(10,200,60)"/>
    </svg>"#;

    let window = web_sys::window().unwrap();
    let encoded = window.btoa(FIXTURE).map_err(|_| "btoa failed".to_string())?;
    let data_uri = format!("data:image/svg+xml;base64,{encoded}");

    assert_view_fragment_switches_viewport("view-fragment-nav", &data_uri).await
}

/// A serialization regression could in principle pass every DOM-shape test above (`defs.view()`, `build_view()`,
/// `set_view_box()`, id validation, cached-id mutation) *and* [`should_switch_rendered_viewport_when_navigating_to_view_fragment`]
/// independently, if the bug were specifically in how a live tree gets turned into exported markup — nothing else
/// connects those two halves. This test closes that gap: the same four-quadrant fixture is built through
/// `SvgRoot`/`SvgDefs::view`/the shape factories, serialized with `XmlSerializer`, and put through the exact same
/// external-reference pixel comparison.
///
/// `XmlSerializer` is used rather than `Element::outer_html` (as tried during earlier development of this test
/// module) because `outer_html`'s HTML-fragment serialization algorithm omits the `xmlns` declaration a standalone
/// SVG document needs to parse correctly on its own — confirmed empirically in both Chromium and Firefox.
/// `XmlSerializer::serialize_to_string` produces genuine XML and includes it.
#[wasm_bindgen_test]
async fn should_switch_rendered_viewport_for_svg_dom_generated_markup() -> Result<(), String> {
    let svg = make_svg("view-fragment-nav-source");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let detail = defs.view("detail").map_err(|e| e.to_string())?;
    detail.set_view_box(100.0, 100.0, 100.0, 100.0).map_err(|e| e.to_string())?;
    // `make_svg` defaults to a 400x300 viewport; without matching it to the 200x200 viewBox below, the mismatched
    // intrinsic aspect ratio (4:3 vs 1:1) would letterbox the serialized SVG when it is scaled down as an image,
    // shifting the quadrants away from where the fixed (20,20)/(180,180) sample points in
    // `assert_view_fragment_switches_viewport` expect them.
    svg.set_viewport(Size::new(200.0, 200.0)).map_err(|e| e.to_string())?;
    svg.set_view_box(0.0, 0.0, 200.0, 200.0).map_err(|e| e.to_string())?;

    for (top_left, fill) in [
        (Point::new(0.0, 0.0), "rgb(10,20,220)"),
        (Point::new(100.0, 0.0), "rgb(230,90,10)"),
        (Point::new(0.0, 100.0), "rgb(230,210,10)"),
        (Point::new(100.0, 100.0), "rgb(10,200,60)"),
    ] {
        svg.rect(top_left, Size::new(100.0, 100.0))
            .map_err(|e| e.to_string())?
            .set_fill(fill)
            .map_err(|e| e.to_string())?;
    }

    let serializer = web_sys::XmlSerializer::new().map_err(|e| format!("{e:?}"))?;
    let markup = serializer.serialize_to_string(&svg.root).map_err(|e| format!("{e:?}"))?;

    let window = web_sys::window().unwrap();
    let encoded = window.btoa(&markup).map_err(|_| "btoa failed".to_string())?;
    let data_uri = format!("data:image/svg+xml;base64,{encoded}");

    assert_view_fragment_switches_viewport("view-fragment-nav-generated", &data_uri).await
}
