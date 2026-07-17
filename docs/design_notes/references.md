# Reference-attribute (`url(#id)`) caching

[← Back to design notes](README.md)

**Contents**

- [`_ref` setters skip revalidating an already-validated id](#_ref-setters-skip-revalidating-an-already-validated-id)
- [Reference handles cache the complete `url(#id)` string, not just the bare id](#reference-handles-cache-the-complete-urlid-string-not-just-the-bare-id)

## `_ref` setters skip revalidating an already-validated id

Every reference-attribute setter on `SvgNode` (`set_marker_start`/`_mid`/`_end`, `set_fill_gradient`/`set_stroke_gradient`, `set_fill_pattern`/`set_stroke_pattern`, `set_clip_path`, `set_filter`) takes a bare `&str` id, validates it, then writes it as `attr="url(#id)"`.
Each also has one or more handle-based `_ref` siblings (`set_marker_start_ref`, `set_fill_linear_gradient`, `set_fill_pattern_ref`, `set_clip_path_ref`, `set_filter_ref`, ...) that take the live element handle (`SvgMarker`, `SvgLinearGradient`, `SvgPattern`, `SvgClipPath`, `SvgFilter`) and forward its cached `id()` through the same bare-id setter — which re-validates a string that cannot possibly be invalid.

Every one of these handle types guarantees its cached id is already valid: `validate_*_id` runs once at construction (`SvgDefs::marker`/`clip_path`/`filter`/`pattern`/`linear_gradient`/`radial_gradient` and their `build_*` siblings) and again inside each handle's own `set_id`, and each handle's generic `set_attr`/`set_attr_display` explicitly reject `"id"` (`Error::ReservedAttribute`) so the normal API can never desynchronise the cache from the DOM.
The only bypass is writing through `as_element()` directly, already documented as an escape hatch that forfeits every crate-level guarantee.
So by the time a `_ref` method reads `handle.id()`, re-scanning it is a repeated check of a string that has already been proven to be correct.

This generalises the same principle that `create_path_from_defs` and `d_from_validated_defs` already apply to path data (see [Path data](path_data.md)): validate once at the untrusted boundary (a bare `&str` from arbitrary caller code), and let a path that started from an already-validated source skip straight to the write.

Concretely, every bare-id setter delegates to a private `SvgNode::set_url_ref(attr, id)` that formats `url(#id)` and writes the attribute, while every `_ref` setter writes its handle's own cached reference directly (see the next section for what that cached reference is and why `_ref` setters no longer go through `set_url_ref` at all).

One private helper shared across all nine reference-attribute pairs, rather than one per attribute kind, since the `url(#...)` wrapping and write are identical regardless of which attribute or id kind is involved — only the attribute name and id string differ, and both are already parameters.

The saving is one string scan per call — the same order of magnitude as the path-validation case above, and for the same reason, it is not worth optimising away on its own merits for a single call.
It compounds when one handle (a shared marker, a reusable gradient) is applied to many elements, which is a common pattern this crate's own demos use (e.g. arrowhead markers applied to several lines).

## Reference handles cache the complete `url(#id)` string, not just the bare id

The previous section removed the redundant *validation* a `_ref` setter performed on an already-valid id, but left a second, separate cost in place: `set_url_ref` still built a fresh `url(#id)` `String` via `format!` on every single call, so `node.set_filter_ref(&filter)` allocated once per call regardless of how many times the same filter was applied.

`SvgMarker`, `SvgPattern`, `SvgClipPath`, `SvgFilter`, and the shared `GradientInner` behind `SvgLinearGradient`/`SvgRadialGradient` now cache the *complete* `url(#id)` reference in a field named `url_ref`, built once in `new` and rebuilt in place by `set_id`, rather than caching the bare id and reformatting it on every reference.

`id()` (the existing public getter) is unaffected from the caller's side: it now slices the bare id back out of `url_ref` (`&url_ref[URL_PREFIX.len()..url_ref.len() - 1]`) instead of returning a separately stored field.
That slice is exact, not just probably-correct: `URL_PREFIX` (`"url(#"`, defined once in `root/defs.rs` and shared by all five types) and the trailing `)` are both pure ASCII, and every id these types accept is validated at construction and by `set_id` to match the pattern `[A-Za-z_][A-Za-z0-9_-]*`, which itself is also pure ASCII.
Therefore, byte offsets always land exactly on the id's boundaries, never mid-character.

A new `pub(crate) fn url_ref(&self) -> &str` exposes the cached string to the `_ref` setters in `node/attrs.rs`, which now write it straight to the attribute via `set_attr`, bypassing `set_url_ref` (and its `format!`) entirely; however, the bare-id setters (`set_fill_gradient`, `set_marker_start`, ...) still need `set_url_ref`, since they only ever receive a bare `&str` from arbitrary caller code and have no cached reference to reuse.

`SvgSymbol` deliberately keeps its plain bare-id cache: a `<symbol>` is referenced via a bare `#id` fragment on `<use>`, not a `url(#...)` wrapper, and has no `_ref`-style setter in `node/attrs.rs` to benefit from a pre-built reference.
The `url(#...)` shape genuinely does not apply there, so extending this change to it would just be a different cached string with no call site to use it.

The cost is six bytes of payload per cached handle (`url(#` and `)`) beyond the bare id; the field count is unchanged from the bare-id-caching version.
In return, `_ref` setters go from "validate nothing, but still allocate a fresh `String` every call" (the state after the previous section's change) to allocating nothing at all, which is the same shape as `create_path_from_defs`/`d_from_validated_defs` and the id-revalidation fix above, extended one step further: not just skip the redundant check, skip the redundant formatting work behind it too.

Construction itself briefly regressed, though: `new` on `SvgMarker`, `SvgPattern`, `SvgClipPath`, and `SvgFilter` kept the pre-`url_ref` signature `new(id: String, ...)`, a holdover from when `id` was stored directly rather than used to build a derived `url_ref` string.
Every call site therefore did `id.to_owned()` (allocation 1) to satisfy that signature, and `new` then built `url_ref` by copying out of that temporary, immediately-discarded `String` (allocation 2) — one avoidable allocation and copy per filter/marker/pattern/clip-path construction.
`GradientInner::new` was never affected, since it already took `id: &str` and built `url_ref` directly from the borrow, having no legacy stored-`id`-field signature to inherit.

`new` on all four now takes `id: &str` to match, and every `SvgDefs::*`/`build_*` call site drops its now-pointless `.to_owned()`.
