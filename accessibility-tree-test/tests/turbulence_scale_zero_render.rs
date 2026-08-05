//! Chrome-DevTools-Protocol (CDP) integration test for `SvgFilter::displacement_map`'s `scale` argument at `0.0`.
//!
//! The demo gallery's own turbulence panel (`demo/panels/panel-turbulence.html`) prominently states that scale 0
//! restores a perfect geometric circle. `demo-app/src/browser_tests/paint/turbulence.rs` proves the DOM half of
//! that claim — the scale slider does reach `scale="0"` on the real `feDisplacementMap` element — but it cannot
//! prove the circle actually *renders* as a perfect circle at that value, since `wasm-bindgen-test`'s WebDriver-run
//! tests have no access to rasterised output. A structural test is satisfied by a `scale="0"` attribute sitting on
//! a filter chain that renders however it likes.
//!
//! This drives a real Chrome instance via CDP and renders two circles built by the sibling `a11y-fixture` wasm
//! crate: `#turbulence-reference` (a plain, unfiltered circle) and `#turbulence-scale-zero` (the same position,
//! radius, and fill, but passed through `turbulence` -> `displacement_map` with `scale` fixed at `0.0`). It samples
//! eight points around each circle's own boundary, at matching angles, 3px inside and 3px outside the nominal
//! radius, and asserts the two circles rasterise to the same pixel values there, within a small antialiasing
//! tolerance. A real displacement would show up at these points first: even a small non-zero `scale` shifts the
//! edge by up to `scale / 2` pixels, far past a 3px margin.
//!
//! Samples are not taken exactly on the mathematical radius. That knife-edge pixel is roughly half-covered by
//! design, so its exact rasterised value is unusually sensitive to any small positional difference between the
//! two circles — the same reason `filter_blend_render.rs` samples its own corner pixel 2px inset from the shape's
//! edge rather than exactly on it. In this sandbox specifically, that turned out to matter for a second, more
//! surprising reason too: `a11y-fixture` pins `#turbulence-scale-zero`'s own filter region to exactly the
//! circle's bounding box (`set_x`/`set_y`/`set_width`/`set_height`, all in `objectBoundingBox` units) rather than
//! leaving it at SVG's own default 10%-margin region. Left at that default, headless Chrome's software rasteriser
//! (`--disable-gpu`, see `launch_browser`'s own `sandbox(false)` reasoning) composited the filtered circle back
//! onto the page with a real, several-pixel positional error — unrelated to `scale`, present even at `0.0`, and
//! large enough on its own to fail this test's boundary samples unpredictably from one run to the next. See
//! `a11y-fixture/src/lib.rs`'s own comment on this scenario for the full account of tracking that down.
//!
//! This intentionally does not attempt broad screenshot testing across every slider position — a single identity
//! test at scale zero is enough to cover this specific, exact semantic claim without turning into a fragile visual
//! regression suite.
//!
//! # How the pixels are read
//!
//! Same technique as `filter_blend_render.rs`: serialise the fixture's `<svg>` to a `data:image/svg+xml` URL, load
//! it into an `Image`, draw it to an offscreen `<canvas>`, and read pixels back via `getImageData`. That script is
//! asynchronous (`Image` loading is not synchronous), so it runs via `Runtime.evaluate` with `awaitPromise: true`
//! and `returnByValue: true`, called directly rather than through `headless_chrome::Tab`'s own `evaluate()`
//! wrapper — that wrapper hardcodes `returnByValue: false`, which only inlines primitive results, not the object
//! this script resolves with.
//!
//! # Why this is a separate test file
//!
//! See `filter_blend_render.rs`'s own module doc comment for the general reasoning: keeping each file honestly
//! scoped to what it actually tests, at the cost of each paying Chrome's startup cost independently, since
//! `tests/*.rs` files are always separate binaries with no way to share a running `Browser`/`Tab` instance.

use std::time::Duration;

use accessibility_tree_test::{build_fixture, fixture_dir, launch_browser, serve};
use headless_chrome::protocol::cdp::Runtime;
use serde_json::Value;

