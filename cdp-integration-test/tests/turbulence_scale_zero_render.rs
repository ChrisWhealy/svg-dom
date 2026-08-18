//! Chrome-DevTools-Protocol (CDP) integration test for `SvgFilter::displacement_map`'s `scale` argument at `0.0`.
//!
//! The demo gallery's own turbulence panel (`demo/panels/panel-turbulence.html`) prominently states that scale 0
//! restores a perfect geometric circle. `demo-app/src/browser_tests/paint/turbulence.rs` proves the DOM half of that
//! claim: that is, the scale slider does reach `scale="0"` on the real `feDisplacementMap` element. However, it cannot
//! prove the circle actually *renders* as a perfect circle at that value, since `wasm-bindgen-test`'s WebDriver-run
//! tests have no access to rasterised output.
//!
//! A structural test is satisfied by a `scale="0"` attribute sitting on a filter chain that could end up being renders
//! quite incorrectly.
//!
//! This test drives a real Chrome instance via CDP and renders three circles built by the sibling `cdp-test-fixture`
//! wasm crate:
//! * `#turbulence-reference` (a plain, unfiltered circle),
//! * `#turbulence-scale-zero` (passed through `turbulence` -> `displacement_map` with `scale` fixed at `0.0`), and
//! * `#turbulence-scale-sixty` (the same chain again, with `scale` fixed at `60.0` - the documented maximum).
//!
//! All three use the same radius and fill, but their centres differ, so samples are always taken at corresponding
//! offsets around each circle's own centre, not at shared absolute coordinates. It samples eight points around each
//! circle's own boundary, at matching angles, 3px inside and 3px outside the nominal radius and then asserts:
//!
//! 1. the reference and scale-zero circles rasterise to the same pixel values at every sample, within a small
//!   antialiasing tolerance.  The negative control is that zero displacement should look unchanged;
//! 2. the reference and scale-sixty circles rasterise to a *materially different* pixel value at, conservatively,
//!   at least one sample.  The positive control is that a real, substantial displacement should look visibly different.
//!
//! Both checks matter together. The first alone is a one-sided claim: it is equally consistent with a correctly working
//! filter chain and with a browser that silently ignores the filter (or fell back to unfiltered `SourceGraphic`), since
//! either would also rasterise like the reference.
//!
//! The second proves this fixture and sampling method can actually detect a real displacement in the first place, so a
//! pass on the first check means what it claims rather than reflecting an insensitive test.
//!
//! The maximum displacement along either axis is `scale / 2` (30px at scale 60) but that is a ceiling, not a guarantee:
//! the actual displacement at any one sampled point depends on the local turbulence channel value there, not on `scale`
//! alone. Below scale 6, `scale / 2` itself is under the 3px sample margin either check uses, and even a much larger
//! scale need not reach its own maximum at any particular sample.
//!
//! Scale 60 was chosen and the threshold below calibrated against what this sandbox's own headless Chrome actually
//! renders at that scale.  This is an observed property of this fixture, not an assumed one.  See the positive
//! control's own comment further down for the measured numbers.
//!
//! Samples are not taken exactly on the mathematical radius. That knife-edge pixel is roughly half-covered by design,
//! so its exact rasterised value is unusually sensitive to any small positional difference between the two circles.
//! This is the same reason `filter_blend_render.rs` samples its own corner pixel 2px inset from the shape's edge rather
//! than exactly on it. In this sandbox specifically, that turned out to matter for a second, more surprising reason
//! too: `cdp-test-fixture` pins `#turbulence-scale-zero`'s own filter region to exactly the circle's bounding box
//! (`set_x`, `set_y`, `set_width` and `set_height` all in `objectBoundingBox` units) rather than leaving it at SVG's
//! own default 10%-margin region.
//!
//! If left at that default, headless Chrome's software rasteriser (run using `--disable-gpu`, see `launch_browser`'s
//! own `sandbox(false)` reasoning) composited the filtered circle back onto the page with a real, several-pixel
//! positional error which was unrelated to `scale`.  This error was present even at `0.0`, and large enough on its own
//! to fail this test's boundary samples unpredictably from one run to the next.
//!
//! See `cdp-test-fixture/src/lib.rs`'s own comment on this scenario for the full account of tracking down that little
//! gremlin.
//!
//! No attempt is made for broad screenshot testing across every slider position — a single identity test at scale zero
//! is enough to cover this specific, exact semantic claim without turning into a fragile visual regression suite.
//!
//! # How the pixels are read
//!
//! Same technique as `filter_blend_render.rs`: serialise the fixture's `<svg>` to a `data:image/svg+xml` URL, load
//! it into an `Image`, draw it to an offscreen `<canvas>` then read pixels back via `getImageData`.
//!
//! Since `Image` loading asynchronous, this script must also be asynchronous, so it runs via `Runtime.evaluate` with
//! `awaitPromise: true` and `returnByValue: true`, called directly, rather than through `headless_chrome::Tab`'s own
//! `evaluate()` wrapper.  That wrapper hardcodes `returnByValue: false`, which only inlines primitive results, not the
//! object by which this script resolves.
//!
//! # Why this is a separate test file
//!
//! See `filter_blend_render.rs`'s own module doc comment for the general reasoning: keeping the scope of test file in
//! its "common sense" location.  This does however come at the cost that each test needs to pay Chrome's startup cost,
//! since `tests/*.rs` files are always separate binaries with no way to share a running `Browser`/`Tab` instance.

