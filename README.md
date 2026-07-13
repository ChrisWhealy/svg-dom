# svg-dom

A lightweight Rust/WebAssembly library for creating and mutating live SVG content directly in the browser DOM.

This crate is a work-in-progress and contains known functional gaps that will be filled in time.
That said, all reasonable, conventional steps have been taken to provide a secure, stable and robust foundation upon which to develop future functionality.

***IMPORTANT***<br>This crate targets WebAssembly only.

## ToDo List

- [x] Define custom `Error` object suitable for handling browser DOM errors
- [x] Define `SvgNode` object
- [x] Define `SvgRoot` object
- [x] Define `AnimationLoop` object
- Implement helper functions for basic SVG shapes
  - [x] `<circle>`
  - [x] `<ellipse>`
  - [x] `<group>`
  - [x] `<line>`
  - [x] `<path>`
  - [x] `<polygon>`
  - [x] `<polyline>`
  - [x] `<rect>`
  - [x] `<text>`
- [x] Implement multi-attribute setter for an SVG node
- [x] Implement reusable `SvgAttrs` / `AttrWriter` for allocation-light attribute writing
- [x] Implement batch-building API that allows elements to be added *en masse*
- [x] Share factory implementation between `SvgRoot` and `SvgBatch`
- [x] Build demo server to illustrate current functionality
- [x] Schedule `cargo-deny` to run as a weekly `cron` job
- Implement remaining SVG elements
  - [x] `<defs>`
  - [x] `<linearGradient>` / `<radialGradient>`
  - [ ] `<pattern>`
  - [x] `<clipPath>`
  - [x] `<marker>`
  - [x] `<image>`
  - [x] `<use>`
  - [ ] `<symbol>`
  - [ ] `<tspan>`
  - [ ] `<textPath>`
  - [ ] `<filter>` and `<fe>` elements

# Table of Contents

