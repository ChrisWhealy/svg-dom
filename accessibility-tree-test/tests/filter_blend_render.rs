//! Chrome-DevTools-Protocol (CDP) integration test for `SvgFilter::blend`'s alpha-preserving tint chain.
//!
//! `tests/filter.rs` (in the main `svg-dom` crate) proves DOM structure for `SvgFilter::blend` and `composite`:
//! the right elements, with the right attributes, in the right order. It cannot prove what those elements actually
//! *render* — and the whole point of the `flood` -> `blend` -> `composite(In)` tint chain documented on
//! `SvgFilter::blend` is a rendering claim: that the chain preserves the source graphic's own transparency instead
//! of leaking the flood colour into it. A structural test that only counts child elements is satisfied by a chain
//! that gets this wrong, which is exactly what happened before that final `composite(In)` step was added — see the
//! `SvgFilter::blend` doc comment's `# ⚠️ Tinting with a flood colour needs a final composite(In)` section and
//! `docs/svg_elements/filters.md`'s matching warning for the full explanation of the bug this test guards against.
//!
//! This drives a real Chrome instance via CDP and renders the `#blend-circle` element built by the sibling
//! `a11y-fixture` wasm crate (a white circle, filtered with `flood("#f0883e", ...)` -> `blend(Multiply)` ->
//! `composite(In)`) to an offscreen canvas, then reads back actual pixel values to confirm:
//!
//! - a pixel at the circle's centre is fully opaque and (approximately) the flood colour — white is `Multiply`'s
//!   identity element, so a correctly alpha-preserving chain paints the flood colour through unchanged, giving an
//!   *exact* expected result rather than an approximate one;
//! - a pixel at a corner of the circle's bounding box — outside the circle, where `SourceGraphic` is fully
//!   transparent — is fully transparent (alpha `0`). Before the `composite(In)` fix, this pixel was opaque and
//!   flood-coloured instead, because `flood` paints across the *entire* filter region regardless of the source
//!   graphic's shape, and `feBlend`'s result alpha is the union of its two inputs' alpha.
//!
//! # How the pixels are read
//!
//! There is no direct "read this SVG's rendered pixels" CDP call, so the in-page JavaScript below serialises the
//! fixture's `<svg>` to a `data:image/svg+xml` URL, loads it into an `Image`, draws it to an offscreen `<canvas>`,
//! and reads two pixels back via `getImageData` — the standard technique for rasterising SVG content in a browser.
//! That script is itself asynchronous (`Image` loading is not synchronous), so it runs via `Runtime.evaluate` with
//! `awaitPromise: true` and `returnByValue: true`, called directly rather than through `headless_chrome::Tab`'s own
//! `evaluate()` wrapper — that wrapper hardcodes `returnByValue: false`, which only inlines primitive results, not
//! the object this script resolves with.
//!
//! # Why this is a separate test file, not more `#[test]`s in `accessibility_tree.rs`
//!
//! Bolting these tests onto the accessibility-tree file would let one file's module doc comment describe two
//! unrelated concerns (accessible-name computation and filter alpha compositing), and would make this test
//! non-obvious to find. The cost is that this file builds and launches its own fixture/Chrome instance independent
//! of `accessibility_tree.rs`'s (`tests/*.rs` files are always separate binaries — there is no way to share a
//! running `Browser`/`Tab` across them, only the setup code in `src/lib.rs` that creates one), so `cargo test -p
//! accessibility-tree-test` pays Chrome's startup cost twice. That is judged worth it for keeping each file
//! honestly scoped to what it actually tests.

use std::time::Duration;

use accessibility_tree_test::{build_fixture, fixture_dir, launch_browser, serve};
use headless_chrome::protocol::cdp::Runtime;
use serde_json::Value;

