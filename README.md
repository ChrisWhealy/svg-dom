# svg-dom

A lightweight Rust/WebAssembly library for creating and mutating live SVG content directly in the browser DOM.

***IMPORTANT***<br>This crate targets WebAssembly only.

## What this crate is

`svg-dom` provides a thin, ergonomic layer over the `web-sys` SVG DOM bindings.

It lets you:

- either attach to an existing `<svg>` element in your HTML page, or create new one programmatically
- add SVG elements (`<rect>`, `<circle>`, `<line>`, `<path>`, `<text>`, `<g>`) and get back cheap-to-clone handles (`SvgNode`) that hold a live reference to the real DOM node
- mutate individual attributes (`fill`, `stroke`, `d`, or any arbitrary attribute) on those handles at any time without to need to rebuild or diff the DOM tree
- attach mouse event listeners (`click`, `mouseover`, `mouseout`) directly to individual elements
- drive reactive updates through a `requestAnimationFrame` loop via `AnimationLoop`

## What this crate is NOT

This crate has nothing to do with the HTML `<canvas>` element.
`<canvas>` is a pixel-based bitmap drawing API which, although it gives you the highest performance ceiling, requires you to take ownership of the entire layout, the render loop and hit-testing.
The implementation cost of such functionality is prohibitive, especially in light of the fact the SVG DOM already provides the bulk of this functionality.

Consequently, `svg-dom` works with the SVG DOM — a retained tree of vector elements — where each element persists between frames and can be individually updated.

## Core types

| Type | Purpose |
|---|---|
| `SvgRoot` | Wraps the root `<svg>` element; entry point for all element creation
| `SvgNode` | Cheap-to-clone handle to a live DOM element; attribute + event API
| `AnimationLoop` | Drives a `requestAnimationFrame` loop; stops on `Drop`
| `Error` | All failure modes: element not found, DOM error, cast failure

## Quick start

```rust,no_run
use wasm_bindgen::prelude::*;
use svg_dom::{AnimationLoop, SvgRoot};

#[wasm_bindgen(start)]
pub fn run() -> Result<(), svg_dom::Error> {
    // Attach to <svg id="diagram"> already present in index.html.
    let svg = SvgRoot::attach("diagram")?;

    // Add a rectangle and give it a colour.
    let rect = svg.rect(20.0, 20.0, 160.0, 80.0)?;
    rect.set_fill("steelblue")?;
    rect.set_stroke("white")?;
    rect.set_stroke_width(2.0)?;

    // Clone the handle so the event closure can refer to the same DOM node.
    let rect_out = rect.clone();
    rect.on_mouseover(move |_evt| { let _ = rect_out.set_fill("gold"); })?;

    let rect_back = rect.clone();
    rect.on_mouseout(move |_evt| { let _ = rect_back.set_fill("steelblue"); })?;

    // Build a <g> group containing a circle and a label.
    let group = svg.group()?;
    let dot   = svg.circle(200.0, 60.0, 8.0)?;
    let label = svg.text(215.0, 65.0, "node A")?;
    group.append(&dot)?;
    group.append(&label)?;

    // Animate: pulse the circle radius each frame.
    // The AnimationLoop must be kept alive (e.g. stored in a static or leaked) for
    // the loop to continue — dropping it cancels the pending frame immediately.
    let _loop = AnimationLoop::start(move |ts| {
        let r = 8.0 + 4.0 * (ts / 500.0).sin();
        let _ = dot.set_attr("r", &r.to_string());
    })?;

    Ok(())
}
```

# Testing

The test suite has two tiers that use different runners.

## Unit tests — `cargo nextest run`

Pure Rust tests with no browser dependency.
Currently covers the `Error` type's `Display` and `Debug` implementations and its inner-value accessors.

```sh
cargo nextest run
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
wasm-pack test --headless --chrome    # or --firefox
```

### How it works

Each function decorated with `#[wasm_bindgen_test]` runs inside the browser's JS engine with full access to the real DOM.
The test file calls `wasm_bindgen_test_configure!(run_in_browser)` once to opt into this mode.

Tests are organised into two integration test files under `tests/`:

| File | What it covers |
|---|---|
| `tests/svg_root.rs` | `SvgRoot` constructors, viewport, and all element factories |
| `tests/svg_node.rs` | `SvgNode` attribute API, clone semantics, `append`, and event handlers |

Shared DOM helpers (creating fixture `<div>` and `<svg>` containers, assertion functions) live in `tests/common.rs` which is included as `mod common` by both test files.

### DOM fixture strategy

Each test appends its own uniquely-named container element to `<body>` so tests do not interfere with each other.
No teardown is needed: the browser page is discarded after the run.

### Event handler tests

Browser events dispatched via `EventTarget::dispatch_event` are **synchronous** — the handler runs inline before `dispatch_event` returns, which means we don't have tp worry about any `async` shenanigans.

A shared `Rc<Cell<bool>>` flag is set inside the handler, and the test checks the flag immediately after dispatch:

```rust
let fired = Rc::new(Cell::new(false));
let fired_c = fired.clone();
node.on_click(move |_| { fired_c.set(true); })?;

let event = MouseEvent::new("click")?;
node.as_element().dispatch_event(&event)?;  // handler fires here, synchronously

assert!(fired.get());
```

### Failure reporting

All test functions return `Result<(), String>`.
If a test fails, `wasm-bindgen-test` displays the `String` message directly without a stack trace, making failures easier to read
in the terminal.

# Building

Use [wasm-pack](https://rustwasm.github.io/wasm-pack/) to build:

```sh
wasm-pack build --target web
```

# Design notes

## `SvgNode` is a reference-counted handle

`SvgNode` wraps an `Rc`, so cloning it is cheap and all clones refer to the same underlying DOM node.
This makes it natural to share a node between an event closure and the surrounding code without the need for any `unsafe` or `Arc` shenanigans.

## Event closures are owned by the node

Closures registered with `on_click` / `on_mouseover` / `on_mouseout` are stored inside the `SvgNode`'s `Rc`.
They live exactly as long as the last clone of the node exists, so you never have to manage their lifetime separately.

## `requestAnimationFrame` self-rescheduling pattern

`AnimationLoop` uses the standard WASM self-referencing closure pattern: the closure holds an `Rc` to itself so it can re-register with `requestAnimationFrame` after each frame.

Calling `stop()` (or dropping the `AnimationLoop`) sets that `Rc` slot to `None`, which prevents the next re-schedule and allows the closure to be freed.
