# Quick Start

## Core Types

| Type | Purpose
|---|---|
| `SvgRoot` | Wraps the root `<svg>` element; entry point for all element creation
| `SvgNode` | Cheap-to-clone handle to a live DOM element; attribute + event API
| `SvgDefs` | `<defs>` container for reusable assets; factory for `SvgMarker`, `SvgClipPath`, `SvgMask`, `SvgPattern`, `SvgSymbol`, `SvgFilter`, `SvgView`, gradients, and shape elements
| `SvgMarker` | `<marker>` element for arrowheads and other path decorations; owned id cache + shape factories
| `SvgClipPath` | `<clipPath>` element that restricts rendered region to an arbitrary shape; owned id cache + shape factories
| `SvgMask` | `<mask>` element that reveals/hides rendered region by luminance or alpha; owned id cache + shape factories
| `SvgPattern` | `<pattern>` element that tiles its content as a fill or stroke paint server; owned id cache + shape factories
| `SvgFilter` | `<filter>` element applying raster effects via a chain of filter-primitive builder methods (`feGaussianBlur`, `feColorMatrix`, `feDiffuseLighting`, ...); owned id cache
| `SvgLinearGradient` / `SvgRadialGradient` | `<linearGradient>`/`<radialGradient>` paint-server gradients; owned id cache
| `SvgSymbol` | `<symbol>` element defining a reusable, scaled viewport; owned id cache + shape factories; stamped via `<use>`
| `SvgView` | `<view>` element defining a named `viewBox`/`preserveAspectRatio`; owned id cache; navigated to via a `#id` URL fragment
| `PathDef` | Type-safe `<path>` `d`-attribute segment; builds a path from `&[PathDef]` via `path_from_defs` that avoids the possibility of creating a malformed `d` string
| `AnimationLoop` | Drives a `requestAnimationFrame` loop; stops on `Drop`
| `SvgAttrs` / `AttrWriter` | Reusable scratch buffer for allocation-light attribute writing
| `Error` | Crate-wide DOM, casting, identifier, path, viewBox, reserved-attribute, accessibility, and feature-specific validation failures — see the type's own rustdoc for the full, current list of variants

## Minimal Demo

