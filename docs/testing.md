# Testing

The test suite has three tiers that use different runners.

## Unit Tests — `cargo test`

Pure Rust tests with no browser dependency.

Currently covers the `Error` type's `Display` and `Debug` implementations and its inner-value accessors, plus the `PathDef` → `d`-string formatting logic in `root::path::unit_tests` (one command per SVG path letter, buffer-reuse behaviour in `write_d`).
Also covers doc tests.

```sh
cargo test
```

## Browser Tests — `wasm-pack test`

Everything that touches the SVG DOM requires a real browser.
These tests use [`wasm-bindgen-test`](https://rustwasm.github.io/wasm-bindgen/wasm-bindgen-test/index.html), which compiles the test suite to WebAssembly, serves it to a headless browser, and streams the results back to the terminal.

### Prerequisites

```sh
cargo install wasm-pack      # one-time install
```

Chrome or Firefox must be installed (headless mode is used — no window opens).

### Running

```sh
wasm-pack test --headless --firefox   # always works

wasm-pack test --headless --chrome    # requires Chrome to be on the latest stable release
```

**Chrome version note.**
wasm-pack 0.15+ always downloads the latest stable ChromeDriver from the Chrome for Testing endpoint rather than detecting the installed Chrome version.
If your Chrome lags behind the stable channel (e.g. managed machines, delayed auto-updates), ChromeDriver and Chrome will be mismatched.
All Chrome tests will then fail with an HTTP 404 session error.
The fix is to update Chrome to the latest stable release so its major version matches the downloaded ChromeDriver.
If you cannot update Chrome immediately, point wasm-pack at a compatible driver with the `--chromedriver` flag:

```sh
# Replace the path with a chromedriver binary whose major version matches your Chrome.
wasm-pack test --headless --chrome \
  --chromedriver ~/.wasm-pack/cache/chromedriver-<hash>/chromedriver
```

`wasm-pack` caches previously downloaded drivers under `~/Library/Caches/.wasm-pack/` on macOS.
Inspect that directory to find one whose version matches your Chrome.

### How It Works

Each function decorated with `#[wasm_bindgen_test]` runs inside the browser's JS engine with full access to the real DOM.
The test file calls `wasm_bindgen_test_configure!(run_in_browser)` once to opt into this mode.

Tests are organised into integration test files under `tests/`:

| File | What it covers |
|---|---|
| `tests/svg_root.rs` | `SvgRoot` constructors, viewport, and all element factories |
| `tests/svg_node/` | `SvgNode` attribute API, clone semantics, `append`, and event handlers — see below |
| `tests/animation_loop.rs` | `AnimationLoop` lifecycle, `start`/`stop` from within callback, and memory retention bug prevention |
| `tests/accessibility.rs` | `title`/`desc` accessible-name/description API: create, read back, update, remove, sibling/ordering behaviour, and blank-value rejection, on both `SvgNode` and `SvgRoot` |
| `tests/clip_path.rs` | `SvgClipPath` construction, id caching, id validation, and `set_id` |
| `tests/defs/` | `SvgDefs` and `SvgMarker` construction, all factory methods, marker ID validation, `build_defs`/`build_marker` deferred-append, `set_id`, and generic attribute surface — see below |
| `tests/filter/` | `SvgFilter` construction, every primitive factory method, id validation, region/coordinate-space attributes, and `SvgNode::set_filter`/`set_filter_ref`/`remove_filter` — see below |
| `tests/gradient.rs` | `SvgLinearGradient`/`SvgRadialGradient` construction, id caching/validation, and `set_fill_gradient`/`set_stroke_gradient` (plus their linear/radial-specific variants) |
| `tests/mask.rs` | `SvgMask` construction, id caching/validation, region/units/type attributes, shape children, and `set_mask_ref`/`remove_mask` |
| `tests/pattern.rs` | `SvgPattern` construction, id caching/validation, region/units/viewBox/transform attributes, shape children, and `set_fill_pattern`/`set_stroke_pattern` |
| `tests/symbol.rs` | `SvgSymbol` construction, id caching/validation, viewBox/preserveAspectRatio, and shape children |
| `tests/text_path.rs` | `<textPath>` construction, content, `href`, and its startOffset/method/spacing/side attributes |
| `tests/tspan.rs` | `<tspan>` construction and nesting, plus the `tspan_dy`/`tspan_line` positioning helpers |
| `tests/view.rs` | `SvgView` construction, id caching/validation, viewBox/preserveAspectRatio, and a real, cross-browser rendering test that samples canvas pixels to prove `<view>` fragment navigation changes the rendered viewport |

Shared DOM helpers (creating fixture `<div>` and `<svg>` containers, assertion functions) live in `tests/common.rs`, included by every test file.
Files directly under `tests/` use a plain `mod common;`.
A file one level down, inside `tests/svg_node/`, `tests/defs/`, or `tests/filter/`, uses `#[path = "../common.rs"] mod common;` instead (see below).

### Promoted-to-folder Test Files

`tests/svg_node.rs`, `tests/defs.rs`, and `tests/filter.rs` each grew past 1000 lines, so each was promoted to a folder.
`tests/svg_node/main.rs`, `tests/defs/main.rs`, and `tests/filter/main.rs` are the actual Cargo-discovered test binaries — Cargo treats `tests/<name>/main.rs` as equivalent to a bare `tests/<name>.rs`.
The rest of the folder splits into one file per concern, indexed in that `main.rs`'s own module doc comment.
This is the same categorisation approach `docs/design_notes/` uses, rather than a `README.md`.

| Folder | Files, each named after (and scoped to) the matching `src/` module | Shared setup |
|---|---|---|
| `tests/svg_node/` | `attrs`, `cached`, `text`, `transform`, `tree`, `events`, `attr_writer`, `geometry` (mirroring `src/node/attrs.rs`, `cached.rs`, `text.rs`, `transform.rs`, `tree.rs`, `event.rs`+`listeners/`, `src/root/attrs/mod.rs`, `geometry.rs`) | `helpers.rs` — a file-local `make_svg` (200×200 canvas, distinct from `common::make_svg`'s 400×300) and the synthetic-event `dispatch`/`dispatch_element` pair `events.rs` uses |
| `tests/defs/` | `svg_defs`, `marker_construction`, `marker_children`, `marker_refs`, `deferred_append`, `marker_id_validation` | None beyond `common` — no file-local helpers were needed |
| `tests/filter/` | `construction`, `apply`, `region`, `gaussian_blur`, `offset`, `merge`, `flood`, `composite`, `blend`, `drop_shadow`, `color_matrix`, `component_transfer`, `turbulence`, `displacement_map`, `morphology`, `image`, `tile`, `convolve_matrix`, `diffuse_lighting`, `specular_lighting`, `chains` (each primitive file mirrors its `src/root/filter/primitives/*.rs` counterpart; `construction`/`region` mirror `src/root/filter/mod.rs`/`attrs.rs`/`region.rs`; `chains` holds cross-primitive integration tests that don't belong to any single primitive) | None beyond `common` — no file-local helpers were needed |

Splitting a large single-function-heavy test file into `main.rs` + siblings, rather than, say, five entirely separate `tests/*.rs` binaries, keeps each concern in its own discoverable file.
It still reports as one `cargo test`/`wasm-pack test` target, and pays only one fixture-setup cost.
The split is organisational, not a change to how the tests run.

### DOM Fixture Strategy

Each test appends its own uniquely-named container element to `<body>` so tests do not interfere with each other.
No teardown is needed: the browser page is discarded after the run.

### Event Handler Tests

Browser events dispatched via `EventTarget::dispatch_event` are **synchronous**.
The handler runs inline before `dispatch_event` returns, so there is no need to worry about any `async` shenanigans.

A shared `Rc<Cell<bool>>` flag is set inside the handler, and the test checks the flag immediately after dispatch:

```rust
let fired = Rc::new(Cell::new(false));
let fired_c = fired.clone();
node.on_click(move |_| { fired_c.set(true); })?;

let event = MouseEvent::new("click")?;
node.as_element().dispatch_event(&event)?;  // handler fires here, synchronously

assert!(fired.get());
```

Additional event wrapper tests dispatch representative synthetic mouse, pointer, wheel, touch, keyboard, focus, drag-and-drop and generic events.
They verify that those managed wrappers fire synchronously too, so demo or application code does not need raw `Closure::forget` listeners for ordinary SVG interaction.

### Failure Reporting

All test functions return `Result<(), String>`.
If a test fails, `wasm-bindgen-test` displays the `String` message directly without a stack trace, making failures easier to read in the terminal.

## CDP Integration Tests — `cargo test -p cdp-integration-test`

The above tests mostly prove DOM structure: that the correct element was created, updated, or removed in the correct place, with the correct attributes.
Most of the above assertions check only the DOM element structure and the attributes of those elements, not what the browser actually paints.

That is not because `wasm-bindgen-test` cannot rasterise since `tests/view.rs` already includes render-level pixel assertions.
Instead, it creates an offscreen `<canvas>`, draws an `SvgImageElement` onto it then calls `getImageData` to sample the actual RGBA pixels.
Those pixels prove that navigating to an SVG `<view>` fragment changes the rendered viewport.
The crate's own dev-dependencies already enable `CanvasRenderingContext2d`, `HtmlCanvasElement`, `ImageData`, `XmlSerializer`, and `wasm-bindgen-futures` for exactly this.
Because the ordinary browser suite runs in both Firefox and Chrome, that view-fragment test is a genuine cross-engine rasterisation check.

So the real distinction is narrower than "wasm-bindgen-test cannot see rendered output."
Most of the ordinary browser suite tests DOM structure and state, but `tests/view.rs` already proves render-level, cross-engine pixel assertions belong there too.

The filter and lighting render checks below currently live in the CDP suite by construction, not by necessity.
`tests/view.rs` shows the same rasterise-and-sample technique already works in plain `wasm-bindgen-test`.
Only the computed accessibility tree genuinely requires CDP: it lives entirely behind the browser's Accessibility CDP domain, with no other route to it.

This leaves an available future route.
Some or all of the filter render checks could move to async `wasm-bindgen-test`s, following `tests/view.rs`'s own precedent.
They could then run in both Chrome and Firefox, for stronger cross-engine coverage.
The current, consolidated CDP suite is not broken, so this is an option worth recording, not a reason to refactor now.

The `cdp-integration-test` crate hosts one Cargo integration-test binary, `tests/cdp/`, with five scenario modules, each driving Chrome directly over the Chrome DevTools Protocol (CDP) via the [`headless_chrome`](https://docs.rs/headless_chrome) crate.
`tests/cdp/common.rs` builds the fixture, starts the static server, and launches Chrome exactly once for the whole binary, via a lazily-initialised `OnceLock`.
To ensure isolation, each scenario module then opens its own tab within the shared browser.
See each module's own doc comment for the full detail on what it verifies.

### `accessibility_tree.rs` — accessible-name/description computation

Queries `Accessibility.getPartialAXTree`, via seven independently reported `#[test]` functions.
These functions confirm:

- A lone `<title>` supplies the accessible name (`title_only_supplies_accessible_name`);
- A `<desc>` supplies the accessible description (`desc_supplies_accessible_description`);
- A value in `aria-label` overrides a `<title>` in name computation (`aria_label_overrides_title`);
- A value in `aria-describedby` overrides a `<desc>` in description computation (`aria_describedby_overrides_desc`);
- A rejected blank `set_title` leaves the element with no accessible name at all (`blank_title_rejection_leaves_no_accessible_name`).
  This proves that the rejection actually prevents the "apparently nameless object exposed to assistive technology" case SVG 2 warns about, not just the DOM mutation;
- A value in `aria-labelledby` overrides *both* `aria-label` and a `<title>` (`aria_labelledby_overrides_title_and_aria_label`).
  `aria-labelledby` has strictly higher precedence than `aria-label` in accessible-name computation, not just parity with it, so this scenario gives an element all three and confirms the referenced text wins over both;
- An `<a>` wrapping visible text is exposed as a named link (`anchor_with_visible_text_is_a_named_link`).
  SVG maps `<a>` to the ARIA "link" role automatically, and the accessible name comes from the linked text content itself, the same way it would for an HTML `<a>`.

### `accessibility_tree.rs`: One shared tab, seven independent results

`tests/cdp/common.rs` already shares the fixture build, static server, and `Browser` process with every other module in this binary.
Opening a fresh tab per scenario here would still cost a navigate-and-wait-for-readiness round trip seven times over, for content that never changes between them, so all seven scenarios additionally share one tab of their own, via a second, module-local `OnceLock`.

`cargo test` runs the seven test functions in parallel, so actual CDP calls against that shared tab are serialised behind a `Mutex`.
`find_element`'s underlying `DOM.getDocument`-then-`DOM.querySelector` sequence is not safe under concurrent access to the same session, even though `Browser` and `Tab` implement `Send + Sync` at the type level.
See the module doc comment in `cdp-integration-test/tests/cdp/accessibility_tree.rs` for the full explanation.

Splitting the original single test (with sequential `assert_eq!` calls in one function) into separate `#[test]` functions was a deliberate correction.
If they were bundled into a single function, only the first failing assertion was ever reported, and `cargo test` counted the whole scenario suite as a monolithic pass/fail.

### `filter_blend_render.rs` — `SvgFilter::blend`'s alpha-preserving tint chain, against real rendered pixels

`svg-dom`'s own `tests/filter/blend.rs`/`chains.rs` prove DOM structure for `SvgFilter::blend`/`composite`: the right elements, with the right attributes.
It cannot prove how those elements are actually *rendered*.
The documented `flood` → `blend` → `composite(In)` tint chain (see `SvgFilter::blend`'s doc comment and [Filters](svg_elements/filters.md)) is fundamentally a rendering claim.
It claims that the chain preserves the source graphic's own transparency, instead of leaking the flood colour into it.

A structural test that only counts child elements can be satisfied by a chain that gets this wrong.
That is exactly what shipped briefly, before a bug report showed a flood-and-blend chain without the final `composite(In)` leaking an opaque flood colour into a circle's transparent bounding-box corners.

This single `#[test]` renders the `#blend-circle` element built by `cdp-test-fixture` — a white circle, filtered with the corrected three-step chain — to an offscreen canvas.
It does this by serialising the fixture's `<svg>` to a `data:image/svg+xml` URL, loading it into an `Image`, then reading the pixels back via `getImageData`: the standard technique for rasterising SVG content in a browser.
It then asserts on the real pixel values:

- a pixel at the circle's centre is fully opaque and (approximately) the flood colour.
  White is `Multiply`'s identity element, so a correctly alpha-preserving chain paints the flood colour through completely unchanged, giving an *exact* expected result rather than an approximate one;
- a pixel at a corner of the circle's bounding box — outside the circle, where `SourceGraphic` is fully transparent — is fully transparent (alpha `0`).
  That is the exact pixel that leaked opaque flood colour before the `composite(In)` fix.

Because the pixel-sampling script is asynchronous — `Image` loading is not synchronous — it runs via the raw `Runtime.evaluate` CDP command, with `awaitPromise: true` and `returnByValue: true`.
It is called directly, rather than through `headless_chrome::Tab`'s own `evaluate()` wrapper, which hardcodes `returnByValue: false`.

This lives in its own module, rather than as more `#[test]`s in `accessibility_tree.rs`.
So each module's doc comment stays retains its "common sense" scope for what it actually verifies — accessible-name computation in one, filter alpha compositing in the other.
It shares `tests/cdp/common.rs`'s one `Browser` instance with every other module in this binary, opening only its own tab.

### `turbulence_scale_zero_render.rs` — `SvgFilter::displacement_map`'s `scale` at `0.0`, against real rendered pixels

The demo gallery's own turbulence panel (`demo/panels/panel-turbulence.html`) prominently states that scale 0 restores a perfect geometric circle.
`demo-app/src/browser_tests/paint/turbulence.rs` proves the DOM half of that claim — the scale slider does reach `scale="0"` on the real `feDisplacementMap` element.
That test only checks DOM structure and attributes, not rendered pixels.
The render-level check for this exact claim lives here, in the CDP suite, instead.
A structural test is satisfied by a `scale="0"` attribute sitting on a filter chain that renders however it likes.

This single `#[test]` renders three circles built by `cdp-test-fixture`: `#turbulence-reference` (a plain, unfiltered circle), `#turbulence-scale-zero` (passed through `turbulence` → `displacement_map` with `scale` fixed at `0.0`), and `#turbulence-scale-sixty` (the same chain again, with `scale` fixed at `60.0` — `demo_turbulence.rs`'s own documented maximum).
All three use the same radius and fill.
Their centres differ, so samples are always taken at corresponding offsets around each circle's own centre, not at shared absolute coordinates.
It samples eight points around each circle's own boundary, 3px inside and 3px outside the nominal radius.
It then asserts two things.
The reference and scale-zero circles rasterise to the same pixel values at every sample, within a small antialiasing tolerance — the negative control.
The reference and scale-sixty circles rasterise to a materially different value at, conservatively, at least one sample — the positive control.

The maximum displacement along either axis is `scale / 2` — 30px at scale 60.
But that is a ceiling, not a guarantee.
The actual displacement at any one sampled point depends on the local turbulence channel value there, not on `scale` alone.
Below scale 6, `scale / 2` itself is under the 3px sample margin either check uses, and even a much larger scale need not reach its own maximum at any particular sample.
Scale 60 and the threshold below were chosen against what this sandbox's own headless Chrome actually renders at that scale, an observed property of this fixture rather than an assumed one.

The positive control exists because the negative control alone is a one-sided claim.
"Scale zero rasterises like the reference" is equally consistent with a correctly working filter chain.
It is also consistent with a browser that silently ignored the filter, or fell back to unfiltered `SourceGraphic` — either would also rasterise like the reference.
Asserting that scale sixty *does* rasterise differently proves this fixture and sampling method can actually detect a real displacement in the first place.
So the negative control's own pass means what it claims to mean.
The two thresholds involved sit far apart.
Measured against this sandbox's own headless Chrome — this fixture's own fixed noise seed makes the render deterministic — boundary samples between the reference and scale-zero circles differ by at most 1 per channel.
Scale-sixty's own genuinely displaced samples, by contrast, differ by 75–255.
The antialiasing tolerance (4) and the displacement threshold (40) both sit in the wide, empty gap between those two, with no realistic risk of one control's own noise tripping the other's threshold.

Getting a stable *negative* comparison here needed one more fix beyond the technique `filter_blend_render.rs` already established.
`cdp-test-fixture` pins `#turbulence-scale-zero`'s own filter region to exactly the circle's bounding box (`set_x`/`set_y`/`set_width`/`set_height`, all `0`/`0`/`1`/`1` in `objectBoundingBox` units) rather than leaving it at SVG's own default 10%-margin region.
Left at that default, this sandbox's headless, software-rendered Chrome (`--disable-gpu`) composited the filtered circle back onto the page with a real, several-pixel positional error.
That error is unrelated to `scale`, present even at `0.0`, and large enough on its own to fail the boundary samples unpredictably from one run to the next.
Pinning the region to a plain 100% box removed that error outright.
`#turbulence-scale-sixty`'s own filter keeps the wider, default-adjacent region `demo_turbulence.rs`'s real, interactive circle uses instead (`widen_filter_region`: -50%/-50%/200%/200%).
A genuine 30px displacement needs room to sample source pixels from outside the bare bounding box, unlike the zero-displacement case, which never reads past its own edge.

This intentionally does not attempt broad screenshot testing across every slider position.
A single identity test at scale zero, backed by one positive control at scale sixty, is enough to cover this specific, exact semantic claim without turning into a fragile visual regression suite.

### `lighting_render.rs` — `demo_lighting.rs`'s own surfaceScale and azimuth sliders, against real rendered pixels

`demo-app/src/browser_tests/paint/lighting.rs` proves the DOM part of both sliders: moving either one mutates `surfaceScale` on all four retained lighting primitives, or `azimuth` on all four `<feDistantLight>` children.
It cannot prove that either mutation actually changes the *rendered* lighting.

This single `#[test]` renders three circles built by `cdp-test-fixture`, all filtered through the exact same `feDiffuseLighting` recipe `demo_lighting.rs`'s own "diffuse-only" column uses: `#lighting-reference` (surfaceScale 6, azimuth 235deg), `#lighting-azimuth-90` (surfaceScale 6, azimuth 90deg), and `#lighting-scale-zero` (surfaceScale 0, azimuth 235deg).
It samples eight points around each circle's own rim, 2px inside its nominal radius, where `feDiffuseLighting`'s own alpha-gradient bump map is non-flat.
It then asserts three things:

- `#lighting-reference`'s own eight rim samples vary meaningfully from one another.
  This is the positive control that this fixture and sampling method actually detect real directional lighting when it is there.
- `#lighting-scale-zero`'s own eight rim samples stay close to one another, well inside the variation shown by `#lighting-reference`.
  This checks `panel-lighting.html`'s own claim that surfaceScale 0 flattens the bump map into a uniform lit surface, against real pixels rather than only the `surfaceScale="0"` attribute.
- There is a material difference between at least one of the eight matching-angle samples of `#lighting-reference` and `#lighting-azimuth-90`.
  Azimuth actually turns the rendered light, not just the `<feDistantLight>` attribute a DOM test can detect.

The first check alone would be a one-sided claim about the sampling method's own sensitivity.
It says nothing about whether a *flattened* bump map also rasterises correctly, which is exactly what the second check proves.
Together they mean a pass on the second check reflects the fixture genuinely losing directional variation, not an insensitive sampling method that would have missed variation either way.

This intentionally does not attempt broader screenshot testing across every slider position.
These three fixed points are enough to cover both sliders' own specific rendering claims without turning into a fragile visual regression suite.

Uses the same pixel-reading technique as `filter_blend_render.rs` and `turbulence_scale_zero_render.rs` — see that section above for the detail.

### `light_sources_render.rs` — `demo_light_sources.rs`'s own four sliders, against real rendered pixels

`demo-app/src/browser_tests/paint/light_sources.rs` can only prove the DOM part of the claims made by the four sliders: that moving each one mutates the correct attribute on the correct retained light-source node, and leaves the other three columns untouched.
It cannot prove that any of those mutations actually change the *rendered* lighting.

This matters more here than for most demos.
One of the four demos' own designs is itself grounded in observed Chrome rendering behaviour, rather than the SVG spec alone.
`demo_light_sources.rs`'s own module doc comment states that a `limiting_cone_angle` of `0.0` renders as a fully open beam, rather than the near-invisible cutoff the spec describes, which is why that slider's own range starts at `5` instead.
Only a test of the rendered pixels can validate that claim.

That same module doc comment also records a second observation: in this sandbox's own Chrome, the spotlight's own `specular_exponent` had no visible effect at any value from `0.01` through `10000`.
This is the reason that field was rejected as this demo's own interactive control for the open-Spot column, in favour of `x`.
This file deliberately does not regression-test that observation — a fixture asserting that varying `specular_exponent` continues to produce *no* visible change would fail the day Chrome fixes whatever causes that, and that failure would represent an improvement, not a regression.
It remains a manually recorded observation in `demo_light_sources.rs`'s own module doc comment, not a claim this file verifies.

This drives a real Chrome instance via CDP and renders rects built by `cdp-test-fixture`, grouped into four checks.
Each check runs `demo_light_sources.rs`'s own exact `feSpecularLighting` recipe on a plain, flat rect, fixed at different slider positions:

1. `#ls-distant-low` (elevation 15deg) vs `#ls-distant-high` (elevation 85deg) — checks that a flat Distant surface's own average brightness genuinely rises with elevation, `panel-light-sources.html`'s own claim.
2. `#ls-point-low-z` (z 20) vs `#ls-point-high-z` (z 180) — checks that a lower light genuinely sharpens the hotspot (a bigger centre-to-corner contrast), rather than spreading it.
3. `#ls-spot-left` (light at its own rect's left edge) vs `#ls-spot-right` (right edge), both with `pointsAtX` trailing by the same 80-unit offset `demo_light_sources.rs`'s own `SPOT_OPEN_AIM_OFFSET` uses — checks that the bright region genuinely moves horizontally with the light, not just the DOM attribute.
4. `#ls-cone-zero` (`limitingConeAngle` 0deg) vs `#ls-cone-narrow` (5deg) vs `#ls-cone-wide` (90deg) — three rects, not two.
   `#ls-cone-narrow` vs `#ls-cone-wide` alone only proves 5deg is a usefully narrow lower bound; it says nothing about 0deg being anomalous.
   `#ls-cone-zero` checks that specific claim directly: the same off-axis sample stays dark at 5deg, but is materially brighter and close to the wide reading, at 0deg.
   This is the highest-value check of the four, since both the slider's own chosen minimum (5, not 0) and `panel-light-sources.html`'s own explanation of why depend on this exact, sandbox-specific Chrome behaviour, rather than a specification guarantee.

### Why this lives outside the main crate

The library's own `cargo test`/`cargo nextest run` stays fast and dependency-light on purpose.
The `cdp` test binary above needs a real, local Chrome/Chromium binary, and pulls in `headless_chrome` (and its own dependency tree).
So, like `demo-server`, the crate hosting it lives in its own workspace member, excluded from the root package's `default-members`.
Plain `cargo build`/`cargo test` at the project root never touch it.

Two supporting crates make this possible:

| Crate | Role |
|---|---|
| `cdp-test-fixture` | A tiny `wasm-bindgen` cdylib that builds real `svg-dom` elements for all five test files: six accessibility scenarios (via `set_title`, `set_desc` and `set_attr`), one `#blend-circle` filter scenario (via `flood`/`blend`/`composite`), a `#turbulence-reference`/`#turbulence-scale-zero`/`#turbulence-scale-sixty` trio (via `turbulence`/`displacement_map`), a `#lighting-reference`/`#lighting-azimuth-90`/`#lighting-scale-zero` trio (via `diffuse_lighting`), and nine light-source rects (via `specular_lighting`) grouped into the four `ls-distant`/`ls-point`/`ls-spot`/`ls-cone` comparisons — and signals readiness by adding a `#fixture-ready` element |
| `cdp-integration-test` | `src/lib.rs` holds the shared `fixture_dir`/`build_fixture`/`serve`/`launch_browser` setup helpers. `tests/cdp/common.rs` calls them once per test run and hands every scenario module its own tab from the one shared `Browser`. `tests/cdp/{accessibility_tree,filter_blend_render,turbulence_scale_zero_render,lighting_render,light_sources_render}.rs` are that binary's five scenario modules, each running its own `#[test]`s |

### Prerequisites

Same `wasm-pack` install as the browser tests, plus a local Chrome or Chromium install (`headless_chrome` auto-discovers it the same way Puppeteer/Playwright do).

### Running

```sh
cargo test -p cdp-integration-test
```

This runs the crate's one `cdp` test binary, covering all five scenario modules.
It rebuilds the `cdp-test-fixture` wasm package once, serves it on an OS-assigned local port, and drives one shared headless Chrome instance against it for the whole run — no manual server or browser setup needed.

***IMPORTANT***<br>
This shared-fixture, shared-browser design depends on every scenario running inside one process, which plain `cargo test` does.
However, `cargo nextest run` does not: by design, nextest isolates every `#[test]` function in its own process, so under nextest, each scenario would rebuild the fixture and relaunch Chrome independently, instead of sharing one run's worth of setup — which defeats the purpose of this consolidation.

If `cargo test` is aliased to `cargo nextest run` on your machine, invoke the real `cargo` binary for this crate instead, bypassing the alias — for example `command cargo test -p cdp-integration-test`.

### Running in CI

Runs as its own job (`cdp-integration-test`) in `.github/workflows/ci.yml`, on every push/PR, using the Chrome installation already present on GitHub's `ubuntu-latest` runner image.
No extra install step is needed, and no per-file CI wiring either, for the same reason noted above.

It was initially added without any CI job at all, so it protected nothing.
The workspace's `default-members` deliberately excludes it (see above), so plain `cargo test`/`cargo nextest run` never runs it.
None of the other CI jobs invoke it either.
A regression here could land on `main` without any CI job noticing.
For example: any test file failing to compile, Chrome's actual accessible-name/description computation drifting away from what the crate assumes, a filter chain silently starting to leak, a scale-zero displacement no longer rendering as a perfect circle, or the lighting/light-source primitives silently losing their directional or positional effect on the rendered pixels.
Being a separate job, rather than an extra step tacked onto `browser-tests`, means its failure is reported independently.
It doesn't obscure, or get obscured by, the unrelated `wasm-bindgen-test` results, while still gating the merge like any other required check.

The Chrome launch in `cdp-integration-test` explicitly passes `sandbox(false)`, rather than using `Browser::default()`'s sandboxed default.
Recent Ubuntu (24.04+, which `ubuntu-latest` now resolves to) restricts unprivileged user namespaces via AppArmor.
That breaks Chrome's own sandbox initialisation, even for the runner's non-root user.
Since this test only ever loads a local fixture page the crate builds itself, there is no untrusted content for the sandbox to matter for.
So it is disabled unconditionally, not just in CI, to keep local and CI runs on the same code path.
See the `# Why the browser is launched with sandbox(false)` section of `tests/cdp/main.rs`'s own module doc comment for the full explanation.
