# Ideas Considered and Rejected

Design suggestions that were evaluated for `svg-dom` and deliberately not adopted.
The reasoning is preserved here so the same ideas are not repeatedly re-proposed.

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
A passive node therefore allocates **no** listener storage at all; it pays only for the inline field, that is, on `wasm32`, the `RefCell` borrow flag (4 bytes) plus a niche-optimised `Option<Box<…>>` pointer that is `null` when empty (4 bytes), so the saving adds up to only ~8 bytes.

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

It was suggested that `EventListener` store the event name as an enum (`Click`, `PointerMove`, … plus `Raw(&'static str)`) rather than a `&'static str`, on the grounds that an enum would be smaller than a fat string pointer, with `Drop` calling `event_name.as_str()`.

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

## 7) Pre-reserving capacity in `write_points`

It was suggested that `write_points` (in `src/root/utils.rs`) call `out.reserve(...)` with an estimated byte size before its formatting loop, so a first write of a large `<polyline>`/`<polygon>` does not grow the `String` repeatedly.

* **The buffer is reused, so steady state is already allocation-free.**<br>
  `write_points` starts with `out.clear()`, which keeps the existing capacity, and every caller (`SvgAttrs::points`, `AttrWriter::points`, `AnimationFrame::set_points`, and the factories) holds *one* buffer reused across calls.
  Once it has been sized by the first write, a same-or-smaller polyline never reallocates again — so an animated polyline (the situation for which the points API exists) sees no per-frame growth regardless.

* **It optimises only the first write, and that cost is already tiny.**<br>
  The reserve would help only the very first write (or a later write that grows past the high-water mark), which is a one-time, setup-shaped event.
  `String` growth is geometric (doubling), so even a 10,000-point polyline reallocates a handful of times totalling a couple of `memcpy`s of the final size — microseconds, paid once.

* **The proposed estimate is heuristic and partly wrong.**<br>
  The full-precision (`None`) constant of 24 bytes per point undershoots real `f64` `Display` output, which, when using full decimal precision, can exceed 30 bytes per coordinate pair, so the reserve would *still* leave the buffer to grow in exactly the case it was meant to cover — while adding per-call arithmetic and two magic constants to an otherwise clean shared helper.

If a profile ever showed first-write reallocation as a genuine bottleneck for enormous polylines, a single plain `reserve(points.len() * k)` could be revisited — but the buffer-reuse design already makes it moot for any repeated or animated use, which is the only hot path here.

## 8) Provide a rendered-size fallback (`getBoundingClientRect`) when seeding the cached viewport

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

## 9) Hiding `SvgRoot::root` behind an `as_element()` accessor

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

## 10) Rewriting `ListenerStore::push` to push into the `Many` vector in place

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

## 11) Deferred listener drops to make self-removal safe from within a handler

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

  An `on_event_once` helper built on `Closure::once` would serve this use case with no Rust-side depth tracking, no deferred store, and no store entry to bookkeep at all, because the browser removes the registration and wasm-bindgen frees the allocation at the right moment.

* **The remaining sub-cases are already safe and need no guards.**<br>
  Removing a *different* event type from the same node is safe: the running closure is not the one being freed.
  Removing listeners on a *different* node is always safe.
  Only the precise sub-case — a handler drops itself — is the footgun the docs warn against, and `on_event_once` is the right API for that pattern.

The deferred-drops design would add memory and CPU overhead to every node and every event in the entire crate in order to guard against a theoretical abstract-machine violation that has no observable consequence in the wasm32 execution model, when the primary motivating use case is already correctly served by `Closure::once`.

The existing documentation warning is the correct and sufficient response; a dedicated `on_event_once` helper is the right follow-on addition if the one-shot pattern proves common in practice.

## 12) Flatten `EventClosure` by simplifying it to `Closure<dyn FnMut(Event)>`

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
