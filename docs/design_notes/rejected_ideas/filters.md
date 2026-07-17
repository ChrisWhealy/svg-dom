# Filter primitives

[← Back to rejected ideas](README.md)

See [`<filter>` primitives return a plain `SvgNode`](../filters.md) for the design notes this rejection confirms.

## `build_gaussian_blur` / `build_offset` / `build_merge` closures for filter primitives

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
This runs against the same restraint already exercised for [`gaussian_blur_xy`](../../../src/root/filter.rs) and `merge`'s slice parameter (see [`<filter>` primitives return a plain `SvgNode`](../filters.md), and its "confirms the plain-`SvgNode` decision" follow-up): add the minimum shape a primitive actually needs, not the maximum shape it could conceivably use.

### Verdict

This review suggestion has not been implemented, not only for the review's own stated reasons, but we have nere provided the additional confirmation that the synchronous call pattern this crate actually uses does not exhibit the intermediate-mutation problem the proposal was framed around.

Revisit only if a genuinely asynchronous dynamic-filter-mutation workload is profiled in a real browser and shown to cause repeated filter-graph recomputation — the same evidence-first bar this crate has applied to every other performance-motivated addition.
