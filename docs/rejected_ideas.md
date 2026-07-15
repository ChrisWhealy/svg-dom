# Ideas Considered and Rejected

Design suggestions that were evaluated for `svg-dom` and deliberately not adopted.
The reasoning is preserved here so the same ideas are not repeatedly re-proposed.

## Contents

1. [Splitting `SvgNode` into passive and interactive types](#1-splitting-svgnode-into-passive-and-interactive-types)
2. [A faster float-to-string crate (`ryu` / `itoa`)](#2-a-faster-float-to-string-crate-ryu--itoa)
3. [`path_fmt` / `text_fmt` factory helpers](#3-path_fmt--text_fmt-factory-helpers)
4. [Handle-light factories for large static scenes (`static_rect`, raw `SvgElement`)](#4-handle-light-factories-for-large-static-scenes-static_rect-raw-svgelement)
5. [An `EventName` enum instead of `&'static str`](#5-an-eventname-enum-instead-of-static-str)
6. [A size-optimised `[profile.release]` baked into the crate](#6-a-size-optimised-profilerelease-baked-into-the-crate)
7. [Provide a rendered-size fallback (`getBoundingClientRect`) when seeding the cached viewport](#7-provide-a-rendered-size-fallback-getboundingclientrect-when-seeding-the-cached-viewport)
8. [Hiding `SvgRoot::root` behind an `as_element()` accessor](#8-hiding-svgrootroot-behind-an-as_element-accessor)
9. [Rewriting `ListenerStore::push` to push into the `Many` vector in place](#9-rewriting-listenerstorepush-to-push-into-the-many-vector-in-place)
10. [Deferred listener drops to make self-removal safe from within a handler](#10-deferred-listener-drops-to-make-self-removal-safe-from-within-a-handler)
11. [Flatten `EventClosure` by simplifying it to `Closure<dyn FnMut(Event)>`](#11-flatten-eventclosure-by-simplifying-it-to-closuredyn-fnmutevent)
12. [Adding `parent_element()` for lighter hot-path parent access](#12-adding-parent_element-for-lighter-hot-path-parent-access)
13. [Hiding raw `web_sys` access behind `raw_element_unchecked()` or a Cargo feature](#13-hiding-raw-web_sys-access-behind-raw_element_unchecked-or-a-cargo-feature)
14. [Restricting or removing `SvgNode::parent()` to prevent split listener state](#14-restricting-or-removing-svgnodeparent-to-prevent-split-listener-state)
15. [Canonicalising on one construction model for `defs`, `marker`, and `batch`](#15-canonicalising-on-one-construction-model-for-defs-marker-and-batch)
16. [Reducing attribute-mutation surface area to one canonical path](#16-reducing-attribute-mutation-surface-area-to-one-canonical-path)
17. [Unifying marker references on handles and making marker IDs immutable](#17-unifying-marker-references-on-handles-and-making-marker-ids-immutable)
18. [Listener removal has documented unsafe lifecycle caveats](#18-listener-removal-has-documented-unsafe-lifecycle-caveats)
19. [Making `AnimationLoop::start_with_frame` the canonical animation API](#19-making-animationloopstart_with_frame-the-canonical-animation-api)
20. [Typed cached-attribute wrappers for scalar values (`CachedF64Attr` / `CachedAttr::set_display`)](#20-typed-cached-attribute-wrappers-for-scalar-values-cachedf64attr--cachedattrset_display)
21. [Reduce error-path formatting machinery to shrink WASM binary size](#21-reduce-error-path-formatting-machinery-to-shrink-wasm-binary-size)
22. [Feature-gate event families and specialised SVG functionality](#22-feature-gate-event-families-and-specialised-svg-functionality)
23. [Optional shared RAF scheduler (`AnimationScheduler`)](#23-optional-shared-raf-scheduler-animationscheduler)
24. [Optional delegated event handling for dense interactive scenes](#24-optional-delegated-event-handling-for-dense-interactive-scenes)
25. [`build_gaussian_blur` / `build_offset` / `build_merge` closures for filter primitives](#25-build_gaussian_blur--build_offset--build_merge-closures-for-filter-primitives)

## 1) Splitting `SvgNode` into passive and interactive types

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

The memory win is tiny because the common case is already optimised.

The listeners field is `RefCell<Option<Box<ListenerStore>>>`, and `store_listener` only allocates on the first `on_*` call.
A passive node therefore allocates **no** listener storage at all; it pays only for the inline field, that is, on `wasm32`, the `RefCell` borrow flag (4 bytes) plus a niche-optimised `Option<Box<...>>` pointer that is `null` when empty (4 bytes), so the saving adds up to only ~8 bytes.

`ListenerStore` is a `One(EventListener)` / `Many(Vec<EventListener>)` enum: the first listener is held inline in the `Box`, so a single-listener node makes one heap allocation rather than the two an empty `Vec` would (the `Box<Vec>` itself plus the element buffer on first push); a second listener upgrades `One` to `Many`.
Registration is a setup-time path, so this is a modest leanness win rather than a hot-path one.

Splitting removes those ~8 inline bytes per node and zero heap allocations, which is negligible next to the `Rc` strong/weak counts and allocation header every node carries regardless.

Against that small saving sit real costs:

* **API surface.**<br>
   Callers must choose passive vs interactive up front.

   Every factory (`rect`, `circle`, `line`, `path`, `text`, `group`) lives in the shared `SvgFactory` used by both `SvgRoot` and `SvgBatch`, so either each factory is duplicated, gaining an `.interactive()` upgrade step, or becomes generic - rippling through two factory surfaces.

   To avoid re-declaring every attribute setter, `InteractiveSvgNode` would also need to `Deref` to `SvgNode`, which is `Deref`-as-inheritance.

* **It breaks the single-identity model.**<br>
  `SvgNode` is `Rc<SvgNodeInner>`, therefore all clones share one ownership root and the listener-lifetime contract ("keep at least one handle alive and the listeners stay alive") depends on that.

  Putting `listeners` on the outer `InteractiveSvgNode` places it *outside* the shared `Rc`, so an "upgrade" forks ownership: the interactive handle owns the listeners independently of any passive clone of the same element.
  Drop the interactive handle while a passive clone is still alive and the listeners die — exactly the footgun the single-type design eliminates.
  Restoring shared semantics would require a second `Rc` layer.

So the structurally "trivial" upgrade is semantically a fork of the very ownership the library deliberately unifies, in exchange for ~8 bytes per node.
The lightweight-passive-node property is better served by the existing lazy `Option<Box<Vec>>`, and any need to signal interactivity is cheaper to meet with documentation than with a second concrete type.

## 2) A faster float-to-string crate (`ryu` / `itoa`)

It was suggested that numeric formatting could be sped up by routing it through a dedicated crate such as `ryu` (floats) or `itoa` (integers) instead of the standard library's `Display`.

Two things undercut it:

* **It does not fit the hot path.**<br>
  The high-frequency formatting in this crate is the transform setters, which use *fixed precision* (`{:.1}`, `{:.3}`).
  `ryu` emits the **shortest round-trip** representation, which is a different output, so it cannot replace that formatting at all.
  It would only touch the default `{value}` `Display` path used at element-creation time and in `set_attr_display` — not the per-event work it was meant to accelerate.

* **The win over std is marginal.**<br>
  Rust's standard `f64` `Display` is itself a shortest-round-trip (Ryū-derived) implementation, so the realistic saving is small and confined to creation-time formatting.

Set against an added dependency in a published crate (which grows every downstream user's dependency tree), the trade is not worth it.
The dominant per-call cost on any real hot path is the `set_attribute` boundary crossing, not the float-to-string conversion, so effort is better spent eliding redundant DOM writes (`CachedAttr`) and reusing format buffers (the transform setters and `set_attr_display`).

## 3) `path_fmt` / `text_fmt` factory helpers

It was suggested that the factories accept `std::fmt::Arguments` directly — `path_fmt(format_args!(...))` and `text_fmt(...)`, plus the `SvgBatch` equivalents — so a caller building a computed `d` or label string need not allocate a `String` before the factory sets the attribute (instead of today's `svg.path(&format!(...))`).
The new methods would format into the factory's existing `SvgAttrs` scratch buffer.

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

## 4) Handle-light factories for large static scenes (`static_rect`, raw `SvgElement`)

It was suggested that for scenes containing thousands of static elements whose handles are discarded immediately, the per-element allocation of an `Rc<SvgNodeInner>` should be avoided.
The factories could skip constructing a managed `SvgNode` by implementing functions such as `static_rect(...)` or `static_path(...)` and return a "naked" `web_sys::SvgElement` instead of a wrapped `SvgNode`.

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

## 5) An `EventName` enum instead of `&'static str`

It was suggested that `EventListener` store the event name as an enum (`Click`, `PointerMove`, ... plus `Raw(&'static str)`) rather than a `&'static str`, on the grounds that an enum would be smaller than a fat string pointer, with `Drop` calling `event_name.as_str()`.

The premise upon which this idea is based is incorrect.

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

## 6) A size-optimised `[profile.release]` baked into the crate

It was suggested that the crate add a wasm-shrinking release profile (`lto = true`, `codegen-units = 1`, `opt-level = "z"`, `panic = "abort"`, `strip = true`) and run `wasm-opt -Oz` as part of packaging, to reduce download and instantiation size for production builds.

This idea cannot be implemented because it does not apply to Rust libraries &mdash; only to applications.

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

## 7) Provide a rendered-size fallback (`getBoundingClientRect`) when seeding the cached viewport

`SvgRoot::attach` reads only the `width` and `height` attributes to seed the cached viewport, so an `<svg>` sized purely with CSS will have cached dimensions of `0 × 0`.
It was suggested that it is necessary to provide a `read_viewport` fall back that returns the rendered measurement such as `getBoundingClientRect()` or the client dimensions when these attributes are absent.
We tightened the documentation instead (`attach` now states that only the two attributes are read, and points CSS-sized callers at `set_viewport`).

* **It would mix two incompatible coordinate spaces and break the write-elision it feeds.**<br>
  The cached viewport is authoritative for `width()`/`height()` *and* for `set_viewport`, which skips redundant DOM writes by comparing the requested size against the cache, then writing `width`/`height` **attributes** (in user units).
  `getBoundingClientRect()` returns rendered **CSS pixels**, which will differ from the attribute units whenever a `viewBox` or CSS scaling is in play.
  Seeding the cache from rendered pixels and then removing attribute writes against it would end up comparing raw values without considering that the units of measure may have become incompatible.
  This would turn a correctness-neutral optimisation into a latent bug.

* **The fallback becomes unreliable exactly when it is needed most.**<br>
  `attach` is frequently called either during module `init`, before first layout/paint, or on a `display:none`/not-yet-attached element — all of which will return a measurement of `0`.
  So the fallback would not even fix the motivating case dependably; it would only paper over some of it, while making the failure mode harder to reason about (sometimes `0`, sometimes a stale pre-layout value).

* **Rendered measurement is already a documented non-goal.**<br>
  `docs/gaps.md` lists `getBoundingClientRect()` among the deliberately out-of-scope DOM-geometry features.
  The crate's contract is that `width()`/`height()` report the *attribute* values read once at attach time; a caller who needs the rendered size can measure it themselves and call `set_viewport`, which keeps the cache coherent with what the crate actually writes.

## 8) Hiding `SvgRoot::root` behind an `as_element()` accessor

`SvgRoot` exposes its `<svg>` as the public field `root`, which lets a caller write `width`/`height` directly and desynchronise the cached viewport that backs `width()`/`height()` and `set_viewport`'s write-elision.

It was suggested that, as a future breaking change, the field should be made private and a `pub fn as_element(&self) -> &SvgsvgElement` accessor be added instead.
We documented the caveat on the field (direct mutation desyncs `width()`/`height()`; `set_viewport` is the cache-aware path) but did **not** make the field private.

* **The accessor would not actually protect the invariant.**<br>
  Every `web_sys` DOM mutator such as `set_attribute`, `set_attribute_ns`, and its typed property setters, take `&self`, not `&mut self` because DOM mutation goes through a shared JS handle.
  So `svg.as_element().set_attribute("width", "500")` desyncs the cache *exactly* as `svg.root.set_attribute("width", "500")` does.
  The change swaps a public field for a method of identical power over the invariant; it does not close the hole the recommendation set out to close.

* **Exposing the element is a deliberate escape hatch.**<br>
  This crate is a thin, minimal wrapper and does not wrap every SVG attribute or property (`viewBox`, `preserveAspectRatio`, CSS classes, focus, and so on — see `docs/gaps.md`).
  Direct access to the root `<svg>` is the supported way to reach those, so the leak is inherent to *exposing the element at all* (which we want to do) not to the field-versus-method spelling.

  The only extra power a public field grants is reassigning `root` wholesale, which needs `&mut SvgRoot` and would obviously corrupt the handle; that is a self-evident misuse, not a footgun worth a breaking API change to forbid.

* **Documentation is therefore the correct and sufficient fix.**<br>
  Since the element must remain reachable and any reference to it can mutate the DOM, the honest contract is a documented one: the field's doc now states that writing `width`/`height` directly desyncs `width()`/`height()` and that `set_viewport` is the cache-aware path.

  Renaming `svg.root` to `svg.as_element()` across every downstream user would churn the public API for no real gain in safety.

## 9) Rewriting `ListenerStore::push` to push into the `Many` vector in place

`ListenerStore::push` replaces `self` with a non-allocating `Many(Vec::new())` placeholder, moves the old value out by value, and matches it exhaustively (`One` → upgrade to a two-element `Many`; `Many` → push and put back).

It was suggested that the `Many` case should instead push into the vector in place (matching `self` by reference, with a separate `mem::replace` + `unreachable!()` only on the `One` → `Many` upgrade path) to avoid moving the whole `Vec` out and back.

This crate makes every attempt to exclude the possibility of generating a WASM binary that conatins an `unreachable!()` call, because if this was statement was ever reached, the WASM runtime would terminate this binary immediately and tear down its memory (effectively self-destructing).

* **The two properties are in tension; the current form is deliberately kept because it is panic-free.**<br>
  To upgrade `One` → `Many` the first listener must be moved out of `&mut self`, which requires `mem::replace`.
  Matching that owned result can either handle *both* variants meaningfully (today's code which is exhaustive and does not panic) *or* pre-narrow `self` by reference and then need `unreachable!()` for the "impossible" arm.
  Rust's move rules do not allow both at once.

* **The saving is nil and the cost is real (if small).**<br>
  `Vec::new()` does not allocate, so the `Many` arm only moves the vector's 24-byte `(ptr, len, cap)` handle out and back — there is no extra heap allocation beyond the `push` itself, and `push` runs at listener-registration time, never on a hot path.
  Against that zero saving, `unreachable!()` adds the possibility of a panic and `#[track_caller]` location data to the **wasm binary** (a size concern this crate takes seriously — see idea 6); the optimiser *may* prove the arm dead and strip it, but that is not guaranteed.

The current code is exhaustive, panic-path-free, allocation-neutral, and documented as such at the call site, so the proposal is a lateral-to-slightly-worse change to working code and was not adopted.

## 10) Deferred listener drops to make self-removal safe from within a handler

`clear_listeners` and `remove_listeners` are safe Rust APIs, but their docs warn that calling them from inside one of the same node's handlers would free the currently-executing wasm-bindgen `Closure` — which is undefined behaviour (UB) in the Rust abstract machine.
This recommendation was to resolve this by making listener removal deferred while a handler is dispatching, using a depth counter and a side-store for closures that are queued for drop:

```rust
struct ListenerState {
    active_dispatch_depth: Cell<u32>,
    deferred_drops: RefCell<Vec<Box<ListenerStore>>>,
}
```

Every managed event wrapper would enter a dispatch guard before invoking the user closure, and leave it afterwards.
`clear_listeners` / `remove_listeners` would still detach the browser-side listener immediately, but if `active_dispatch_depth > 0`, the removed `ListenerStore` would be parked in `deferred_drops` and dropped only after the outermost handler returns.

This idea was rejected fior the following reasons:

* **The UB is practically inert in the wasm32 execution model.**<br>
  wasm-bindgen's closure trampoline calls the `FnMut` through a raw pointer, then returns.
  It does not dereference the pointer again after the call returns.
  WebAssembly is single-threaded, so the freed allocation cannot be concurrently reclaimed and overwritten during the same synchronous call frame.
  No bytes in linear memory are read through the dangling reference after the call exits.
  In the Rust abstract machine this is formally UB, but in the concrete wasm32 execution environment there is no observable memory corruption, no torn state, and no crash — the warning documents a theoretical violation, not a practical hazard.

* **Deferred drops impose a permanent per-node memory cost.**<br>
  Adding `Cell<u32>` plus `RefCell<Vec<Box<ListenerStore>>>` to `SvgNodeInner` costs roughly 30 bytes on every node - depth counter, borrow flag, and `Vec` metadata - even though the self-removal pattern is vanishingly rare.
  The whole-`Vec` cost only applies when the deferred path actually fires, but the field overhead is paid by every node in every scene regardless.

* **Tracking dispatch depth adds overhead on the hot path.**<br>
  To know whether a removal should be deferred or immediate, every closure invocation would need to increment and decrement the counter.
  Today `on_event` stores the raw user closure directly; tracking depth would require wrapping each user closure in an outer bookkeeping closure which adds an extra heap allocation per listener registration and one extra increment / decrement / branch on every event fired.
  This crate deliberately avoids per-invocation overhead; the transform setters, `CachedAttr`, and the scratch-buffer APIs all exist specifically to reduce work done per event.
  This "fix" costs more than the problem.

* **The motivating use case (a one-shot handler that removes itself) already has a correct, zero-cost solution.**<br>
  wasm-bindgen provides `Closure::once`, which wraps a `FnOnce` and frees itself after the first invocation.
  Combined with the browser-native `addEventListener` option `{ once: true }`, the closure is called exactly once and the memory is reclaimed by wasm-bindgen's own cleanup path after the call returns - never while the closure is still running.

  `on_event_once` was implemented using a `FnOnce` wrapped in a `FnMut` via `Option::take`, combined with `{ once: true }`, so the browser removes the DOM listener and the user handler capture is freed after the first matching dispatch.
  A small listener shell (the `Closure<dyn FnMut(Event)>` wrapper) remains in the node's listener store until `clear_listeners`, `remove_listeners`, or node drop — giving the same ownership guarantees as every other managed listener without requiring a separate store design.

* **The remaining sub-cases are already safe and need no guards.**<br>
  Removing a *different* event type from the same node is safe: the running closure is not the one being freed.
  Removing listeners on a *different* node is always safe.
  Only the precise sub-case — a handler drops itself — is the footgun the docs warn against, and `on_event_once` is the right API for that pattern.

The deferred-drops design would add memory and CPU overhead to every node and every event in the entire crate in order to guard against a theoretical abstract-machine violation that has no observable consequence in the wasm32 execution model, when the primary motivating use case is already correctly served by `Closure::once`.

The existing documentation warning is the correct and sufficient response; a dedicated `on_event_once` helper is the right follow-on addition if the one-shot pattern proves common in practice.

## 11) Flatten `EventClosure` by simplifying it to `Closure<dyn FnMut(Event)>`

`EventClosure` is an enum with one variant per supported event type (`Drag`, `Focus`, `Keyboard`, `Mouse`, `Pointer`, `Touch`, `Wheel`, and `Event`), each of which wraps the corresponding `Closure<dyn FnMut(T)>`.

The `callback_ref` method has an 8-arm match with identical bodies (`closure.as_ref().unchecked_ref()`), and every `on_*` helper constructs the matching variant.
The suggestion was to replace all of this with a single `Closure<dyn FnMut(Event)>`, moving the typed event conversion into a small wrapper at each registration site:

```rust
pub fn on_click<F: FnMut(MouseEvent) + 'static>(&self, mut handler: F) -> Result<(), Error> {
    self.store_listener(
        "click",
        Closure::new(move |e: Event| handler(e.unchecked_into::<MouseEvent>())),
    )
}
```

The 35-line `EventClosure` enum and the `callback_ref` match would disappear, and `EventListener` would shrink by the discriminant — roughly 4 bytes per listener on wasm32.

This idea is rejected for the following reasons:

* **It adds an extra `vtable` dispatch on every event fire.**<br>
  Currently the browser → JS trampoline → user-handler path has a single `FnMut::call_mut` `vtable` dispatch into the user's closure.
  The proposed wrapper inserts a second `call_mut` dispatch into the wrapper before reaching the user closure.
  `unchecked_into()` is itself a no-op cast, but the extra `dyn` call is real.

  The cost of calling event handlers is already dominated by the JS/WASM boundary crossing, so this extra dispatch does not dominate either.  However, adding overhead to the dispatch path conflicts with the crate's explicit design philosophy of keeping hot-path costs minimal.

* **The saving is marginal and the code change is lateral, not a reduction.**<br>
  The 35 lines removed from `event.rs` are simple, correct, and rarely touched.
  In exchange, every `on_*` helper (and there are roughly 25–30 of them) gains a wrapper closure expression, and `store_listener` changes its parameter type.
  The net difference is roughly neutral in lines and the resulting registration sites are slightly harder to read because the typed relationship between method name, event type, and closure is no longer encoded at the call-to-store boundary.

* **The WASM binary impact is speculative.**<br>
  The current approach emits 8 separate `wasm-bindgen` trampolines (one per event-type closure type) but lets the compiler call the user closure directly in each.

  The proposed approach emits a single trampoline class but requires one concrete wrapper-closure monomorphization per event type, since `F` is generic.

  The net binary size impact is unknown without measuring; the recommendation itself says "I would only keep it if the resulting wasm size ... looks good", meaning the outcome is uncertain.

  This crate does not accept speculative changes that may worsen size without a measurement gate.

* **The per-listener struct size saving is negligible.**<br>
  On wasm32 a `Closure<dyn FnMut(T)>` is the same size regardless of `T`: only the `vtable` pointer differs and that is stored in the heap-allocated `Box<dyn FnMut>`, not in the `Closure` struct itself.

  The saved discriminant is ~4 bytes per `EventListener`; with `ListenerStore::One` that is a single listener, and `ListenerStore::Many` allocates a `Vec` whose elements are individually ~4 bytes smaller.

  The saving is real but not significant enough to justify the change.

The current `EventClosure` enum is boilerplate, but nonetheless it is working, auditable boilerplate that encodes a clear structural invariant.
Any new `on_*` helper must explicitly state which variant it wraps without touching the dispatch path.
The right response to the boilerplate concern is documentation, not type erasure.

## 12) Adding `parent_element()` for lighter hot-path parent access

`SvgNode::parent()` creates a fresh managed `SvgNode` around the parent element, with its own independent listener store.
It was suggested that a lighter sibling method (`pub fn parent_element(&self) -> Option<SvgElement>`) be added for hot-path callers (e.g. inside `pointermove`, `wheel`, or RAF callbacks) that only need to inspect or compare the parent without constructing a new managed node.

The suggestion explicitly frames this as a future addition ("if parent traversal becomes common in hot paths") and recommends not replacing the current `parent()` behaviour.
For those reasons, and the following, it is not adopted now.

* **No current consumer demonstrates the to be a realistic need.**<br>
  The crate targets simple SVG diagrams; hot-path parent traversal has not (so far) appeared in any demo or real use case.
  Extending the API surface for a hypothetical future requirement conflicts with the crate's principle of not designing for speculative requirements.

* **`as_element()` already provides the raw-DOM escape hatch.**<br>
  A caller who needs the parent element for inspection can call `node.parent().map(|n| n.as_element().clone())`.
  It is verbose but accurate for the rare case, and avoids widening the public API for a path that has no demonstrated users.

* **Returning `SvgElement` directly would break the crate's abstraction layer.**<br>
  Every other navigation method (`parent`, `downgrade` / `upgrade`) returns a crate-managed type.
  Introducing `Option<SvgElement>` as a return type on a navigation method would be the first raw `web-sys` type surfaced from a traversal API, setting an inconsistent precedent.

If a future profile shows parent-traversal allocation as a measured bottleneck in a real hot path, the method can be added then with evidence to justify the API surface cost.

## 13) Hiding raw `web_sys` access behind `raw_element_unchecked()` or a Cargo feature

An external review flagged three sites where the crate exposes raw `web_sys` elements:

* `SvgRoot::root` — a public field of type `SvgsvgElement`.
* `SvgNode::as_element()` — returns `&SvgElement`.
* `SvgDefs::as_element()` and `SvgMarker::as_element()` — likewise.

The argument was that callers can bypass cached state, marker-id validation, and listener ownership through these handles, and that raw DOM access should instead live behind an explicit escape API — `raw_element_unchecked()` — or a Cargo feature such as `features = ["raw-dom-access"]`.

The recommendation was evaluated against each site individually, because the facts differ.

### `SvgRoot::root`

This was already analysed in **idea 8**, where it was also suggested that the public field be hidden behind an `as_element()` accessor.
The conclusion there stands for any spelling of the accessor: every `web_sys` DOM-mutating method takes `&self`, not `&mut self`, so `svg.raw_element_unchecked().set_attribute("width", "500")` desyncs the cached viewport *exactly* as `svg.root.set_attribute("width", "500")` does.
The accessor name changes the ergonomics but does not close the invariant hole, because the hole is inherent to exposing the element at all — which we want to do.

### `SvgNode::as_element()`

`SvgNode` has **no cached state** that direct DOM access can desync.
Its managed state is the listener `Rc<SvgNodeInner>`, and a caller who adds a listener directly to the raw `SvgElement` does not corrupt that store — they merely bypass the crate's automatic listener cleanup for that one listener.
This is the unavoidable consequence of exposing any DOM handle, not a unique defect of `as_element()`.

More importantly, `SvgNode::set_attr()` is already a full escape hatch: it accepts an arbitrary attribute name and value and writes it verbatim to the DOM.
Any "bypassing" of attribute-writer consistency or text-safety helpers is equally possible through that route — the existing security note on `set_attr` already documents this.
Removing `as_element()` would only deny access to non-attribute DOM methods (`get_bounding_client_rect`, `tag_name`, typed property accessors that need a `dyn_ref` cast, and so on) — which are the *legitimate* reasons to reach the raw element.

### `SvgDefs::as_element()`

`SvgDefs` has **no cached state**.
Its element is only ever used internally as an append target, and there is nothing for a caller to desync by writing attributes directly.
The accessor is an unrestricted escape hatch; treating it as a backdoor would make it indistinguishable from a justified feature.

### `SvgMarker::as_element()`

This is the one site where the concern has real substance: `SvgMarker` caches the `id` in a Rust `String`, and writing `id` through the raw element bypasses both the cache update and the `validate_marker_id` check.

Two layers already guard against the most common path:

* `SvgMarker::set_attr()` and `set_attr_display()` both reject `"id"` (case-insensitively) with `Error::ReservedAttribute`.
* The doc comment on `as_element()` explicitly warns that the `id` attribute must not be written through this handle, and points to `set_id`.

The only unsuppressed path is `marker.as_element().set_attribute("id", ...)`, which is a deliberate choice by a caller who has read the docs.
A rename to `as_element_unchecked()` would be marginally more self-documenting, but it would be a breaking API change to all existing callers in exchange for a signal the existing doc comment already delivers.
It would also create a naming inconsistency between `SvgMarker` and the other types, which expose `as_element()` with no cached state to protect.

### The Cargo feature approach

Gating `as_element()` behind `features = ["raw-dom-access"]` would impose a feature dependency on every caller who needs `get_bounding_client_rect`, `computed_text_length`-style casts, or any other non-attribute DOM method that the crate does not wrap — exactly the legitimate use cases the method exists for.
The `docs/gaps.md` list makes clear that this crate deliberately does not wrap large swaths of SVG DOM; a feature gate would make those gaps less accessible, not more documented.

### Conclusion

The naming concern is legitimate: `as_element()` does not signal that it bypasses crate invariants.
But renaming alone does not protect invariants, and the invariants that actually exist are either already documented (`SvgRoot` viewport, `SvgMarker` id) or do not exist at all (`SvgNode`, `SvgDefs`).

The correct and sufficient response is precise documentation at each escape hatch, which is already in place.
A breaking rename or a Cargo feature would impose a real cost on legitimate callers in exchange for a naming signal that cannot substitute for reading the docs.

## 14) Restricting or removing `SvgNode::parent()` to prevent split listener state

`SvgNode::parent()` wraps the raw parent DOM element in a brand-new `SvgNode` with its own independent `Rc<SvgNodeInner>` and an empty listener store.
The external review flagged this as a "serious consistency issue": the same SVG element now has two unrelated managed handles, and listeners registered through the `parent()` handle are neither visible to nor cleaned up by the factory handle for the same element.
Four alternatives were proposed: remove `parent()`, return a read-only `SvgElementRef<'a>`, return a raw `SvgElement`, or maintain a canonical registry so the same DOM element always maps to the same `Rc<SvgNodeInner>`.

None of these changes will be adopted.

### The reviewer's premise is incorrect

The crate's invariant is **per-lineage**, not per-DOM-element.

A *lineage* is an `SvgNode` and all its `clone()`s — they all share the same `Rc<SvgNodeInner>`, the same listener store, and the same lifetime contract ("keep one handle alive, listeners stay alive").
The crate never claims that every `SvgNode` wrapping a given DOM element belongs to the same lineage.

The doc comments for `parent()` makes this explicit: it states that listener tracking is "per handle lineage (a handle together with its clones) and **not** per DOM element".  Furthermore, the returned handle's listener store is independent of any factory handle for the same element, and that the handle should be treated as read-only navigation.
The same caution is extended to discourage registering listeners through a `parent()` handle.
This is not a consistency violation — it is the documented, intended behavior of the per-lineage model.

### Why the proposed alternatives do not improve the situation

**Option A — Remove `parent()`.**<br>
This removes the legitimate use cases that motivated the method in the first place: walking up the tree to inspect or modify an ancestor attribute from inside an event callback, without having to retain every ancestor handle at construction time.
A caller who needs to navigate upward and cannot or does not want to hold the ancestor handle would be left with `node.as_element().parent_node()` (a raw DOM call) or `SvgNode::as_element()` plus `dyn_ref` casting, both of which are worse than the current API.

**Option B — `SvgElementRef<'a>`, a lifetime-bound read-only wrapper.**<br>
The wrapper would need to re-expose a meaningful subset of `SvgNode`'s API (attribute reads, `tag_name`, bounding-box queries) without any of the mutation or listener methods.
That is a large parallel surface, with viral lifetime annotations that flow into any closure or data structure that holds the ref.
The underlying DOM element is still reachable (since there is no way to make the underlying `SvgElement` truly read-only) so the wrapper only provides a "soft" guarantee backed by `unsafe` at the boundary.

The doc comments already discourage this pattern; therefore, there is no benefit in adding a second public type that introduces a parallel API surface and allows for viral (i.e. unpredictable) lifetimes.

**Option C — Return `SvgElement` directly.**<br>
Evaluated and rejected in idea 12 for the same reasons: it is the first raw `web-sys` type surfaced from a traversal API, it denies access to the typed attribute setters and transform helpers that make `SvgNode` useful, and it sets an inconsistent precedent relative to every other navigation method (`downgrade` / `upgrade`, which return crate-managed types).

**Option D — A canonical registry mapping DOM elements to `Rc<SvgNodeInner>`.**<br>
This would require a `HashMap<JsValue, Weak<SvgNodeInner>>` (or similar), a strategy for removing stale entries when handles are dropped, and logic to detect when a factory creates an element that was already registered.

Such a change would require a fundamental change to the ownership model (every factory call would need to consult the registry) and adds allocations and hash lookups to what is currently an allocation-free construction.
The problem it aims to solve (i.e. ensuring that `parent()` returns a handle in the same lineage as the factory handle) is not a problem that callers actually need solved, because the correct use of `parent()` (read-only navigation and attribute mutation) does not require shared listener state.

### Conclusion

The per-lineage model is the correct invariant.
`parent()` returns an independent handle that forms its own lineage; that is documented behavior, not a bug.
The doc comment is the appropriate and sufficient fix.
It was tightened to explicitly state "Do not register listeners through a handle obtained from `parent()`" alongside the existing explanation of why listener state is not shared.

## 15) Canonicalising on one construction model for `defs`, `marker`, and `batch`

The external review observed that the crate exposes two construction styles for the same conceptual objects:

```rust
// direct — appends to the live DOM immediately
let defs = svg.defs()?;
let marker = defs.marker("arrow")?;
marker.set_ref_x(10.0)?;

// closure — appends only after the closure succeeds
let defs = svg.build_defs(|defs| {
    defs.build_marker("arrow", |m| { m.set_ref_x(10.0) })
})?;
```

The claim was that these have different failure semantics — direct appends happen before all child content is set, so a mid-build error can leave partial DOM behind — and that having two public models for the same operation is a consistency problem.
The recommendation was to pick the closure builders as canonical and rename or remove the direct APIs.

This is not adopted.

### The batch pair does not have inconsistent atomicity

`batch()` and `build_batch()` are both fully transactional: both create a detached `DocumentFragment` and commit it to the live DOM only when `commit()` is called.
`build_batch()` is a thin convenience wrapper that calls `batch()` then `commit()`.
There is no atomicity difference between them; the reviewer incorrectly included the batch API in the diagnosis.

### The `defs`/`marker` pairs serve genuinely different use cases

`defs()` and `marker()` exist specifically for cases where `<defs>` content cannot be known upfront:
for example, adding markers dynamically in response to user actions, or building a marker incrementally across several calls.
`build_defs()` and `build_marker()` exist for one-shot construction where all children are known before any DOM mutation.

These are complementary, not competing.
Removing the direct APIs would leave no public way to extend a `<defs>` section after initial construction, which is a legitimate use case the docs already describe ("For dynamically extending `<defs>` after initial construction, use `defs` instead").

### The atomicity difference is real but the practical risk is near-zero

With the direct APIs, a failure partway through construction leaves a partial element in the live DOM.
For `defs()`: if a subsequent `marker()` call fails, an empty `<defs>` remains in the SVG.
For `marker()`: if a shape or attribute setter fails after `marker()` returns, a partial `<marker>` remains in `<defs>`.

However, the realistic failure points are:

- `validate_marker_id` — fires before any DOM mutation, so an invalid id leaves nothing in the DOM.
- `create_svg_element` — creates a detached element; failure leaves nothing attached.
- Attribute setters (`set_ref_x`, `set_marker_width`, ...) — call `set_attribute` on standard SVG
  attribute names, which browsers essentially never reject.

The only realistic path to a "partial DOM" is a browser DOM error on a standard SVG attribute setter,
which is theoretically possible but does not occur in practice.
Furthermore, an incomplete `<marker>` in `<defs>` is completely harmless — it renders nothing unless a
`marker-end` or similar attribute actively references its id.

### The naming proposal does not justify a breaking change

The proposed rename to `open_defs()` / `append_marker()` communicates "live mutation" better than
`defs()` / `marker()`, but not enough better to justify breaking every existing caller.
The existing names already convey the distinction from their `build_*` counterparts.

### Documentation is the correct response

The direct API doc comments were tightened to explicitly state the partial-DOM-on-failure risk and to
recommend `build_defs`/`build_marker` for one-shot use.
The `build_*` doc comments already stated that the closure path defers the append.
Together these give callers a clear picture of the trade-off without changing any public API.

## 16) Reducing attribute-mutation surface area to one canonical path

An external review observed that the crate exposes many ways to set the same attribute — using
`set_fill`, `set_attr`, `SvgAttrs::set`, `AttrWriter::set`, `AnimationFrame::set_attr`, or
raw `as_element().set_attribute` — and that some internal implementations were inconsistent:
`SvgMarker::set_orient` and `set_units` call `element.set_attribute` directly; marker-reference
setters use `format!("url(#{id})")` to build the URL; `SvgNode::set_stroke_width` creates a
`SvgAttrs::new()` locally rather than using a shared buffer.

The recommendation was to collapse these into four strict layers: one internal primitive, one
ordinary public mutation path, explicitly named performance helpers, and no raw DOM in the normal API.

This will not be adopted for the following reasons.

### The "many paths" observation conflates composition with competition

For *string-valued* attributes, the paths are not alternatives; they are layered compositions:

```rust
node.set_fill("red")                         // typed helper → calls set_attr
node.set_attr("fill","red")                  // generic helper → calls element.set_attribute
attrs.set(&node,"fill","red")                // SvgAttrs::set → calls node.set_attr
node.attrs(&mut a).set("fill","red").apply() // AttrWriter → calls SvgAttrs::set → node.set_attr
```

Each level wraps the one below.
`SvgAttrs::set` is a one-line pass-through to `node.set_attr`; using it for a string attribute adds no behaviour: it exists so the `AttrWriter` chain can mix string and numeric attributes uniformly.

For *numeric* attributes, `SvgAttrs::display` / `AttrWriter::display` / `AnimationFrame::set_attr_fmt` genuinely add a distinct behaviour: they format the value into a reusable scratch `String` and avoid the per-call allocation that a naïve `value.to_string()` would make.
These are not competing paths; they are a deliberate performance tier.

Raw `as_element()` access is already documented as an escape hatch (rejected idea 13) and is excluded from the "normal" operating model.

### The internal inconsistencies have explanations

**`SvgMarker::set_orient` and `set_units` call `element.set_attribute` directly.**

`SvgAttrs` (and the `display_element` / `display` methods on it) exists for *numeric formatting*: it reuses a scratch `String` to convert `f64`/`Display` values without per-call allocation.
`set_orient` accepts a `&str`; `set_units` accepts a `MarkerUnits` enum that produces a `&'static str`.
Neither needs scratch-buffer formatting.
Routing them through `self.attrs.borrow_mut()` would add a `RefCell` borrow and an indirection for no benefit; calling `element.set_attribute` directly is the correct primitive for string-valued writes.

**Marker-reference setters use `format!("url(#{marker_id})")`.**

`SvgNode` does not own a shared scratch buffer (its `SvgNodeInner` holds only `element` and `listeners`).
Adding a shared `RefCell<SvgAttrs>` to every node just to avoid this one `format!` at setup time would add ~40 bytes to every node in every scene.
A single `format!` at marker-reference setup time is the right tradeoff since `set_marker_end` is not in the hot-path.

**`SvgNode::set_stroke_width` creates `SvgAttrs::new()` locally.**

`SvgMarker` and `SvgDefs` hold a shared `RefCell<SvgAttrs>` because they are factory types that may need to format many numeric attributes; `SvgNode` does not.
The local `SvgAttrs::new()` in `set_stroke_width` is the convenience path; the doc comment already directs hot-path callers to `set_attr_display` with a caller-supplied buffer.
The inconsistency is deliberate and documented.

### The "four layers" recommendation already describes the crate's design

| | Reviewer's label | Existing equivalent |
|--|--|---|
| Layer 1 | One internal primitive | `element.set_attribute` via `web_sys` |
| Layer 2 | Ordinary public mutation | `node.set_attr` + typed helpers (`set_fill`, `set_stroke`, ...) |
| Layer 3 | Performance helpers | `SvgAttrs`, `CachedAttr`, `AnimationFrame` |
| Layer 4 | Remove raw DOM from normal API | See rejected idea 13 |

The recommendation names an architecture that already exists; it does not propose a change.


## 17) Unifying marker references on handles and making marker IDs immutable

The external review raised two concerns:

1. The fact that there are two marker-reference styles (`line.set_marker_end("arrow")?`: a string id, and `line.set_marker_end_ref(&marker)?`: a handle) reintroduces string-typed ids alongside the safer handle form.

2. `SvgMarker::set_id` allows renaming a marker after it has been referenced, leaving all previously written `url(#...)` attributes stale.
   The recommendation was to make ids immutable after construction and to make the handle form the only reference path.

### String-based references cannot be removed

The string form (`set_marker_start/mid/end`) exists for markers that are not created through this crate: a pre-existing `<marker>` defined in inline HTML, a CSS `<defs>` block written by hand, or a third-party library.
In those cases the caller has only the marker's id — there is no `SvgMarker` handle to supply.
Removing the string form would make those markers unreferenceable through the crate's API, forcing callers back to raw `element.set_attribute`.

### Staleness is inherent to the SVG reference model, not a Rust API problem

Both the string and handle forms ultimately write a DOM attribute string: `marker-end="url(#arrow)"`.
SVG marker references are not live pointers; they are strings that the browser resolves by id when it paints the element.
Even if the string form were removed and the handle form were the only path, the written attribute would still be a plain string that becomes stale if the marker is later renamed.
Tracking live references would require the crate to maintain a list of every element that references each marker and update all of them on rename — essentially a live reference-update system akin to the canonical registry rejected in idea 14.
That is far outside the scope of a minimal wrapper crate.

### `set_id` with `&mut self` already provides the strongest practical protection

`set_id` takes `&mut self`, meaning Rust's borrow checker prevents any code from holding a shared `&SvgMarker` reference while a rename is in progress.
The remaining staleness risk is in the DOM, not in Rust: a caller can call `set_id` after the marker has already been referenced, and the previously written DOM attributes will not be updated.
This is fully documented on both `set_id` and the `_ref` methods ("reapply the reference after renaming if needed").

### Removing `set_id` would limit legitimate use cases without closing the hole

A marker whose id must be computed at runtime (a generated unique name, a user-provided label, an id derived from data) needs to be renamed after initial construction.
Removing `set_id` would force callers to delete and recreate the marker and reapply all references (a more error-prone and DOM-intensive path) rather than a single guarded rename.
Because staleness is inherent to DOM string references, making ids immutable does not actually eliminate the risk; it only hides it.

### The genuine inconsistency: `set_marker_start` and `set_marker_mid` lacked the "prefer `_ref`" recommendation

`set_marker_end` already carried the note "Prefer `set_marker_end_ref` when you have the `SvgMarker` handle available." `set_marker_start` and `set_marker_mid` did not.
This asymmetry was a real inconsistency.
The doc comments on `set_marker_start` and `set_marker_mid` have been updated to match `set_marker_end`, so all three string-id setters now consistently recommend the `_ref` form when a handle is available.

## 18) Listener removal has documented unsafe lifecycle caveats

A follow-up to idea 10 was to replace the deferred-drop guard with a listener-handle model:

```rust
let handle = node.on_click(...)?;
handle.remove();
```

The claim was that this model "marks the listener as removed, detaches it from DOM, and defers dropping the closure until after the current dispatch completes", making self-removal safe without the dispatch-depth counter.

The claim is wrong on its central premise: **a handle does not eliminate the need for dispatch tracking.**
To defer a closure drop until after dispatch, you still need to know when dispatch is active.
The only mechanism that provides this is the dispatch-depth counter; which is the same infrastructure rejected in idea 10 for adding overhead to every event on every node.
The handle is an alternative API surface for triggering deferred drops, not an alternative to the mechanism.

### The handle ownership dilemma

`on_click` (and every other `on_*` helper) currently returns `Result<(), Error>`.
Changing it to `Result<ListenerHandle, Error>` forces a choice between two semantics:

**Handle drop removes the listener.**<br>
A caller who writes `node.on_click(|_| {})?;` would immediately drop the handle and silently remove the listener.
Every listener registration would require the caller to explicitly store a handle, or risk losing the listener instantly.
This is a significant ergonomic regression for the common case where a listener is intended to last as long as the node.

**Handle drop is a no-op; listener lives independently.**<br>
The listener is still owned by the node, exactly as now.
The handle is an alternate API for explicit removal, alongside `clear_listeners` and `remove_listeners`.
This adds no safety property: calling `handle.remove()` from within the executing handler has the same abstract-machine footgun as `clear_listeners()`, and without the deferred-drop guard (which requires the dispatch-depth counter), the closure is still freed while execution is in progress.

Neither option solves the stated problem.

### The underlying analysis from idea 10 still applies

- The abstract-machine UB is practically inert in the wasm32 execution model: the trampoline does not dereference the freed pointer again after the user closure returns, and WebAssembly is single-threaded, so no concurrent reuse can occur.
- The motivating use case (a handler that removes itself) is already correctly served by `on_event_once`, which uses `{ once: true }` and `Option::take` so the browser removes the DOM listener and the user handler's captures are freed after the first dispatch, never while executing.
- Removing listeners on a *different* node, or for a *different* event type on the same node, is already safe.

### What handles would genuinely add

A handle that carries the identity of a specific listener would enable removal of one listener out of several registered for the same event type: something `remove_listeners("click")` cannot do as it removes all listeners for the specified event type e.g. `click`.
This is a new, incremental feature request, not a safety fix.

If specific-listener removal proves necessary in practice, it can be evaluated on its own merits, with an API designed for that use case rather than framed as a correction to the existing safety model.

## 19) Making `AnimationLoop::start_with_frame` the canonical animation API

The external review observed that `AnimationLoop` exposes two starting styles:

```rust
// Plain callback — timestamp only.
AnimationLoop::start(|ts| { /* ... */ })?;

// Frame callback — timestamp plus a reusable scratch buffer.
AnimationLoop::start_with_frame(|ts, frame| { /* ... */ })?;
```

The recommendation was that `start_with_frame` should be made canonical: which implies that `start` should either be removed or deprecated because it is *"more consistent with the crate's current performance direction"*.

This idea will not be adopted.

### The `start` and `start_with_frame` function are layered, not competing

`start_with_frame` is implemented as a thin wrapper around `start_inner`, the same private entry point used by `start`:

```rust
pub fn start_with_frame<F: FnMut(f64, &mut AnimationFrame) + 'static>(mut callback: F) -> Result<Self, Error> {
    let mut frame = AnimationFrame::new();
    Self::start_inner(move |ts| callback(ts, &mut frame))
}
```

`AnimationFrame` is an adapter that adds one allocation (the scratch `String`, made once) and no other overhead.
The two constructors are not alternatives at the same abstraction level; one wraps the other.

### `start` is the right API for a large class of callbacks

Not every animation callback needs to format numeric attributes.
A callback that reads a game state, calls `set_transform()` on a pre-computed matrix string, or invokes a typed setter (`set_cx`, `set_cy`) has no use for `AnimationFrame`.
Forcing the `AnimationFrame` parameter onto every caller imposes an unused parameter and the mental overhead of an API whose purpose is irrelevant to the callback ("why is there a frame here?").

### The relationship exactly mirrors the `defs` / `batch` API pairs

`defs()` / `build_defs()`, `marker()` / `build_marker()`, `batch()` / `build_batch()`, and `start()` / `start_with_frame()` all follow the same pattern: a simpler form for the common case and a richer form that adds behaviour such as atomicity and a scratch buffer for callers that need it.
Idea 15 has already been rejected for the same reasons, that the `defs`/`batch` pairs should be collapsed into a single canonical form.

### The doc already guides callers to the right form

The `start` doc comment includes the remark "For a crate-managed buffer, see `start_with_frame`" and shows, in its example, how to manage a manually-owned buffer when the crate's buffer is not wanted.
Callers are directed to the performance path without it being forced on them.

Removing `start` would be a breaking change that delivers no benefit for the majority of animation callbacks, while adding a mandatory, unused parameter for every caller that does not need to format attributes.

## 20) Typed cached-attribute wrappers for scalar values (`CachedF64Attr` / `CachedAttr::set_display`)

An external review suggested two related additions to reduce the verbosity of caching small numeric or scalar attribute states:

```rust
// Option A — a new public type that bakes in the attribute name and precision
let mut opacity = CachedF64Attr::new("opacity", 3);
opacity.set(&node, alpha)?;

// Option B — a new method on the existing CachedAttr
cached.set_display(&node, "opacity", alpha, &mut scratch)?;
```

The reviewer correctly hedged Option B by stating "The latter may already be close to `CachedAttr::set_fmt`; if so, I would not add another API merely for convenience" &mdash; and that hedge applies.

Neither addition was adopted.

### `CachedAttr::set_fmt` already covers every stated use case

`CachedAttr` already exposes four methods: `set` (string value), `set_text` (text content), `set_fmt` (formatted attribute), and `set_text_fmt` (formatted text content).
`set_fmt` takes a caller-owned scratch buffer and `fmt::Arguments`, formats into the buffer and delegates to `set`, which skips the DOM write when the formatted string matches the value already present in the cache.

Every use case the review named is already handled today:

```rust
// snapped coordinate
cache.set_fmt(&node, "x",        &mut scratch, format_args!("{:.1}", snapped_x))?;

// zoom percentage  
cache.set_fmt(&node, "font-size",&mut scratch, format_args!("{:.0}%", zoom))?;

// rounded opacity
cache.set_fmt(&node, "opacity",  &mut scratch, format_args!("{:.2}", alpha))?;

// frame counter (text content)
cache.set_text_fmt(&mut scratch, &node,         format_args!("frame {frame_n}"))?;
```

The no-allocation no-DOM-write path is already present: the unchanged case is a plain `&str` comparison against the cache's `String`, with no JS crossing.

### Against `CachedF64Attr`

A new public type that bakes in an attribute name and a precision would require a `String` scratch buffer as a struct field and would narrow the formatting to a fixed-precision float &mdash; `CachedF64Attr::new("opacity", 3)` implies `"{:.3}"`.
That is a subset of what `set_fmt` already provides.
The only benefit is moving the attribute name and precision from the call site to the constructor; the caller must still provide a value per call.

Against that minor ergonomic shift sits a new `pub struct`, its documentation and its `impl` block.
Further, every future caller must still answer the question "which one do I use?": which amounts to a permanent surface cost for a temporary convenience gain.

### Against `CachedAttr::set_display`

`set_display<T: Display>` taking a generic value rather than `fmt::Arguments` would save writing `format_args!("{}", n)` at the call site, replacing it with just `n`.
This is a real but extremely narrow ergonomic win.

More importantly, it does not help the specific cases the review lists.
Floating-point values displayed as `"{:.2}"` or `"{:.0}%"` require a format string beyond `{}`, so `set_display` would not apply and callers would still reach for `set_fmt`.
The only realistic beneficiaries are integer-valued states (frame counters, selection indices) whose `{}` output is exactly what is needed — but for those, `format_args!("{}", n)` is the entire overhead being eliminated and adding a new method to remove it is not worth the extra API surface.

Adding `set_display` alongside `set_fmt` would also create a two-method decision: callers who see both would need to reason about when each applies.
The cognitive load added by saying *"use `set_display` only when `{}` is the right format and you don't need a format string"* outweighs any call-site saving.

### The Profiling Caveat

The review correctly notes that cached writes are worthwhile only when values repeat after quantisation and that for continuously changing values that never cause a cache hit, the comparison cost is pure overhead.

This consideration is already documented in `cached.rs`: the module-level notes contrast it with `SvgNode::set_attr_if_changed`, which pays the cost of a JS round-trip for the comparison, whereas `CachedAttr` keeps the last value on the Rust side.

The guidance that a `CachedAttr` should be dedicated to a single *frequently-touched-but-rarely-changing* attribute (a cursor style, a discrete state indicator, a grid-snapped position) is already in place.

No new type or method is needed to convey it.

## 21) Reduce error-path formatting machinery to shrink WASM binary size

An external review suggested three approaches to reduce the code generated for DOM error handling, all framed as a binary-size experiment to be verified with `twiggy` or `wasm-tools`:

```rust
// Option A — store the raw JsValue instead of formatting eagerly
pub enum Error { Dom(JsValue), ... }

// Option B — store both the raw value and a static context label
pub enum Error { Dom { operation: &'static str, value: JsValue }, ... }

// Option C (less intrusive) — mark dom_err cold and non-inline
#[cold] #[inline(never)]
pub(crate) fn dom_err(e: JsValue) -> Error { ... }
```

None of these changes will be adopted.

### Options A and B (storing `JsValue`) have a hard target blocker

`JsValue` is a `wasm_bindgen` type that does not exist on non-WASM targets.
The crate compiles on `x86_64` for its 19 host-side unit tests; several of those tests construct `Error::Dom("...".into())` directly and pattern-match on the `String` payload.
Changing `Error::Dom` to hold `JsValue` would break every one of them and would require `#[cfg(target_arch = "wasm32")]` gating on the error type itself.
That eliminates host-target compilation and the test suite entirely — neither option is acceptable.

Even setting that blocker aside, the binary impact is neutral.
The argument for both options is that not formatting eagerly in `dom_err` removes the `format!("{e:?}")` call from that one site.
But error values must eventually be presented to the caller; both options still require a `Display` impl that calls the same formatter.
The formatting machinery is simply moved from `dom_err` to `Display` — it is not removed from the binary.
There is no net change in compiled code unless `Display` for `Error::Dom` were also eliminated, which would make the error type unpresentable and defeat its purpose.

### Option C — `#[cold]` / `#[inline(never)]` — is likely a no-op for binary size

All 109 `dom_err` call sites use the pattern `.map_err(dom_err)`, which passes `dom_err` as a function item to `Result::map_err`.
When a function item (not a closure) is passed to a higher-order generic function like `map_err`, the compiler treats the call as opaque — `dom_err`'s body is not inlined at any of those sites under normal optimisation.
`#[inline(never)]` would therefore have no measurable effect: the function is already not being inlined.

`#[cold]` provides a branch-prediction hint that the error branch is rarely taken.
Because DOM calls in this crate fail only under pathological conditions (the browser rejects a standard SVG attribute name, or memory is exhausted), the optimizer already infers the error path is cold from context; the explicit attribute would at most confirm what the optimizer already assumed.

### The measurement gate is the correct bar

The RFC itself acknowledges the speculative nature of the suggestion: *"this must be verified with twiggy or wasm-tools size; wasm-bindgen representation costs might offset the saving."*

The crate's existing position on implementing speculative binary size reductions is to decline unless a measurement showing a real reduction is available, but no such measurement exists for these proposals.
This has been applied consistently across ideas 3, 4, 6 and 11

The dominant contributors to the compiled WASM binary size are the DOM interaction trampolines generated by wasm-bindgen and the formatting and attribute-writing infrastructure, not the error-path formatting code that runs only when standard browser calls fail.
Rearranging how `Error::Dom` stores its payload would not address any of those contributors.

## 22) Feature-gate event families and specialised SVG functionality

An external review proposed optional Cargo features that gate individual event type families:

```toml
[features]
default = ["events-pointer", "events-mouse"]
events-drag     = ["web-sys/DragEvent"]
events-focus    = ["web-sys/FocusEvent"]
events-keyboard = ["web-sys/KeyboardEvent"]
events-touch    = ["web-sys/TouchEvent"]
events-wheel    = ["web-sys/WheelEvent"]
```

Plus optional feature gates for gradients and markers as self-contained modules.
The motivation was to allow downstream apps to opt out of event families (and the associated web-sys bindings) they never use.

None of these changes will be adopted.

### Dead-code elimination already handles unused event types

`EventClosure` is `pub(super)`, meaning its constructors are only reachable within the `node` module.
The only construction sites are the private helpers `add_drag_listener`, `add_touch_listener`, `add_wheel_listener`, and so on, which are each called by exactly one `on_*` family of methods in `listeners.rs`.
If a downstream application never calls any `on_drag*` method, the LLVM can prove that `EventClosure::Drag` is never constructed and therefore eliminates:

- all monomorphizations of `add_drag_listener` and its generic closure parameter
- the `EventClosure::Drag` constructor
- every `EventClosure::Drag` match arm in `callback_ref` (reachable only from a variant that is provably dead)
- the web-sys `DragEvent` binding methods, which are `extern "C"` declarations that the linker and `wasm-opt` strip when not referenced

The web-sys extern type itself (the struct wrapping a `JsValue`) carries no code of its own; it is an opaque handle that generates no WASM instructions.
After LTO and wasm-opt's dead-import removal pass, an unused event family contributes nothing to the final binary.
Feature gates would not provide a binary reduction beyond what the existing optimisation pipeline already delivers.

### The `EventClosure` enum makes cfg-gating architecturally fragile

Feature-gating individual variants of an enum in Rust requires identical `#[cfg(feature = "...")]` guards on every match arm at every match site.
`EventClosure` is currently matched exhaustively in three places (`callback_ref`, `EventListener::detach`, and `ListenerStore::remove_by_type` via the chain to `callback_ref`).
Any mismatch between cfg guards at a construction site and the corresponding match sites is a compile error.
Adding a new event type or renaming a feature flag requires synchronized edits across all match sites.

This maintenance surface is disproportionate to the savings, which are zero after DCE (see above).

### Changing default features is a breaking semver change

Today every event type is unconditionally available to callers.
The RFC proposes `default = ["events-pointer", "events-mouse"]`, which would silently remove `on_key_down`, `on_drag`, `on_wheel`, `on_touchstart`, and related methods from any downstream app that does not explicitly request the corresponding feature.
That is a breaking change by semver convention, requiring a 0.2.0 version bump and coordinated migration for all existing users.
An accidental breakage of a working build is a significantly worse outcome than a marginal binary size increase.

### Gradient and marker feature gates save no web-sys bindings

Neither `SvgLinearGradient`, `SvgRadialGradient`, nor `SvgMarker` depend on their own web-sys feature flags.
All three types hold an `SvgElement` internally and manipulate it through the same DOM interfaces that every other node in the library uses.
Gating these modules behind optional features would remove only their Rust-level impl code — not any web-sys bindings — and DCE already removes that code if the types are never instantiated.

### CI combinatorial explosion

Seven event-family feature flags produce up to 128 combinations.
Testing even a representative subset — six to ten configurations — adds meaningful CI complexity for maintenance that provides no binary savings over the current unconditional build.
The crate's `#[deny(missing_docs)]` requirement also means every conditionally-compiled public item needs doc comments in each cfg branch, compounding the maintenance surface further.

### The measurement gate

The RFC itself acknowledges the risk: *"LLVM and `wasm-bindgen` already eliminate much unused code, so the real reduction could be small."*
No size profile comparing a representative downstream application compiled with and without these gates has been produced.

The consistent position for speculative binary-size changes is to decline until a measurement can show that a real reduction exists; this has been applied to ideas 3, 4, 6, 11, and 21, and it applies here as well.

## 23) Optional shared RAF scheduler (`AnimationScheduler`)

An external review proposed an `AnimationScheduler` abstraction that owns one `requestAnimationFrame` registration and dispatches `N` registered callbacks through it each frame.
The motivation was that `N` independent `AnimationLoop` values make `N` JS → WASM boundary crossings per frame and issue `N` RAF registrations, all carrying essentially the same timestamp.
It was proposed that a shared scheduler would reduce both to one.

None of the proposed changes will be adopted.

### The aggregation is already available and the crate already demonstrates it

`AnimationLoop::start` and `start_with_frame` both accept a single `FnMut` callback that can call any number of sub-functions.
The existing animation demo drives three geometrically independent animations (a pulsing circle, a travelling circle, and a hue-rotating rectangle) from one callback, with one RAF registration.
This is already the idiomatic pattern: if several animations must run concurrently, put all their updates in one closure.
No library change is needed for the common case; the scheduler adds a layer of infrastructure to solve a problem the API already does not impose.

### Mutation during dispatch multiplies a problem already shown to be subtle

`AnimationLoop` already required careful handling for the "stop from inside the callback" case: `AnimLoopState::Dispatching` prevents an immediate closure drop (which would be a use-after-free of `Rc` fields still on the stack), and a deferred `setTimeout(0)` cleans up the slot once the callback has fully returned.
A scheduler with `N` callbacks inherits all of this complexity and multiplies it: any callback can deregister *itself*, deregister *another* callback, add new callbacks, or drop the whole scheduler.

The RFC acknowledges this directly and proposes a slot table with tombstones followed by post-dispatch compaction.

A tombstone-based slot table is the correct solution, but every frame then pays:

- a linear scan of the slot table to collect live indices before dispatch (to avoid iterating while callbacks mutate the collection)
- a second pass to compact tombstoned slots
- `RefCell` borrow management around each callback invocation to keep the collection accessible for mutation

This introduces O(N) bookkeeping per frame even when the slot table is perfectly stable, and the borrow hygiene is at least as finicky as the existing `AnimLoopState` trick &mdash; but with more interacting mutation paths.
A panic or use-after-free in this code in production would cause a silent WASM failure with no stack trace.

### The feature is not SVG-specific

A multi-callback RAF multiplexer does not interact with the SVG DOM, `SvgNode`, `SvgRoot`, attributes, or any other part of the crate.
It is a general-purpose WASM/browser utility whose correct home is a dedicated crate (`raf-scheduler` or similar), where it can be composed with any browser WASM project.
Adding it here conflates the library's scope without offering any SVG-specific advantage.

### The hosting model has no good answer

A standalone `AnimationScheduler` value can be instantiated many times; multiple schedulers in the same application simply recreate the N-loop problem.
For the scheduler to actually unify all animations it must be treated as a shared singleton, which in WASM means `thread_local!` state or `Rc<RefCell<...>>` indirection passed through every code module that registers a callback.
That burden falls on the application developer, who would not have needed it at all had they put their animations in one `AnimationLoop` callback from the start.

Hosting the scheduler on `SvgRoot` (one per SVG) is a cleaner ownership model but adds surface to `SvgRoot` that has nothing to do with SVG element creation or mutation.

### The overhead is marginal at realistic `N`

* One JS → WASM boundary crossing at 60 Hz costs roughly 1–2 µs on modern hardware.

* For `N = 3` concurrent loops, the "waste" over a unified scheduler is 2 extra crossings per frame — approximately 120–240 µs per second, well under 0.02 % of CPU time on any current device.

* The benefit is only significant at large `N` (the RFC uses ten as its example), and no profile of a downstream application with that many concurrent `AnimationLoop` values has been produced.

As with previous rejections, the measurement gate that has been consistently applied to speculative performance work (entries 3, 4, 6, 11, 21, and 22) applies here as well.

## 24) Optional delegated event handling for dense interactive scenes

An External review proposed implementing a delegated listener API where a single bubbling event handler attached to an `SvgRoot` or group would serve all matching descendants, reducing `N` per-node registrations down to one:

```rust
let delegated = svg.on_delegated_click("[data-item]", |node, event| {
    // handle clicks for all matching descendants
})?;
```

The stated benefit for a scene with `N` interactive nodes was that browser listener registrations, wasm-bindgen closures, listener-store entries and setup/teardown costs all fall from `N` to 1.

None of the proposed changes will be adopted.

### The current API already achieves event delegation with no changes

Attaching any bubbling event listener to a group or root node is already event delegation.
The handler fires for events originating from all descendants; `event.target()` identifies the originating element.
A scene with 1000 interactive items needs exactly one `on_click` registration on their common ancestor: one listener-store entry, one wasm-bindgen closure and one browser callback:-

```rust
let group = svg.group()?;
// … append 1000 child nodes to group …

group.on_click(|event| {
    if let Some(target) = event.target() {
        if let Ok(element) = target.dyn_into::<web_sys::Element>() {
            // inspect element.get_attribute("data-item") or element.id()
        }
    }
})?;
```

The RFC's proposed `on_delegated_click` API adds nothing that the current API cannot express; it would be a thin convenience wrapper around exactly the pattern shown above.

### The handler signature `|node, event|` cannot be implemented without a hidden registry

The distinguishing feature of the proposed API over `on_click` on a group is that the closure receives a typed `SvgNode` for the originating element, not a raw `event.target()`.

`SvgNode` is `Rc<SvgNodeInner>`; which is a reference-counted smart pointer to an opaque inner struct.
There is no constructor that builds one from a bare `web_sys::SvgElement`.
Reconstructing an `SvgNode` from a delegated event target therefore requires an external lookup table mapping DOM element identity to `Rc<SvgNodeInner>`.

Such a table would need to:
- be updated on every `SvgNode` construction and destruction;
- use DOM-object identity keys (since `web_sys::Element` does not implement `Hash` or `Eq` any comparison would require the use of `JsValue::loose_eq` or `Object.is()`);
- hold at most `Weak<SvgNodeInner>` references to avoid keeping dead nodes alive;
- be stored somewhere with a lifetime long enough to serve delegated callbacks — either on `SvgRoot` (which would become an always-on overhead for every app) or in a separate opt-in type.

This is a significant architectural addition that adds a non-trivial runtime overhead on every node creation and destruction, solely to avoid calling `get_attribute` inside the closure.
If the delegated handler receives a raw `web_sys::Element` instead (not a `SvgNode`), the API differs only cosmetically from `on_click` on a group, which the user can already write.

### The CSS selector check introduces per-event JS boundary crossings

The proposed API filters targets with a CSS selector string such as `"[data-item]"`.
Evaluating this filter requires a JS call via `element.closest(selector)` or `element.matches(selector)` inside every event dispatch.
For click events this is harmless, but the RFC also mentions `pointermove`, `keyboard events`, and `drag events` as appropriate event types.
For `pointermove` at 60 Hz with an active scene, a JS call per event for selector evaluation reintroduces boundary crossings in the hot path, partially erasing the registration savings the feature was designed to provide.

### Non-bubbling events limit the useful scope to a narrow set

The RFC correctly notes that `pointerenter`, `pointerleave`, `mouseenter`, `mouseleave`, `focus`, and `blur` have different bubbling semantics and cannot be delegated.
These are the events most commonly used for hover and focus effects — exactly the class of per-element behaviour most likely to be attached to many nodes in an interactive scene.
The events that do bubble cleanly (`click`, `pointerdown`, `pointerup`) are lower-frequency and better served by the already-available group-listener pattern.

### The ownership model documentation cost

The crate's central invariant is *"a listener belongs to the node that registered it."*
Delegated listeners break this invariant: the handler fires for events originating on descendant nodes, not the node that holds the registration.
Introducing a parallel ownership model requires documentation carve-outs, lifecycle caveats distinct from the existing listener-management guidance and decisions about whether delegated registrations participate in `clear_listeners` and `remove_listeners` — whose semantics have already been carefully defined for the node-owned case.

## 25) `build_gaussian_blur` / `build_offset` / `build_merge` closures for filter primitives

An external review suggested that each `SvgFilter` primitive method (`gaussian_blur`, `offset`, `merge`) could gain a `build_*` closure-based sibling, mirroring `SvgDefs::build_filter`'s detached-until-success pattern:

```rust
filter.build_gaussian_blur(4.0, |b| {
    b.set_attrs([("in", "SourceAlpha"), ("result", "blur")])?;
    Ok(())
})?;
```

versus the current two-step shape:

```rust
filter
    .gaussian_blur(4.0)?
    .set_attrs([("in", "SourceAlpha"), ("result", "blur")])?;
```

The review's own recommendation was **not** to add this, reasoning that `build_filter` already keeps the whole filter detached for normal construction, that dynamically extending an already-referenced filter is an uncommon path, and that a `build_*` sibling would double the API surface per primitive.
That conclusion is correct, and this rejection entry is to provide confirmation.

### `build_filter` already makes the *only* case that matters impossible to observe

`SvgDefs::build_filter` (`src/root/defs.rs`) creates the `<filter>` element, runs the caller's closure against it, and only calls `self.element.append_child(filter.as_element())` — attaching it to `<defs>` — after the closure returns `Ok`.
Every primitive method called inside that closure (`gaussian_blur`, `offset`, `merge`) therefore mutates an element that is not yet part of the document tree at all, so there is nothing for a renderer to recompute regardless of how many intermediate attribute writes happen.

`<defs>` content is never rendered even once attached, and nothing can reference the filter via `url(#id)` before the caller explicitly applies it with `set_filter_ref`/`set_filter`, which necessarily happens after `build_filter` returns.
So for ordinary construction (build a filter, then apply it) the multi-mutation sequence the review is concerned about is not just unlikely to matter, it is architecturally unobservable.

### The remaining case is narrower than "dynamic filter modification" in general, and mostly synchronous anyway

The one case where intermediate mutations could, at least in principle, be observed is when calling a primitive method on a filter that is *already* attached to `<defs>` *and already referenced* by a live, rendered node — e.g. `existing_filter.gaussian_blur(4.0)?.set_attrs([...])?` invoked some time after the filter was first built and applied.

Even then, the two-call chain shown in both the review's example and every example in this crate's own docs is fully synchronous Rust/WASM: `gaussian_blur` returns, then `set_attrs` runs, with no `await` or event-loop yield between them.

Browsers coalesce a style recalculation and a paint to happen after the current task finishes, not synchronously after each individual `setAttribute`/`appendChild` call — that only happens on a forced synchronous layout, which requires the script to *read* a layout-dependent property (`getBBox`, `offsetWidth`, ...) between the writes, which nothing here does.

So even in the narrow live-filter-mutation case, a chained call sequence like the one above produces exactly one paint with the final, fully-configured primitive — not a flash of intermediate states.
The scenario the review's own stated motivation describes would need the caller to hold the returned `SvgNode` across an actual event-loop boundary (store it, mutate it later from a different callback or after a `setTimeout`).
This is a real, but considerably narrower case than a braod "dynamic filter modification".

### Cost is not zero

The review's own third point stands on its own regardless of the above: a `build_*` sibling for every primitive doubles the primitive API surface (`gaussian_blur` + `build_gaussian_blur`, `offset` + `build_offset`, `merge` + `build_merge`, and every primitive added after them) for a benefit that only applies to an already-narrow set of cases.
This runs against the same restraint already exercised for [`gaussian_blur_xy`](../src/root/filter.rs) and `merge`'s slice parameter (see `docs/design_notes.md`, "`<filter>` primitives return a plain `SvgNode`" and its "confirms the plain-`SvgNode` decision" follow-up): add the minimum shape a primitive actually needs, not the maximum shape it could conceivably use.

### Verdict

This review suggestion has not been implemented, not only for the review's own stated reasons, but we have nere provided the additional confirmation that the synchronous call pattern this crate actually uses does not exhibit the intermediate-mutation problem the proposal was framed around.

Revisit only if a genuinely asynchronous dynamic-filter-mutation workload is profiled in a real browser and shown to cause repeated filter-graph recomputation — the same evidence-first bar this crate has applied to every other performance-motivated addition.
