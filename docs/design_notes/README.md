# Design notes

This directory records the *why* behind non-obvious implementation choices in `svg-dom` — not what the code does.
The functionality is described in the doc comments.
Instead, what is documented here is the rationale for why it does it this way, what alternatives were considered, and what was measured rather than assumed.

Where a corrected claim needed recording, it is kept as a correction rather than silently rewritten — see the note under [Geometry read-back](geometry.md#ctmscreen_ctm-are-accumulated-matrices-not-generally-the-elements-own-local-transform) for an example of that convention.

## Core node model

- [Node identity, ownership, and tree navigation](node_and_tree.md) — `SvgNode`'s reference-counted handle, event-listener lifetime and ownership, the shared element factory behind `SvgRoot`/`SvgBatch`, and why `parent`/`children`/`query_selector*` reuse one independent-handle pattern rather than introducing a second handle type.
- [`requestAnimationFrame` self-rescheduling pattern](animation.md) — how `AnimationLoop` manages its own dispatch lifecycle safely, including the two rare failure paths that can leak captured state.

## Performance patterns

- [Performance patterns](performance.md) — reusable scratch buffers for per-frame formatting and transforms, redundant-write avoidance (`set_attr_if_changed`, `CachedAttr`), multi-attribute updates, and high-frequency event coalescing (`pointermove`/`touchmove`/`wheel`).
- [Transform API design](transforms.md) — why `set_matrix` takes a role-named `Matrix2D` struct rather than positional arguments or `a`/`b`/`c`/`d`/`e`/`f`, and why `set_matrix_precise` exists alongside it.

## Attribute & reference helpers

- [Reference-attribute (`url(#id)`) caching](references.md) — why `_ref` setters (`set_filter_ref`, `set_clip_path_ref`, ...) skip revalidating an id that is already known to be valid, and why the referenced handles cache the complete `url(#id)` string rather than the bare id.
- [Small `SvgNode`/`SvgRoot` attribute-helper decisions](svg_node_api.md) — `SvgRoot::set_view_box` and its shared validator, and why `classList` helpers live on `SvgNode` once rather than being duplicated per element type.

## Path data

- [Typesafe Path Data Builder](path_data.md) — the two-enum `PathDef` design and its measured layout cost, allocation-tier strategy, whitespace-elision in `d` strings, and precisely what "prevents malformed path data" does and does not guarantee.

## Filters

- [`<filter>` primitives return a plain `SvgNode`](filters.md) — why filter-primitive builder methods (`gaussian_blur`, `merge`, `flood`, `composite`, `drop_shadow`, `color_matrix`, ...) hand back a plain `SvgNode` instead of a typed wrapper per primitive, and the design of each primitive added so far.

## Geometry read-back

- [Geometry read-back methods gate on the DOM interface, not the element type](geometry.md) — `bounding_box`, `ctm`/`screen_ctm`, `total_length`/`point_at_length`, `bounding_client_rect`: the `dyn_ref` interface-gating pattern, the `Result`/`Option`/plain-value split, `bounding_box`'s object/fill-only scope, and the coordinate-space and write-back caveats around `ctm`/`screen_ctm`.

## Ideas considered and rejected

- [Ideas Considered and Rejected](rejected_ideas/README.md) — recommendations that were evaluated and *not* adopted, with the reasoning for each, grouped by topic the same way as this document.
