# Design notes

## `SvgNode` is a reference-counted handle

`SvgNode` wraps an `Rc`, so cloning it is cheap and all clones refer to the same underlying DOM node.
This makes it natural to share a node between an event closure and the surrounding code without the need for any `unsafe` or `Arc` shenanigans.

## Event listeners are owned by the node

Listeners registered through the managed helpers such as `on_click`, `on_mousedown`, `on_mousemove`, `on_contextmenu`, `on_pointerdown`, `on_pointermove`, `on_pointerenter`, `on_pointerleave`, `on_wheel`, `on_touchstart`, `on_keydown`, `on_focus`, and the drag-and-drop helpers are stored inside the `SvgNode`'s `Rc`.
Each stored entry keeps the event type together with its wasm-bindgen closure, so the DOM listener can be removed before the closure is dropped.

The built-in listener helpers use fixed browser event names, so event types can be stored as `&'static str` values.
They live exactly as long as the last clone of the node exists, so you never have to manage their lifetime separately or call `Closure::forget` for normal `SvgNode` interactions.

This lifetime rule is important for long-lived browser demos and applications: if a function creates a DOM node, attaches a managed listener, and then drops every `SvgNode` handle before returning, the listener is deliberately removed. Keep at least one handle to every listener-owning node for as long as the interaction should remain active. The demo gallery does this with a small page-lifetime owner for interactive nodes.

For uncommon browser events, `on_event` provides the same managed lifetime behaviour while using a generic `web_sys::Event`.

## `requestAnimationFrame` self-rescheduling pattern

`AnimationLoop` uses the standard WASM self-referencing closure pattern: the closure holds an `Rc` to itself so it can re-register with `requestAnimationFrame` after each frame.

Calling `stop()` (or dropping the `AnimationLoop`) sets that `Rc` slot to `None`, which prevents the next re-schedule and allows the closure to be freed.

## Per-frame formatting uses a reusable scratch buffer

`AnimationLoop::start_with_frame` supplies an `AnimationFrame` value to each RAF callback.
`AnimationFrame` owns one reusable `String` scratch buffer and exposes helpers such as `set_attr_fmt`, `set_fill_fmt`, `set_d_fmt`, and `set_text_fmt`.

Use these helpers for values that change every frame instead of writing `set_attr(..., &format!(...))` or `set_attr(..., &value.to_string())` inside the RAF callback.

The DOM still receives a normal `&str`, but on the Rust/WASM side, the same allocation is used across frames.

## Multi-attribute updates

`SvgNode::set_attrs` accepts any `IntoIterator` of `(name, value)` pairs where both sides implement `AsRef<str>`.
This keeps the public API ergonomic for string literals and precomputed string values.

Use `set_attrs` when all values are already strings and you want a compact method for setting multiple attributes at once.

## Reusable attribute formatting

`SvgAttrs` owns a reusable `String` scratch buffer, then `AttrWriter` binds that buffer to a single `SvgNode` for chainable writes.
Use this in order to avoid the need to call `to_string()` or `format!` for numeric or formatted attribute values.

The browser still receives one normal SVG `setAttribute` operation per attribute, but the Rust/WASM side reuses the formatting allocation.
The built-in root and batch element factories use the same mechanism for initial numeric geometry attributes, so repeated element creation does not allocate a fresh formatting `String` per element.

## Shared element factory implementation

`SvgRoot` and `SvgBatch` expose the same basic element factories (`rect`, `circle`, `line`, `path`, `text`, and `group`).
Internally, those factories delegate to a shared `SvgFactory` implementation, so shape-specific creation logic and initial attribute writes exist in one place only.

The only difference between the two paths is the append target: `SvgRoot` appends directly to the live `<svg>`, while `SvgBatch` appends to its `DocumentFragment` until `commit()` is called.
