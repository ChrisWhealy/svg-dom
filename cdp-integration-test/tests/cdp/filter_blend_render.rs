//! `SvgFilter::blend`'s alpha-preserving tint chain, against real rendered pixels.
//!
//! `tests/filter/blend.rs` and `chains.rs` (in the main `svg-dom` crate) can only prove the DOM structure for
//! `SvgFilter::blend` and `composite`: that the right elements, with the right attributes, exist in the right
//! order.
//! Neither can prove what those elements actually *render*.
//! The whole point of the `flood` → `blend` → `composite(In)` tint chain documented by `SvgFilter::blend` is a
//! rendering claim: that the chain preserves the source graphic's own transparency, instead of leaking the flood
//! colour into it.
//!
//! A structural test that only counts child elements is satisfied by a chain that still gets the rendering wrong —
//! exactly what happened before that final `composite(In)` step was added.
//! For the full explanation of the bug this test guards against, see the doc comment sections
//! `# ⚠️ Tinting with a flood colour needs a final composite(In)` and `docs/svg_elements/filters.md`'s matching
//! warning in `SvgFilter::blend`.
//!
//! This renders the `#blend-circle` element built by the sibling `cdp-test-fixture` wasm crate (a white circle,
//! filtered with `flood("#f0883e", ...)` → `blend(Multiply)` → `composite(In)`) to an offscreen canvas, then reads
//! back actual pixel values to confirm that:
//!
//! 1. A pixel at the circle's centre is fully opaque and (approximately) the flood colour.
//!    White is `Multiply`'s identity element, so a correctly alpha-preserving chain paints the flood colour
//!    through unchanged, giving an *exact* expected result rather than an approximate one.
//! 2. A pixel at a corner of the circle's bounding box lying outside the circle, where `SourceGraphic` is fully
//!    transparent, is also fully transparent (alpha `0`).
//!    Before the `composite(In)` fix, this pixel was opaque and flood-coloured instead, because `flood` paints
//!    across the *entire* filter region regardless of the source graphic's shape, and `feBlend`'s result alpha is
//!    the union of its two inputs' alpha.
//!
//! # How the pixels are read
//!
//! CDP offers no call that can read an SVG's rendered pixels directly, so this goes through in-page JavaScript
//! instead:
//! 1. Serialise the fixture's `<svg>` to a `data:image/svg+xml` URL.
//! 2. Load that into an `Image`.
//! 3. Draw it to an offscreen `<canvas>`.
//! 4. Read the two pixels back via `getImageData`.
//!
//! That script must itself be asynchronous, since `Image` loading is asynchronous, so it runs via
//! `Runtime.evaluate` with `awaitPromise: true` and `returnByValue: true`.
//! This is called directly, rather than through `headless_chrome::Tab`'s own `evaluate()` wrapper, which hardcodes
//! `returnByValue: false` and only inlines primitive results, not the object this script resolves with.

use headless_chrome::protocol::cdp::Runtime;
use serde_json::Value;

/// The in-page async script: rasterises the fixture's `<svg>` and returns `{ center: [r,g,b,a], corner: [r,g,b,a] }`
/// for the `#blend-circle` element, sampling its centre and a corner of its bounding box (inset by 2px so the sample
/// point is never itself antialiased edge pixel).
const SAMPLE_SCRIPT: &str = r#"
(async () => {
    const circle = document.querySelector('#blend-circle')
    const svg = circle.closest('svg')
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

    const cx = parseFloat(circle.getAttribute('cx'))
    const cy = parseFloat(circle.getAttribute('cy'))
    const r = parseFloat(circle.getAttribute('r'))

    function pixelAt(x, y) {
        const d = ctx.getImageData(Math.round(x), Math.round(y), 1, 1).data
        return [d[0], d[1], d[2], d[3]]
    }

    const center = pixelAt(cx, cy)
    const corner = pixelAt(cx - r + 2, cy - r + 2)

    URL.revokeObjectURL(url)
    return { center, corner }
})()
"#;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Component-wise `(r, g, b, a)` from a JSON `[r, g, b, a]` array.
fn rgba(value: &Value, context: &str) -> Result<(u8, u8, u8, u8), String> {
    let arr = value
        .as_array()
        .ok_or_else(|| format!("{context}: expected a 4-element array, got {value}"))?;
    let component = |i: usize| -> Result<u8, String> {
        arr.get(i)
            .and_then(Value::as_u64)
            .map(|n| n as u8)
            .ok_or_else(|| format!("{context}: expected a numeric component at index {i}, got {value}"))
    };
    Ok((component(0)?, component(1)?, component(2)?, component(3)?))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn blend_tint_chain_preserves_source_alpha() -> Result<(), String> {
    let tab = super::common::new_tab()?;

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
        .map_err(|e| format!("Runtime.evaluate failed: {e}"))?;

    if let Some(exception) = evaluated.exception_details {
        return Err(format!("pixel-sampling script threw: {exception:?}"));
    }
    let value = evaluated
        .result
        .value
        .ok_or_else(|| "evaluate did not return a value — was returnByValue set?".to_owned())?;

    let center = rgba(&value["center"], "center pixel")?;
    let corner = rgba(&value["corner"], "corner pixel")?;

    // The flood colour is #f0883e = (240, 136, 62). White is Multiply's identity element, so a correctly
    // alpha-preserving chain paints it through completely unchanged inside the circle — allow +/-2 per channel for
    // canvas rasterisation rounding, not for any real uncertainty about the expected colour.
    let close = |actual: u8, expected: u8| actual.abs_diff(expected) <= 2;
    if !(close(center.0, 240) && close(center.1, 136) && close(center.2, 62) && center.3 == 255) {
        return Err(format!(
            "expected the circle's centre to be opaque and (approximately) the flood colour (240, 136, 62, 255), \
             got {center:?}"
        ));
    }

    // Outside the circle but inside its bounding box, SourceGraphic is fully transparent. Before the composite(In)
    // fix, this pixel was opaque and flood-coloured (the exact bug this test exists to catch); it must now be
    // fully transparent.
    if corner != (0, 0, 0, 0) {
        return Err(format!(
            "expected a corner of the circle's bounding box to be fully transparent (the composite(In) step must \
             clip the flood back to the source's own alpha coverage), got {corner:?} — the flood colour is \
             leaking through wherever SourceGraphic is transparent"
        ));
    }

    Ok(())
}
