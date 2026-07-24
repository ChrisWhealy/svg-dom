# `<filter>` primitives return a plain `SvgNode`

[← Back to design notes](README.md)

**Contents**

- [`feOffset` and `feMerge` confirm the plain-`SvgNode` decision, rather than forcing a redesign](#feoffset-and-femerge-confirm-the-plain-svgnode-decision-rather-than-forcing-a-redesign)
- [`gaussian_blur_xy` shares a private `fmt::Arguments` core with `gaussian_blur`, rather than duplicating it](#gaussian_blur_xy-shares-a-private-fmtarguments-core-with-gaussian_blur-rather-than-duplicating-it)
- [`flood` and `composite` complete a *true* tinted drop shadow, and `CompositeOperator` follows the existing typed-enum precedent](#flood-and-composite-complete-a-true-tinted-drop-shadow-and-compositeoperator-follows-the-existing-typed-enum-precedent)
- [`drop_shadow` takes five positional parameters, because these value must be supplied to the underlying SVG primitive](#drop_shadow-takes-five-positional-parameters-because-these-value-must-be-supplied-to-the-underlying-svg-primitive)
- [`color_matrix` uses a data-carrying enum](#color_matrix-uses-a-data-carrying-enum)
- [`turbulence`/`turbulence_xy` have no `in`, and `displacement_map` reuses `Channel` rather than a new enum](#turbulenceturbulence_xy-have-no-in-and-displacement_map-reuses-channel-rather-than-a-new-enum)
- [`morphology`/`morphology_xy` are the third `fmt::Arguments`-core primitive pair, and `MorphologyOperator` orders `Erode` first to match the SVG default](#morphologymorphology_xy-are-the-third-fmtarguments-core-primitive-pair-and-morphologyoperator-orders-erode-first-to-match-the-svg-default)
- [`image` takes `href` positionally and adds no `PreserveAspectRatio` type, both by analogy with `SvgRoot::image`](#image-takes-href-positionally-and-adds-no-preserveaspectratio-type-both-by-analogy-with-svgrootimage)
- [Filter region and coordinate-space attributes get named setters, `FilterUnits` reuses the `PatternUnits` shape](#filter-region-and-coordinate-space-attributes-get-named-setters-filterunits-reuses-the-patternunits-shape)
- [`tile` is the first primitive with zero parameters](#tile-is-the-first-primitive-with-zero-parameters)
- [`convolve_matrix` and `convolve_matrix_xy` take a plain `&[f64]` kernel and document, rather than validate, a length mismatch](#convolve_matrix-and-convolve_matrix_xy-take-a-plain-f64-kernel-and-document-rather-than-validate-a-length-mismatch)

`SvgFilter` (`src/root/filter/`) is structurally identical to `SvgClipPath` and `SvgPattern`: that is, it is an id-cached container obtained from `SvgDefs::filter`/`build_filter`, applied to any element via `SvgNode::set_filter_ref`/`set_filter`, with the usual `set_attr`/`set_attrs`/`set_attr_display` escape hatch for attributes not yet wrapped by a named setter.
That much follows established precedent directly; the one new decision is what a filter-primitive *builder method* — `gaussian_blur`, and whatever `fe*` methods follow it — should hand back.

The SVG filter primitives are a large, mostly-orthogonal vocabulary: around fifteen elements (`feGaussianBlur`, `feOffset`, `feColorMatrix`, `feComposite`, `feMerge`/`feMergeNode`, `feFlood`, `feBlend`, and others), each with its own attribute grammar, but sharing two attributes across nearly all of them — `in` (identifies the upstream input or named result to be read) and `result` (the name under which this primitive's output is published, and which a later primitive's `in`/`in2` can reference).

Two designs were available for the return type of a method like `gaussian_blur`:

1. A typed wrapper per primitive (`FeGaussianBlur`, mirroring `SvgClipPath`'s own typed methods), or a `FilterPrimitive` enum in the `PathDef` style, with `in`/`result` as named fields or setters.
2. A plain `SvgNode` — the same handle already returned by every ordinary shape factory (`create_rect`, `create_circle`, ...) — relying on the existing generic `SvgNode::set_attr` for `in`, `result`, and any primitive-specific attribute not yet promoted to a named parameter.

Option 2 was chosen for this first primitive.

Unlike `PathDef` (see [Path data](path_data.md)), which models a single, closed, well-understood grammar (SVG path data) that benefits from exhaustive compile-time coverage, the filter primitive vocabulary is still only one primitive deep in this crate; committing to a typed wrapper (or a `PathDef`-style enum) per primitive now would mean guessing at a shape for fourteen more elements this crate does not yet implement, several of which (`feMerge`'s ordered `feMergeNode` children, `feComponentTransfer`'s per-channel `feFunc*` children) have structure closer to `SvgClipPath`'s child-shape factories than to a flat attribute bag.

It costs nothing to add primitives around a plain `SvgNode` — `gaussian_blur` is a thin `create_svg_element` + attribute write + `append_child`, the same shape as `GradientInner::add_stop` — and does not pre-commit the crate to an API surface for primitives not yet built.

This decision will be revisited once several more primitives exist and a genuine cross-primitive pattern (such as a shared `in`/`result` typed setter, or a `feMerge`-shaped child-list builder) becomes visible from real usage rather than anticipated in advance.

## `feOffset` and `feMerge` confirm the plain-`SvgNode` decision, rather than forcing a redesign

`offset` was a second flat-attribute primitive (`dx`, `dy`), no surprises — the same shape as `gaussian_blur`.

`merge` was the first real test: `<feMerge>` has ordered `<feMergeNode>` children rather than a flat attribute bag, exactly the case flagged above as a possible reason to introduce a typed, child-list builder.

In practice it does not need one.

`merge(&["offset-blur", "SourceGraphic"])` takes the list of `in` values as a plain `&[&str]` parameter and builds the `<feMergeNode>` children internally in one pass, still handing back a plain `SvgNode` for the outer `<feMerge>` (which has nothing but `result` left to set).
There was no ordering, mutation, or per-node configuration requirement that a closure-based builder (in the `SvgClipPath`/`build_clip_path` style) would have served better.
Each `feMergeNode` is only ever an `in` value, so a slice is already the natural shape for "an ordered list of input names."

The general shape of the decision therefore still stands after three primitives: reach for a closure/child-builder API only when a primitive's children need more than one attribute each, or when they must be added incrementally rather than known upfront — neither of which has come up yet.

## `gaussian_blur_xy` shares a private `fmt::Arguments` core with `gaussian_blur`, rather than duplicating it

`stdDeviation` is one of several SVG attributes with a `<number-optional-number>` grammar: one number for an isotropic value, or two space-separated numbers (`"x y"`) for independent horizontal/vertical (anisotropic) values.

`gaussian_blur` only ever wrote the one-number form, so a caller wanting the two-number form had no direct route to it — the closest workaround was calling `gaussian_blur` (one `stdDeviation` write), then overwriting the same attribute on the returned `SvgNode` with a hand-formatted `"x y"` string (a second write, and ordinarily a `format!`-allocated `String` to supply it).

`gaussian_blur_xy` closes that gap as a second public constructor for the same `<feGaussianBlur>` element, not a new primitive.
Both public methods delegate to a private `gaussian_blur_args(&self, std_deviation: fmt::Arguments<'_>)` that does the actual element creation, single attribute write, and append; `gaussian_blur` calls it with `format_args!("{std_deviation}")` and `gaussian_blur_xy` with `format_args!("{x} {y}")`.
Passing `fmt::Arguments` rather than a `&str` means neither caller needs to pre-format a `String`: `Arguments` implements `Display`, so it flows straight through `SvgAttrs::display_element`'s existing `write!(scratch, "{value}")` into the retained scratch buffer — the same technique `SvgPattern::set_view_box` and `SvgSymbol::set_view_box` already use to combine several numbers into one attribute (see [Performance patterns](performance.md), "Reusable attribute formatting").

This is a second data point (after `merge`'s slice-of-`&str` parameter) that a filter primitive needing a slightly richer call shape than "one flat attribute, one method" does not need a bigger abstraction — a second thin public method sharing a private core is enough as long as the underlying element is still just attributes, no child structure.

## `flood` and `composite` complete a *true* tinted drop shadow, and `CompositeOperator` follows the existing typed-enum precedent

`gaussian_blur` + `offset` + `merge` alone can only produce a shadow that is a blurred *copy* of the source graphic's own fill.
However, using these tools alone cannot give a shadow an independent colour or opacity without hand-writing the `feFlood` and `feComposite` effects through the generic escape hatch on some other primitive's returned `SvgNode` (which does not exist, since `<feFlood>` and `<feComposite>` are not children of another primitive).

This gap is now closed with the implementation of `flood` and `composite`, and together with the above three effects are enough for the textbook drop-shadow recipe: blur `SourceAlpha`, flood a colour, composite it `In` the blurred mask, offset, then merge underneath the original.

`flood(color, opacity)` takes both parameters positionally, unlike `gaussian_blur`'s single `std_deviation`, both `flood-color` and `flood-opacity` are core to what a flood *is* for the shadow-tinting use case (an untinted, fully-opaque flood is rarely useful on its own), so neither belongs behind the generic `set_attr` escape hatch the way `in` and `result` do for every primitive.

`composite(in2, operator)` takes `in2` positionally for the same reason `offset` takes `dx`/`dy` positionally: every meaningful use of `feComposite` supplies this value, not an optional cross-primitive linking attribute like `in`/`result`.

`operator` is a `CompositeOperator` enum (`Over`/`In`/`Out`/`Atop`/`Xor`/`Lighter`/`Arithmetic`) rather than a bare `&str`, matching the crate's existing convention for closed sets of SVG keyword attributes (`ClipPathUnits`, `PatternUnits`, `ArcSize`/`ArcSweep`, ...).
A typo in a bare string silently produces an unrecognised operator the browser ignores, while a typo in an enum variant is a compile error.

`Arithmetic`'s `k1`–`k4` coefficients are deliberately left to the generic escape hatch rather than added as further parameters: they only apply to one of the seven operators, and every other operator this crate exposes needs no extra configuration beyond `in2` and the operator keyword itself — the same "cover what's common, defer what's rare" judgement already applied to `gaussian_blur_xy` and `merge`.

## `drop_shadow` takes five positional parameters, because these value must be supplied to the underlying SVG primitive

`<feDropShadow>` is not a new effect; the SVG specification defines it as a browser-native shorthand for exactly this effect sequence:

   `gaussian_blur` → `flood` → `composite` → `offset` → `merge`

This is the chain the previous section describes, collapsed into one element that the browser expands internally.

`drop_shadow(std_deviation, dx, dy, color, opacity)` mirrors that definition directly: `std_deviation` is the same value `gaussian_blur` takes, `dx`/`dy` the same as `offset`, `color`/`opacity` the same as `flood`.

Five positional parameters is more than any other primitive method in this crate, but it is not a departure from the "cover what's common, defer what's rare" rule the other primitives already follow; instead,it is that rule applied to a primitive whose entire point is to bundle five otherwise-separate attributes into a single call.

Every one of these five values is something the caller of `drop_shadow`  specifically needs to control; there is no smaller "common case" subset to expose while deferring the rest, the way `composite` defers `Arithmetic`'s rarely-used `k1`–`k4`.
`in`/`result` remain on the generic escape hatch, as for every other primitive, since they are optional cross-primitive linking attributes rather than part of what a drop shadow actually *is*.

The one behaviour that you should be aware of (noted in `drop_shadow`'s own doc comment, not just here) is that `<feDropShadow>`'s result already includes its `in` input composited on top, exactly as the manual chain's final `merge` step does.
Therefore, a `<filter>` containing only `drop_shadow(...)` is already a complete, ready-to-use effect.

Calling `merge` again after `drop_shadow` is unnecessary as it would paint a second copy of the original graphic on top.

## `color_matrix` uses a data-carrying enum

This is the first filter primitive whose "type" changes the shape of another attribute.

`<feColorMatrix>` is unlike every primitive before it: its `type` attribute (`matrix` / `saturate` / `hueRotate` / `luminanceToAlpha`) does not just select a keyword, it determines what the SVG `values` attribute is allowed to contain — twenty numbers for `matrix`, one number for `saturate`/`hueRotate`, and nothing at all for `luminanceToAlpha`.

A fieldless enum like `CompositeOperator`, matched only to pick an `as_str()`, cannot express that: it would leave `values` as a separate loose parameter a caller could mismatch against `type` (a `values: [f64; 20]` supplied alongside `type: "saturate"`, or a lone `f64` alongside `type: "matrix"`), the exact class of error this crate's typed setters exist to rule out at compile time.

`ColorMatrixType` instead carries each type's own payload directly in the matching variant: `Matrix([f64; 20])`, `Saturate(f64)`, `HueRotate(f64)`, `LuminanceToAlpha` (no payload).

`color_matrix(matrix_type: ColorMatrixType)` matches on the variant to write both `type` (via `as_str()`) and `values` (or no `values` attribute at all for `LuminanceToAlpha`) together, so there is exactly one way to call it for each type and no way to supply a `values` shape that does not match the `type` you asked for.

`Matrix` is a fixed-size `[f64; 20]` rather than `Vec<f64>` or `&[f64]`, for the same reason `PathDef` prefers typed variants over a free-form string: the SVG grammar for this `values` form is exactly 20 numbers, so a matrix with the wrong element count cannot be constructed in the first place.
This avoids the possibility of a failure at the DOM boundary or worse, being silently truncated or padded by whatever formatting code assembled the string.

`ColorMatrixType` deliberately does not derive `Copy`, unlike every other filter-related enum in this crate (`CompositeOperator`, `ClipPathUnits`, `ArcSize`, ...).
Technically, however, it could since `[f64; 20]` is `Copy` — but doing so would make an easy-to-miss 160-byte copy happen implicitly at every call site that only meant to move or borrow the value, silently working against the same allocation-and-copy-consciousness the rest of this crate explicitly tries to avoid.

`Clone` is still derived for the rare case a caller genuinely wants to reuse one matrix definition (e.g. applying the same custom transform to several filters), but reuse now has to be spelled `matrix_type.clone()`, an explicit, visible cost rather than a free implicit one.

Writing the 20-number `values` string still avoids a heap allocation: `format_args!` with a literal 20-placeholder format string (verbose in the source, but optimised by the compiler) flows through `SvgAttrs::display_element`'s existing scratch buffer exactly as `gaussian_blur_xy`'s two-placeholder case already does (see "`gaussian_blur_xy` shares a private `fmt::Arguments` core" above).

A shared "write N space-separated numbers" helper (mirroring `write_points`'s technique for a runtime-length list) was not worth building for this: `feColorMatrix` is the only primitive in the entire SVG filter specification with a fixed 20-number attribute, so there is no second call site to justify factoring the loop out.

See [`docs/gaps.md`](../gaps.md) for the primitives still to be added.

## `turbulence`/`turbulence_xy` have no `in`, and `displacement_map` reuses `Channel` rather than a new enum

`<feTurbulence>` is the first primitive in this crate with no `in` attribute at all — it is a generator, not a filter over an existing input, so the "`in` defers to the generic escape hatch" convention every prior primitive follows has nothing to defer here: there is no `in` to set.
This is noted explicitly in `turbulence`'s own doc comment (not just here), since every other primitive's doc comment mentions `in` in the same breath as `result`, and a caller skimming past would otherwise reasonably expect the same here.

`turbulence`/`turbulence_xy` follow the same split as `gaussian_blur`/`gaussian_blur_xy`: a private `turbulence_args(&self, base_frequency: fmt::Arguments<'_>, ...)` core shared by both public methods, since `baseFrequency` is another `<number-optional-number>` attribute (see "`gaussian_blur_xy` shares a private `fmt::Arguments` core" above).

`num_octaves` is a `u32` rather than a signed integer: the SVG Filter Effects spec classifies a negative value as unsupported (not as an error — the spec's terminology distinguishes the two), so `u32` rules out at compile time the possibility of supplying such values instead of needing a runtime check — the same "make the invalid state unrepresentable where it's cheap to do so" judgement `ColorMatrixType::Matrix`'s fixed-size array already makes for a different attribute.

`displacement_map`'s `xChannelSelector`/`yChannelSelector` select one of the same four channels (`R`/`G`/`B`/`A`) `Channel` already names for `component_transfer`'s `<feFuncX>` children — a second, unrelated SVG attribute pair that happens to draw from the identical four-value vocabulary.
Rather than add a second, word-for-word-duplicate enum, `Channel` gained a second method, `selector_str()` (returning `"R"`/`"G"`/`"B"`/`"A"`, distinct from `tag()`'s `"feFuncR"`/`"feFuncG"`/`"feFuncB"`/`"feFuncA"`), and `displacement_map` takes two plain `Channel` parameters.
This mirrors the `FilterUnits` decision below (one enum shared by `filterUnits`/`primitiveUnits` rather than two identical enums) more than it resembles anything new: reuse an existing closed vocabulary wherever a new attribute draws from the same one, rather than mechanically minting a new type per attribute name.

## `morphology`/`morphology_xy` are the third `fmt::Arguments`-core primitive pair, and `MorphologyOperator` orders `Erode` first to match the SVG default

`<feMorphology>`'s `radius` is a third `<number-optional-number>` attribute (after `stdDeviation` and `baseFrequency`), so `morphology`/`morphology_xy` follow the identical private-core split already used twice: `morphology_args(&self, radius: fmt::Arguments<'_>, operator: MorphologyOperator)` does the actual element creation and attribute writes, with `morphology` calling it via `format_args!("{radius}")` and `morphology_xy` via `format_args!("{radius_x} {radius_y}")` (see "`gaussian_blur_xy` shares a private `fmt::Arguments` core" above).

The shared *code* shape does not mean shared *semantics*, though, and an early draft of `morphology_xy`'s doc comment wrongly assumed it did: it claimed `morphology_xy(3.0, 0.0, Dilate)` performs a horizontal-only dilation, by analogy with `gaussian_blur_xy(0.0, 6.0)`'s genuinely-supported one-axis blur.

The SVG spec defines `feMorphology` differently from `feGaussianBlur` — a `radius` component that is zero (or negative) disables the *entire* primitive, not just that axis, so the example above was actually documenting a no-op.

This is now corrected in `morphology_xy`'s own doc comment (with a `⚠️` specifically contrasting it against `gaussian_blur_xy`'s different behaviour) and in the guide; the lesson generalises past this one primitive: two primitives sharing an implementation pattern for encoding a `<number-optional-number>` attribute does not guarantee they share the same interpretation of a zero component, and each still needs checking against its own section of the specification rather than assumed by analogy.

Unlike `CompositeOperator` (`Over` first) and `BlendMode` (`Normal` first), `MorphologyOperator` is a plain two-variant enum with no data — the same shape as `FilterUnits`.
`Erode` is listed first because it is the SVG default for `operator` (mirroring the crate-wide convention of ordering a keyword enum's variants with the spec default first, already followed by every other filter enum here), not because it is expected to be reached for more often than `Dilate`; the two are equally common in practice, growing and shrinking a silhouette being symmetric operations.

**Correction:** an earlier draft of this section, and of `morphology`'s own doc comment, deliberately hedged `radius`'s negative-value behaviour ("may well produce an unspecified rendering result") rather than asserting a specific spec classification, since the exact wording the Filter Effects specification uses for a negative `feMorphology` `radius` had not been confirmed at the time — the same caution that led to the `numOctaves` correction above.

The specification does in fact define it, and plainly: "a negative or zero value disables the effect ... the result is the filter input image."
That is, a negative `radius` behaves identically to `0.0`, not merely "unsupported" or "unspecified" — both disable the primitive outright, with `in` passed through unchanged.
`morphology`'s doc comment now states this directly instead of hedging, and there is accordingly no reason for `morphology`/`morphology_xy` to reject or clamp a negative value in Rust: the SVG-defined pass-through behaviour is already the correct, useful result, so preserving it (rather than adding a redundant `Error` variant purely to special-case a value the renderer already handles safely) is the simpler and equally correct choice.

See [`docs/gaps.md`](../gaps.md) for the primitives still to be added.

## `image` takes `href` positionally and adds no `PreserveAspectRatio` type, both by analogy with `SvgRoot::image`

Alongside `turbulence` and `turbulence_xy`, `<feImage>` takes its content from resolving an `href`, rather than an `in` generator.
So `image`'s doc comment carries the same explicit callout `turbulence`'s does rather than leaving a reader to assume every primitive reads `in`.

`href` is `image`'s one positional parameter — the same "cover what's common, defer what's rare" judgement already applied to `flood`'s `color`/`opacity` and `offset`'s `dx`/`dy`: every meaningful use of `feImage` supplies it, so it does not belong behind the generic `set_attr` escape hatch the way `result` (and here, `preserveAspectRatio`) does.

`preserveAspectRatio` is deliberately *not* given a named parameter or a new typed enum, even though `<feImage>` shares this attribute with `<image>`/`<symbol>`/`<marker>`.
Checking those three first found no existing `PreserveAspectRatio` type anywhere in the crate: `SvgRoot::image` has no dedicated setter for it at all (its own doc comment just points at using `set_attr`), `SvgMarker` explicitly documents the same gap, and only `SvgSymbol::set_preserve_aspect_ratio` exists, taking a plain `&str`, not an enum.

`image` follows the majority convention (no dedicated setter), the same way `MorphologyOperator` above chose to reuse `Channel` rather than create a single-use type.
Here, we reached for a typed enum when a closed vocabulary is already established crate-wide, not merely because an attribute recurs.

`href` is written verbatim with no validation, under a `# Security` doc section that is close to word-for-word the one on `SvgRoot::image`/`SvgNode::set_href` — the risk (an attacker-controlled `javascript:` URL reaching `setAttribute`) and the crate's stance on it (document, don't silently sanitise) are unchanged by which element carries the attribute.

See [`docs/gaps.md`](../gaps.md) for the primitives still to be added.

## Filter region and coordinate-space attributes get named setters, `FilterUnits` reuses the `PatternUnits` shape

Previously, `SvgFilter`'s own `x`, `y`, `width`, `height`, `filterUnits`, and `primitiveUnits` attributes could only be modified using the generic `set_attr`/`set_attr_display` escape hatch — none of them block constructing a *primitive*, only tuning the filter region a primitive renders into, so there was no forcing function to add them sooner.

They are common enough to need their own typed setters now: `set_width`/`set_height`, in particular, are the fix for the SVG default filter region (`-10% -10% 120% 120%` of the referencing element's bounding box) clipping a wide `gaussian_blur`, a problem real enough to already be called out as an aside in this crate's own doc comments before a typed setter existed to fix it.

`set_x`/`set_y`/`set_width`/`set_height` follow `SvgPattern`'s identical four setters exactly — plain `f64` via `SvgAttrs::display_element`, no unit suffix, since under the SVG default `filterUnits="objectBoundingBox"` a plain number is already interpreted as a bounding-box fraction (`1.4` means `140%`), and under `userSpaceOnUse` it is a user-space coordinate.

`filterUnits` and `primitiveUnits` share one `FilterUnits` enum (`UserSpaceOnUse`/`ObjectBoundingBox`) rather than two separate enums with identical variants.
This is the same choice `PatternUnits` already made for `patternUnits`/`patternContentUnits`: both attributes draw from the same two-value SVG vocabulary, so a second enum only duplicates `as_str()` with no type-safety benefit.

Note the two attributes default to *different* variants (`filterUnits` defaults to `ObjectBoundingBox`, `primitiveUnits` to `UserSpaceOnUse`) — `FilterUnits` only fixes which values are legal, not which one a bare `<filter>` starts with; each setter's own doc comment states its attribute's default explicitly so callers do not have to guess or check the SVG specification.

## `tile` is the first primitive with zero parameters

`<feTile>` has no numeric or enum-valued attributes needing a typed parameter: no `<number>`, `<number-optional-number>`, or enum-typed attribute to wrap here.
It accepts the optional input selector `in` (unlike the generators — `turbulence`, `turbulence_xy`, `image` or `flood` — which do not consume one, as their own doc comments already explain), together with the common filter-primitive attributes `x`, `y`, `width`, `height`, and `result`.
`tile(&self) -> Result<SvgNode, Error>` is accordingly the simplest possible instance of the crate-wide primitive pattern: create the element, append it, return the node, with no attribute-writing step in between.

The interesting design question was not the method signature but where the explanation of *how tiling actually works* belongs.
`feTile` does not choose what gets repeated — that is entirely a function of the *input* primitive's own subregion (defined by `x`, `y`, `width`, `height`), which every primitive already has via the generic `set_attr` and `set_attrs` escape hatch.
However, no earlier primitive's doc comment had a reason to focus on this aspect, since none of them changed behaviour based on whether their own subregion was left at its default or narrowed.
That default is not uniformly "the whole filter region".
An ordinary primitive's default subregion is generally the union of its own referenced inputs' subregions — but a generator with no referenced input, such as `feTurbulence` in its own example, defaults instead to the whole filter region.

`tile` is the first primitive whose entire visible effect depends on a *different* primitive's subregion being deliberately narrowed first, so its doc comment states this as a `⚠️` warning rather than folding it into ordinary prose — the same treatment already given to other easy-to-miss no-op conditions (`morphology_xy`'s zero-or-negative-component case, `gaussian_blur`'s wide-blur clipping).

`tile` is documented as the filter-graph counterpart to `SvgDefs::pattern` and `SvgDefs::build_pattern`, cross-referenced in both directions: a `<pattern>` is a paint server, applied via `fill` or `stroke`, and cannot appear inside a `<filter>`.

`feTile` repeats a filter-generated tile as one step inside a filter graph instead, so its output can feed further primitives (colour-transformed, blended, composited, ...) the same way any other primitive's output can.
Neither replaces the other; they solve the same "repeat this pattern" problem in two different parts of the SVG rendering model.

See [`docs/gaps.md`](../gaps.md) for the primitives still to be added.

## `convolve_matrix` and `convolve_matrix_xy` take a plain `&[f64]` kernel and document, rather than validate, a length mismatch

`<feConvolveMatrix>` is the highest-attribute-count primitive in this crate: `order`, `kernelMatrix`, `divisor`, `bias`, `targetX`/`targetY`, `edgeMode`, `kernelUnitLength`, and `preserveAlpha`, on top of the usual `in`/`result`.

Following the same "cover what's common, defer what's rare" judgement already used by `drop_shadow` and `composite`, five of these become named parameters — `order` (or `order_x`/`order_y`), `kernel_matrix`, `divisor`, `edge_mode`, `preserve_alpha` — since every meaningful use of this primitive supplies all five: there is no useful "just convolve" call that skips any of them.

`bias`, `targetX`, `targetY`, and `kernelUnitLength` are deferred to the generic `set_attr`/`set_attrs` escape hatch, which is the same treatment `composite` gives `Arithmetic`'s `k1`–`k4`: each only matters for a minority of kernels:

* `bias` for a kernel whose output can go negative
* `targetX`/`targetY` for an asymmetric kernel whose "centre" isn't the geometric middle
* `kernelUnitLength`, a deprecated legacy attribute for requesting explicit kernel sampling intervals — the current Filter Effects specification marks it deprecated and slated for removal, since it does not reliably achieve the device-independent rendering it was meant to provide

so promoting all four to positional parameters would bloat the common call for a benefit needed only by a few callers — and, for `kernelUnitLength` specifically, one the specification itself now discourages relying on.
`kernelUnitLength` remains reachable through `set_attr` regardless, since a deprecated attribute is not the same as a removed one, and this crate's escape hatch does not gatekeep by spec status.

`order`, `order_x` and `order_y` follow the fourth occurrence of the `fmt::Arguments`-core split already used by `gaussian_blur`/`gaussian_blur_xy` (`stdDeviation`), `turbulence`/`turbulence_xy` (`baseFrequency`), and `morphology`/`morphology_xy` (`radius`): that is, private `convolve_matrix_args` do the actual element creation and attribute writes, with `convolve_matrix` calling it via `format_args!("{order}")` and `convolve_matrix_xy` via `format_args!("{order_x} {order_y}")`.

`kernel_matrix` is a plain `&[f64]` slice, not a fixed-size array the way `ColorMatrixType::Matrix` uses `[f64; 20]`.

`feColorMatrix`'s `values` attribute has exactly one valid length (20) across every use of that element, so a fixed-size array of the wrong length can be caught at compile time for free; however, `feConvolveMatrix`'s `kernelMatrix` has no such single valid length, since the slice length must equal `order_x * order_y`, which is itself a parameter the caller chooses per call.

A `[f64; N]` parameter for `kernel_matrix` would therefore need `N` to be a const generic tied to two further const generics (the product of `order_x` and `order_y`), which is an expression Rust's stable const-generics cannot yet support (this crate's `rust-version` is 1.85, well before the still-nightly-only `generic_const_exprs`).

A runtime slice is therefore the only shape available, so a mismatch between `kernel_matrix.len()` and `order_x * order_y` cannot be rejected at compile time the way an out-of-range `ColorMatrixType::Matrix` element count can.

The natural next question is whether that mismatch should be validated at runtime, the way `component_transfer` validates `TransferFunction::Table`/`Discrete`'s value-count edge cases via `Error::InvalidTransferFunction`.

Checking the SVG specification settles it the other way: a `kernelMatrix` whose length does not equal `orderX * orderY` is explicitly defined to make `<feConvolveMatrix>` *"act as a pass through filter"*.
This therefore defines a well-formed (albeit inert) rendering outcome, rather than an error condition whose handling browsers may well disagree on (in the same way they disagree on how an *undefined* `TransferFunction::Table`/`Discrete` length is handled).

This is exactly the distinction the `morphology`/`morphology_xy` correction (below) already draws for a negative/zero `radius`: a defined-but-perhaps-surprising SVG behaviour gets documented with a `⚠️` warning on the affected method, not a new `Error` variant.

`convolve_matrix`'s own doc comment states this directly, mirroring `morphology_xy`'s *"not a no-op you'd expect"* treatment rather than inventing a new validation for a case the specification has already settled.

The same reasoning applies to `divisor: 0.0`: the specification defines this as falling back to the sum of `kernel_matrix`'s own values (or `1.0` if that sum is itself `0.0`), rather than an error.
So this crate passes it through unvalidated and documents the fallback rather than rejecting it or treating it as a special case.

**Correction:** an external review of the first `convolve_matrix`/`convolve_matrix_xy` implementation pointed out that `order`, `order_x` and `order_y` were left unvalidated too, on the mistaken assumption that the same "document, don't validate" reasoning above covered every edge case this primitive has.

It does not.

The Filter Effects specification requires `order`'s components to be integers greater than zero, but it gives no defined fallback for a zero component.

Unlike the `kernelMatrix` length mismatch (defined as a pass-through) and the zero `divisor` (defined as falling back to the kernel's sum): this is not a "defined but surprising" outcome, but simply a value outside the attribute's permitted range with no defined rendering behaviour.

That is exactly the other branch of the "validate vs. document" distinction this crate already draws elsewhere (see `component_transfer`'s `Error::InvalidTransferFunction` above, for a value list whose length has no defined SVG semantics): an undefined outcome gets rejected before it reaches the DOM, not documented and passed through.

`convolve_matrix`/`convolve_matrix_xy` now check `order`/`order_x`/`order_y` for zero and return the new `Error::InvalidConvolveMatrixOrder` before creating any element, rather than serializing a value the specification never assigns a meaning to — `u32` already rules out negative and fractional `order` values at compile time, so zero was the one remaining gap `u32` alone could not close.

`SpaceSeparated` — to avoid duplicating functionality, the private `fmt::Display` wrapper `component_transfer` already used internally to provide an allocation-free way to write `tableValues`, has been moved from `component_transfer.rs` to `primitives/mod.rs` as `pub(super)` so `convolve_matrix` can reuse it for `kernelMatrix`.

`EdgeMode` is a fresh three-variant fieldless enum (`Duplicate`/`Wrap`/`None`), not a reuse of any existing crate enum — unlike `MorphologyOperator` reusing `Channel`'s selector vocabulary, nothing else in this crate's filter API shares `edgeMode`'s specific `duplicate`/`wrap`/`none` keyword set, so a new type is the correct call here, following the same "reuse only when the vocabulary is genuinely identical" rule that section already established.

`Duplicate` is listed first as `<feConvolveMatrix>`'s own SVG default — the same "spec default first" convention `MorphologyOperator` follows — even though `<feGaussianBlur>` also has an `edgeMode` attribute sharing this vocabulary with a *different* default (`None`); `EdgeMode`'s own doc comment notes this explicitly so a reader does not assume one shared default across both elements.
