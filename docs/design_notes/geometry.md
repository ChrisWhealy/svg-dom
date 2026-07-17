# Geometry read-back methods gate on the DOM interface, not the element type

[← Back to design notes](README.md)

**Contents**

- [`dyn_ref` gating, not a closed element-type enum](#dyn_ref-gating-not-a-closed-element-type-enum)
- [Three-way split between `Result`, `Option`, and a plain value](#three-way-split-between-result-option-and-a-plain-value)
- [`bounding_box()` wraps only the no-argument `getBBox` — the object/fill box, not everything painted](#bounding_box-wraps-only-the-no-argument-getbbox--the-objectfill-box-not-everything-painted)
- [`ctm`/`screen_ctm` reuse `Matrix2D`, not a new type](#ctmscreen_ctm-reuse-matrix2d-not-a-new-type)
- [`Rect` composes `Point` and `Size`, and its two producers are not interchangeable](#rect-composes-point-and-size-and-its-two-producers-are-not-interchangeable)
- [`ctm`/`screen_ctm` are accumulated matrices, not generally the element's own local transform](#ctmscreen_ctm-are-accumulated-matrices-not-generally-the-elements-own-local-transform)
  - [Converting a point between viewport and local coordinates](#converting-a-point-between-viewport-and-local-coordinates)
  - [Recovering the local matrix](#recovering-the-local-matrix)
- [`point_at_length`'s `distance` is saturated to `f32` range, not just cast](#point_at_lengths-distance-is-saturated-to-f32-range-not-just-cast)

Six new `SvgNode` methods in `src/node/geometry.rs` have been implemented: `bounding_box`, `ctm`, `screen_ctm`, `total_length`, `point_at_length`, `bounding_client_rect`.

## `dyn_ref` gating, not a closed element-type enum

`SvgGraphicsElement` (`getBBox`/`getCTM`/`getScreenCTM`) and `SvgGeometryElement` (`getTotalLength`/`getPointAtLength`) are runtime DOM interfaces, not something this crate tracks statically.
`SvgNode` is one type shared by every element this crate's factories produce, the same way `set_attr`/`attr` work on any element regardless of tag.
Confirmed by checking `web-sys`'s own `extends` declarations: every element type this crate hands back as a plain `SvgNode` (`rect`, `circle`, `ellipse`, `line`, `polyline`, `polygon`, `path`, `text`, `tspan`, `textPath`, `use`, `image`, `g`, and the root `svg`) implements `SVGGraphicsElement`, so the interface check on `bounding_box`/`ctm`/`screen_ctm` is a defensive safety net rather than something reachable through this crate's own API today.
`SVGGeometryElement`, in contrast, is implemented only by `rect`/`circle`/`ellipse`/`line`/`polyline`/`polygon`/`path` — `text`/`tspan`/`textPath`/`use`/`image`/`g`/`svg` genuinely lack it, so `total_length`/`point_at_length`'s "does not apply" branch is real and tested (calling either on a `<text>` or `<g>` node).

The gating itself follows the existing precedent set by `computed_text_length` (`src/node/text.rs`): `self.inner.element.dyn_ref::<web_sys::SvgGraphicsElement>()` returns `None` cleanly if the underlying element does not implement the interface, rather than the call panicking or throwing an uncaught JS exception.

## Three-way split between `Result`, `Option`, and a plain value

- `bounding_box()`/`point_at_length()` return `Result<_, Error>`: even once the interface check passes, the underlying browser call (`getBBox`/`getPointAtLength`) is itself declared fallible in the DOM Standard, and the "wrong interface" case folds into the same `Error::Dom` rather than a separate error shape.
- `ctm()`/`screen_ctm()`/`total_length()` return `Option<_>`: `getCTM`/`getScreenCTM` are nullable-but-not-throwing in the DOM Standard itself (`null` when not currently rendered), and `getTotalLength` is infallible once the interface check passes — `computed_text_length` already collapses an interface mismatch into a single `None` rather than a nested `Option<Result<_>>`, and these follow the same shape for consistency.
- `bounding_client_rect()` returns a plain `Rect`: `Element.getBoundingClientRect()` is infallible and universal on every `Element`, so there is nothing to wrap.

## `bounding_box()` wraps only the no-argument `getBBox` — the object/fill box, not everything painted

Called with no arguments, `getBBox()` returns the SVG specification's default `SVGBoundingBoxOptions`: `fill=true`, `stroke=false`, `markers=false`, `clipped=false` — the **object/fill bounding box**, geometry only.
A wide `stroke-width`, marker decorations (arrowheads and the like), and any `clip-path` applied to the element are not included, so `bounding_box()` can report a rect visibly smaller than everything the element actually paints.
This is not a bug in the wrapper; it is exactly what the no-argument DOM call returns, and it is why `bounding_box()`'s own doc comment calls this out explicitly rather than leaving "bounding box" to be read as "everything visibly painted."

`web-sys` 0.3.102 does expose the options-taking overload as `get_b_box_with_a_options` (gated behind the `SvgBoundingBoxOptions`/`SvgRect` features), so a `bounding_box_with_options`-style method is mechanically possible.
It was not added here: the options-taking form of `getBBox()` is a newer part of the SVG2 spec, and unlike the plain no-argument form (long-supported, exercised throughout this crate's own test suite), its cross-browser support was not verified as reliable across the browser/toolchain range this crate targets.
Wrapping a DOM method whose actual runtime behaviour is uncertain outside of Chromium would trade a well-understood gap (documented, no-argument only) for a worse one (an API that appears to let a caller opt into the stroked/decorated box, but might silently ignore the options argument, or behave inconsistently, depending on the browser).
If and when that support picture is confirmed reliable, the overload can be added the same way the no-argument form was: a thin wrapper, `dyn_ref`-gated the same as the other five methods.

Note also that `getBoundingClientRect()` (`bounding_client_rect()`) is not a reliable substitute for "the stroked/painted extent" either — checked empirically (Chromium/Playwright) against a `<rect>` with `stroke-width="20"`, `getBoundingClientRect()` reported the exact same fill-only extent as `getBBox()`, not a stroke-widened one.
`Rect`'s own doc comment flags this rather than letting a reader assume `bounding_client_rect()` is the "everything painted" alternative to `bounding_box()`'s fill-only box.

## `ctm`/`screen_ctm` reuse `Matrix2D`, not a new type

`SVGMatrix`'s `a()`/`b()`/`c()`/`d()`/`e()`/`f()` getters map onto exactly the role layout `Matrix2D` already uses for `set_matrix`/`set_matrix_precise` (`a`→`h_scale`, `b`→`v_skew`, `c`→`h_skew`, `d`→`v_scale`, `e`→`h_trans`, `f`→`v_trans`) — see [Transforms](transforms.md).
Reusing the existing struct rather than introducing a second matrix type avoids a second, parallel matrix representation for what is structurally the same 2D affine transform — see below for what this does, and does not, buy a caller wanting to write a matrix back with `set_matrix`.

## `Rect` composes `Point` and `Size`, and its two producers are not interchangeable

`Rect { origin: Point, size: Size }` reuses the crate's existing coordinate types rather than duplicating four `f64` fields — the same reasoning that keeps `Matrix2D` a single shared type instead of one-off structs per caller.

`bounding_box()` (local, user-space, `getBBox()`) and `bounding_client_rect()` (rendered CSS pixels relative to the viewport, `getBoundingClientRect()`) both return a `Rect`, but the two coordinate spaces are not interchangeable — they differ whenever any transform, `viewBox`, or CSS scaling is in play.
This is the same mistake `docs/rejected_ideas.md` ("Provide a rendered-size fallback...") already documents from the other direction: an earlier proposal to seed the cached viewport from `getBoundingClientRect()` was rejected specifically because doing so would silently compare CSS pixels against attribute user-units.
`Rect`'s own doc comment states the distinction explicitly, rather than leaving it to be discovered the same way twice.

## `ctm`/`screen_ctm` are accumulated matrices, not generally the element's own local transform

An earlier draft of this note (and of `docs/elements.md`) claimed a matrix read via `ctm()` could be "mutated and written straight back" with `set_matrix`.
That is wrong in general, and worth recording as a corrected claim rather than silently rewriting it (this document's convention for corrections — kept even across this reorganisation into topic files).

Per the SVG specification, `getCTM()` returns the matrix mapping the element's own coordinate system into its **nearest SVG viewport's** coordinate system.
That is the accumulation of the element's own `transform` **and** every intervening ancestor's `transform`, up to (but not through) the nearest viewport ancestor — for ordinary content with no nested `<svg>`/`<symbol>` boundary, that viewport is the root `<svg>` itself, so `ctm()` on a nested shape already reflects the *combined* chain of every intermediate `<g>`'s transform, not just the shape's own.
`screen_ctm()` continues past that point into the **document viewport's CSS-pixel coordinates** — despite the DOM method's name (`getScreenCTM`), this is not physical monitor/screen coordinates; it additionally carries the root `<svg>`'s own position on the page (normal document flow, any CSS transform on an HTML ancestor, and so on).

The browser test `should_accumulate_ancestor_transforms_in_ctm_up_to_the_root_viewport` (`tests/svg_node.rs`) demonstrates exactly why the original claim was wrong: a `<g>` translated `(100, 0)` contains a `<rect>` translated `(0, 50)`.
The rect's own local transform is `translate(0, 50)`, but `rect.ctm()` reports `(100, 50)` — the parent's translation is already folded in.
Writing that `ctm()` reading straight back as the rect's own `transform` via `set_matrix` would leave the parent's translation in place *and* add the already-accumulated `(100, 50)` again, producing an effective translation of `(200, 50)`, not the intended `(0, 50)`.

**Direct write-back of a `ctm()`/`screen_ctm()` reading is therefore only correct when the parent-to-viewport transform is the identity matrix** — informally, when the element being measured has no relevantly-transformed ancestor between it and its nearest viewport.
`screen_ctm()` additionally requires the page position itself to contribute nothing (rarely true), so it is very rarely safe to write back directly at all.

This leaves two genuinely different operations, easy to conflate because both start from a `ctm()`/`screen_ctm()` reading — each is covered in its own subsection below:

- **Converting a point** between document-viewport coordinates and this element's own local coordinates.
- **Recovering this element's own writable local `transform`** matrix.

### Converting a point between viewport and local coordinates

This uses only **this element's own** `screen_ctm()` — a parent's matrix is not involved at all, because `screen_ctm()` already maps this element's local coordinate system straight to the document viewport in one step:

```text
viewport_point = element.screen_ctm() · local_point
local_point     = inverse(element.screen_ctm()) · viewport_point
```

An earlier revision of this note conflated this with the *different* local-transform-recovery operation below, and incorrectly suggested inverting the *parent's* `screen_ctm()` to do it.
Inverting the parent's matrix only gets a point as far as the parent's own coordinate system — it is the tool the next subsection uses to recover a local *transform*, not the tool for converting an arbitrary *point*.
Converting a point needs only the element's own `screen_ctm()`, inverted, applied directly; nothing about the parent enters into it.

### Recovering the local matrix

Using this crate's own `matrix(a, b, c, d, e, f)` convention (documented on `Matrix2D` itself, see [Transforms](transforms.md)) — a point is transformed as `p' = M · p` in homogeneous column-vector form:

```text
| h_scale  h_skew   h_trans |
| v_skew   v_scale  v_trans |
| 0        0        1       |
```

For an element and its immediate parent that share the same nearest viewport ancestor (i.e. no `<svg>`/`<symbol>` boundary between them), composition gives `child.ctm() = parent.ctm() · child.local()`.
Solving for the child's own local matrix:

```text
child.local() = inverse(parent.ctm()) · child.ctm()
```

For a general 2D affine `Matrix2D { h_scale: a, v_skew: b, h_skew: c, v_scale: d, h_trans: e, v_trans: f }`, with `det = a·d - b·c`:

```text
inverse.h_scale = d / det          inverse.h_skew  = -c / det
inverse.v_skew  = -b / det         inverse.v_scale =  a / det
inverse.h_trans = (c·f - d·e) / det
inverse.v_trans = (b·e - a·f) / det
```

and composing two matrices `P · C` (`P` applied after `C`):

```text
result.h_scale = P.h_scale·C.h_scale + P.h_skew·C.v_skew
result.v_skew  = P.v_skew·C.h_scale  + P.v_scale·C.v_skew
result.h_skew  = P.h_scale·C.h_skew  + P.h_skew·C.v_scale
result.v_scale = P.v_skew·C.h_skew   + P.v_scale·C.v_scale
result.h_trans = P.h_scale·C.h_trans + P.h_skew·C.v_trans + P.h_trans
result.v_trans = P.v_skew·C.h_trans  + P.v_scale·C.v_trans + P.v_trans
```

Checked against the test above: `parent.ctm()` is a pure `translate(100, 0)`, whose inverse is `translate(-100, 0)`.
Composing that inverse with `child.ctm() = translate(100, 50)` gives `translate(0, 50)` — exactly the rect's actual local `set_translate(0, 50)`, confirming the formula.

This crate deliberately does not ship an `inverse`/`compose` method on `Matrix2D` — `Matrix2D` remains a plain data struct with no matrix-composition API of its own (the same scope boundary already noted above), so a caller who needs this implements it from the formula above rather than this crate silently growing a small linear-algebra library.

Both `set_matrix` and `set_matrix_precise`'s doc comments carry a short pointer to this note.

## `point_at_length`'s `distance` is saturated to `f32` range, not just cast

`SVGGeometryElement.getPointAtLength()` takes an IDL `float` (32-bit), so `point_at_length(&self, distance: f64)` has to narrow its argument before crossing into the browser.
The first implementation did this with a plain `distance as f32`, and the doc comment claimed an out-of-range `distance` never errors — the browser clamps it to the path's start or end.
That claim was correct for ordinary finite values, but not for the entire `f64` domain, and the gap was only found by checking the exact browser behaviour rather than assuming the DOM Standard's clamping rule was the whole story.

Rust's documented behaviour for an overflowing float-to-float `as` cast is to saturate to infinity: `f64::MAX as f32` is `f32::INFINITY`, not a large finite `f32`.
Passing that to the browser does not clamp — `getPointAtLength`'s IDL parameter is a *restricted* `float`, not `unrestricted float`, so the browser's own WebIDL binding rejects `NaN`/`±Infinity` before the SVG path-clamping algorithm ever runs.
Confirmed empirically (Chromium): `rect.getPointAtLength(Infinity)` throws `TypeError: ... The provided float value is non-finite`, while `rect.getPointAtLength(1e30)` — large, but still finite and well within `f32`'s range — clamps to the path start exactly as the original doc comment described.
So `f64::MAX`, `f64::MIN`, and any other overflowing-but-finite `f64` all silently became a `TypeError` (mapped to `Err` by this crate's own `dom_err`), directly contradicting the "never errors for an out-of-range distance" claim for that slice of the input domain.

Two fixes were available: document the narrower true behaviour (finite, `f32`-representable distances clamp; anything that would overflow to infinity errors instead), or preserve the originally-documented behaviour across the full `f64` domain by saturating rather than letting the cast overflow.
The second was chosen: it keeps the public `f64` signature behaving intuitively (any finite distance clamps, exactly as documented) without adding a second browser boundary crossing (an alternative fix — clamping in Rust against a `total_length()` reading before ever calling `get_point_at_length` — would have doubled the number of calls into the browser for every use, including the overwhelmingly common in-range case that never needed clamping at all).

The fix has two parts, in this order:

1. Reject genuinely non-finite `distance` (`NaN`, `+Infinity`, `-Infinity`) explicitly, with a clear `Error::Dom` before any browser call is made — these were never meaningful path measurements to begin with, so turning them into a clean, immediate `Err` is more useful than letting them surface as an opaque browser `TypeError` (or, worse, a different opaque error depending on which browser is running).
2. For everything else — every finite `f64` — compare against `f32::MAX`/`f32::MIN` (widened to `f64` for an exact, lossless comparison) and saturate to the nearest `f32` boundary rather than casting directly. A saturated `f32::MAX`/`f32::MIN` is still a perfectly ordinary finite `f32` from the browser's point of view, so it clamps to the path's start/end exactly like `1e30` does above — no special-casing needed on the browser side, and no behavioural difference from an "ordinary" large out-of-range distance a caller might pass by hand.

Three new browser tests (`tests/svg_node.rs`) lock this in: `should_reject_non_finite_point_at_length_distance` (`NaN`/both infinities all return `Err`), and `should_saturate_out_of_f32_range_finite_distance_instead_of_erroring` (`f64::MAX` and `f64::MIN` both return `Ok`, matching the same point as calling with `total_length()`/a large negative number would).
