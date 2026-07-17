# Event and listener system

[← Back to rejected ideas](README.md)

**Contents**

- [An `EventName` enum instead of `&'static str`](#an-eventname-enum-instead-of-static-str)
- [Rewriting `ListenerStore::push` to push into the `Many` vector in place](#rewriting-listenerstorepush-to-push-into-the-many-vector-in-place)
- [Deferred listener drops to make self-removal safe from within a handler](#deferred-listener-drops-to-make-self-removal-safe-from-within-a-handler)
- [Flatten `EventClosure` by simplifying it to `Closure<dyn FnMut(Event)>`](#flatten-eventclosure-by-simplifying-it-to-closuredyn-fnmutevent)
- [Listener removal has documented unsafe lifecycle caveats](#listener-removal-has-documented-unsafe-lifecycle-caveats)
- [Feature-gate event families and specialised SVG functionality](#feature-gate-event-families-and-specialised-svg-functionality)
- [Optional delegated event handling for dense interactive scenes](#optional-delegated-event-handling-for-dense-interactive-scenes)

## An `EventName` enum instead of `&'static str`

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

## Rewriting `ListenerStore::push` to push into the `Many` vector in place

`ListenerStore::push` replaces `self` with a non-allocating `Many(Vec::new())` placeholder, moves the old value out by value, and matches it exhaustively (`One` → upgrade to a two-element `Many`; `Many` → push and put back).

It was suggested that the `Many` case should instead push into the vector in place (matching `self` by reference, with a separate `mem::replace` + `unreachable!()` only on the `One` → `Many` upgrade path) to avoid moving the whole `Vec` out and back.

This crate makes every attempt to exclude the possibility of generating a WASM binary that conatins an `unreachable!()` call, because if this was statement was ever reached, the WASM runtime would terminate this binary immediately and tear down its memory (effectively self-destructing).

* **The two properties are in tension; the current form is deliberately kept because it is panic-free.**<br>
  To upgrade `One` → `Many` the first listener must be moved out of `&mut self`, which requires `mem::replace`.
  Matching that owned result can either handle *both* variants meaningfully (today's code which is exhaustive and does not panic) *or* pre-narrow `self` by reference and then need `unreachable!()` for the "impossible" arm.
  Rust's move rules do not allow both at once.

* **The saving is nil and the cost is real (if small).**<br>
  `Vec::new()` does not allocate, so the `Many` arm only moves the vector's 24-byte `(ptr, len, cap)` handle out and back — there is no extra heap allocation beyond the `push` itself, and `push` runs at listener-registration time, never on a hot path.
  Against that zero saving, `unreachable!()` adds the possibility of a panic and `#[track_caller]` location data to the **wasm binary** (a size concern this crate takes seriously — see [A size-optimised `[profile.release]` baked into the crate](performance.md#a-size-optimised-profilerelease-baked-into-the-crate)); the optimiser *may* prove the arm dead and strip it, but that is not guaranteed.

The current code is exhaustive, panic-path-free, allocation-neutral, and documented as such at the call site, so the proposal is a lateral-to-slightly-worse change to working code and was not adopted.

## Deferred listener drops to make self-removal safe from within a handler

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

## Flatten `EventClosure` by simplifying it to `Closure<dyn FnMut(Event)>`

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

## Listener removal has documented unsafe lifecycle caveats

A follow-up to [deferred listener drops](#deferred-listener-drops-to-make-self-removal-safe-from-within-a-handler) was to replace the deferred-drop guard with a listener-handle model:

```rust
let handle = node.on_click(...)?;
handle.remove();
```

The claim was that this model "marks the listener as removed, detaches it from DOM, and defers dropping the closure until after the current dispatch completes", making self-removal safe without the dispatch-depth counter.

The claim is wrong on its central premise: **a handle does not eliminate the need for dispatch tracking.**
To defer a closure drop until after dispatch, you still need to know when dispatch is active.
The only mechanism that provides this is the dispatch-depth counter; which is the same infrastructure rejected [above](#deferred-listener-drops-to-make-self-removal-safe-from-within-a-handler) for adding overhead to every event on every node.
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

### The underlying analysis from the deferred-drops rejection still applies

- The abstract-machine UB is practically inert in the wasm32 execution model: the trampoline does not dereference the freed pointer again after the user closure returns, and WebAssembly is single-threaded, so no concurrent reuse can occur.
- The motivating use case (a handler that removes itself) is already correctly served by `on_event_once`, which uses `{ once: true }` and `Option::take` so the browser removes the DOM listener and the user handler's captures are freed after the first dispatch, never while executing.
- Removing listeners on a *different* node, or for a *different* event type on the same node, is already safe.

### What handles would genuinely add

A handle that carries the identity of a specific listener would enable removal of one listener out of several registered for the same event type: something `remove_listeners("click")` cannot do as it removes all listeners for the specified event type e.g. `click`.
This is a new, incremental feature request, not a safety fix.

If specific-listener removal proves necessary in practice, it can be evaluated on its own merits, with an API designed for that use case rather than framed as a correction to the existing safety model.

## Feature-gate event families and specialised SVG functionality

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

This crate's consistent position on speculative binary-size proposals is to decline until a measurement can show that a real reduction exists — see [Performance & binary size](performance.md) for the other entries this has been applied to (the `ryu`/`itoa` crate, `path_fmt`/`text_fmt`, handle-light factories, and the error-path formatting machinery), and [flattening `EventClosure`](#flatten-eventclosure-by-simplifying-it-to-closuredyn-fnmutevent) above within this category — it applies here as well.

## Optional delegated event handling for dense interactive scenes

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
