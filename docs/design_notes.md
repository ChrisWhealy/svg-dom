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
Passive noeds can remain small by keeping the buffer external whilst hot paths can opt in explicitly.
In an event handler, the buffer typically lives in an `Rc<RefCell<String>>` shared by the closures, as the drag/touch demo does.

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
For a label whose value changes on every event (E,G. a coordinate or status readout updated each time `pointermove` is handled) we now avoid allocating and discarding a `String` by calling `set_text(&format!(...))`.

When the text instead *repeats* between events, `CachedAttr::set_text` is the better fit since, the DOM write is skipped entirely if the value is unchanged.
The drag/touch demo uses `set_text_fmt` for its live coordinate readout, sharing one scratch buffer with the card's transform; its `last:` event readout has many interleaving writers (including native `drag` events that fire between `pointermove`s), so it is deliberately left uncached — partial caching there would skip a needed write.

## Shared element factory implementation

`SvgRoot` and `SvgBatch` expose the same basic element factories (`rect`, `circle`, `line`, `path`, `text`, and `group`).
Internally, those factories delegate to a shared `SvgFactory` implementation, so shape-specific creation logic and initial attribute writes exist in one place only.

The only difference between the two paths is the append target: `SvgRoot` appends directly to the live `<svg>`, while `SvgBatch` appends to its `DocumentFragment` until `commit()` is called.

# Ideas Considered and Rejected

## Splitting `SvgNode` into passive and interactive types

It was suggested that a benefit could be obtained by splitting `SvgNode` into passive and interactive types.
Only the interactive type would carry the listener storage:

```rust
struct SvgNode {
    element: SvgElement
}

struct InteractiveSvgNode {
    node: SvgNode,
    listeners: RefCell<Vec<EventListener>>
}
```

The motivation was to stop passive geometry nodes from carrying listener state they never use.
This option has been evaluated and **will not** be pursued.

The memory win is tiny because the common case is already optimised.

The listeners field is `RefCell<Option<Box<ListenerStore>>>`, and `store_listener` only allocates on the first `on_*` call.
A passive node therefore allocates **no** listener storage at all; it pays only for the inline field, that is, on `wasm32`, the `RefCell` borrow flag (4 bytes) plus a niche-optimised `Option<Box<…>>` pointer that is `null` when empty (4 bytes), so the saving adds up to only ~8 bytes.

`ListenerStore` is a `One(EventListener)` / `Many(Vec<EventListener>)` enum: the first listener is held inline in the `Box`, so a single-listener node makes one heap allocation rather than the two an empty `Vec` would (the `Box<Vec>` itself plus the element buffer on first push); a second listener upgrades `One` to `Many`. Registration is a setup-time path, so this is a modest leanness win rather than a hot-path one.

Splitting removes those ~8 inline bytes per node and zero heap allocations, which is negligible next to the `Rc` strong/weak counts and allocation header every node carries regardless.

Against that small saving sit real costs:

* **API surface.**<br>
   Callers must choose passive vs interactive up front.

   Every factory (`rect`, `circle`, `line`, `path`, `text`, `group`) lives in the shared `SvgFactory` used by both `SvgRoot` and `SvgBatch`, so either each factory is duplicated, gaining an `.interactive()` upgrade step, or becomes generic - rippling through two factory surfaces.

   To avoid re-declaring every attribute setter, `InteractiveSvgNode` would also need to `Deref` to `SvgNode`, which is `Deref`-as-inheritance.

* **It breaks the single-identity model.**<br>
  `SvgNode` is `Rc<SvgNodeInner>`, therefore all clones share one ownership root and the listener-lifetime contract ("keep at least one handle alive and the listeners stay alive") depends on that.

  Putting `listeners` on the outer `InteractiveSvgNode` places it *outside* the shared `Rc`, so an "upgrade" forks ownership: the interactive handle owns the listeners independently of any passive clone of the same element. Drop the interactive handle while a passive clone is still alive and the listeners die — exactly the footgun the single-type design eliminates. Restoring shared semantics would require a second `Rc` layer.

So the structurally "trivial" upgrade is semantically a fork of the very ownership the library deliberately unifies, in exchange for ~8 bytes per node.
The lightweight-passive-node property is better served by the existing lazy `Option<Box<Vec>>`, and any need to signal interactivity is cheaper to meet with documentation than with a second concrete type.

## A faster float-to-string crate (`ryu` / `itoa`)

It was suggested that numeric formatting could be sped up by routing it through a dedicated crate such as `ryu` (floats) or `itoa` (integers) instead of the standard library's `Display`.
This was evaluated and **will not** be pursued.

Two things undercut it:

