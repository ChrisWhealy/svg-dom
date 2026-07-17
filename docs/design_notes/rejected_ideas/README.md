# Ideas Considered and Rejected

[← Back to design notes](../README.md)

Design suggestions that were evaluated for `svg-dom` and deliberately not adopted.
The reasoning is preserved here — grouped by topic and split across files, mirroring how the [design notes](../README.md) themselves are organised — so the same ideas are not repeatedly re-proposed.

Several entries share a recurring theme referred to as **the measurement gate**: this crate declines a speculative
performance or binary-size change until a real measurement (a profile, a `wasm-opt`/MD5 comparison, a
`twiggy`/`wasm-tools` size report) shows an actual benefit, rather than accepting a plausible-sounding argument on
its own. See [Performance and binary-size proposals](performance.md) for the fullest articulation of this, with
cross-links from the other categories it has also been applied to.

## Node identity, ownership, and tree navigation

- [Node identity, ownership, and tree navigation](node_and_tree.md) — splitting `SvgNode` into passive/interactive types, a lighter-weight `parent_element()`, and restricting/removing `SvgNode::parent()` to avoid split listener state.

## Event and listener system

- [Event and listener system](events.md) — an `EventName` enum, an in-place `ListenerStore::push`, deferred listener drops for self-removal safety, flattening `EventClosure`, listener handles, feature-gating event families, and delegated event handling.

## Animation API

- [Animation API](animation.md) — making `start_with_frame` the canonical entry point, and an optional shared `requestAnimationFrame` scheduler.

## API surface and escape hatches

- [API surface and escape hatches](api_surface.md) — hiding `SvgRoot::root`/raw `web_sys` access behind accessors or a Cargo feature, canonicalising the `defs`/`marker`/`batch` construction models, and reducing attribute-mutation paths to one canonical route.

## Reference-attribute (`url(#id)`) design

- [Reference-attribute (`url(#id)`) design](references.md) — unifying marker references on handles and making marker ids immutable.

## Performance and binary-size proposals

- [Performance and binary-size proposals](performance.md) — a faster float-to-string crate, creation-time `path_fmt`/`text_fmt` helpers, handle-light factories for static scenes, a size-optimised release profile, typed cached-attribute wrappers, and reducing error-path formatting machinery.

## Geometry and viewport

- [Geometry and viewport](geometry.md) — a rendered-size (`getBoundingClientRect`) fallback for seeding `SvgRoot`'s cached viewport.

## Filter primitives

- [Filter primitives](filters.md) — `build_*` closure-based siblings for filter primitive methods.
