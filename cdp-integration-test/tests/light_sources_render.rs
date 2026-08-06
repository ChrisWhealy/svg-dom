//! Chrome-DevTools-Protocol (CDP) integration test for `demo_light_sources.rs`'s own four sliders.
//!
//! `demo-app/src/browser_tests/paint/light_sources.rs` proves the DOM half of all four sliders' own claims —
//! that moving each one reaches the right attribute on the right retained light-source node, and leaves the
//! other three columns untouched. It cannot prove any of those mutations actually change the *rendered*
//! lighting. This matters more here than for most demos: one of the four columns' own designs is itself grounded
//! in observed Chrome rendering rather than the SVG spec alone — `demo_light_sources.rs`'s own module doc
//! comment records that `limiting_cone_angle` `0.0` renders as a fully open beam rather than the near-invisible
//! cutoff the spec describes, which is why that slider's own range starts at `5` instead. A structural DOM test
//! cannot check that claim; only real rendered pixels can.
//!
//! That same module doc comment also records a second observation: the Spot light's own `specular_exponent` had
//! no visible effect in this sandbox's own Chrome, at any value from `0.01` through `10000`, which is why that
//! field was rejected as this demo's own interactive control for the open-Spot column in favour of `x`. This
//! file does not regression-test that observation, deliberately: a fixture asserting that varying
//! `specular_exponent` continues to produce *no* visible change would fail the day Chrome fixes whatever causes
//! that, and that failure would represent an improvement, not a regression. It remains a manually recorded
//! observation in `demo_light_sources.rs`'s own module doc comment, not a claim this file verifies.
//!
//! This drives a real Chrome instance via CDP and renders rects built by the sibling `cdp-test-fixture` wasm
//! crate, grouped into four checks, each running `demo_light_sources.rs`'s own exact `feSpecularLighting` recipe
//! on a plain, flat rect, fixed at different slider positions:
//!
//! - `#ls-distant-low` (elevation 15deg) vs `#ls-distant-high` (elevation 85deg) — checks that a flat Distant
//!   surface's own average brightness genuinely rises with elevation, panel-light-sources.html's own claim.
//! - `#ls-point-low-z` (z 20) vs `#ls-point-high-z` (z 180) — checks that a lower light genuinely sharpens the
//!   hotspot (a bigger centre-to-corner contrast) rather than spreading it.
//! - `#ls-spot-left` (light at its own rect's left edge) vs `#ls-spot-right` (right edge), both with `pointsAtX`
//!   trailing by the same 80-unit offset `demo_light_sources.rs`'s own `SPOT_OPEN_AIM_OFFSET` uses — checks that
//!   the bright region genuinely moves horizontally with the light, not just the DOM attribute.
//! - `#ls-cone-zero` (`limitingConeAngle` 0deg) vs `#ls-cone-narrow` (5deg) vs `#ls-cone-wide` (90deg) — three
//!   rects, not two. `#ls-cone-narrow` vs `#ls-cone-wide` alone only proves 5deg is a usefully narrow lower
//!   bound; it says nothing about 0deg being anomalous. `#ls-cone-zero` checks that specific claim directly: the
//!   same off-axis sample stays dark at 5deg, but is materially brighter, and close to the wide reading, at
//!   0deg — this is the highest-value check of the four, since the slider's own chosen minimum (5, not 0) and
//!   panel-light-sources.html's own explanation of why both depend on this exact, sandbox-specific Chrome
//!   behaviour rather than a specification guarantee.
//!
//! # How the pixels are read
//!
//! Same technique as `lighting_render.rs`: serialise the fixture's `<svg>` to a `data:image/svg+xml` URL, load
//! it into an `Image`, draw it to an offscreen `<canvas>`, and read pixels back via `getImageData`. That script
//! is asynchronous (`Image` loading is not synchronous), so it runs via `Runtime.evaluate` with
//! `awaitPromise: true` and `returnByValue: true`, called directly rather than through `headless_chrome::Tab`'s
//! own `evaluate()` wrapper — that wrapper hardcodes `returnByValue: false`, which only inlines primitive
//! results, not the object this script resolves with.
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

