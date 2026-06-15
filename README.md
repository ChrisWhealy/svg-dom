# svg-dom

A lightweight Rust/WebAssembly library for creating and mutating live SVG content directly in the browser DOM.

This crate is still just a PoC and has known functional gaps that will be filled in time...

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

## What this crate is

The `svg-dom` crate acts as a thin wrapper for `web-sys` SVG DOM bindings that allows you to:

- Attach to an existing `<svg>` element in your HTML page
- Create new `<svg>` element programmatically
- Add a basic set of SVG elements using helper functions (`<rect>`, `<circle>`, `<line>`, `<path>`, `<text>`, `<g>`)
   - You get back a cheap-to-clone handle (`SvgNode`) that holds a live reference to the real DOM node
- Mutate an element's individual attributes (`fill`, `stroke`, `d`, or any arbitrary attribute) on those handles without the need to rebuild or diff the DOM tree
- Attach mouse event listeners (`click`, `mouseover`, `mouseout`) directly to individual elements
- Drive reactive updates through a `requestAnimationFrame` loop via `AnimationLoop`

## What this crate is NOT

This crate does not use an HTML `<canvas>` element!

The `<canvas>` element offers a pixel-based, bitmap drawing API which, although it gives you the highest performance ceiling, requires you to take ownership of the entire layout, the render loop and hit-testing.

Not only is the implementation cost of such functionality is high, it becomes somewhat redundant in light of the fact that the SVG DOM is already a persistent tree of vector elements that can be individually updated.
Consequently, this crate works only with the SVG DOM.

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

Then visit <http://127.0.0.1:8000/demo>

# Quick start

## Core types

| Type | Purpose |
|---|---|
| `SvgRoot` | Wraps the root `<svg>` element; entry point for all element creation
| `SvgNode` | Cheap-to-clone handle to a live DOM element; attribute + event API
| `AnimationLoop` | Drives a `requestAnimationFrame` loop; stops on `Drop`
| `Error` | All failure modes: element not found, DOM error, cast failure

## Minimal Demo

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
    let dot = svg.circle(200.0, 60.0, 8.0)?;
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
