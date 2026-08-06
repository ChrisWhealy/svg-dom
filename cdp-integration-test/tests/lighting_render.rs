//! Chrome-DevTools-Protocol (CDP) integration test for `demo_lighting.rs`'s own surfaceScale and azimuth sliders.
//!
//! `demo-app/src/browser_tests/paint/lighting.rs` proves the DOM half of both sliders' own claims — that moving
//! either one reaches `surfaceScale` on all four retained lighting primitives, or `azimuth` on all four
//! `<feDistantLight>` children. It cannot prove either mutation actually changes the *rendered* lighting. A
//! structural test is satisfied by a `<feDiffuseLighting surfaceScale="0">` sitting in a filter chain that
//! rasterises however it likes.
//!
//! This drives a real Chrome instance via CDP and renders three circles built by the sibling `cdp-test-fixture`
//! wasm crate, all filtered through the exact same `feDiffuseLighting` recipe `demo_lighting.rs`'s own
//! "diffuse-only" column uses: `#lighting-reference` (surfaceScale 6, azimuth 235deg, this demo's own defaults),
//! `#lighting-azimuth-90` (surfaceScale 6, azimuth 90deg instead), and `#lighting-scale-zero` (surfaceScale 0,
//! azimuth 235deg). It samples eight points around each circle's own rim, 2px inside its nominal radius — where
//! `feDiffuseLighting`'s own alpha-gradient bump map is non-flat, since a plain circle's own interior alpha never
//! varies — and asserts:
//!
//! - `#lighting-reference`'s own eight rim samples vary meaningfully from one another — the positive control that
//!   this fixture and sampling method actually detect real directional lighting when it is there;
//! - `#lighting-scale-zero`'s own eight rim samples stay close to one another, well inside the variation
//!   `#lighting-reference` shows — `panel-lighting.html`'s own claim that surfaceScale 0 flattens the bump map
//!   into a uniform lit surface, checked against real pixels rather than only the `surfaceScale="0"` attribute;
//! - at least one of the eight matching-angle samples differs materially between `#lighting-reference` and
//!   `#lighting-azimuth-90` — azimuth actually turns the rendered light, not just the `<feDistantLight>`
//!   attribute a DOM test can already see.
//!
//! The first check alone would be a one-sided claim about the sampling method's own sensitivity: it says nothing
//! about whether a *flattened* bump map also rasterises correctly, which is exactly what the second check closes.
//! Together they mean a pass on the second check reflects the fixture genuinely losing directional variation, not
//! an insensitive sampling method that would have missed variation either way.
//!
//! This intentionally does not attempt broader screenshot testing across every slider position — these three
//! fixed points are enough to cover both sliders' own specific rendering claims without turning into a fragile
//! visual regression suite.
//!
//! # How the pixels are read
//!
//! Same technique as `filter_blend_render.rs`/`turbulence_scale_zero_render.rs`: serialise the fixture's `<svg>`
//! to a `data:image/svg+xml` URL, load it into an `Image`, draw it to an offscreen `<canvas>`, and read pixels
//! back via `getImageData`. That script is asynchronous (`Image` loading is not synchronous), so it runs via
//! `Runtime.evaluate` with `awaitPromise: true` and `returnByValue: true`, called directly rather than through
//! `headless_chrome::Tab`'s own `evaluate()` wrapper — that wrapper hardcodes `returnByValue: false`, which only
//! inlines primitive results, not the object this script resolves with.
//!
//! # Why this is a separate test file
//!
//! See `filter_blend_render.rs`'s own module doc comment for the general reasoning: keeping each file honestly
//! scoped to what it actually tests, at the cost of each paying Chrome's startup cost independently, since
//! `tests/*.rs` files are always separate binaries with no way to share a running `Browser`/`Tab` instance.

use std::time::Duration;

use cdp_integration_test::{build_fixture, fixture_dir, launch_browser, serve};
use headless_chrome::protocol::cdp::Runtime;
use serde_json::Value;

