# Performance patterns

[← Back to design notes](README.md)

Allocation-avoidance and DOM-write-avoidance patterns for hot paths — per-frame animation callbacks and high-frequency
event handlers (`pointermove`, `touchmove`, `wheel`).
See [Transforms](transforms.md) for the `Matrix2D`/`set_matrix` API-design decisions that sit alongside the
buffer-reuse pattern described here.

**Contents**

- [Per-frame formatting uses a reusable scratch buffer](#per-frame-formatting-uses-a-reusable-scratch-buffer)
- [Transform setters reuse a caller-owned buffer](#transform-setters-reuse-a-caller-owned-buffer)
- [Redundant attribute writes are skipped on hot paths](#redundant-attribute-writes-are-skipped-on-hot-paths)
- [Caller-owned attribute cache for genuinely hot paths](#caller-owned-attribute-cache-for-genuinely-hot-paths)
- [Multi-attribute updates](#multi-attribute-updates)
- [Reusable attribute formatting](#reusable-attribute-formatting)
- [High-frequency event coalescing](#high-frequency-event-coalescing)
- [`children`/`query_selector_all` pre-reserve `Vec` capacity, rather than collecting through `filter_map`](#childrenquery_selector_all-pre-reserve-vec-capacity-rather-than-collecting-through-filter_map)

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
* `set_matrix` / `set_matrix_precise`
* `set_transform_fmt`

Reusing one buffer across calls means no new allocation happens unless the formatted text outgrows the buffer's capacity.
For shapes that the typed helpers do not cover, your escape hatch is `set_transform_fmt`: it accepts `std::fmt::Arguments` so `format_args!(...)` can build any transform string without the heap allocation that `format!` would otherwise incur.

See [Transforms](transforms.md) for why `set_matrix` takes a `Matrix2D` struct rather than positional arguments, and why `set_matrix_precise` exists alongside it.

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

## `children`/`query_selector_all` pre-reserve `Vec` capacity, rather than collecting through `filter_map`

Both methods (`src/node/tree.rs`) know the DOM collection's length before iterating it, so the original implementation looked like it should collect in one allocation:

```rust
(0..collection.length())
    .filter_map(|i| collection.item(i))
    .filter_map(|el| el.dyn_into::<SvgElement>().ok())
    .map(SvgNode::new)
    .collect()
```

It does not.
`Iterator::size_hint` for `FilterMap` reports a **lower** bound of `0`, because it cannot know in advance how many elements its predicate will discard — confirmed directly (`(0..10).filter_map(...).size_hint()` prints `(0, Some(10))`, and the bound is unchanged after chaining `.map(SvgNode::new)`, since `Map` just forwards its inner iterator's hint).

`Vec`'s `Extend`/`FromIterator` implementation reserves against that lower bound, not the upper one, so a lower bound of `0` means `collect()` cannot make a single exact allocation here — it falls back to the same amortised doubling every `Vec::push` loop uses, reallocating and moving already-collected `SvgNode`s (each an `Rc` pointer, so the copy itself is cheap, but the repeated reallocation is not) as the vector grows past each capacity step.

The fix replaces the iterator chain with `Vec::with_capacity(len)` followed by a manual `for` loop that pushes into it, using the DOM count as an **upper bound**, since filtering can only ever shrink the result, never grow it:

```rust
let len = collection.length();
let mut nodes = Vec::with_capacity(len as usize);
for i in 0..len {
    if let Some(el) = collection.item(i) {
        if let Ok(svg_el) = el.dyn_into::<SvgElement>() {
            nodes.push(SvgNode::new(svg_el));
        }
    }
}
nodes
```

For the ordinary case (an SVG subtree with no `<foreignObject>` content), every DOM entry survives both casts, so this is now exactly one allocation with zero wasted growth-copies — deterministically, not just usually, since the improvement follows directly from `Vec::with_capacity`'s documented behaviour rather than from anything an optimiser might or might not do.
This is a different kind of change from the `write_default`/`write_fixed` experiment described in [path data](path_data.md) ("`dps` is clamped once per `write_d_fixed` call..."): that needed an empirical `wasm-opt`/MD5 comparison because the question was whether LLVM had already erased a source-level difference; this one needs no such benchmark, because `Vec`'s allocation strategy is a documented, guaranteed contract, not an optimisation the compiler is free to skip.

The trade-off is bounded, not open-ended: a selector matching mostly non-SVG elements (the flagged case being a `<foreignObject>` full of HTML) leaves the `Vec` holding unused capacity — at most `(dom_count - svg_count) * size_of::<SvgNode>()` bytes, i.e. one pointer per discarded element, which is freed as soon as the caller drops the `Vec`.
This is a transient memory cost, not a correctness issue or an unbounded one, and for `query_selector_all` in particular it is dwarfed by the browser's own `querySelectorAll` DOM walk that produces the `NodeList` in the first place.

`shrink_to_fit()` was deliberately not added after the loop: trimming the reserved-but-unused capacity would itself cost a second allocation and copy, which would negate the improvement on the common, all-SVG path in exchange for shaving a transient memory cost that already frees itself when the `Vec` is dropped.
