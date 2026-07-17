# API surface and escape hatches

[← Back to rejected ideas](README.md)

**Contents**

- [Hiding `SvgRoot::root` behind an `as_element()` accessor](#hiding-svgrootroot-behind-an-as_element-accessor)
- [Hiding raw `web_sys` access behind `raw_element_unchecked()` or a Cargo feature](#hiding-raw-web_sys-access-behind-raw_element_unchecked-or-a-cargo-feature)
- [Canonicalising on one construction model for `defs`, `marker`, and `batch`](#canonicalising-on-one-construction-model-for-defs-marker-and-batch)
- [Reducing attribute-mutation surface area to one canonical path](#reducing-attribute-mutation-surface-area-to-one-canonical-path)

## Hiding `SvgRoot::root` behind an `as_element()` accessor

`SvgRoot` exposes its `<svg>` as the public field `root`, which lets a caller write `width`/`height` directly and desynchronise the cached viewport that backs `width()`/`height()` and `set_viewport`'s write-elision.

It was suggested that, as a future breaking change, the field should be made private and a `pub fn as_element(&self) -> &SvgsvgElement` accessor be added instead.
We documented the caveat on the field (direct mutation desyncs `width()`/`height()`; `set_viewport` is the cache-aware path) but did **not** make the field private.

* **The accessor would not actually protect the invariant.**<br>
  Every `web_sys` DOM mutator such as `set_attribute`, `set_attribute_ns`, and its typed property setters, take `&self`, not `&mut self` because DOM mutation goes through a shared JS handle.
  So `svg.as_element().set_attribute("width", "500")` desyncs the cache *exactly* as `svg.root.set_attribute("width", "500")` does.
  The change swaps a public field for a method of identical power over the invariant; it does not close the hole the recommendation set out to close.

* **Exposing the element is a deliberate escape hatch.**<br>
  This crate is a thin, minimal wrapper and does not wrap every SVG attribute or property (`preserveAspectRatio`, focus management, and so on — see `docs/gaps.md`). Geometry read-back (`getBBox`, `getCTM`, `getTotalLength`, ...) was in this category at the time of this rejection; it has since been wrapped as `SvgNode::bounding_box`/`ctm`/`total_length`/etc. — see [Geometry read-back](../geometry.md) — which does not change the argument here: the crate still does not, and will not, wrap *every* SVG attribute or property, only the ones with a demonstrated need.
  Direct access to the root `<svg>` is the supported way to reach those, so the leak is inherent to *exposing the element at all* (which we want to do) not to the field-versus-method spelling.

  The only extra power a public field grants is reassigning `root` wholesale, which needs `&mut SvgRoot` and would obviously corrupt the handle; that is a self-evident misuse, not a footgun worth a breaking API change to forbid.

* **Documentation is therefore the correct and sufficient fix.**<br>
  Since the element must remain reachable and any reference to it can mutate the DOM, the honest contract is a documented one: the field's doc now states that writing `width`/`height` directly desyncs `width()`/`height()` and that `set_viewport` is the cache-aware path.

  Renaming `svg.root` to `svg.as_element()` across every downstream user would churn the public API for no real gain in safety.

## Hiding raw `web_sys` access behind `raw_element_unchecked()` or a Cargo feature

An external review flagged three sites where the crate exposes raw `web_sys` elements:

* `SvgRoot::root` — a public field of type `SvgsvgElement`.
* `SvgNode::as_element()` — returns `&SvgElement`.
* `SvgDefs::as_element()` and `SvgMarker::as_element()` — likewise.

The argument was that callers can bypass cached state, marker-id validation, and listener ownership through these handles, and that raw DOM access should instead live behind an explicit escape API — `raw_element_unchecked()` — or a Cargo feature such as `features = ["raw-dom-access"]`.

The recommendation was evaluated against each site individually, because the facts differ.

### `SvgRoot::root`

This was already analysed [above](#hiding-svgrootroot-behind-an-as_element-accessor), where it was also suggested that the public field be hidden behind an `as_element()` accessor.
The conclusion there stands for any spelling of the accessor: every `web_sys` DOM-mutating method takes `&self`, not `&mut self`, so `svg.raw_element_unchecked().set_attribute("width", "500")` desyncs the cached viewport *exactly* as `svg.root.set_attribute("width", "500")` does.
The accessor name changes the ergonomics but does not close the invariant hole, because the hole is inherent to exposing the element at all — which we want to do.

### `SvgNode::as_element()`

`SvgNode` has **no cached state** that direct DOM access can desync.
Its managed state is the listener `Rc<SvgNodeInner>`, and a caller who adds a listener directly to the raw `SvgElement` does not corrupt that store — they merely bypass the crate's automatic listener cleanup for that one listener.
This is the unavoidable consequence of exposing any DOM handle, not a unique defect of `as_element()`.

More importantly, `SvgNode::set_attr()` is already a full escape hatch: it accepts an arbitrary attribute name and value and writes it verbatim to the DOM.
Any "bypassing" of attribute-writer consistency or text-safety helpers is equally possible through that route — the existing security note on `set_attr` already documents this.
Removing `as_element()` would only deny access to non-attribute DOM methods (`tag_name`, `preserveAspectRatio` reads, typed property accessors that need a `dyn_ref` cast, and so on — `get_bounding_client_rect` was an example of this category at the time of this rejection, and has since been wrapped as [`SvgNode::bounding_client_rect`](../geometry.md)) — which are the *legitimate* reasons to reach the raw element.

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

Gating `as_element()` behind `features = ["raw-dom-access"]` would impose a feature dependency on every caller who needs `computed_text_length`-style `dyn_ref` casts or any other non-attribute DOM method that the crate does not wrap — exactly the legitimate use cases the method exists for.
The `docs/gaps.md` list makes clear that this crate deliberately does not wrap large swaths of SVG DOM; a feature gate would make those gaps less accessible, not more documented.

### Conclusion

The naming concern is legitimate: `as_element()` does not signal that it bypasses crate invariants.
But renaming alone does not protect invariants, and the invariants that actually exist are either already documented (`SvgRoot` viewport, `SvgMarker` id) or do not exist at all (`SvgNode`, `SvgDefs`).

The correct and sufficient response is precise documentation at each escape hatch, which is already in place.
A breaking rename or a Cargo feature would impose a real cost on legitimate callers in exchange for a naming signal that cannot substitute for reading the docs.

## Canonicalising on one construction model for `defs`, `marker`, and `batch`

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

## Reducing attribute-mutation surface area to one canonical path

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

Raw `as_element()` access is already documented as an escape hatch (see [Hiding raw `web_sys` access behind `raw_element_unchecked()` or a Cargo feature](#hiding-raw-web_sys-access-behind-raw_element_unchecked-or-a-cargo-feature) above) and is excluded from the "normal" operating model.

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
| Layer 4 | Remove raw DOM from normal API | See [Hiding raw `web_sys` access behind `raw_element_unchecked()` or a Cargo feature](#hiding-raw-web_sys-access-behind-raw_element_unchecked-or-a-cargo-feature) above |

The recommendation names an architecture that already exists; it does not propose a change.