/// The in-page async script: rasterises the fixture's `<svg>` and returns the alpha channel (the intensity
/// signal, since `feSpecularLighting`'s own alpha is the max of its own lit R/G/B — the red channel is not
/// usable here, since a dim, mostly-transparent pixel composites against the canvas's own default white
/// background and reads as bright regardless of its own true intensity) at a handful of named local offsets
/// within each of the nine rects, keyed by rect id then offset name.
const SAMPLE_SCRIPT: &str = r#"
(async () => {
    const ids = [
        'ls-distant-low', 'ls-distant-high',
        'ls-point-low-z', 'ls-point-high-z',
        'ls-spot-left', 'ls-spot-right',
        'ls-cone-zero', 'ls-cone-narrow', 'ls-cone-wide',
    ];
    const rects = Object.fromEntries(ids.map((id) => [id, document.querySelector('#' + id)]));
    const svg = rects['ls-distant-low'].closest('svg');
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

    // feSpecularLighting's own alpha is the max of its own lit R/G/B, so it scales with lit intensity directly.
    // The red channel does not: transparent (dim) pixels composite against the canvas's own default white
    // background, reading as bright regardless of how dim the underlying lit colour actually is.
    function alphaAt(x, y) {
        return ctx.getImageData(Math.round(x), Math.round(y), 1, 1).data[3];
    }

    function rectOrigin(el) {
        return { x: parseFloat(el.getAttribute('x')), y: parseFloat(el.getAttribute('y')) };
    }

    // Named local offsets, shared across every rect (each is 160x100): centre, four inset corners, and a point
    // near where the Spot columns' own aim point lands.
    const offsets = {
        centre: [80, 50],
        topLeft: [15, 15],
        topRight: [145, 15],
        bottomLeft: [15, 85],
        bottomRight: [145, 85],
        nearAim: [110, 80],
        // The Spot columns' own aim point lands at exactly this local offset (pointsAtX/pointsAtY, relative to
        // each rect's own origin) — the one point on the surface exactly on the beam's own aim axis.
        exactAim: [120, 90],
        // The open Spot columns' own light sits at local y 20, not the rect's own mid-height, so these two sit
        // level with it instead, close to each side's own edge.
        leftAtLightHeight: [20, 20],
        rightAtLightHeight: [140, 20],
    };

    const samples = {};
    for (const id of ids) {
        const origin = rectOrigin(rects[id]);
        samples[id] = {};
        for (const [name, [dx, dy]] of Object.entries(offsets)) {
            samples[id][name] = alphaAt(origin.x + dx, origin.y + dy);
        }
    }

    URL.revokeObjectURL(url);
    return samples;
})()
"#;

/// `samples[id][offset_name]` as a plain `u8`, panicking with context on any malformed value.
fn sample(value: &Value, id: &str, offset_name: &str) -> u8 {
    value[id][offset_name]
        .as_u64()
        .unwrap_or_else(|| panic!("samples[{id:?}][{offset_name:?}] missing or not a number: {value}")) as u8
}

