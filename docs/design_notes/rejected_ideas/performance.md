# Performance and binary-size proposals

[← Back to rejected ideas](README.md)

Every entry in this file was declined for the same underlying reason.
Elsewhere in this document, that reason is called **the measurement gate**.
This crate's consistent position: decline a speculative performance or binary-size change until a real measurement shows an actual benefit.
A real measurement means a profile, a `wasm-opt`/MD5 comparison, or a `twiggy`/`wasm-tools` size report — not a plausible-sounding argument alone.
The same gate has also been applied to two proposals filed under other categories: [feature-gating event families](events.md#feature-gate-event-families-and-specialised-svg-functionality) and [an optional shared RAF scheduler](animation.md#optional-shared-raf-scheduler-animationscheduler).
Both are cross-referenced from here.

**Contents**

- [A faster float-to-string crate (`ryu` / `itoa`)](#a-faster-float-to-string-crate-ryu--itoa)
- [`path_fmt` / `text_fmt` factory helpers](#path_fmt--text_fmt-factory-helpers)
- [Handle-light factories for large static scenes (`static_rect`, raw `SvgElement`)](#handle-light-factories-for-large-static-scenes-static_rect-raw-svgelement)
- [A size-optimised `[profile.release]` baked into the crate](#a-size-optimised-profilerelease-baked-into-the-crate)
- [Typed cached-attribute wrappers for scalar values (`CachedF64Attr` / `CachedAttr::set_display`)](#typed-cached-attribute-wrappers-for-scalar-values-cachedf64attr--cachedattrset_display)
- [Reduce error-path formatting machinery to shrink WASM binary size](#reduce-error-path-formatting-machinery-to-shrink-wasm-binary-size)

## A faster float-to-string crate (`ryu` / `itoa`)

One suggestion: accelerate numeric formatting by routing it through a dedicated crate, such as `ryu` (floats) or `itoa` (integers), instead of the standard library's `Display`.

Two things undercut it:

* **It does not fit the hot path.**<br>
  The high-frequency formatting in this crate is the transform setters, which use *fixed precision* (`{:.1}`, `{:.3}`).
  `ryu` emits the **shortest round-trip** representation.
  That is a different output, so it cannot replace that formatting at all.
  It would only touch the default `{value}` `Display` path used at element-creation time and in `set_attr_display` — not the per-event work it was meant to accelerate.

* **The win over std is marginal.**<br>
  Rust's standard `f64` `Display` is itself a shortest-round-trip (Ryū-derived) implementation, so the realistic saving is small and confined to creation-time formatting.

Set against an added dependency in a published crate (which grows every downstream user's dependency tree), the trade is not worth it.
The dominant per-call cost on any real hot path is the `set_attribute` boundary crossing, not the float-to-string conversion.
Effort is better spent eliding redundant DOM writes (`CachedAttr`) and reusing format buffers (the transform setters and `set_attr_display`).

## `path_fmt` / `text_fmt` factory helpers

One suggestion: have the factories accept `std::fmt::Arguments` directly — `path_fmt(format_args!(...))` and `text_fmt(...)`, plus the `SvgBatch` equivalents.
This way, a caller building a computed `d` or label string need not allocate a `String` before the factory sets the attribute (instead of today's `svg.path(&format!(...))`).
The new methods would format into the factory's existing `SvgAttrs` scratch buffer.

* **It optimises a cold path.**<br>
  Element creation runs at setup time, not per frame or per event.
  Every allocation-light helper in this crate — `AnimationFrame::set_*_fmt`, the transform setters, `SvgAttrs`/`AttrWriter` — exists to remove churn from genuinely *hot* paths.
  One allocation at creation is not that.
  This is the same distinction noted for the [`ryu`/`itoa` idea above](#a-faster-float-to-string-crate-ryu--itoa).

* **The saved string is dwarfed by what the factory already does.**<br>
  Every factory call already performs a `create_element_ns` (a wasm/JS boundary crossing that allocates a live DOM node) and a DOM append.
  A caller's `format!` for the `d`/text is negligible beside those, so nothing measurable is changed by removing it.
  This is the same "boundary crossing dominates" reasoning used to reject `ryu`.

* **The hot case is already covered.**<br>
  A path or label whose `d` or text *changes over time* should be created once, then mutated on the live node — never recreated.
  Use `AnimationFrame::set_d_fmt`/`set_text_fmt` inside a RAF loop, or `SvgAttrs::fmt`/`SvgNode::set_text_fmt` in an event handler.
  The crate's model is mutating live nodes rather than rebuilding the tree, so per-frame element re-creation is already a non-goal.

Against those benefits, the cost is four new public methods (`path_fmt`/`text_fmt` on both `SvgRoot` and `SvgBatch`).
Each carries documentation and tests, under `#![deny(missing_docs)]`, just to remove a single setup-time allocation.
Callers who format at creation time can simply write `svg.path(&format!(...))`.
If a future profile ever shows element-creation churn dominating (for example frequent full rebuilds), the right response is to mutate existing nodes, not to add creation-time formatting helpers.

## Handle-light factories for large static scenes (`static_rect`, raw `SvgElement`)

One suggestion targeted scenes containing thousands of static elements whose handles are discarded immediately.
It proposed avoiding the per-element allocation of an `Rc<SvgNodeInner>`.
The factories could skip constructing a managed `SvgNode`, via functions such as `static_rect(...)` or `static_path(...)`.
These would return a "naked" `web_sys::SvgElement` instead of a wrapped `SvgNode`.

* **The `Rc` is dwarfed by the per-element DOM cost.**<br>
  Every factory call already creates a real browser DOM node via `create_element_ns` (thus crossing the wasm/JS boundary) and makes one `set_attribute` crossing per attribute.
  A single `Rc::new` of a two-field struct is noise beside that.
  It is a one-time setup cost, not a hot-path one.

  This is the same "boundary crossing dominates" reasoning used to reject `ryu`/`itoa` and the [`path_fmt` helpers above](#path_fmt--text_fmt-factory-helpers).

* **The real cost of bulk creation is already addressed.**<br>
  `SvgBatch` (`build_batch` / `build_batch_into`) appends many elements through a single `DocumentFragment` operation, which targets the DOM-mutation and reflow cost that actually scales with element count.
  A `static_*` variant cannot remove the cost of boundary crossing — each element and its attributes still have to be created.
  It would only save a negligible handle allocation on top of the work `SvgBatch` already minimises.

* **It bifurcates the API for a speculative gain.**<br>
  A `static_*` form of every factory across both `SvgRoot` and `SvgBatch` is a large, permanent public surface, with docs and tests under `#![deny(missing_docs)]`.
  It also forces a "which one do I use?" decision on every caller.
  The recommendation is itself conditional ("if this crate will be used for thousands of static elements").
  No profile shows the handle as a bottleneck.

* **It re-exposes raw `web-sys`.**<br>
  Returning a bare `web_sys::SvgElement` (or nothing) discards the cheap-to-clone live handle that is the crate's reason to exist.
  It leaves a caller who later wants to mutate the element with no `SvgNode`.
  The rare need to reach raw `web-sys` is already met, after the fact, by [`SvgNode::as_element`](../../../src/node/mod.rs).

If a real workload ever proves the handle allocation to be a measurable bottleneck, this can be revisited.
But the DOM-node and `set_attribute` costs will still dominate, so the saving would remain marginal.

## A size-optimised `[profile.release]` baked into the crate

One suggestion: add a wasm-shrinking release profile — `lto = true`, `codegen-units = 1`, `opt-level = "z"`, `panic = "abort"`, `strip = true`.
It would also run `wasm-opt -Oz` as part of packaging, to reduce download and instantiation size for production builds.

This idea cannot be implemented because it does not apply to Rust libraries — only to applications.

* **A dependency's `[profile.release]` is ignored.**<br>
  `svg-dom` is a library.
  A `[profile.release]` here would govern only builds where `svg-dom` itself is the root — i.e. the demo's own wasm build.
  It would never govern a downstream application's production build, which is the thing the recommendation wants to shrink.
  Cargo only honours the `[profile.*]` of the crate being built as the root, or the workspace root.
  Profiles declared by dependencies have no effect.

* **These settings belong to the application, not the library.**<br>
  `opt-level = "z"`, `panic = "abort"`, `lto`, and `strip` are whole-binary trade-offs (size vs speed vs unwinding) that are the application author's call.
  The right home for them is the consumer's own `Cargo.toml`.
  Imposing them from a dependency would be both ineffective (see above) and presumptuous.

* **The demo artifact is the only thing it would actually affect, and that is a local dev tool.**<br>
  The ~200 KiB `pkg/` build is produced locally by `wasm-pack`, and git-ignored via wasm-pack's own `pkg/.gitignore`.
  It is served only by `cargo demo`.
  Its download size is not a shipped concern.
  `wasm-pack` already runs `wasm-opt` on release builds, configurable through `[package.metadata.wasm-pack.profile.release]`, so the size lever for the demo already exists in the toolchain.

This recommendation does, however, contain a useful kernel: *how to minimise wasm size*.
But it belongs as guidance for application authors — set the size-optimised profile in **your** app and let `wasm-pack`/`wasm-opt` run — not in any configuration of the library manifest.

## Typed cached-attribute wrappers for scalar values (`CachedF64Attr` / `CachedAttr::set_display`)

An external review suggested two related additions to reduce the verbosity of caching small numeric or scalar attribute states:

```rust
// Option A — a new public type that bakes in the attribute name and precision
let mut opacity = CachedF64Attr::new("opacity", 3);
opacity.set(&node, alpha)?;

// Option B — a new method on the existing CachedAttr
cached.set_display(&node, "opacity", alpha, &mut scratch)?;
```

The reviewer correctly hedged Option B, stating "The latter may already be close to `CachedAttr::set_fmt`; if so, I would not add another API merely for convenience."
That hedge applies.

Neither addition was adopted.

### `CachedAttr::set_fmt` already covers every stated use case

`CachedAttr` already exposes four methods: `set` (string value), `set_text` (text content), `set_fmt` (formatted attribute), and `set_text_fmt` (formatted text content).
`set_fmt` takes a caller-owned scratch buffer and `fmt::Arguments`.
It formats into the buffer and delegates to `set`.
`set` skips the DOM write when the formatted string matches the value already present in the cache.

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

A new public type that bakes in an attribute name and a precision would require a `String` scratch buffer as a struct field.
It would also narrow the formatting to a fixed-precision float — `CachedF64Attr::new("opacity", 3)` implies `"{:.3}"`.
That is a subset of what `set_fmt` already provides.
The only benefit is moving the attribute name and precision from the call site to the constructor.
The caller must still provide a value per call.

Against that minor ergonomic shift sits a new `pub struct`, its documentation and its `impl` block.
Every future caller must still answer the question "which one do I use?"
That amounts to a permanent surface cost for a temporary convenience gain.

### Against `CachedAttr::set_display`

`set_display<T: Display>` taking a generic value rather than `fmt::Arguments` would save writing `format_args!("{}", n)` at the call site, replacing it with just `n`.
This is a real but extremely narrow ergonomic win.

More importantly, it does not help the specific cases the review lists.
Floating-point values displayed as `"{:.2}"` or `"{:.0}%"` require a format string beyond `{}`, so `set_display` would not apply and callers would still reach for `set_fmt`.
The only realistic beneficiaries are integer-valued states — frame counters, selection indices — whose `{}` output is exactly what is needed.
But for those, `format_args!("{}", n)` is the entire overhead being eliminated.
Adding a new method to remove it is not worth the extra API surface.

Adding `set_display` alongside `set_fmt` would also create a two-method decision: callers who see both would need to reason about when each applies.
The cognitive load added by saying *"use `set_display` only when `{}` is the right format and you don't need a format string"* outweighs any call-site saving.

### The Profiling Caveat

The review correctly notes that cached writes are worthwhile only when values repeat after quantisation.
For continuously changing values that never cause a cache hit, the comparison cost is pure overhead.

This consideration is already documented in `cached.rs`.
The module-level notes contrast it with `SvgNode::set_attr_if_changed`, which pays the cost of a JS round-trip for the comparison.
`CachedAttr`, by contrast, keeps the last value on the Rust side.

The guidance that a `CachedAttr` should be dedicated to a single *frequently-touched-but-rarely-changing* attribute (a cursor style, a discrete state indicator, a grid-snapped position) is already in place.

No new type or method is needed to convey it.

## Reduce error-path formatting machinery to shrink WASM binary size

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
The crate compiles on `x86_64` for its 19 host-side unit tests.
Several of those tests construct `Error::Dom("...".into())` directly, and pattern-match on the `String` payload.
Changing `Error::Dom` to hold `JsValue` would break every one of them.
It would also require `#[cfg(target_arch = "wasm32")]` gating on the error type itself.
That eliminates host-target compilation and the test suite entirely — neither option is acceptable.

Even setting that blocker aside, the binary impact is neutral.
The argument for both options is that not formatting eagerly in `dom_err` removes the `format!("{e:?}")` call from that one site.
But error values must eventually be presented to the caller.
Both options still require a `Display` impl that calls the same formatter.
The formatting machinery is simply moved from `dom_err` to `Display` — it is not removed from the binary.
There is no net change in compiled code unless `Display` for `Error::Dom` were also eliminated, which would make the error type unpresentable and defeat its purpose.

### Option C — `#[cold]` / `#[inline(never)]` — is likely a no-op for binary size

All 109 `dom_err` call sites use the pattern `.map_err(dom_err)`, which passes `dom_err` as a function item to `Result::map_err`.
When a function item, not a closure, is passed to a higher-order generic function like `map_err`, the compiler treats the call as opaque.
`dom_err`'s body is not inlined at any of those sites under normal optimisation.
`#[inline(never)]` would therefore have no measurable effect: the function is already not being inlined.

`#[cold]` provides a branch-prediction hint that the error branch is rarely taken.
DOM calls in this crate fail only under pathological conditions — the browser rejects a standard SVG attribute name, or memory is exhausted.
Because of that, the optimizer already infers the error path is cold from context.
The explicit attribute would at most confirm what the optimizer already assumed.

### The measurement gate is the correct bar

The RFC itself acknowledges the speculative nature of the suggestion: *"this must be verified with twiggy or wasm-tools size; wasm-bindgen representation costs might offset the saving."*

The crate's existing position is to decline speculative binary-size reductions unless a measurement shows a real reduction.
No such measurement exists for these proposals.
This has been applied consistently across the [`ryu`/`itoa`](#a-faster-float-to-string-crate-ryu--itoa), [`path_fmt`/`text_fmt`](#path_fmt--text_fmt-factory-helpers), [handle-light factory](#handle-light-factories-for-large-static-scenes-static_rect-raw-svgelement), and [flatten `EventClosure`](events.md#flatten-eventclosure-by-simplifying-it-to-closuredyn-fnmutevent) entries.

The dominant contributors to the compiled WASM binary size are the DOM interaction trampolines generated by wasm-bindgen, and the formatting and attribute-writing infrastructure.
They are not the error-path formatting code that runs only when standard browser calls fail.
Rearranging how `Error::Dom` stores its payload would not address any of those contributors.
