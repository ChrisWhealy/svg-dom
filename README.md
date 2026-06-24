# svg-dom

A lightweight Rust/WebAssembly library for creating and mutating live SVG content directly in the browser DOM.

This crate is still just an MVP and contains known functional gaps that will be filled in time&hellip;

***IMPORTANT***<br>This crate targets WebAssembly only.

# Table of Contents

- [What this crate is](#what-this-crate-is)
- [What this crate is NOT](#what-this-crate-is-not)
- [Building](#building)
- [Demo Server](#demo-server)
- [Quick start](#quick-start)
- [Testing](https://github.com/ChrisWhealy/svg-dom/blob/main/docs/testing.md)
- [Design Notes](https://github.com/ChrisWhealy/svg-dom/blob/main/docs/design_notes.md)
- [Gap Analysis](https://github.com/ChrisWhealy/svg-dom/blob/main/docs/gaps.md)

## ToDo List

- [x] Define custom `Error` object suitable for handling browser DOM errors
- [x] Define `SvgNode` object
- [x] Define `SvgRoot` object
- [x] Define `AnimationLoop` object
- Implement helper functions for basic SVG shapes
  - [x] `<circle>`
  - [x] `<group>`
  - [x] `<line>`
  - [x] `<path>`
  - [x] `<rect>`
  - [x] `<text>`
- [x] Implement multi-attribute setter for an SVG node
- [x] Implement reusable `SvgAttrs` / `AttrWriter` for allocation-light attribute writing
- [x] Implement batch-building API that allows elements to be added *en masse*
- [x] Share factory implementation between `SvgRoot` and `SvgBatch`
- [x] Build demo server to illustrate current functionality
- Implement remaining SVG elements
  - [ ] `<ellipse>`
  - [ ] `<polyline>` / `<polygon>`
  - [ ] `<defs>`
  - [ ] `<linearGradient>` / `<radialGradient>`
  - [ ] `<pattern>`
  - [ ] `<clipPath>`
  - [ ] `<marker>`
  - [ ] `<image>`
  - [ ] `<use>` / `<symbol>`
  - [ ] `<tspan>`
  - [ ] `<textPath>`
  - [ ] `<filter>` and `<fe>` elements

## What this crate is

The `svg-dom` crate acts as a thin wrapper for `web-sys` SVG DOM bindings that allows you to:

- Attach to an existing `<svg>` element in your HTML page
- Create new `<svg>` element programmatically
- Add a basic set of SVG elements using helper functions (`<rect>`, `<circle>`, `<line>`, `<path>`, `<text>`, `<g>`)
   - You get back a cheap-to-clone handle (`SvgNode`) that holds a live reference to the real DOM node
- Using the element's handle, you can mutate attributes without the need to rebuild or diff the DOM tree
   - via helpers such as `fill`, `stroke`, `d`
   - Any arbitrary attribute
   - Several attributes using `set_attrs`
   - Formatted values via `SvgAttrs`
- Attach managed event listeners directly to individual elements — mouse, pointer, wheel, touch, keyboard, focus/blur, drag-and-drop, and generic `Event` handlers — with listener event names stored as `&'static str` (i.e. allocation-free)
- Drive reactive updates through a `requestAnimationFrame` loop via `AnimationLoop`

## What this crate is NOT

This crate does not use an HTML `<canvas>` element!

The `<canvas>` element offers a pixel-based, bitmap drawing API which, although it gives you the highest performance ceiling, requires you to take ownership of the entire layout, the render loop and hit-testing.

Not only is the implementation cost of such functionality high, it becomes somewhat redundant in light of the fact that the SVG DOM is already a persistent tree of vector elements that can be individually updated and with which JavaScript (via `web-sys`) can already interact.

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

Then visit <http://127.0.0.1:8000/demo>.

The demo gallery includes examples for the managed event wrappers. Interactive demo nodes are kept alive explicitly for the lifetime of the page because managed listeners are removed automatically when their owning `SvgNode` is dropped.

# Quick start

## Core types

| Type | Purpose
|---|---|
| `SvgRoot` | Wraps the root `<svg>` element; entry point for all element creation
| `SvgNode` | Cheap-to-clone handle to a live DOM element; attribute + event API
| `AnimationLoop` | Drives a `requestAnimationFrame` loop; stops on `Drop`
| `SvgAttrs` / `AttrWriter` | Reusable scratch buffer for allocation-light attribute writing
| `Error` | All failure modes: element not found, DOM error or client-side cast failure

## Minimal Demo

```rust,no_run
use wasm_bindgen::prelude::*;
use svg_dom::{AnimationLoop, SvgAttrs, SvgRoot, root::utils::{Point, Size}};

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

    // Clone the handle so the event closure can refer to the same DOM node.
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
    // immediately. Here we leak it with `std::mem::forget` so it runs for the lifetime of the
    // page; a real app would instead store it somewhere lasting (a `static`, app state, or a
    // field on a long-lived struct).
    let anim = AnimationLoop::start_with_frame(move |ts, frame| {
        let r = 8.0 + 4.0 * (ts / 500.0).sin();
        let _ = frame.set_attr_fmt(&dot, "r", format_args!("{r}"));
    })?;
    std::mem::forget(anim);

    Ok(())
}
```

> This example is mirrored in [`examples/readme_minimal.rs`](examples/readme_minimal.rs) and compiled for `wasm32` in CI, so it cannot silently fall out of step with the crate.

## Managed event handlers

`SvgNode` owns the closures registered by its event helpers and removes the matching DOM listener before those closures are dropped. Use these helpers instead of registering raw `web-sys` callbacks and calling `Closure::forget`.

The managed wrappers cover common SVG interaction events: click/double-click/context menu, mouse down/up/move/enter/leave/over/out, pointer down/up/move/enter/leave/over/out/cancel, wheel, touch start/move/end/cancel, key down/up, focus/blur, and drag-and-drop. For less common events, `on_event("event-name", handler)` provides the same managed lifetime with a generic `web_sys::Event`.

```rust,no_run
let pad = svg.rect(Point::new(20.0, 20.0), Size::new(160.0, 80.0))?;
pad.set_attrs([("tabindex", "0"), ("style", "cursor:pointer")])?;

let pressed = pad.clone();
pad.on_mousedown(move |evt| {
    if evt.button() == 0 {
        let _ = pressed.set_attr("transform", "translate(2,2)");
    }
})?;

let released = pad.clone();
pad.on_mouseup(move |_| {
    let _ = released.set_attr("transform", "translate(0,0)");
})?;

pad.on_contextmenu(move |evt| evt.prevent_default())?;
```

## Setting several attributes at once

Use `SvgNode::set_attrs` when a geometry or style update naturally changes several attributes together.
It accepts string literals and owned `String` values, so it is convenient both for fixed style values and computed geometry:

```rust,no_run
let rect = svg.rect(Point::origin(), Size::new(80.0, 40.0))?;
rect.set_attrs([
    ("fill", "steelblue"),
    ("stroke", "white"),
    ("stroke-width", "2"),
    ("rx", "6"),
])?;
```

For repeated numeric or formatted writes, use `SvgAttrs` instead.  It owns a reusable scratch `String`, so display/format values do not require a fresh allocation each time:

```rust,no_run
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

```rust,no_run
let _loop = AnimationLoop::start_with_frame(move |ts, frame| {
    let x = 100.0 + 50.0 * (ts / 600.0).sin();
    let _ = frame.set_attr_fmt(&dot, "cx", format_args!("{x:.1}"));
    let _ = frame.set_fill_fmt(&dot, format_args!("hsl({:.0},70%,50%)", ts % 360.0));
})?;
```

The existing `AnimationLoop::start` API is still available for callbacks that do not need reusable formatting storage.