#[test]
fn light_sources_sliders_change_rendered_pixels() {
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

    // --- Distant: average brightness (five samples) should rise clearly with elevation ---
    let distant_avg = |id: &str| -> f64 {
        let names = ["centre", "topLeft", "topRight", "bottomLeft", "bottomRight"];
        names.iter().map(|n| f64::from(sample(&value, id, n))).sum::<f64>() / names.len() as f64
    };
    let distant_low_avg = distant_avg("ls-distant-low");
    let distant_high_avg = distant_avg("ls-distant-high");
    const MIN_DISTANT_BRIGHTNESS_GAP: f64 = 20.0;
    assert!(
        distant_high_avg - distant_low_avg >= MIN_DISTANT_BRIGHTNESS_GAP,
        "expected elevation 85deg to average at least {MIN_DISTANT_BRIGHTNESS_GAP} brighter than elevation \
         15deg, but got {distant_low_avg:.1} (low) vs {distant_high_avg:.1} (high) — panel-light-sources.html's \
         own claim that raising elevation brightens the uniform sheen"
    );

    // --- Point: centre-to-corner contrast should be bigger (a sharper hotspot) at low z than high z ---
    let point_contrast =
        |id: &str| -> i32 { i32::from(sample(&value, id, "centre")) - i32::from(sample(&value, id, "topLeft")) };
    let point_low_z_contrast = point_contrast("ls-point-low-z");
    let point_high_z_contrast = point_contrast("ls-point-high-z");
    const MIN_POINT_CONTRAST_GAP: i32 = 15;
    assert!(
        point_low_z_contrast - point_high_z_contrast >= MIN_POINT_CONTRAST_GAP,
        "expected z 20's own centre-to-corner contrast to exceed z 180's own by at least \
         {MIN_POINT_CONTRAST_GAP}, but got {point_low_z_contrast} (low z) vs {point_high_z_contrast} (high z) — \
         a lower light should sharpen the hotspot, not spread it"
    );

    // Spot (no cone): the bright region should move with the light, not stay fixed. Compared within each
    // rect (its own side nearest the light against its own side farthest from it), not across the two rects at
    // the same absolute point — the two lights' own reflection peaks are not equally strong at their own
    // opposite edges, an asymmetry of this particular geometry, not a sign either rect is failing to track its
    // own light. The direction each rect's own bright region leans is the thing being checked here.
    let spot_left_bias = i32::from(sample(&value, "ls-spot-left", "leftAtLightHeight"))
        - i32::from(sample(&value, "ls-spot-left", "rightAtLightHeight"));
    let spot_right_bias = i32::from(sample(&value, "ls-spot-right", "rightAtLightHeight"))
        - i32::from(sample(&value, "ls-spot-right", "leftAtLightHeight"));
    const MIN_SPOT_BIAS: i32 = 20;
    assert!(
        spot_left_bias >= MIN_SPOT_BIAS,
        "expected the light-left rect's own left-edge sample to exceed its own right-edge sample by at least \
         {MIN_SPOT_BIAS}, but got a bias of only {spot_left_bias}"
    );
    assert!(
        spot_right_bias >= MIN_SPOT_BIAS,
        "expected the light-right rect's own right-edge sample to exceed its own left-edge sample by at least \
         {MIN_SPOT_BIAS}, but got a bias of only {spot_right_bias}"
    );

    // Spot (with cone): a sample well off the beam's own aim axis should stay dark under the narrow cone,
    // but light up once the cone widens. This is the check that backs the slider's own chosen minimum (5, not
    // 0) and panel-light-sources.html's own explanation of why.
    let cone_narrow_off_axis = sample(&value, "ls-cone-narrow", "topLeft");
    let cone_wide_off_axis = sample(&value, "ls-cone-wide", "topLeft");
    const MAX_NARROW_OFF_AXIS_BRIGHTNESS: u8 = 30;
    const MIN_CONE_WIDENING_GAP: i32 = 15;
    assert!(
        cone_narrow_off_axis <= MAX_NARROW_OFF_AXIS_BRIGHTNESS,
        "expected the narrow cone (5deg) to leave a sample well off its own aim axis at or below \
         {MAX_NARROW_OFF_AXIS_BRIGHTNESS}, but got {cone_narrow_off_axis} — the hard-edged cutoff should exclude \
         it almost entirely"
    );
    assert!(
        i32::from(cone_wide_off_axis) - i32::from(cone_narrow_off_axis) >= MIN_CONE_WIDENING_GAP,
        "expected widening the cone from 5deg to 90deg to brighten that same off-axis sample by at least \
         {MIN_CONE_WIDENING_GAP}, but got {cone_narrow_off_axis} (narrow) vs {cone_wide_off_axis} (wide) — \
         widening the cone should illuminate a region the narrow cone left dark"
    );

    // The 0deg Chrome anomaly itself: demo_light_sources.rs's own module doc comment and
    // panel-light-sources.html both state that limitingConeAngle 0 renders as a fully open beam in this
    // sandbox's own Chrome, not the near-invisible cutoff a zero-width cone should be, which is why the
    // slider's own range starts at 5 rather than 0. The 5deg-vs-90deg comparison above proves 5deg is a usefully
    // narrow lower bound; it says nothing about 0deg, which this checks directly: the same off-axis sample
    // should be materially brighter at 0deg than at 5deg, and close to the wide (90deg) reading, matching "fully
    // open" rather than "even narrower than 5deg".
    let cone_zero_off_axis = sample(&value, "ls-cone-zero", "topLeft");
    const MIN_ZERO_VS_NARROW_GAP: i32 = 20;
    const MAX_ZERO_VS_WIDE_DIFF: i32 = 10;
    assert!(
        i32::from(cone_zero_off_axis) - i32::from(cone_narrow_off_axis) >= MIN_ZERO_VS_NARROW_GAP,
        "expected the 0deg cone to leave that same off-axis sample at least {MIN_ZERO_VS_NARROW_GAP} brighter \
         than the 5deg cone does, but got {cone_zero_off_axis} (0deg) vs {cone_narrow_off_axis} (5deg) — this is \
         the specific claim that the slider's own minimum (5, not 0) and panel-light-sources.html's own \
         explanation both depend on"
    );
    assert!(
        (i32::from(cone_zero_off_axis) - i32::from(cone_wide_off_axis)).abs() <= MAX_ZERO_VS_WIDE_DIFF,
        "expected the 0deg cone's own off-axis sample to land within {MAX_ZERO_VS_WIDE_DIFF} of the wide \
         (90deg) cone's own reading, matching a fully open beam rather than a partial cutoff, but got \
         {cone_zero_off_axis} (0deg) vs {cone_wide_off_axis} (90deg)"
    );

    // Positive control: a sample exactly on the beam's own aim axis should stay lit even under the narrow
    // cone, proving the off-axis reading above reflects the hard-edged cutoff itself, not the whole beam going
    // dark for an unrelated reason. A 5deg cone is tight enough (cos(5deg) ~ 0.996) that even `nearAim`, 10
    // units off axis in each direction, is not a safe positive control for it — confirmed empirically, that
    // point alone reads far dimmer under the narrow cone than under the wide one. `exactAim` sits precisely on
    // the axis instead, where the cutoff excludes nothing regardless of how narrow the cone is. Its own measured
    // brightness there (23) is well below this rect's own brightest points elsewhere (the surface's own normal
    // and the light's own halfway-vector geometry still shape the result away from a flat maximum), but it is
    // clearly non-zero, unlike the off-axis reading above — the threshold checks for exactly that contrast, not
    // for a peak brightness.
    let cone_narrow_exact_aim = sample(&value, "ls-cone-narrow", "exactAim");
    let cone_wide_near_aim = sample(&value, "ls-cone-wide", "nearAim");
    const MIN_EXACT_AIM_BRIGHTNESS: u8 = 15;
    const MIN_NEAR_AIM_BRIGHTNESS: u8 = 60;
    assert!(
        cone_narrow_exact_aim >= MIN_EXACT_AIM_BRIGHTNESS,
        "expected a sample exactly on the beam's own aim axis to stay at or above {MIN_EXACT_AIM_BRIGHTNESS} \
         even under the narrow (5deg) cone, but got {cone_narrow_exact_aim}"
    );
    assert!(
        cone_wide_near_aim >= MIN_NEAR_AIM_BRIGHTNESS,
        "expected a sample near the beam's own aim axis to stay at or above {MIN_NEAR_AIM_BRIGHTNESS} under the \
         wide (90deg) cone, but got {cone_wide_near_aim}"
    );
}