- [What this crate is](#what-this-crate-is)
- [What this crate is NOT](#what-this-crate-is-not)
- [Building](#building)
- [Demo Server](#demo-server)
- [Quick start](#quick-start)
- [Testing](https://github.com/ChrisWhealy/svg-dom/blob/main/docs/testing.md)
- [Design Notes](https://github.com/ChrisWhealy/svg-dom/blob/main/docs/design_notes.md)
- [Supported SVG Elements](https://github.com/ChrisWhealy/svg-dom/blob/main/docs/elements.md)
- [Gap Analysis](https://github.com/ChrisWhealy/svg-dom/blob/main/docs/gaps.md)

## What this crate is

The `svg-dom` crate acts as a thin wrapper for `web-sys` SVG DOM bindings that allows you to:

- Attach to an existing `<svg>` element in your HTML page
- Create new `<svg>` element programmatically
- Add a basic set of SVG elements:
   - Helper functions exist for `<rect>`, `<circle>`, `<ellipse>`, `<line>`, `<polyline>`, `<polygon>`, `<path>`, `<text>`, `<g>`
   - `<defs>` (`SvgDefs`), `<marker>` (`SvgMarker`), and `<clipPath>` (`SvgClipPath`) are supported for defining reusable assets, with deferred-append helpers (`build_defs` / `build_marker` / `build_clip_path`) that only commit the element to the DOM once construction succeeds; apply a clip path to any element with `set_clip_path_ref`
   - `<use>` is supported via `SvgRoot::use_node` / `SvgBatch::use_node` — stamps a copy of any element referenced by `id` without duplicating DOM nodes; each copy is independently positionable and styleable
   - `<image>` is supported via `SvgRoot::image` / `SvgBatch::image` — embeds a raster image or SVG by URL or `data:` URI with full `preserveAspectRatio` control
   - You get back a cheap-to-clone handle (`SvgNode`) that holds a live reference to the real DOM node
- Using the element's handle, you can mutate individual, multiple or arbitrary attributes:
   - without the need to rebuild or diff the DOM tree
   - via helpers such as `fill`, `stroke`, `d`
   - using `set_attrs` (multiple attributes in one call)
   - formatted values via `SvgAttrs` (scratch-buffer backed; reuses a single `String` allocation across calls)
- Attach managed event listeners directly to individual elements (listener event names are stored as `&'static str` making them allocation-free)
   - `mouse`
   - `pointer`
   - `wheel`
   - `touch`
   - `keyboard`
   - `focus/blur`
   - `drag-and-drop`
   - and generic `Event` handlers
- Drive reactive updates through a `requestAnimationFrame` loop via `AnimationLoop`

## What this crate is NOT

This crate does not use an HTML `<canvas>` element!

Whilst the `<canvas>` element offers a pixel-based, bitmap drawing API that gives you the highest performance ceiling, it also requires you to take ownership of the entire layout, the render loop and hit-testing.

Not only is the implementation cost of such functionality high, it becomes somewhat redundant in light of the fact that the SVG DOM is already a persistent tree of DOM elements that can be individually updated and with which JavaScript (via `web-sys`) can already interact.

Consequently, this crate works exclusively with the SVG DOM.

# Building

Use [wasm-pack](https://rustwasm.github.io/wasm-pack/) to build:

```sh
wasm-pack build --target web
```

# Demo Server

To run a basic demo, start the demo Web Server using

```sh
cargo demo
```

Then visit <http://127.0.0.1:8080/demo>.

The demo gallery includes examples for the managed event wrappers.
Interactive demo nodes are kept alive explicitly for the lifetime of the page because managed listeners are removed automatically when their owning `SvgNode` is dropped.

The coding used in the actual demo implementation is shown beneath each example.

# Quick start

## Core types

| Type | Purpose
|---|---|
| `SvgRoot` | Wraps the root `<svg>` element; entry point for all element creation
| `SvgNode` | Cheap-to-clone handle to a live DOM element; attribute + event API
| `SvgDefs` | `<defs>` container for reusable assets; factory for `SvgMarker`, `SvgClipPath`, gradients, and shape elements
| `SvgMarker` | `<marker>` element for arrowheads and other path decorations; owned id cache + shape factories
| `SvgClipPath` | `<clipPath>` element that restricts rendered region to an arbitrary shape; owned id cache + shape factories
| `AnimationLoop` | Drives a `requestAnimationFrame` loop; stops on `Drop`
| `SvgAttrs` / `AttrWriter` | Reusable scratch buffer for allocation-light attribute writing
| `Error` | All failure modes: element not found, DOM error, cast failure, invalid id (marker / gradient / clip-path), or reserved attribute

## Minimal Demo

```rust
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use svg_dom::{AnimationLoop, SvgAttrs, SvgRoot, root::utils::{Point, Size}};

// An app must keep its AnimationLoop alive somewhere lasting, or it stops the moment the
// handle is dropped. `thread_local!` gives us exactly one such slot per thread (a wasm page
// is single-threaded), initialised lazily on first access.
thread_local! {
    static ANIM: RefCell<Option<AnimationLoop>> = const { RefCell::new(None) };
}

// wasm-bindgen entry point. An exported function's error type must be `Into<JsValue>`, and
// `svg_dom::Error` is not, so the boundary returns `Result<(), JsValue>` and converts there;
// the actual work lives in `build`, which uses `?` with `svg_dom::Error` throughout.
#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    build().map_err(|e| JsValue::from_str(&e.to_string()))
}

fn build() -> Result<(), svg_dom::Error> {
    // Attach to <svg id="diagram"> already present in index.html.
    let svg = SvgRoot::attach("diagram")?;

    // Add a rectangle and give it a colour.
    let rect = svg.rect(Point::new(20.0, 20.0), Size::new(160.0, 80.0))?;
    let mut attrs = SvgAttrs::new();
    rect.attrs(&mut attrs)
        .fill("steelblue")?
        .stroke("white")?
        .stroke_width(2.0)?;

    // `rect` is a page-lifetime node that is intentionally kept alive by the strong clones inside
    // the closures — that is the exception, not the rule.  For nodes that should be discardable,
    // use `rect.downgrade()` and call `upgrade()` inside the closure instead; see `WeakSvgNode`.
    let rect_out = rect.clone();
    rect.on_pointerenter(move |_evt| { let _ = rect_out.set_fill("gold"); })?;

    let rect_back = rect.clone();
    rect.on_pointerleave(move |_evt| { let _ = rect_back.set_fill("steelblue"); })?;

    // Build a <g> group containing a circle and a label.
    let group = svg.group()?;
    let dot = svg.circle(Point::new(200.0, 60.0), 8.0)?;
    let label = svg.text(Point::new(215.0, 65.0), "node A")?;
    group.append(&dot)?;
    group.append(&label)?;

    // Animate: pulse the circle radius each frame.
    //
    // The AnimationLoop must outlive this function — dropping it cancels the pending frame
    // immediately. Park it in the thread-local slot so it runs for the page's lifetime; because
    // AnimationLoop stops on Drop, clearing or replacing that slot later stops the animation
    // cleanly (whereas `std::mem::forget` would simply leak it).
    let anim = AnimationLoop::start_with_frame(move |ts, frame| {
        let r = 8.0 + 4.0 * (ts / 500.0).sin();
        let _ = frame.set_attr_fmt(&dot, "r", format_args!("{r}"));
    })?;
    ANIM.with(|slot| *slot.borrow_mut() = Some(anim));

    Ok(())
}
```

> This example is mirrored in [`examples/readme_minimal.rs`](examples/readme_minimal.rs) and compiled for `wasm32` in CI, so it cannot silently fall out of step with the crate.

### Keeping the loop alive

An `AnimationLoop` stops as soon as its handle is dropped, so it must be held somewhere that lives for as long as the animation should run.

This example parks it in a [`thread_local!`](https://doc.rust-lang.org/std/macro.thread_local.html) slot.
Thread-local storage is a variable with one independent instance *per thread*, created lazily the first time that thread touches it.
A WASM page runs on a single thread, so in practice this is one page-global slot that is initialised on first use and then lives for the lifetime of the page.

This approach is preferable to calling `std::mem::forget(anim)`, since forgetting the loop leaks it permanently and throws away the crate's `Drop`-based stop; whereas a stored loop can be cleared (or replaced) later to stop it cleanly.

A larger app would instead hold the loop in its own long-lived state: maybe an application struct, a framework component, or some similar structure, rather than a free-standing slot in `thread_local!`.

## Managed event handlers

`SvgNode` owns the closures registered by its event helpers and removes the matching DOM listener before those closures are dropped.
Use these helpers instead of registering raw `web-sys` callbacks and calling `Closure::forget`.

The managed wrappers cover common SVG interaction events: click/double-click/context menu, mouse down/up/move/enter/leave/over/out, pointer down/up/move/enter/leave/over/out/cancel, wheel, touch start/move/end/cancel, key down/up, focus/blur, and drag-and-drop.
For less common events, `on_event("event-name", handler)` provides the same managed lifetime with a generic `web_sys::Event`.

When a listener needs to mutate the node it is registered on, capture a `WeakSvgNode` rather than a strong clone.
A strong self-capture creates a cycle — node → listener store → closure → node — that keeps the node alive indefinitely and prevents automatic listener cleanup.

```rust
// `pad` must be kept alive in application state for as long as it should remain interactive.
// The weak handles inside the closures do not count as strong references.
let pad = svg.rect(Point::new(20.0, 20.0), Size::new(160.0, 80.0))?;
pad.set_attrs([("tabindex", "0"), ("style", "cursor:pointer")])?;

let weak = pad.downgrade();
pad.on_mousedown(move |evt| {
    if evt.button() == 0 {
        if let Some(pad) = weak.upgrade() {
            let _ = pad.set_attr("transform", "translate(2,2)");
        }
    }
})?;

let weak = pad.downgrade();
pad.on_mouseup(move |_| {
    if let Some(pad) = weak.upgrade() {
        let _ = pad.set_attr("transform", "translate(0,0)");
    }
})?;

pad.on_contextmenu(move |evt| evt.prevent_default())?;
```

## Setting several attributes at once

Use `SvgNode::set_attrs` when a geometry or style update naturally changes several attributes together.
It accepts string literals and owned `String` values, so it is convenient both for fixed style values and computed geometry:

```rust
let rect = svg.rect(Point::origin(), Size::new(80.0, 40.0))?;
rect.set_attrs([
    ("fill", "steelblue"),
    ("stroke", "white"),
    ("stroke-width", "2"),
    ("rx", "6"),
])?;
```

For repeated numeric or formatted writes, use `SvgAttrs` instead.
It owns a reusable scratch `String`, so display/format values do not require a fresh allocation each time:

```rust
let mut attrs = SvgAttrs::new();
rect.attrs(&mut attrs)
    .fill("steelblue")?
    .stroke("white")?
    .stroke_width(2.0)?
    .fmt("transform", format_args!("translate({:.1}, {:.1})", x, y))?;
```

Element factory methods use `SvgAttrs` internally for initial numeric geometry attributes, so repeated shape creation reuses scratch storage instead of allocating a new formatting buffer per element.

## Allocation-light animation formatting

For attributes that change every animation frame, prefer `AnimationLoop::start_with_frame` over building fresh strings with `format!` inside the RAF callback.
The callback receives an `AnimationFrame` scratch buffer that is allocated once and reused for formatted attributes and text:

```rust
let _loop = AnimationLoop::start_with_frame(move |ts, frame| {
    let x = 100.0 + 50.0 * (ts / 600.0).sin();
    let _ = frame.set_attr_fmt(&dot, "cx", format_args!("{x:.1}"));
    let _ = frame.set_fill_fmt(&dot, format_args!("hsl({:.0},70%,50%)", ts % 360.0));
})?;
```

The existing `AnimationLoop::start` API is still available for callbacks that do not need reusable formatting storage.