use cdp_integration_test::{build_fixture, fixture_dir, launch_browser, serve};
use headless_chrome::protocol::cdp::Runtime;
use serde_json::Value;
use std::time::Duration;

/// The in-page async script: rasterises the fixture's `<svg>` and returns sixteen samples each for the
/// `#turbulence-reference`, `#turbulence-scale-zero`, and `#turbulence-scale-sixty` circles respectively.
/// ```
/// { referenceSamples: [[r,g,b,a], ...],
///   scaleZeroSamples: [[r,g,b,a], ...],
///   scaleSixtySamples: [[r,g,b,a], ...]
/// }
/// ```
/// 
/// Each sample sits 3px inside or 3px outside its own circle's own radius, at one of eight angles around it (inside
/// samples first, then outside, in the same angle order), so index `i` in one array corresponds to the same offset and
/// angle as index `i` in either other.
const SAMPLE_SCRIPT: &str = r#"
(async () => {
    const reference = document.querySelector('#turbulence-reference')
    const scaleZero = document.querySelector('#turbulence-scale-zero')
    const scaleSixty = document.querySelector('#turbulence-scale-sixty')
    const svg = reference.closest('svg')
    const xml = new XMLSerializer().serializeToString(svg)
    const blob = new Blob([xml], { type: 'image/svg+xml' })
    const url = URL.createObjectURL(blob)
    const img = new Image()
    const loaded = new Promise((resolve, reject) => {
        img.onload = resolve
        img.onerror = reject
    })
    img.src = url
    await loaded

    const canvas = document.createElement('canvas')
    canvas.width = img.width
    canvas.height = img.height
    const ctx = canvas.getContext('2d')
    ctx.drawImage(img, 0, 0)

    function pixelAt(x, y) {
        const d = ctx.getImageData(Math.round(x), Math.round(y), 1, 1).data
        return [d[0], d[1], d[2], d[3]]
    }

    function circleGeometry(el) {
        return {
            cx: parseFloat(el.getAttribute('cx')),
            cy: parseFloat(el.getAttribute('cy')),
            r: parseFloat(el.getAttribute('r')),
        };
    }

    const angles = [0, 45, 90, 135, 180, 225, 270, 315]
    const inset = 3
    function insetSamples(geo) {
        const inside = angles.map(deg => {
            const rad = (deg * Math.PI) / 180
            return pixelAt(geo.cx + (geo.r - inset) * Math.cos(rad), geo.cy + (geo.r - inset) * Math.sin(rad))
        })
        const outside = angles.map((deg) => {
            const rad = (deg * Math.PI) / 180
            return pixelAt(geo.cx + (geo.r + inset) * Math.cos(rad), geo.cy + (geo.r + inset) * Math.sin(rad))
        })
        return inside.concat(outside)
    }

    const referenceSamples = insetSamples(circleGeometry(reference))
    const scaleZeroSamples = insetSamples(circleGeometry(scaleZero))
    const scaleSixtySamples = insetSamples(circleGeometry(scaleSixty))

    URL.revokeObjectURL(url)
    return { referenceSamples, scaleZeroSamples, scaleSixtySamples }
})()
"#;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
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

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
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

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn turbulence_scale_zero_matches_reference_while_scale_sixty_visibly_differs() {
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
    let scale_sixty_samples = rgba_list(&value["scaleSixtySamples"], "scaleSixtySamples");

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
    assert_eq!(
        scale_sixty_samples.len(),
        16,
        "expected 8 inside + 8 outside samples for the scale-sixty circle"
    );

    // The largest single-channel gap between two samples — the same metric both the negative and positive
    // control below judge their own threshold against, just on opposite sides of it.
    let max_component_diff = |a: (u8, u8, u8, u8), b: (u8, u8, u8, u8)| {
        [a.0.abs_diff(b.0), a.1.abs_diff(b.1), a.2.abs_diff(b.2), a.3.abs_diff(b.3)]
            .into_iter()
            .max()
            .expect("four elements")
    };

    // --- negative control: zero displacement should rasterise unchanged ---
    //
    // +/-4 per channel for canvas rasterisation rounding — not for any real uncertainty about the claim. Every
    // sample point sits 3px clear of the nominal edge (see the module doc comment for why exactly-on-the-edge
    // samples are unusable here), so a genuinely displaced edge would flip a sample from solid fill to fully
    // transparent (or vice versa) rather than miss by single digits — categorically larger than rounding noise.
    const ANTIALIASING_TOLERANCE: u8 = 4;
    for (i, (reference, scale_zero)) in reference_samples.iter().zip(scale_zero_samples.iter()).enumerate() {
        let diff = max_component_diff(*reference, *scale_zero);
        assert!(
            diff <= ANTIALIASING_TOLERANCE,
            "sample {i}: reference circle {reference:?} vs scale-zero circle {scale_zero:?} differ by {diff}, \
             more than the antialiasing tolerance — scale 0 should rasterise identically to an unfiltered \
             circle, not produce a visibly different edge"
        );
    }

    // --- positive control: a real, substantial displacement should rasterise visibly different ---
    //
    // Without this half, the negative control above is a one-sided claim: a browser that silently ignored the
    // filter entirely, or fell back to unfiltered SourceGraphic, would pass it too, since an ignored filter also
    // rasterises like the reference. This proves the fixture and sampling method above can actually detect a
    // real displacement, so the negative control's own pass means what it claims to mean.
    //
    // A conservative threshold, well clear of both the antialiasing tolerance above and the small cross-pipeline
    // noise `cdp-test-fixture`'s own comment on this scenario describes (a filtered element composites back onto the
    // page slightly differently from an unfiltered one, even with no real displacement) — turbulence rendering
    // can vary a little between runs, so this only requires at least one sample to clear a wide margin, not an
    // exact or universal difference across all sixteen. Measured against this sandbox's own headless Chrome (this
    // fixture's own fixed noise seed makes it deterministic), the sixteen samples' own differences land at either
    // ~1 or 75-255 with nothing in between, so 40 sits in the middle of a wide, empty gap rather than close to
    // either side.

    const DISPLACEMENT_THRESHOLD: u8 = 40;
    let differing_samples = reference_samples
        .iter()
        .zip(scale_sixty_samples.iter())
        .filter(|(reference, scale_sixty)| max_component_diff(**reference, **scale_sixty) > DISPLACEMENT_THRESHOLD)
        .count();
    assert!(
        differing_samples >= 1,
        "expected at least one of the 16 boundary samples to differ from the reference circle by more than {\
         DISPLACEMENT_THRESHOLD} at scale 60, but none did — either a real displacement is not reaching the \
         page, or this fixture/sampling method cannot detect one when it is there, which would silently \
         undermine the negative control above"
    );
}
