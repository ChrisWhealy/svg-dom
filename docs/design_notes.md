# Design notes

## `SvgNode` is a reference-counted handle

`SvgNode` wraps an `Rc`, so cloning it is cheap and all clones refer to the same underlying DOM node.
This makes it natural to share a node between an event closure and the surrounding code without the need for any `unsafe` or `Arc` shenanigans.

## Event listeners are owned by the node

Listeners registered with `on_click` / `on_pointerenter` / `on_pointerleave` are stored inside the `SvgNode`'s `Rc`.
Each stored entry keeps the event type together with its wasm-bindgen closure, so the DOM listener can be removed before the closure is dropped.
They live exactly as long as the last clone of the node exists, so you never have to manage their lifetime separately.

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
This keeps the public API ergonomic for string literals while also allowing computed `String` values from helpers such as `Point::get_x_str()`.

The built-in element factories and the browser demos use this setter for grouped initial geometry, text, and presentation attributes.  The browser still receives one normal SVG `setAttribute` operation per attribute, but crate code now has a single path for grouped attribute application and callers no longer need to spell out several repeated `set_attr` calls.
