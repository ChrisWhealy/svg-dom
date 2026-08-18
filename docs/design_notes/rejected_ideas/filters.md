# Filter primitives

[← Back to rejected ideas](README.md)

See [`<filter>` primitives return a plain `SvgNode`](../filters.md) for the design notes this rejection confirms.

## `build_gaussian_blur` / `build_offset` / `build_merge` closures for filter primitives

An external review suggested that each `SvgFilter` primitive method (`gaussian_blur`, `offset`, `merge`) could gain a `build_*` closure-based sibling.
This would mirror `SvgDefs::build_filter`'s detached-until-success pattern:

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

The review's own recommendation was **not** to add this.
Its reasoning had three parts: `build_filter` already keeps the whole filter detached for normal construction, dynamically extending an already-referenced filter is an uncommon path, and a `build_*` sibling would double the API surface per primitive.
That conclusion is correct.
This entry confirms it.

### `build_filter` already makes the *only* case that matters impossible to observe

`SvgDefs::build_filter` (`src/root/defs.rs`) creates the `<filter>` element.
It runs the caller's closure against that element.
Only after the closure returns `Ok` does it call `self.element.append_child(filter.as_element())`, attaching the filter to `<defs>`.
Every primitive method called inside that closure (`gaussian_blur`, `offset`, `merge`) therefore mutates an element that is not yet part of the document tree.
There is nothing for a renderer to recompute, no matter how many intermediate attribute writes happen.

`<defs>` content is never rendered, even once attached.
Nothing can reference the filter via `url(#id)` until the caller explicitly applies it, with `set_filter_ref`/`set_filter`.
That application necessarily happens after `build_filter` returns.
So for ordinary construction (build a filter, then apply it) the multi-mutation sequence the review is concerned about is not just unlikely to matter.
It is architecturally unobservable.

### The remaining case is narrower than "dynamic filter modification" in general, and mostly synchronous anyway

The one case where intermediate mutations could, in principle, be observed is narrow.
It requires calling a primitive method on a filter that is already attached to `<defs>` and already referenced by a live, rendered node.
For example: `existing_filter.gaussian_blur(4.0)?.set_attrs([...])?`, invoked some time after the filter was first built and applied.

Even then, the two-call chain is fully synchronous Rust/WASM.
This applies to both the review's example and every example in this crate's own docs.
`gaussian_blur` returns, then `set_attrs` runs, with no `await` or event-loop yield between them.

Browsers coalesce style recalculation and paint to happen after the current task finishes.
They do not run synchronously after each individual `setAttribute`/`appendChild` call.
A synchronous layout is only forced when the script reads a layout-dependent property, such as `getBBox` or `offsetWidth`, between the writes.
Nothing here does that.

So even in the narrow live-filter-mutation case, a chained call sequence like the one above produces exactly one paint.
That paint shows the final, fully-configured primitive, not a flash of intermediate states.
The scenario the review's own stated motivation describes needs something more.
The caller would have to hold the returned `SvgNode` across an actual event-loop boundary — storing it, then mutating it later from a different callback or after a `setTimeout`.
This is a real case, but considerably narrower than broad "dynamic filter modification."

### Cost is not zero

The review's own third point stands on its own, regardless of the above.
A `build_*` sibling for every primitive doubles the primitive API surface: `gaussian_blur` + `build_gaussian_blur`, `offset` + `build_offset`, `merge` + `build_merge`, and so on for every primitive added thereafter.
That cost applies for a benefit that only reaches an already-narrow set of cases.
This follows the same restraint already exercised for [`gaussian_blur_xy`](../../../src/root/filter/primitives/gaussian_blur.rs) and `merge`'s slice parameter.
See [`<filter>` primitives return a plain `SvgNode`](../filters.md) and its "confirms the plain-`SvgNode` decision" follow-up.
The rule: add the minimum shape a primitive actually needs, not the maximum shape it could conceivably use.

### Verdict

This review suggestion has not been implemented, for the review's own stated reasons.
This entry adds one more confirmation: the synchronous call pattern this crate actually uses does not show the intermediate-mutation problem the proposal was framed around.

Revisit this only if a genuinely asynchronous dynamic-filter-mutation workload is profiled in a real browser and shown to cause repeated filter-graph recomputation.
That is the same evidence-first bar this crate applies to every other performance-motivated addition.
