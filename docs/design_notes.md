# Design notes

**Contents**

- [`SvgNode` is a reference-counted handle](#svgnode-is-a-reference-counted-handle)
- [Event listeners are owned by the node](#event-listeners-are-owned-by-the-node)
- [`requestAnimationFrame` self-rescheduling pattern](#requestanimationframe-self-rescheduling-pattern)
- [Per-frame formatting uses a reusable scratch buffer](#per-frame-formatting-uses-a-reusable-scratch-buffer)
- [Transform setters reuse a caller-owned buffer](#transform-setters-reuse-a-caller-owned-buffer)
- [Redundant attribute writes are skipped on hot paths](#redundant-attribute-writes-are-skipped-on-hot-paths)
- [Caller-owned attribute cache for genuinely hot paths](#caller-owned-attribute-cache-for-genuinely-hot-paths)
- [Multi-attribute updates](#multi-attribute-updates)
- [Reusable attribute formatting](#reusable-attribute-formatting)
- [Shared element factory implementation](#shared-element-factory-implementation)
- [Typesafe Path Data Builder](#typesafe-path-data-builder)
- [Ideas Considered and Rejected](rejected_ideas.md)
- [Performance Patterns](#performance-patterns)

## `SvgNode` is a reference-counted handle

`SvgNode` wraps an `Rc`, so cloning it is cheap and all clones refer to the same underlying DOM node.
This makes it natural to share a node between an event closure and the surrounding code without the need for any `unsafe` or `Arc` shenanigans.

## Event listeners are owned by the node

Listeners registered through the managed helpers such as `on_click`, `on_mousedown`, `on_mousemove`, `on_contextmenu`, `on_pointerdown`, `on_pointermove`, `on_pointerenter`, `on_pointerleave`, `on_wheel`, `on_touchstart`, `on_keydown`, `on_focus`, and the drag-and-drop helpers are stored inside the `SvgNode`'s `Rc`.
Each stored entry keeps the event type together with its wasm-bindgen closure, so the DOM listener can be removed before the closure is dropped.

The built-in listener helpers use fixed browser event names, so event types can be stored as `&'static str` values.
They live exactly as long as the last clone of the node exists, so you never have to manage their lifetime separately or call `Closure::forget` for normal `SvgNode` interactions.

This lifetime rule is important for long-lived browser demos and applications: if a function creates a DOM node, attaches a managed listener, and then drops every `SvgNode` handle before returning, the listener is deliberately removed.
Keep at least one handle to every listener-owning node for as long as the interaction should remain active.
The demo gallery does this with a small page-lifetime owner for interactive nodes.

For uncommon browser events, `on_event` provides the same managed lifetime behaviour while using a generic `web_sys::Event`.
`on_event_once` is the one-shot counterpart: the browser removes the listener automatically after the first dispatch (via the native `{ once: true }` option), and the handler's captured values are freed on that dispatch if the `instanceof` check passes.
It accepts a generic typed event `E`; the cast from the raw `Event` is checked at runtime via `instanceof`, so a mismatched `E` silently suppresses the handler and leaves the captured values alive until the node is dropped or its listeners are cleared.
This is preferable to allowing undefined behaviour (that you hope won't do anything bad...)

Handlers are bound as `FnMut`, not `Fn`, so a handler can own and mutate captured state directly — typically a reusable `SvgAttrs` or `String` scratch buffer for a hot `pointermove`/`mousemove` path — without an `Rc<RefCell<...>>` wrapper.
The only constraint, inherited from wasm-bindgen's `Closure<dyn FnMut>`, is that a handler must not run re-entrantly (synchronously dispatching the same event to the same node from inside the handler), which will cause a panic — the same outcome a re-entrant `RefCell` borrow would produce.

Registration is otherwise append-only: the usual way to retire a listener is to drop the entire node.
However, for a long-lived node whose behaviour changes over time (e.g. one that swaps in mode-specific handlers) `clear_listeners` and `remove_listeners(event_type)` detach managed listeners without dropping the node, mirroring the detach-then-drop sequence that `SvgNodeInner::drop` performs (the browser-side callback is removed before its closure is freed).

`remove_listeners` reuses the per-listener event type already stored for cleanup, and compacts a `Many` store back to `One` when a single listener survives.
Since removing a listener frees its closure, neither method may be used to remove the listener that is currently executing; that is documented as the caller's responsibility.

## `requestAnimationFrame` self-rescheduling pattern

`AnimationLoop` uses the standard WASM self-referencing closure pattern: the closure holds an `Rc` to itself so it can re-register with `requestAnimationFrame` after each frame.

Calling `stop()` (or dropping the `AnimationLoop`) cancels the pending handle and sets the `Rc` slot to `None`, which prevents the next re-schedule and allows the closure to be freed.

When `stop()` is called from *inside* the running callback (e.g. a one-shot animation that stops itself on the first frame), freeing the closure immediately would create a use-after-free error on the still-executing closure body.

`AnimationLoop` tracks the dispatch lifecycle via the enum `AnimLoopState` (with members `Idle` / `Dispatching` / `StopPending` / `Stopped`).

When `stop()` detects the `Dispatching` state, it transitions to `StopPending` and defers the slot clear by scheduling a zero-delay `setTimeout`; by the time that timer fires the callback has fully returned and the closure (and all it has captured) are released.
The post-callback code in the RAF wrapper detects `StopPending`, transitions to `Stopped`, and skips re-scheduling.

`StopPending` exists specifically to make `stop()` **genuinely idempotent during dispatch**.
Without it, a second `stop()` call during the same dispatch — whether an explicit second call or the `Drop` impl firing because the handle is dropped inside the callback — would see `Stopped` instead of `Dispatching`, enter the synchronous cleanup branch, and drop the `FrameClosure` while the wrapper body was still executing past `callback(ts)`.
That recreates the exact use-after-free error the dispatch guard was added to prevent.

With `StopPending`, subsequent calls to `stop()` during the same dispatch see `StopPending` and collapse to a no-op: the deferred timer fires exactly once, the closure is never freed mid-execution, and both the "stop twice from inside callback" and "stop then drop from inside callback" scenarios are safe.

This mechanism is shared by the "drop from inside callback", "stop from inside callback (handle kept alive)", and "stop then drop from inside callback" paths, so captured values are released promptly without relying on when the `AnimationLoop` handle is eventually dropped.

Two rare failure paths are worth noting:

1. If `requestAnimationFrame` fails during re-scheduling (after the callback returns), the loop cannot continue; the failure path immediately sets the state to `Stopped` and clears the slot and frees any captured values at that moment rather than waiting for the `AnimationLoop` to be dropped.

   If `setTimeout` scheduling itself fails (a near-impossible browser-level error), the deferred cleanup cannot be registered.
   The post-callback code still transitions the state from `StopPending` to `Stopped`, so *if another `AnimationLoop` handle survives*, a later `stop()` or `Drop` sees `Stopped` and clears the slot synchronously, releasing the RAF closure and its captures.
   But if the handle that called `stop()` was the last `AnimationLoop` handle — i.e. it was dropped from inside the running callback — no later `stop()`/`Drop` call exists to perform that cleanup, and the RAF closure, the shared slot, and everything the user callback captured remain permanently leaked.

1. The callback created by `Closure::once_into_js` for the deferred `setTimeout` is a Rust `FnOnce` handed to JavaScript as a one-shot function; wasm-bindgen only deallocates it when it is *invoked* — an uninvoked `once_into_js` closure is not reclaimed merely by JavaScript garbage collection.

   If `setTimeout` registration fails, that callback is never invoked, so it — and its cloned `Rc` reference to the closure slot — leaks for the life of the page.
   In the recoverable case where another `AnimationLoop` handle survives (see above), that leaked callback's `Rc` ends up pointing at an already-cleared (`None`) slot, so the RAF closure and the user's captured state are not doubled up in the leak; only the one-shot closure and the empty slot allocation remain leaked.

## Per-frame formatting uses a reusable scratch buffer

`AnimationLoop::start_with_frame` supplies an `AnimationFrame` value to each RAF callback.
`AnimationFrame` owns one reusable `String` scratch buffer and exposes helpers such as `set_attr_fmt`, `set_fill_fmt`, `set_d_fmt`, `set_text_fmt`, and `set_points` / `set_points_fixed` (for animating `<polyline>`/`<polygon>` vertices, the latter at a fixed decimal precision to keep the per-frame string short).

Use these helpers for values that change every frame instead of writing `set_attr(..., &format!(...))` or `set_attr(..., &value.to_string())` inside the RAF callback.

The DOM still receives a normal `&str`, but on the Rust/WASM side, the same allocation is used across frames.

## Transform setters reuse a caller-owned buffer

`AnimationFrame`'s reusable buffer only helps callbacks that run *through* `AnimationLoop`.
Event-driven handlers such as drag, pan/zoom, sliders, knobs, follow-the-pointer cursors, resize/selection handles etc. do not, yet they update `transform` just as often.
Writing `set_attr("transform", &format!("translate({x:.1}, {y:.1})"))` inside a `pointermove` handler allocates and then drops a new `String` on every event - which adds up to unnecessary churn.

`src/node/transform.rs` adds a set of helpers that take a caller-owned `&mut String` scratch buffer, clear it, format the new transform into it, and hand it to `set_attr`.
These are

* `set_translate`
* `set_rotate`
* `set_rotate_about`
* `set_scale`
* `set_scale_xy`
* `set_translate_scale`
* `set_transform_fmt`

Reusing one buffer across calls means no new allocation happens unless the formatted text outgrows the buffer's capacity.
For shapes that the typed helpers do not cover, your escape hatch is `set_transform_fmt`: it accepts `std::fmt::Arguments` so `format_args!(...)` can build any transform string without the heap allocation that `format!` would otherwise incur.

The scratch buffer is deliberately **not** stored inside `SvgNode`.
Most nodes are passive geometry that never animate, do folding formatting state into every node would cause them all to grow while benefiting only a few.
Passive nodes can remain small by keeping the buffer external whilst hot paths can opt in explicitly.
Because managed handlers are `FnMut`, a handler that is the sole user of a buffer can simply own it (`let mut buf = String::new()` captured by the closure), as the colour-wheel demo does.
An `Rc<RefCell<String>>` is needed only when one buffer is *shared across several* closures, as the drag/touch demo does for its coordinate readout.

In spite of the fact that writing into a `String` is infallible, `write!` is typed to return `std::fmt::Error`.
`Error` implements `From<std::fmt::Error>`, mapping to the existing `Error::Dom` variant so the helpers can use `?` without a dedicated error variant.

## Redundant attribute writes are skipped on hot paths

`set_attr_if_changed` reads the current value with `get_attribute` and writes only when it differs.
This avoids a redundant DOM write in high-frequency handlers where the same value repeats between frames such as cursor style, `opacity` flags or selected state.

It is not a universal win and the cost can be bigger than it first appears: `get_attribute` **allocates a fresh `String` for the current value and crosses the wasm/JS boundary on every call**, even when it then writes nothing.
So for values that change on every call (such as a drag `transform`) calling `set_attr` remains the cheaper option, keeping `set_attr_if_changed` as best kept for *occasional* de-duplication rather than a per-event hot path.

## Caller-owned attribute cache for genuinely hot paths

For a high-frequency path where an element's attribute value usually repeats (E.G. a cursor style or `opacity` flag touched on every `pointermove`), using `CachedAttr` (in `src/node/cached.rs`) is preferable to calling `set_attr_if_changed`.

`CachedAttr` remembers the last value it wrote on the **Rust** side, so the unchanged case is a plain `&str` comparison against an owned `String`: I.E. no allocation takes place and no call into JS.

The DOM is touched only on a genuine change and even then, the `String` backing buffer is reused (`clear` + `push_str`) rather than reallocated.

This is the same design used for the transform scratch buffer: the cache is **caller-owned** and deliberately not stored inside `SvgNode`, so passive geometry nodes carry no caching state.
Keep one `CachedAttr` per frequently-updated value (typically captured in an event handler's state), dedicated to a single attribute on a single node.

If that attribute is changed by some other path, call `invalidate()` so the cache does not skip a needed write on the strength of a now-stale remembered value.

The same cache also covers text content via `CachedAttr::set_text`, for the equivalent case of a status readout rewritten with the same string on every `pointermove`.
Dedicate a given `CachedAttr` to *either* an attribute or text content, not both, since they share the single remembered value.

For a *formatted* cached value, `CachedAttr::set_fmt` / `set_text_fmt` format into a caller-owned `&mut String` scratch and then cache, so a frequently-touched but rarely-changing formatted readout (a grid-snapped coordinate, a zoom percentage) avoids both the per-call `format!` allocation and the redundant DOM write.
The scratch is a *separate* buffer from the cache, because the cache's own buffer holds the last-written value the new one is compared against.

## Multi-attribute updates

`SvgNode::set_attrs` accepts any `IntoIterator` of `(name, value)` pairs where both sides implement `AsRef<str>`.
This keeps the public API ergonomic for string literals and precomputed string values.

Use `set_attrs` when all values are already strings and you want a compact method for setting multiple attributes at once.

## Reusable attribute formatting

`SvgAttrs` owns a reusable `String` scratch buffer, then `AttrWriter` binds that buffer to a single `SvgNode` for chainable writes.
Use this in order to avoid the need to call `to_string()` or `format!` for numeric or formatted attribute values.

The browser still receives one normal SVG `setAttribute` operation per attribute, but the Rust/WASM side reuses the formatting allocation.
The built-in root and batch element factories use the same mechanism for initial numeric geometry attributes, so repeated element creation does not allocate a fresh formatting `String` per element.

For a single numeric attribute updated on a hot path, `SvgNode::set_attr_display` is the lightweight counterpart taking a caller-owned `&mut String` directly (the same shape as the transform setters), without the ceremony of binding an `AttrWriter`.
The convenience numeric setters such as `set_stroke_width` instead allocate a short-lived `String` per call; that is fine for one-off styling but should be swapped for `set_attr_display` (or an `AttrWriter`) when the value is animated.
The same caveat applies to the `Point`/`Size` `get_*_str` helpers, which each allocate; they are documented as one-off conveniences, not for per-event or per-frame use.

`SvgNode::set_text_fmt` and the `set_text_display` convenience for a single value both format into a caller-owned `&mut String` and set the result as text content.
For a label whose value changes on every event (e.g. a coordinate or status readout updated each time `pointermove` is handled), use `set_text_fmt` or `set_text_display` rather than `set_text(&format!(...))` or `set_text(&value.to_string())`, which allocate and discard a fresh `String` on every call.

When the text instead *repeats* between events, `CachedAttr::set_text` is the better fit since the DOM write only takes place when the value actually changes.
Both the pointer-lifecycle and drag/touch demos route *every* `last: ...` readout writer such as the hot `pointermove`/`touchmove`/`dragover` streams and the discrete transitions alike, through one shared `CachedAttr`, so a burst of identical label updates only touches the DOM on the first write.
The essential rule is that *all* writers should share one cache: partial caching, where some writers bypass it, is what would let the cache skip a genuinely needed write (which is why the cache is fed even from handlers, such as the native `drag` wrappers, that fire between `pointermove`s).

The drag/touch demo's live *coordinate* readout is a separate node that changes on every move, so it keeps using `set_text_fmt` with a scratch buffer shared with the card's transform rather than the cache.

## Shared element factory implementation

`SvgRoot` and `SvgBatch` expose the same basic element factories (`rect`, `circle`, `line`, `path`, `text`, and `group`).
Internally, those factories delegate to a shared `SvgFactory` implementation, so shape-specific creation logic and initial attribute writes exist in one place only.

The only difference between the two paths is the append target: `SvgRoot` appends directly to the live `<svg>`, while `SvgBatch` appends to its `DocumentFragment` until `commit()` is called.

## Typesafe Path Data Builder

`SvgRoot::path(d: &str)` (and its siblings on `SvgBatch`, `SvgDefs`, `SvgClipPath`, `SvgMarker`, `SvgPattern`, `SvgSymbol`) writes a `d` path verbatim.
A hand-written `d` string is free text, so there are no safeguards against it being malformed such as a wrong command letter, a missing argument or a transposed flag.
The SVG parser does not reject a malformed `d` string outright; it simply stops rendering at the first token it cannot parse, so the failure is silent and possibly quite difficult to debug

`PathDef` (in `root::path::path_def`, re-exported at the crate root) removes that failure mode by definition.
A `<path>`'s `d` attribute is built from an ordered `&[PathDef]` slice instead of a string; `build_d` / `write_d` do the formatting.

Since a `PathDef` can only ever represent one well-formed SVG command, there is no possibility of creating a malformed `d` string.

### Two enums, not one, wrapped in a third

`PathDefAbsolute` and `PathDefRelative` mirror each other variant-for-variant (`MoveTo`, `LineTo`, `EllipticalArcTo` etc.), differing only in whether the emitted command is upper- or lower-case.

Real SVG path data routinely mixes both within a single path: an initial absolute move command (`M`) followed by a run of relative line (`l`) or curve (`c`) commands is the idiomatic, compact way to define path data by hand.
It is commonplace for callers to mix both absolute and relative path definitions within the same `d` string.

`PathDef::{Abs, Rel}` is the thinnest possible wrapper that permits this: a single `Vec<PathDef>` (or array/slice literal) can freely interleave absolute and relative segments, exactly as hand-written path data would, while each individual segment stays unambiguous about which coordinate space it uses.

### `HorizontalLineTo` / `VerticalLineTo` take `f64`, not `Point`

The SVG `H`/`h` and `V`/`v` commands each take a single coordinate.
`H` takes a bare `x` and `V` takes a bare `y`, not a full `(x, y)` coordinate pair.

### `EllipticalArc` is a named-field struct, not a five-element tuple

The SVG arc commands (`A`/`a`) take two boolean flags (`large-arc-flag`, `sweep-flag`) to select between the (up to) four geometric solutions for an arc between two points at a given radius.

As adjacent positional `bool`s in a tuple variant, they are easy to transpose — `(true, false)` vs `(false, true)` looks the same at a glance and the compiler cannot catch the swap.
`ArcSize` (`Small`/`Large`) and `ArcSweep` (`CounterClockwise`/`Clockwise`) turn each flag into a self-documenting enum, and bundling all five arc parameters into one named-field `EllipticalArc` struct (rather than a five-argument tuple variant) means every field is labelled at the construction site instead of positional.

### Formatting matches the existing `write_points` convention

Coordinates are written with plain `{}` (`Display`) formatting (Rust's shortest round-trip representation) rather than a fixed decimal count, mirroring `write_points`'s default-precision path in `root::utils`.
This keeps whole-number demo coordinates (`"M 70 10"`, not `"M 70.0 10.0"`) exactly as compact as the hand-written strings they replace.

There is deliberately no fixed-precision variant analogous to `points_fixed`: unlike a `<polyline>`'s point list, path data mixes several different argument shapes (coordinates, a rotation angle, two single-bit flags), so a uniform *"n decimal places for everything"* knob would not obviously do the right thing across all of them; a caller who needs shorter numbers can round the `f64` before constructing the `PathDef`.

### Two allocation tiers, mirroring `points` / `set_attr_display`

An earlier version of this feature had `path_from_defs` and `SvgNode::set_d_from_defs` both call `build_d`, which allocates a fresh `String` on every call.
That included the shared `SvgFactory::create_path_from_defs` default method used by every `path_from_defs` factory sibling — nothing in the shipped API actually called `write_d` outside of `build_d`'s own body, contradicting `write_d`'s own documentation, which describes it as the buffer-reusing path for hot call sites.

The fix follows the crate's existing two-tier split for `points`, verbatim:

- **Node *creation*** (`path_from_defs` on `SvgRoot` and its factory siblings) now writes `d` through the factory's own retained `SvgAttrs` buffer — the same `self.attrs().borrow_mut()` pattern `create_rect` and friends already use — so repeated calls on one factory allocate at most once (for buffer growth), not once per call.

- **Node *updates* on a live `SvgNode`** still have two tiers, exactly as `set_font_size` (allocating) and `set_attr_display` (caller-owned buffer) do for other attributes: `SvgNode::set_d_from_defs` remains a convenience that allocates a short-lived `String` per call (which is fine for an occasional update) while `SvgAttrs::d_from_defs` / `AttrWriter::d_from_defs` and `AnimationFrame::set_d_from_defs` reuse a caller-owned buffer for a path that is morphed on every `pointermove` event or every animation frame.

`SvgNode` has no buffer of its own to reuse (it is a lightweight `Rc` handle, not a factory), which is exactly why the crate's hot-path attribute setters — `set_attr_display`, the transform setters, `AnimationFrame` — all take the scratch buffer as a parameter rather than owning one.
`d_from_defs` follows that same shape rather than inventing a new one.

# Performance Patterns

## High-frequency event coalescing

On some modern devices, the events generated by `pointermove`, `touchmove`, and `wheel` can arrive at the hardware polling rate, which could be as high as 1000 Hz (i.e. one event per millisecond); while the various browser events arrive at a rate between 60 and 120 Hz.

A handler that is called at the hardware polling rate could potentially call `set_translate` or `set_attr` on every delivered event, even though all but the last position before the next paint is immediately discarded.
This might involve performing a Rust → JavaScript crossing, then a possible `setAttribute` DOM call, and a potential SVG layout invalidation for each event.
Such an architecture is highly wasteful of computing resources and can result in jerky or laggy scrolling.

The `AnimationFrame` scratch buffer (see `AnimationLoop::start_with_frame`) removes per-event *allocation*, but it does not reduce the *count* of those crossings.

The fix is standard: record the latest value in the event handler and apply it at most once per animation frame.

Modern browsers partially automate this for pointer events via `getCoalescedEvents()`, but the pattern below works uniformly across all high-frequency event types.

The crate does not yet provide a built-in coalescer type, but the pattern can be built from existing primitives:

```rust
use std::{cell::{Cell, RefCell}, rc::Rc};
use svg_dom::{SvgRoot, WeakSvgNode, root::utils::{Point, Size}};
use wasm_bindgen::prelude::*;

let svg  = SvgRoot::attach("diagram")?;
let node = svg.rect(Point::origin(), Size::new(60.0, 60.0))?;
node.set_fill("steelblue")?;

// --- coalescer state ---
// `pending` holds the latest position submitted by the event handler.
// `scheduled` is true while a RAF has been requested but not yet dispatched.
let pending:   Rc<Cell<Option<Point>>> = Rc::new(Cell::new(None));
let scheduled: Rc<Cell<bool>>          = Rc::new(Cell::new(false));

// Clones for the RAF closure.
let pending_raf   = pending.clone();
let scheduled_raf = scheduled.clone();

// Use a weak handle so the RAF closure does not keep `node` alive, avoiding a
// reference cycle (listener store → closure → Rc<node> → listener store).
// See `WeakSvgNode` for the full explanation.
let node_weak = node.downgrade();

// The RAF callback: read the latest position, clear the scheduled flag, apply.
let raf_cb = Closure::<dyn FnMut()>::new(move || {
    scheduled_raf.set(false);
    if let Some(pt) = pending_raf.take() {
        if let Some(n) = node_weak.upgrade() {
            let _ = n.set_translate(pt.x, pt.y);
        }
    }
});

// Clones for the event handler.
let pending_ev   = pending.clone();
let scheduled_ev = scheduled.clone();
let window       = web_sys::window().unwrap();

node.on_pointermove(move |evt| {
    // Always overwrite with the freshest position.
    pending_ev.set(Some(Point::new(evt.client_x() as f64, evt.client_y() as f64)));
    // Request a frame only if one is not already queued.
    if !scheduled_ev.get() {
        scheduled_ev.set(true);
        let _ = window.request_animation_frame(raf_cb.as_ref().unchecked_ref());
    }
})?;

Ok::<(), svg_dom::Error>(())
```

Key points:

- `pending.set(...)` replaces whatever intermediate position was stored; only the last one matters.
- The `scheduled` flag ensures at most one `requestAnimationFrame` is queued at a time.
  When the RAF fires it clears the flag, so the next event will queue a fresh one.
- The RAF closure uses a `WeakSvgNode` to avoid a reference cycle.
  A strong `SvgNode` clone inside both the event handler and the RAF closure would keep the node (and its listeners) alive indefinitely.
- If you also need the `AnimationFrame` scratch buffer for formatted attribute writes, allocate one `AnimationFrame` outside both closures and put it behind an `Rc<RefCell<AnimationFrame>>`, then borrow it inside the RAF closure.
