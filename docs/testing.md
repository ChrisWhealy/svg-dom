# Testing

The test suite has three tiers that use different runners.

## Unit tests — `cargo test`

Pure Rust tests with no browser dependency.

Currently covers the `Error` type's `Display` and `Debug` implementations and its inner-value accessors, plus the `PathDef` → `d`-string formatting logic in `root::path::unit_tests` (one command per SVG path letter, buffer-reuse behaviour in `write_d`).
Also covers doc tests.

```sh
cargo test
```

## Browser tests — `wasm-pack test`

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
If your Chrome lags behind the stable channel (e.g. managed machines, delayed auto-updates), ChromeDriver and Chrome will be mismatched and all Chrome tests will fail with an HTTP 404 session error.
The fix is to update Chrome to the latest stable release so its major version matches the downloaded ChromeDriver.
If you cannot update Chrome immediately, point wasm-pack at a compatible driver with the `--chromedriver` flag:

```sh
# Replace the path with a chromedriver binary whose major version matches your Chrome.
wasm-pack test --headless --chrome \
  --chromedriver ~/.wasm-pack/cache/chromedriver-<hash>/chromedriver
```

`wasm-pack` caches previously downloaded drivers under `~/Library/Caches/.wasm-pack/` on macOS; inspect that directory to find one whose version matches your Chrome.

### How it works

Each function decorated with `#[wasm_bindgen_test]` runs inside the browser's JS engine with full access to the real DOM.
The test file calls `wasm_bindgen_test_configure!(run_in_browser)` once to opt into this mode.

Tests are organised into integration test files under `tests/`:

| File | What it covers |
|---|---|
| `tests/svg_root.rs` | `SvgRoot` constructors, viewport, and all element factories |
| `tests/svg_node.rs` | `SvgNode` attribute API, clone semantics, `append`, and event handlers |
| `tests/animation_loop.rs` | `AnimationLoop` lifecycle, `start`/`stop` from within callback, and memory retention bug prevention |
| `tests/defs.rs` | `SvgDefs` and `SvgMarker` construction, all factory methods, marker ID validation, `build_defs`/`build_marker` deferred-append, `set_id`, and generic attribute surface |

Shared DOM helpers (creating fixture `<div>` and `<svg>` containers, assertion functions) live in `tests/common.rs` which is included as `mod common` by each test file.

### DOM fixture strategy

Each test appends its own uniquely-named container element to `<body>` so tests do not interfere with each other.
No teardown is needed: the browser page is discarded after the run.

### Event handler tests

Browser events dispatched via `EventTarget::dispatch_event` are **synchronous** — the handler runs inline before `dispatch_event` returns, which means we don't have to worry about any `async` shenanigans.

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

### Failure reporting

All test functions return `Result<(), String>`.
If a test fails, `wasm-bindgen-test` displays the `String` message directly without a stack trace, making failures easier to read in the terminal.

## Accessibility-tree integration test — `cargo test -p accessibility-tree-test`

Everything above proves DOM structure: the right element was created, updated, or removed in the right place.
None of it can see the actual, browser-*computed* accessibility tree — the accessible name/description a screen reader would receive after ARIA precedence, role computation, and pruning are applied — because that lives behind the browser's Accessibility CDP domain, which `wasm-bindgen-test`'s WebDriver-run tests have no access to.

This drives a real Chrome instance directly over the Chrome DevTools Protocol (via the [`headless_chrome`](https://docs.rs/headless_chrome) crate) and queries `Accessibility.getPartialAXTree`, via five independently reported `#[test]` functions confirming:

- a lone `<title>` supplies the accessible name (`title_only_supplies_accessible_name`);
- a `<desc>` supplies the accessible description (`desc_supplies_accessible_description`);
- `aria-label` overrides a `<title>` in name computation (`aria_label_overrides_title`);
- `aria-describedby` overrides a `<desc>` in description computation (`aria_describedby_overrides_desc`);
- a rejected blank `set_title` leaves the element with no accessible name at all (`blank_title_rejection_leaves_no_accessible_name`) — proving the rejection actually prevents the "apparently nameless object exposed to assistive technology" case SVG 2 warns about, not just the DOM mutation.

### Why this lives outside the main crate

The library's own `cargo test`/`cargo nextest run` stays fast and dependency-light on purpose.
This test needs a real, local Chrome/Chromium binary and pulls in `headless_chrome` (and its own dependency tree), so — like `demo-server` — it lives in its own workspace member excluded from the root package's `default-members`.
Plain `cargo build`/`cargo test` at the project root never touch it.

Two supporting crates make this possible:

| Crate | Role |
|---|---|
| `a11y-fixture` | A tiny `wasm-bindgen` cdylib that builds five real `svg-dom` elements (via `set_title`/`set_desc`/`set_attr`) covering the five scenarios above, and signals readiness by adding a `#fixture-ready` element |
| `accessibility-tree-test` | The five `#[test]` functions: build the fixture with `wasm-pack build --target web`, serve it locally over `tiny_http`, launch Chrome via `headless_chrome`, and assert against the `Accessibility.getPartialAXTree` result for each scenario element |

### One shared browser session, multiple independent results

Building the test fixture and launching Chrome are both expensive actions, so all five tests share the same fixture build, static server, and Chrome tab via a lazily-initialised `OnceLock`, rather than each paying that startup cost independently.

`cargo test` still runs the five test functions in parallel, so actual CDP calls against the shared tab are serialised behind a `Mutex`.
`find_element`'s underlying `DOM.getDocument`-then-`DOM.querySelector` sequence is not safe under concurrent access to the same session, even though `Browser` and `Tab` implement `Send + Sync` at the type level.
See the module doc comment in `accessibility-tree-test/tests/accessibility_tree.rs` for the full explanation.

Splitting the original single test (with five sequential `assert_eq!` calls in one function) into five separate `#[test]` functions was a deliberate correction: if they were bundled into a single function, then only the first failing assertion was ever reported and `cargo test` counted the whole scenario suite as a monolithic pass/fail.

### Prerequisites

Same `wasm-pack` install as the browser tests, plus a local Chrome or Chromium install (`headless_chrome` auto-discovers it the same way Puppeteer/Playwright do).

### Running

```sh
cargo test -p accessibility-tree-test
```

This rebuilds the `a11y-fixture` wasm package, serves it on an OS-assigned local port, and drives a headless Chrome instance against it — no manual server or browser setup needed.