```rust
use std::cell::RefCell;
use svg_dom::{
    AnimationLoop, SvgAttrs, SvgRoot,
    root::utils::{Point, Size},
};
use wasm_bindgen::prelude::*;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// An app must keep its AnimationLoop alive in some long lasting structure; otherwise it will stop the moment the handle
// is dropped. `thread_local!` gives us one slot per execution thread; this DOM-facing code runs on the page's main
// thread, so it acts as a slot whose lifetime equals that of the Web page. It is initialised lazily on first access.
// In a `#[wasm_bindgen(start)]` app, this is the idiomatic place to park state.
thread_local! {
    static ANIM: RefCell<Option<AnimationLoop>> = const { RefCell::new(None) };
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    build().map_err(|e| JsValue::from_str(&e.to_string()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn build() -> Result<(), svg_dom::Error> {
    // Attach to <svg id="diagram"> already present in index.html.
    let svg = SvgRoot::attach("diagram")?;

    // Add a rectangle and give it a colour.
    let rect = svg.rect(Point::new(20.0, 20.0), Size::new(160.0, 80.0))?;
    let mut attrs = SvgAttrs::new();
    rect.attrs(&mut attrs).fill("steelblue")?.stroke("white")?.stroke_width(2.0)?;

    // `rect` is a page-lifetime node that is intentionally kept alive by the strong clones inside the closures —
    // that is the exception, not the rule.  For nodes that should be discardable, use `rect.downgrade()` and call
    // `upgrade()` inside the closure instead; see `svg_dom::WeakSvgNode`.
    let rect_out = rect.clone();
    rect.on_pointerenter(move |_evt| {
        let _ = rect_out.set_fill("gold");
    })?;

    let rect_back = rect.clone();
    rect.on_pointerleave(move |_evt| {
        let _ = rect_back.set_fill("steelblue");
    })?;

    // Build a <g> group containing a circle and a label.
    let group = svg.group()?;
    let dot = svg.circle(Point::new(200.0, 60.0), 8.0)?;
    let label = svg.text(Point::new(215.0, 65.0), "node A")?;
    group.append(&dot)?;
    group.append(&label)?;

    // Animate: pulse the circle radius each frame. The AnimationLoop must outlive this function — dropping it cancels
    // the pending frame immediately — so park it in the thread-local slot for the page's lifetime. Because
    // AnimationLoop stops on Drop, clearing or replacing that slot later stops the animation cleanly.
    let anim = AnimationLoop::start_with_frame(move |ts, frame| {
        let r = 8.0 + 4.0 * (ts / 500.0).sin();
        let _ = frame.set_attr_fmt(&dot, "r", format_args!("{r}"));
    })?;
    ANIM.with(|slot| *slot.borrow_mut() = Some(anim));

    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn main() {}
```

> This example is mirrored in [`examples/readme_minimal.rs`](examples/readme_minimal.rs) and compiled for `wasm32` in CI, so it cannot silently fall out of step with the crate.

### Keeping the loop alive

An `AnimationLoop` stops as soon as its handle is dropped, so it must be held somewhere that lives for as long as the animation should run.

This example parks it in a [`thread_local!`](https://doc.rust-lang.org/std/macro.thread_local.html) slot.
Thread-local storage is a variable with one independent instance *per thread*, created lazily the first time that thread touches it.
This DOM-facing code runs on the page's main thread.
In practice, that makes it one page-global slot, initialised on first use and living for the lifetime of the page.

This approach is preferable to calling `std::mem::forget(anim)`, since forgetting the loop leaks it permanently and throws away the crate's `Drop`-based stop; whereas a stored loop can be cleared (or replaced) later to stop it cleanly.

A larger app would instead hold the loop in its own long-lived state: maybe an application struct, a framework component, or some similar structure, rather than a free-standing slot in `thread_local!`.

## Managed Event Handlers

`SvgNode` owns the closures registered by its event helpers and removes the matching DOM listener before those closures are dropped.
Use these helpers instead of registering raw `web-sys` callbacks and calling `Closure::forget`.

The managed wrappers cover most of the common SVG interaction events:
* click
* double-click
* context menu
* mouse[down, up, move, enter, leave, over, out]
* pointer[down, up, move, enter, leave, over, out, cancel]
* wheel
* touch[start, move, end, cancel]
* key[down, up]
* focus and blur
* drag-and-drop

For less common events, `on_event("event-name", handler)` provides the same managed lifetime with a generic `web_sys::Event`.

When a listener needs to mutate the node on which it is registered, capture a `WeakSvgNode` rather than a strong clone.
The problem here is that if you create a strong clone, it will end up creating a reference cycle:

```text
node → listener store → closure → node
```

This then keeps the node alive indefinitely and inhibits automatic listener cleanup.

```rust
// `pad` must be kept alive in the application state for as long as it is required to be interactive.
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

## Setting Several Attributes at Once

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

## Allocation-light Animation Formatting

For attributes that change every animation frame, the use of `AnimationLoop::start_with_frame` is preferable over building fresh strings with `format!` inside the RAF callback.
The callback receives an `AnimationFrame` scratch buffer with initial capacity that retains and reuses its previous allocation size between frames; thus, it grows only when a formatted value exceeds its current capacity:

```rust
let _loop = AnimationLoop::start_with_frame(move |ts, frame| {
    let x = 100.0 + 50.0 * (ts / 600.0).sin();
    let _ = frame.set_attr_fmt(&dot, "cx", format_args!("{x:.1}"));
    let _ = frame.set_fill_fmt(&dot, format_args!("hsl({:.0},70%,50%)", ts % 360.0));
})?;
```

The existing `AnimationLoop::start` API is still available for callbacks that do not need reusable formatting storage.