* **It does not fit the hot path.**<br>
  The high-frequency formatting in this crate is the transform setters, which use *fixed precision* (`{:.1}`, `{:.3}`). `ryu` emits the **shortest round-trip** representation, which is a different output, so it cannot replace that formatting at all. It would only touch the default `{value}` `Display` path used at element-creation time and in `set_attr_display` — not the per-event work it was meant to accelerate.

* **The win over std is marginal.**<br>
  Rust's standard `f64` `Display` is itself a shortest-round-trip (Ryū-derived) implementation, so the realistic saving is small and confined to creation-time formatting.

Set against an added dependency in a published crate (which grows every downstream user's dependency tree), the trade is not worth it.
The dominant per-call cost on any real hot path is the `set_attribute` boundary crossing, not the float-to-string conversion, so effort is better spent eliding redundant DOM writes (`CachedAttr`) and reusing format buffers (the transform setters and `set_attr_display`).

## `path_fmt` / `text_fmt` factory helpers

It was suggested that the factories accept `std::fmt::Arguments` directly — `path_fmt(format_args!(...))` and `text_fmt(...)`, plus the `SvgBatch` equivalents — so a caller building a computed `d` or label string need not allocate a `String` before the factory sets the attribute (instead of today's `svg.path(&format!(...))`).
The new methods would format into the factory's existing `SvgAttrs` scratch buffer.

This recommendation has been evaluated and **will not** be pursued.

* **It optimises a cold path.**<br>
  Element creation runs at setup time, not per frame or per event.
  Every allocation-light helper in this crate — `AnimationFrame::set_*_fmt`, the transform setters, `SvgAttrs`/`AttrWriter` — exists to remove churn from genuinely *hot* paths; one allocation at creation is not that.
  This is the same distinction noted for the `ryu`/`itoa` idea above.

* **The saved string is dwarfed by what the factory already does.**<br>
  Every factory call already performs a `create_element_ns` (a wasm/JS boundary crossing that allocates a live DOM node) and a DOM append.
  A caller's `format!` for the `d`/text is negligible beside those, so nothing measurable is changed by removing it. 
  The same "the cost of boundary crossing dominates" reasoning as was used to reject the use of the `ryu` crate.

* **The hot case is already covered.**<br>
  A path or label whose `d` or text *changes over time* should be created once and then mutated on the live node with `AnimationFrame::set_d_fmt` / `set_text_fmt` (inside a RAF loop) or `SvgAttrs::fmt` / `SvgNode::set_text_fmt` (in an event handler) — never recreated.
  The crate's model is mutating live nodes rather than rebuilding the tree, so per-frame element re-creation is already a non-goal.

Against those, the cost is four new public methods (`path_fmt` / `text_fmt` on both `SvgRoot` and `SvgBatch`), each carrying documentation and tests under `#![deny(missing_docs)]`, simple to remove a single setup-time allocation.
Callers who format at creation time can simply write `svg.path(&format!(...))`.
If a future profile ever shows element-creation churn dominating (for example frequent full rebuilds), the right response is to mutate existing nodes, not to add creation-time formatting helpers.

## Handle-light factories for large static scenes (`static_rect`, raw `SvgElement`)

It was suggested that for scenes containing thousands of static elements whose handles are discarded immediately, the per-element allocation of an `Rc<SvgNodeInner>` should be avoided.
The factories could skip constructing a managed `SvgNode` by implementing functions such as `static_rect(...)` or `static_path(...)` and return a "naked" `web_sys::SvgElement` instead of a wrapped `SvgNode`.

This recommendation has been evaluated and **will not** be pursued.

* **The `Rc` is dwarfed by the per-element DOM cost.**<br>
  Every factory call already creates a real browser DOM node via `create_element_ns` (thus crossing the wasm/JS boundary) and makes one `set_attribute` crossing per attribute.
  A single `Rc::new` of a two-field struct is noise beside that, and is a one-time setup cost rather than occurring on a hot path.

  This is the same "the cost of boundary crossing dominates" reasoning used to reject `ryu`/`itoa` and the `path_fmt` helpers above.

* **The real cost of bulk creation is already addressed.**<br>
  `SvgBatch` (`build_batch` / `build_batch_into`) appends many elements through a single `DocumentFragment` operation, which targets the DOM-mutation and reflow cost that actually scales with element count.
  A `static_*` variant cannot remove the cost of boundary crossing &mdash; each element and its attributes still have to be created &mdash; so it would only shave off a negligible handle allocation on top of the work `SvgBatch` already minimises.

* **It bifurcates the API for a speculative gain.**<br>
  A `static_*` form of every factory across both `SvgRoot` and `SvgBatch` is a large, permanent public surface (with docs and tests under `#![deny(missing_docs)]`), plus a "which one do I use?" decision forced on every caller.
  The recommendation is itself conditional ("if this crate will be used for thousands of static elements"), and no profile shows the handle as a bottleneck.

* **It re-exposes raw `web-sys`.**<br>
  Returning a bare `web_sys::SvgElement` (or nothing) discards the cheap-to-clone live handle that is the crate's reason to exist, and leaves a caller who later wants to mutate the element with no `SvgNode`.
  The rare need to reach raw `web-sys` is already met, after the fact, by [`SvgNode::as_element`](../src/node/mod.rs).

If a real workload ever proves the handle allocation to be a measurable bottleneck, this can be revisited — but the DOM-node and `set_attribute` costs will still dominate, so the saving would remain marginal.

## An `EventName` enum instead of `&'static str`

It was suggested that `EventListener` store the event name as an enum (`Click`, `PointerMove`, … plus `Raw(&'static str)`) rather than a `&'static str`, on the grounds that an enum would be smaller than a fat string pointer, with `Drop` calling `event_name.as_str()`.

This recommendation has been evaluated and **will not** be pursued on the grounds that the premise is incorrect.

* **The enum is larger, not smaller.**<br>
  The `Raw(&'static str)` variant is mandatory as it is what backs the `on_event` escape hatch for arbitrary event names; so the enum must not only be able to hold a `&'static str`, it must also be able to distinguish it from the ~30 builtin named variants.
  On wasm32 a `&'static str` is 8 bytes (a 4-byte pointer plus a 4-byte length).
  The payload offers a single niche (the pointer is non-null, so one forbidden value), which can absorb only one unit variant for free; with the ~30 built-in event names, the layout falls back to an explicit discriminant, making the enum roughly 12 bytes.
  So the change would *grow* `EventListener` by ~4 bytes per listener, which moves in the opposite direction of the stated intent.

* **The field is already a negligible part of `EventListener`.**<br>
  An `EventListener` also owns the `SvgElement` handle and the wasm-bindgen `Closure` (itself a heap-allocated, boxed `dyn Fn` plus a `JsValue`), which dominate its size.
  Trimming the event-name field (if this is possible) would not acheive a meaningful reduction in total size, and the field is already optimal at `&'static str` (the crate has already deliberately moved off the use of `String`).

* **It adds standing maintenance for a negative payoff.**<br>
  The enum would have to enumerate every supported browser event name and stay in lockstep with the `on_*` helpers, plus carry an `as_str()` mapping, in exchange for making the struct bigger.

The recommendation sat behind the caveat that it is "only worth doing if listener-heavy scenes are expected", but listener-heavy scenes are exactly where the larger per-listener struct cost would be greatest.

## A size-optimised `[profile.release]` baked into the crate

It was suggested that the crate add a wasm-shrinking release profile (`lto = true`, `codegen-units = 1`, `opt-level = "z"`, `panic = "abort"`, `strip = true`) and run `wasm-opt -Oz` as part of packaging, to reduce download and instantiation size for production builds.

This recommendation has been evaluated and **will not** be baked into the crate because a Rust library cannot set this for its consumers.

* **A dependency's `[profile.release]` is ignored.**<br>
  `svg-dom` is a library, so a `[profile.release]` here would govern only builds where `svg-dom` itself is the root — i.e. the demo's own wasm build — and never a downstream application's production build, which is the thing the recommendation wants to shrink.
  Cargo only honours the `[profile.*]` of the crate being built as the root (or the workspace root); the profiles declared by dependencies have no effect.

* **These settings belong to the application, not the library.**<br>
  `opt-level = "z"`, `panic = "abort"`, `lto`, and `strip` are whole-binary trade-offs (size vs speed vs unwinding) that are the application author's call.
  The right home for them is the consumer's own `Cargo.toml`; imposing them from a dependency would be both ineffective (see above) and presumptuous.

* **The demo artifact is the only thing it would actually affect, and that is a local dev tool.**<br>
  The ~200 KiB `pkg/` build is produced locally by `wasm-pack` (and git-ignored via wasm-pack's own `pkg/.gitignore`), served only by `cargo demo`; its download size is not a shipped concern.
  `wasm-pack` already runs `wasm-opt` on release builds, configurable through `[package.metadata.wasm-pack.profile.release]`, so the size lever for the demo already exists in the toolchain.

This recommendation does however contain a useful kernel — *how to minimise wasm size* — but it belongs as guidance for application authors (set the size-optimised profile in **your** app and let `wasm-pack`/`wasm-opt` run), not in any configuration of the library manifest.