/// The in-page async script: rasterises the fixture's `<svg>` and returns `{ center: [r,g,b,a], corner: [r,g,b,a] }`
/// for the `#blend-circle` element, sampling its centre and a corner of its bounding box (inset by 2px so the
/// sample point is never itself antialiased edge pixel).
const SAMPLE_SCRIPT: &str = r#"
(async () => {
    const circle = document.querySelector('#blend-circle');
    const svg = circle.closest('svg');
    const xml = new XMLSerializer().serializeToString(svg);
    const blob = new Blob([xml], { type: 'image/svg+xml' });
    const url = URL.createObjectURL(blob);
    const img = new Image();
    const loaded = new Promise((resolve, reject) => {
        img.onload = resolve;
        img.onerror = reject;
    });
    img.src = url;
    await loaded;

    const canvas = document.createElement('canvas');
    canvas.width = img.width;
    canvas.height = img.height;
    const ctx = canvas.getContext('2d');
    ctx.drawImage(img, 0, 0);

    const cx = parseFloat(circle.getAttribute('cx'));
    const cy = parseFloat(circle.getAttribute('cy'));
    const r = parseFloat(circle.getAttribute('r'));

    function pixelAt(x, y) {
        const d = ctx.getImageData(Math.round(x), Math.round(y), 1, 1).data;
        return [d[0], d[1], d[2], d[3]];
    }

    const center = pixelAt(cx, cy);
    const corner = pixelAt(cx - r + 2, cy - r + 2);

    URL.revokeObjectURL(url);
    return { center, corner };
})()
"#;

/// Component-wise `(r, g, b, a)` from a JSON `[r, g, b, a]` array, panicking with `context` on any malformed value.
fn rgba(value: &Value, context: &str) -> (u8, u8, u8, u8) {
    let arr = value
        .as_array()
        .unwrap_or_else(|| panic!("{context}: expected a 4-element array, got {value}"));
    let component = |i: usize| {
        arr.get(i)
            .and_then(Value::as_u64)
            .unwrap_or_else(|| panic!("{context}: expected a numeric component at index {i}, got {value}"))
            as u8
    };
    (component(0), component(1), component(2), component(3))
}

#[test]
fn blend_tint_chain_preserves_source_alpha() {
    let dir = fixture_dir();
    build_fixture(&dir);
    let port = serve(dir);

    let browser = launch_browser().expect("failed to launch Chrome — is it installed locally?");
    let tab = browser.new_tab().expect("failed to open a new tab");
    tab.navigate_to(&format!("http://127.0.0.1:{port}/index.html"))
        .expect("failed to navigate to fixture page");
    tab.wait_for_element_with_custom_timeout("#fixture-ready", Duration::from_secs(10))
        .expect("fixture did not signal readiness in time");

    let evaluated = tab
        .call_method(Runtime::Evaluate {
            expression: SAMPLE_SCRIPT.to_owned(),
            object_group: None,
            include_command_line_api: None,
            silent: None,
            context_id: None,
            return_by_value: Some(true),
            generate_preview: None,
            user_gesture: None,
            await_promise: Some(true),
            throw_on_side_effect: None,
            timeout: None,
            disable_breaks: None,
            repl_mode: None,
            allow_unsafe_eval_blocked_by_csp: None,
            unique_context_id: None,
            serialization_options: None,
        })
        .expect("Runtime.evaluate failed");

    if let Some(exception) = evaluated.exception_details {
        panic!("pixel-sampling script threw: {exception:?}");
    }
    let value = evaluated
        .result
        .value
        .expect("evaluate did not return a value — was returnByValue set?");

    let center = rgba(&value["center"], "center pixel");
    let corner = rgba(&value["corner"], "corner pixel");

    // The flood colour is #f0883e = (240, 136, 62). White is Multiply's identity element, so a correctly
    // alpha-preserving chain paints it through completely unchanged inside the circle — allow +/-2 per channel for
    // canvas rasterisation rounding, not for any real uncertainty about the expected colour.
    let close = |actual: u8, expected: u8| actual.abs_diff(expected) <= 2;
    assert!(
        close(center.0, 240) && close(center.1, 136) && close(center.2, 62) && center.3 == 255,
        "expected the circle's centre to be opaque and (approximately) the flood colour (240, 136, 62, 255), got {center:?}"
    );

    // Outside the circle but inside its bounding box, SourceGraphic is fully transparent. Before the composite(In)
    // fix, this pixel was opaque and flood-coloured (the exact bug this test exists to catch); it must now be
    // fully transparent.
    assert_eq!(
        corner,
        (0, 0, 0, 0),
        "expected a corner of the circle's bounding box to be fully transparent (the composite(In) step must clip \
         the flood back to the source's own alpha coverage), got {corner:?} — the flood colour is leaking through \
         wherever SourceGraphic is transparent"
    );
}