/// The in-page async script: rasterises the fixture's `<svg>` and returns
/// `{ referenceSamples: [[r,g,b,a], ...], scaleZeroSamples: [[r,g,b,a], ...] }`, sixteen samples each, for the
/// `#turbulence-reference` and `#turbulence-scale-zero` circles respectively. Each sample sits 3px inside or 3px
/// outside its own circle's own radius, at one of eight angles around it (inside samples first, then outside, in
/// the same angle order), so index `i` in one array corresponds to the same offset and angle as index `i` in the
/// other.
const SAMPLE_SCRIPT: &str = r#"
(async () => {
    const reference = document.querySelector('#turbulence-reference');
    const scaleZero = document.querySelector('#turbulence-scale-zero');
    const svg = reference.closest('svg');
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

    function pixelAt(x, y) {
        const d = ctx.getImageData(Math.round(x), Math.round(y), 1, 1).data;
        return [d[0], d[1], d[2], d[3]];
    }

    function circleGeometry(el) {
        return {
            cx: parseFloat(el.getAttribute('cx')),
            cy: parseFloat(el.getAttribute('cy')),
            r: parseFloat(el.getAttribute('r')),
        };
    }

    const angles = [0, 45, 90, 135, 180, 225, 270, 315];
    const inset = 3;
    function insetSamples(geo) {
        const inside = angles.map((deg) => {
            const rad = (deg * Math.PI) / 180;
            return pixelAt(geo.cx + (geo.r - inset) * Math.cos(rad), geo.cy + (geo.r - inset) * Math.sin(rad));
        });
        const outside = angles.map((deg) => {
            const rad = (deg * Math.PI) / 180;
            return pixelAt(geo.cx + (geo.r + inset) * Math.cos(rad), geo.cy + (geo.r + inset) * Math.sin(rad));
        });
        return inside.concat(outside);
    }

    const referenceSamples = insetSamples(circleGeometry(reference));
    const scaleZeroSamples = insetSamples(circleGeometry(scaleZero));

    URL.revokeObjectURL(url);
    return { referenceSamples, scaleZeroSamples };
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

/// A JSON array of `[r, g, b, a]` arrays, decoded via [`rgba`].
fn rgba_list(value: &Value, context: &str) -> Vec<(u8, u8, u8, u8)> {
    value
        .as_array()
        .unwrap_or_else(|| panic!("{context}: expected an array, got {value}"))
        .iter()
        .enumerate()
        .map(|(i, v)| rgba(v, &format!("{context}[{i}]")))
        .collect()
}

#[test]
fn turbulence_scale_zero_matches_unfiltered_circle_at_every_boundary_sample() {
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

    let reference_samples = rgba_list(&value["referenceSamples"], "referenceSamples");
    let scale_zero_samples = rgba_list(&value["scaleZeroSamples"], "scaleZeroSamples");

    assert_eq!(
        reference_samples.len(),
        16,
        "expected 8 inside + 8 outside samples for the reference circle"
    );
    assert_eq!(
        scale_zero_samples.len(),
        16,
        "expected 8 inside + 8 outside samples for the scale-zero circle"
    );

    // +/-4 per channel for canvas rasterisation rounding — not for any real uncertainty about the claim. Every
    // sample point sits 3px clear of the nominal edge (see the module doc comment for why exactly-on-the-edge
    // samples are unusable here), so a genuinely displaced edge would flip a sample from solid fill to fully
    // transparent (or vice versa) rather than miss by single digits — categorically larger than rounding noise.
    let close = |a: u8, b: u8| a.abs_diff(b) <= 4;
    for (i, (reference, scale_zero)) in reference_samples.iter().zip(scale_zero_samples.iter()).enumerate() {
        assert!(
            close(reference.0, scale_zero.0)
                && close(reference.1, scale_zero.1)
                && close(reference.2, scale_zero.2)
                && close(reference.3, scale_zero.3),
            "sample {i}: reference circle {reference:?} vs scale-zero circle {scale_zero:?} differ by more than \
             the antialiasing tolerance — scale 0 should rasterise identically to an unfiltered circle, not \
             produce a visibly different edge"
        );
    }
}
