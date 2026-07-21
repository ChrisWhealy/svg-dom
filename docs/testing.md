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

## Accessibility-tree Integration Test — `cargo test -p accessibility-tree-test`

The above tests are designed to prove the DOM structure: the right element was created, updated, or removed in the right place.
None of it can see the actual, browser-*computed* accessibility tree; that is, the accessible name and description a screen reader would receive after ARIA precedence, role computation, and pruning have been applied.
This is because this functionality lives behind the browser's Accessibility CDP domain, to which `wasm-bindgen-test`'s WebDriver-run tests have no access.

This drives a real Chrome instance directly over the Chrome DevTools Protocol (CDP) via the [`headless_chrome`](https://docs.rs/headless_chrome) crate and queries `Accessibility.getPartialAXTree`, via six independently reported `#[test]` functions.
These functions confirm:

- A lone `<title>` supplies the accessible name (`title_only_supplies_accessible_name`);
- A `<desc>` supplies the accessible description (`desc_supplies_accessible_description`);
- A value in `aria-label` overrides a `<title>` in name computation (`aria_label_overrides_title`);
- A value in `aria-describedby` overrides a `<desc>` in description computation (`aria_describedby_overrides_desc`);
- A rejected blank `set_title` leaves the element with no accessible name at all (`blank_title_rejection_leaves_no_accessible_name`), thus proving that the rejection actually prevents the "apparently nameless object exposed to assistive technology" case SVG 2 warns about, not just the DOM mutation;
- A value in `aria-labelledby` overrides *both* `aria-label` and a `<title>` (`aria_labelledby_overrides_title_and_aria_label`) — `aria-labelledby` has strictly higher precedence than `aria-label` in accessible-name computation, not just parity with it, so this scenario gives an element all three and confirms the referenced text wins over both.

### Why this lives outside the main crate

The library's own `cargo test`/`cargo nextest run` stays fast and dependency-light on purpose.
This test needs a real, local Chrome/Chromium binary and pulls in `headless_chrome` (and its own dependency tree), so — like `demo-server` — it lives in its own workspace member excluded from the root package's `default-members`.
Plain `cargo build`/`cargo test` at the project root never touch it.

Two supporting crates make this possible:

| Crate | Role |
|---|---|
| `a11y-fixture` | A tiny `wasm-bindgen` cdylib that builds six real `svg-dom` elements (via `set_title`, `set_desc` and `set_attr`) covering the six scenarios above, and signals readiness by adding a `#fixture-ready` element |
| `accessibility-tree-test` | The six `#[test]` functions: build the fixture with `wasm-pack build --target web`, serve it locally over `tiny_http`, launch Chrome via `headless_chrome`, and assert against the `Accessibility.getPartialAXTree` result for each scenario element |

### One Shared Browser Session, Multiple Independent Results

Building the test fixture and launching Chrome are both expensive actions, so all six tests share the same fixture build, static server, and Chrome tab via a lazily-initialised `OnceLock`, rather than each paying that startup cost independently.

`cargo test` still runs the six test functions in parallel, so actual CDP calls against the shared tab are serialised behind a `Mutex`.
`find_element`'s underlying `DOM.getDocument`-then-`DOM.querySelector` sequence is not safe under concurrent access to the same session, even though `Browser` and `Tab` implement `Send + Sync` at the type level.
See the module doc comment in `accessibility-tree-test/tests/accessibility_tree.rs` for the full explanation.

Splitting the original single test (with sequential `assert_eq!` calls in one function) into separate `#[test]` functions was a deliberate correction: if they were bundled into a single function, then only the first failing assertion was ever reported and `cargo test` counted the whole scenario suite as a monolithic pass/fail.

### Prerequisites

Same `wasm-pack` install as the browser tests, plus a local Chrome or Chromium install (`headless_chrome` auto-discovers it the same way Puppeteer/Playwright do).

### Running

```sh
cargo test -p accessibility-tree-test
```

### Running in CI

Runs as its own job (`accessibility-tree-test`) in `.github/workflows/ci.yml`, on every push/PR, using the Chrome installation already present on GitHub's `ubuntu-latest` runner image — no extra install step.

It was initially added without any CI job at all, so it protected nothing: the workspace's `default-members` deliberately excludes it (see above), so plain `cargo test`/`cargo nextest run` never runs it, and none of the other CI jobs invoke it either.
A regression here — the test failing to compile, or Chrome's actual accessible-name/description computation drifting away from what the crate assumes — could land on `main` without any CI job noticing.
Being a separate job (rather than an extra step tacked onto `browser-tests`) means its failure is reported independently and doesn't obscure or get obscured by the unrelated `wasm-bindgen-test` results, while still gating the merge like any other required check.

The Chrome launch in `accessibility-tree-test` explicitly passes `sandbox(false)` rather than using `Browser::default()`'s sandboxed default — recent Ubuntu (24.04+, which `ubuntu-latest` now resolves to) restricts unprivileged user namespaces via AppArmor, which breaks Chrome's own sandbox initialisation even for the runner's non-root user.
Since this test only ever loads a local fixture page the crate builds itself, there is no untrusted content for the sandbox to matter for, so it is disabled unconditionally (not just in CI) to keep local and CI runs on the same code path.
See the `# Why the browser is launched with sandbox(false)` section of the module doc comment for the full explanation.

This rebuilds the `a11y-fixture` wasm package, serves it on an OS-assigned local port, and drives a headless Chrome instance against it — no manual server or browser setup needed.