/// The in-page async script: rasterises the fixture's `<svg>` and returns
/// `{ referenceSamples: [[r,g,b,a], ...], azimuth90Samples: [[r,g,b,a], ...], scaleZeroSamples: [[r,g,b,a], ...] }`,
/// eight samples each, for the `#lighting-reference`, `#lighting-azimuth-90`, and `#lighting-scale-zero` circles
/// respectively. Each sample sits 2px inside its own circle's own radius, at one of eight angles around it, so
/// index `i` in one array corresponds to the same angle as index `i` in either other.
const SAMPLE_SCRIPT: &str = r#"
(async () => {
    const reference = document.querySelector('#lighting-reference');
    const azimuth90 = document.querySelector('#lighting-azimuth-90');
    const scaleZero = document.querySelector('#lighting-scale-zero');
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
    const inset = 2;
    function rimSamples(geo) {
        return angles.map((deg) => {
            const rad = (deg * Math.PI) / 180;
            return pixelAt(geo.cx + (geo.r - inset) * Math.cos(rad), geo.cy + (geo.r - inset) * Math.sin(rad));
        });
    }

    const referenceSamples = rimSamples(circleGeometry(reference));
    const azimuth90Samples = rimSamples(circleGeometry(azimuth90));
    const scaleZeroSamples = rimSamples(circleGeometry(scaleZero));

    URL.revokeObjectURL(url);
    return { referenceSamples, azimuth90Samples, scaleZeroSamples };
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

/// The largest single-channel gap between two samples.
fn max_component_diff(a: (u8, u8, u8, u8), b: (u8, u8, u8, u8)) -> u8 {
    [a.0.abs_diff(b.0), a.1.abs_diff(b.1), a.2.abs_diff(b.2), a.3.abs_diff(b.3)]
        .into_iter()
        .max()
        .expect("four elements")
}

/// The largest single-channel gap between any two samples in `samples`, checked against every other sample once.
fn max_spread(samples: &[(u8, u8, u8, u8)]) -> u8 {
    let mut spread = 0;
    for (i, a) in samples.iter().enumerate() {
        for b in &samples[i + 1..] {
            spread = spread.max(max_component_diff(*a, *b));
        }
    }
    spread
}

#[test]
fn lighting_sliders_change_rendered_pixels() {
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
    let azimuth_90_samples = rgba_list(&value["azimuth90Samples"], "azimuth90Samples");
    let scale_zero_samples = rgba_list(&value["scaleZeroSamples"], "scaleZeroSamples");

    assert_eq!(reference_samples.len(), 8, "expected 8 rim samples for the reference circle");
    assert_eq!(azimuth_90_samples.len(), 8, "expected 8 rim samples for the azimuth-90 circle");
    assert_eq!(scale_zero_samples.len(), 8, "expected 8 rim samples for the scale-zero circle");

    // --- positive control: real directional lighting at a non-zero surfaceScale should vary meaningfully around
    // the rim, not sit flat like an unlit silhouette. Without this, a low `max_spread` on the scale-zero circle
    // below would be equally consistent with a correctly flattened bump map and with a sampling method too coarse
    // to see any lighting variation at all. Calibrated against what this sandbox's own headless Chrome actually
    // renders for this fixture, the same way turbulence_scale_zero_render.rs's own thresholds are: not every
    // angle lands inside the bump map's own narrow rim-gradient band (a few sample points fall back on the flat
    // interior value instead), but the angles that do land inside it swing all the way from full black to a
    // bright grey, an observed spread of 234 against this threshold of 20. ---
    let reference_spread = max_spread(&reference_samples);
    const MIN_REFERENCE_SPREAD: u8 = 20;
    assert!(
        reference_spread >= MIN_REFERENCE_SPREAD,
        "expected the reference circle's own 8 rim samples to vary by at least {MIN_REFERENCE_SPREAD} between \
         its own brightest and darkest points, but the largest gap found was only {reference_spread} — either \
         directional lighting is not reaching the page, or this sampling method cannot detect it, which would \
         silently undermine the surfaceScale-zero check below"
    );

    // --- surfaceScale 0 flattens the bump map: panel-lighting.html's own claim, checked against real pixels. The
    // threshold sits well under MIN_REFERENCE_SPREAD, so a bump map that only partly flattened would still fail
    // this. The observed spread in this sandbox is exactly 0: all eight samples rasterise to the identical
    // (234, 234, 234, 255), the same flat-normal value a few of the reference circle's own angles happen to
    // land on too. ---
    let scale_zero_spread = max_spread(&scale_zero_samples);
    const MAX_SCALE_ZERO_SPREAD: u8 = 8;
    assert!(
        scale_zero_spread <= MAX_SCALE_ZERO_SPREAD,
        "expected the scale-zero circle's own 8 rim samples to stay within {MAX_SCALE_ZERO_SPREAD} of one \
         another (a uniformly lit flat surface), but the largest gap found was {scale_zero_spread} — surfaceScale \
         0 should flatten the bump map entirely, leaving no rim-to-rim variation for a non-zero surfaceScale to \
         create"
    );

    // --- azimuth 90 actually turns the rendered light: at least one of the 8 matching-angle samples should
    // differ materially from the reference circle's own azimuth-235 rendering. A conservative threshold, well
    // clear of ordinary canvas rasterisation rounding. The eight observed per-angle differences in this sandbox
    // are [39, 213, 70, 8, 0, 0, 0, 21] — four clear this threshold with room to spare, and the four that do not
    // are exactly the angles whose own reference sample already sat on the flat interior value rather than
    // inside the rim-gradient band. ---
    const AZIMUTH_CHANGE_THRESHOLD: u8 = 20;
    let differing_samples = reference_samples
        .iter()
        .zip(azimuth_90_samples.iter())
        .filter(|(reference, azimuth_90)| max_component_diff(**reference, **azimuth_90) > AZIMUTH_CHANGE_THRESHOLD)
        .count();
    assert!(
        differing_samples >= 1,
        "expected at least one of the 8 rim samples to differ from the reference circle by more than \
         {AZIMUTH_CHANGE_THRESHOLD} at azimuth 90, but none did — either turning the light is not reaching the \
         page, or this fixture/sampling method cannot detect it"
    );
}
