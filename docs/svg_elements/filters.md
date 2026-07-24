# Filters

[← Back to supported elements](README.md)

**Contents**

## `<filter>`

A `<filter>` applies raster effects (such as blur, colour manipulation or compositing etc.) to any element that references it.
The browser evaluates the filter's primitive children in document order and paints the result in place of the referencing element.

To live-append a filter to the DOM, call `SvgDefs::filter(id)`

To generate a filter using a closure, call `SvgDefs::build_filter(id, closure)`.
The filter will remain detached from the DOM until the closure succeeds.

Apply it to any element with `SvgNode::set_filter_ref(&filter)` or `SvgNode::set_filter("id")`.

Remove the filter with `SvgNode::remove_filter()`.

## Primitive Factories Available on `SvgFilter`

| Method | Element | Description |
|---|---|---|
| `gaussian_blur(std_deviation)` | `<feGaussianBlur>` | Blurs the input equally on both axes; larger `std_deviation` blurs more. `std_deviation` is interpreted in the coordinate system established by `primitiveUnits`, which is user-space units by default, or bounding-box-relative values under `ObjectBoundingBox`. Returns an `SvgNode`, so `in`/`result` (not yet wrapped by a named setter) can be set via `set_attr`. |
| `gaussian_blur_xy(std_deviation_x, std_deviation_y)` | `<feGaussianBlur>` | Blurs the input independently along the horizontal and vertical axes, writing the SVG two-number `stdDeviation="x y"` form internally. Both values are interpreted in the same `primitiveUnits`-dependent coordinate system as `gaussian_blur`. Pass `0.0` to switch off blurring along that particular axis. |
| `offset(dx, dy)` | `<feOffset>` | Shifts the input by `(dx, dy)` in the coordinate system established by `primitiveUnits` — user-space units by default, or bounding-box-relative values under `ObjectBoundingBox`. Returns an `SvgNode` for `in`/`result`, as above. |
| `merge(inputs)` | `<feMerge>` (with `<feMergeNode>` children) | Stacks each `&str` in `inputs` as one `<feMergeNode in="...">` child, in order (later entries painted on top). The standard way to layer a shadow underneath the original graphic. |
| `flood(color, opacity)` | `<feFlood>` | Fills the filter region with a solid `flood-color`/`flood-opacity`. Combined with `composite`, gives a shadow an independent colour rather than a blurred copy of the source graphic's own fill. |
| `composite(in2, operator)` | `<feComposite>` | Combines this primitive's `in` input with `in2` using a [Porter-Duff](https://en.wikipedia.org/wiki/Alpha_compositing) `CompositeOperator` (`Over`/`In`/`Out`/`Atop`/`Xor`/`Lighter`/`Arithmetic`). `operator: In` against a blurred alpha mask is the standard way to tint a shadow. **`Arithmetic` needs extra setup — see the warning below before using it.** |
| `blend(in2, mode)` | `<feBlend>` | Mixes this primitive's `in` input with `in2` using a `BlendMode` — the sixteen standard `<blend-mode>` keywords shared by CSS compositing and SVG (`Normal`, `Multiply`, `Screen`, `Darken`, `Lighten`, `Overlay`, `ColorDodge`, `ColorBurn`, `HardLight`, `SoftLight`, `Difference`, `Exclusion`, `Hue`, `Saturation`, `Color`, `Luminosity`) — not the full CSS `mix-blend-mode` value set, which also has two CSS-only modes (`plus-lighter`/`plus-darker`) this enum does not offer. Unlike `composite`, which combines opaque inputs geometrically, `blend` combines their *colours* photometrically, by default in `linearRGB` rather than the `sRGB` space CSS and most image editors use — see the warning below. **IMPORTANT**: Tinting with a flood colour needs a final `composite(In)` step to preserve transparency — see the warning below before using it. |
| `drop_shadow(std_deviation, dx, dy, color, opacity)` | `<feDropShadow>` | Implements the browser-native shorthand for the entire `gaussian_blur` → `flood` → `composite` → `offset` → `merge` chain described below. `std_deviation` and `dx`/`dy` are interpreted in the same `primitiveUnits`-dependent way as their `gaussian_blur`/`offset` counterparts. Its result already has the original graphic merged on top: a `<filter>` containing only `drop_shadow` is a complete effect, so there is no need to call `merge` after it. |
| `color_matrix(matrix_type)` | `<feColorMatrix>` | Transforms colours via a `ColorMatrixType`: `Saturate(amount)`, `HueRotate(degrees)`, `LuminanceToAlpha`, or a full custom `Matrix([f64; 20])` (the fixed-size array rules out a wrong element count at compile time). Independent of the shadow primitives above — greyscale, saturation, and hue effects, not compositing. |
| `component_transfer(funcs)` | `<feComponentTransfer>` (with `<feFuncR>`/`<feFuncG>`/`<feFuncB>`/`<feFuncA>` children) | Remaps one or more colour channels independently. `funcs` is a `&[(Channel, TransferFunction)]` slice — one `<feFuncX>` child per entry, in order; a channel not mentioned gets no child at all (the SVG default, equivalent to `TransferFunction::Identity`). `TransferFunction` selects `type`/its attributes: `Table(Vec<f64>)`/`Discrete(Vec<f64>)` write `tableValues` as the space-separated list, `Linear { slope, intercept }` and `Gamma { amplitude, exponent, offset }` write their own named attributes. The standard way to do gamma correction, contrast/levels adjustment, posterisation (`Discrete`), or an alpha fade/clip — none of which `color_matrix`'s whole-pixel linear transform can express. |
| `turbulence(base_frequency, num_octaves, seed, turbulence_type)` | `<feTurbulence>` | Generates Perlin-noise output without reading from the `in` parameter, since it fabricates its own image rather than reading an upstream one. `turbulence_type` is a `TurbulenceType` (`Turbulence`, higher-contrast/marbled; `FractalNoise`, softer/cloud-like). `num_octaves` is a `u32`, ruling out negative values (unsupported by SVG) at compile time. Almost always paired with `displacement_map` — see the warning below. |
| `turbulence_xy(base_frequency_x, base_frequency_y, num_octaves, seed, turbulence_type)` | `<feTurbulence>` | As `turbulence`, but with independent horizontal/vertical base frequencies, writing the SVG two-number `baseFrequency="x y"` form. Unequal frequencies stretch the noise along whichever axis has the lower one — useful for wood-grain/brushed-metal textures that should read as directional rather than isotropic. |
| `displacement_map(in2, scale, x_channel_selector, y_channel_selector)` | `<feDisplacementMap>` | Warps this primitive's `in` input using `in2`'s `x_channel_selector`/`y_channel_selector` channel values (each a `Channel`) as a per-pixel displacement field, scaled by `scale`. `in2` is typically `turbulence`/`turbulence_xy`'s `result`. Selecting different channels for `x`/`y` (e.g. `Red`/`Green`) gives free two-dimensional displacement; `Channel::Alpha` for both is the SVG default but constrains displacement to one diagonal — see the warning below. |
| `morphology(radius, operator)` | `<feMorphology>` | Takes a component-wise minimum (`Erode`, SVG default) or maximum (`Dilate`) over `radius`, across the input's premultiplied R/G/B/A channels — a `MorphologyOperator` selects which. Against `SourceAlpha`, the common case, this shrinks/thins or grows/thickens the source silhouette; against `SourceGraphic` it can also shift or bleed colours at edges, since colour channels are processed the same way. `radius` is interpreted in the same `primitiveUnits`-dependent way as `gaussian_blur`'s `std_deviation`; `0.0`, or any negative value, disables the effect (`in` passed through unchanged). |
| `morphology_xy(radius_x, radius_y, operator)` | `<feMorphology>` | As `morphology`, but with independent horizontal/vertical radii, writing the SVG two-number `radius="x y"` form. Both values must be positive: unlike `gaussian_blur_xy`, a zero (or negative) component on *either* axis disables the whole primitive rather than giving a one-dimensional effect — see the warning below. |
| `image(href)` | `<feImage>` | Uses the image or SVG content at `href` as this primitive's own generated output — like `turbulence`/`turbulence_xy`, a generator with no meaningful `in`. `preserveAspectRatio` is not wrapped by a named parameter, the same choice already made for `SvgRoot::image`. Supplies a *second*, independent image source that can be combined with the filtered element's own `SourceGraphic`/`SourceAlpha` — a plain filtered `<image>` already becomes its own `SourceGraphic` and can be colour-transformed or blended on its own, but combining it with unrelated content needs either `image` or a second layered element. `href` accepts a same-document `"#id"` reference as well as an external/`data:` URL, and is written verbatim; do not pass a `javascript:` URL or other attacker-controlled string without validation. Loading is asynchronous: a successful return means only that the DOM node was constructed, not that `href` has loaded; a missing/unsupported/zero-sized/failed resource renders as transparent black. A *tainted* result (an SVG element reference, or an image fetched in no-CORS mode) consumed as `displacement_map`'s `in2` makes that displacement silently become a pass-through — see the warning below. |
| `tile()` | `<feTile>` | Repeats its input across this primitive's own subregion. Has no numeric or enum-valued attributes needing a typed parameter — the optional `in`, and the common `result`, `x`, `y`, `width` and `height` are all reachable via `set_attr`/`set_attrs` on the returned `SvgNode`. The repeated rectangle is the selected input's *effective* subregion, not anything `tile` itself chooses — see the warning below, since that subregion must be smaller than `tile`'s own output subregion or tiling has no visible effect. |
| `convolve_matrix(order, kernel_matrix, divisor, edge_mode, preserve_alpha)` | `<feConvolveMatrix>` | Applies a square `order`×`order` matrix convolution — the general image-processing operation behind sharpening, blurring, embossing, and edge-detection kernels. `kernel_matrix` is a `&[f64]` slice that must contain exactly `order * order` values, in row-major order; `edge_mode` is an `EdgeMode` (`Duplicate`/`Wrap`/`None`) selecting how the kernel reads beyond the input's border. `bias`, `targetX`, `targetY`, and `kernelUnitLength` are not wrapped by named parameters — set them via `set_attr`/`set_attrs` on the returned node. `kernelUnitLength` is a deprecated legacy attribute for requesting explicit kernel sampling intervals; the current Filter Effects specification marks it deprecated and slated for removal, since it does not reliably achieve platform-independent rendering, so it remains reachable through `set_attr` but should not be relied upon. **A `kernel_matrix` whose length does not match `order * order` is not rejected, but neither will it result in any visible alteration to the input — see the warning below.** **`order: 0` *is* rejected, returning `Error::InvalidConvolveMatrixOrder` — see the warning below.** |
| `convolve_matrix_xy(order_x, order_y, kernel_matrix, divisor, edge_mode, preserve_alpha)` | `<feConvolveMatrix>` | As `convolve_matrix`, but with an `order_x`×`order_y` rectangular kernel, writing the SVG two-number `order="x y"` form — the natural shape for a directional (horizontal-only or vertical-only) effect that a square kernel cannot express. |

***⚠️ `CompositeOperator::Arithmetic` requires `k1`–`k4` to be set manually***

`composite()` does not write these arguments and their SVG initial value is `0`.
Choosing `Arithmetic` and stopping there evaluates `0*i1*i2 + 0*i1 + 0*i2 + 0` for every pixel: that is, a **transparent black** not a blend of the two inputs - and no error to signal the mistake.
Always set all four coefficients immediately after the call:

```rust,no_run
use svg_dom::{SvgRoot, root::filter::CompositeOperator};

let svg  = SvgRoot::attach("diagram")?;
let defs = svg.defs()?;
let flt  = defs.filter("blend")?;
flt.gaussian_blur(6.0)?.set_attrs([("in", "SourceGraphic"), ("result", "blur")])?;

// A straightforward 50/50 blend of the sharp source and its blurred copy: k2 = k3 = 0.5, k1 = k4 = 0.
flt.composite("blur", CompositeOperator::Arithmetic)?.set_attrs([
    ("in", "SourceGraphic"),
    ("k1", "0"), ("k2", "0.5"), ("k3", "0.5"), ("k4", "0"),
])?;
Ok::<(), svg_dom::Error>(())
```

A drop-shadow can be constructed by chaining `gaussian_blur` + `flood` + `composite` + `offset` + `merge` together: blur the source alpha, composite a flood colour into the blurred mask, offset it, then merge it underneath the original — see the `<filter>` demo panel or `SvgFilter::composite`'s doc example.

For the common case (where the shadow is the same as the graphic it decorates), the above chaining is not needed; simply call `drop_shadow`.
The manual chain is only useful in situations where you need to name an intermediate result (`result="blur"`) for reuse by another primitive, or shadow one graphic while merging a different one on top.

A "poor man's" drop-shadow can be constructed by chaining `gaussian_blur` + `offset` + `merge`; however using these effects alone, you cannot have an independent shadow colour.

`blend` tints an entire graphic by mixing its colours with a flood colour, rather than isolating a mask the way `composite`'s `In` operator does against a blurred shadow:

```rust,no_run
use svg_dom::{SvgRoot, root::filter::{BlendMode, CompositeOperator}};

let svg  = SvgRoot::attach("diagram")?;
let defs = svg.defs()?;
let tint = defs.filter("tint")?;
tint.flood("steelblue", 1.0)?.set_attr("result", "colour")?;
tint.blend("colour", BlendMode::Multiply)?.set_attrs([("in", "SourceGraphic"), ("result", "tinted")])?;
// Clip back to the source's own alpha coverage — see the warning below.
tint.composite("SourceGraphic", CompositeOperator::In)?.set_attr("in", "tinted")?;
Ok::<(), svg_dom::Error>(())
```

***⚠️ Tinting with a flood colour needs a final `composite(In)` to preserve transparency***

`flood` paints its colour *opaquely* across the entire filter region — a rectangle, unrelated to whatever shape or transparency the source graphic actually has.

Blending that flood straight against `SourceGraphic` only changes the *colour*, not *alpha*: so `feBlend`'s result alpha is the union of its two inputs' alpha, so the flood's opacity carries straight through and the flood colour shows through wherever the source graphic was itself transparent (the corners of a circle's bounding box, the transparent parts of an image, and so on).

The final `composite(in2: "SourceGraphic", operator: In)` step above clips the blended result back to the source's own alpha coverage, discarding the leaked flood outside it.
Skipping it is only safe when the source graphic is known to be fully opaque across its entire filter region — a plain rectangle, for instance, which is why this mistake is easy to miss during development.

***⚠️ `BlendMode` is not quite CSS `mix-blend-mode`***

Two qualifications on the "same as CSS" comparison above:

`BlendMode` covers the sixteen *standard* `<blend-mode>` keywords shared by CSS compositing and SVG `feBlend` — `mix-blend-mode` itself also accepts two CSS-only, property-specific modes (`plus-lighter`/`plus-darker`) that have no `feBlend` equivalent and so are not offered here.

SVG filter primitives operate in the `linearRGB` colour space by default, unlike CSS `mix-blend-mode` and most image editors, which operate in `sRGB`.
The same `BlendMode` can therefore produce a visibly different result here than the "same" mode elsewhere, even given identical input colours.
Set `color-interpolation-filters="sRGB"` on the `<filter>` element (or on an individual primitive's own `SvgNode` to override it just for that primitive) when an sRGB-space result is required to match CSS or an image editor.

`component_transfer` gamma-corrects all three colour channels identically, and fades alpha via a linear remap:

```rust,no_run
use svg_dom::{SvgRoot, root::filter::{Channel, TransferFunction}};

let svg  = SvgRoot::attach("diagram")?;
let defs = svg.defs()?;
let flt  = defs.filter("gamma-fade")?;
let gamma = TransferFunction::Gamma { amplitude: 1.0, exponent: 0.45, offset: 0.0 };

flt.component_transfer(&[
    (Channel::Red, gamma.clone()),
    (Channel::Green, gamma.clone()),
    (Channel::Blue, gamma),
    (Channel::Alpha, TransferFunction::Linear { slope: 0.6, intercept: 0.0 }),
])?;

Ok::<(), svg_dom::Error>(())
```

***⚠️ An `Alpha` transfer function with `f(0) > 0` can paint a background across the whole filter region***

`feComponentTransfer` runs on every pixel of its input using non-premultiplied values, including pixels that started fully transparent.
If the `Channel::Alpha` function maps `0.0` to something above `0.0` — for example `Linear { intercept: 0.2, .. }`, `Gamma { offset: 0.2, .. }`, or a `Table`/`Discrete` list whose first entry is above `0.0` — every previously-transparent pixel becomes visible too, not just the ones that were already part of the shape.
When `in` is `SourceGraphic` (the default for the first primitive), the primitive subregion is the whole filter region, so the result is a rectangular halo or background fill across that entire region, not a subtler change confined to the original silhouette.
The example above avoids this: its `Linear { slope: 0.6, intercept: 0.0 }` alpha function has `f(0) == 0`, so fully transparent pixels stay fully transparent.
Only give `Channel::Alpha` a function with `f(0) == 0` unless a background fill across the whole region is the intended effect.

***⚠️ `TransferFunction::Table` with exactly one value, or `Discrete` with zero values, is rejected***

The SVG spec defines `tableValues` for `Table` as `n+1` values describing `n` interpolation regions.

An empty list (`n = 0`) is explicitly defined as equivalent to `Identity`, but a single value also leaves `n = 0` with no region to which to apply the interpolation formula — the spec does not define what a lone value means, so browsers are free to differ.

For a portable constant `Table` transfer function, supply the same value twice instead: `TransferFunction::Table(vec![0.5, 0.5])`.

`Discrete` has the opposite asymmetry: a *single* value is well-defined by the SVG "discrete" stepping formula (every input maps to that one entry), but an *empty* list is not — the formula divides the input by the value count and indexes into the list with the result, both undefined for zero values, and the spec gives the empty list no identity fallback the way it does for `Table`.

`component_transfer` returns `Error::InvalidTransferFunction` for either case rather than silently emitting a `tableValues` whose meaning depends on which browser renders it.

***⚠️ Naming the same `Channel` twice in `funcs` does not compose — the last one wins***

`funcs` is not deduplicated, so `component_transfer` will happily create two `<feFuncR>` children if `Channel::Red` appears twice.
Per the SVG spec, when a `<feComponentTransfer>` has more than one child for the same channel, only the last one has any effect; the earlier ones are created in the DOM but ignored, not chained or applied in sequence.

Avoid supplying the same `Channel` more than once when building `funcs` programmatically.

`turbulence`/`turbulence_xy` and `displacement_map` together distort a shape's edge with noise — the standard route to hand-drawn/organic textures:

```rust,no_run
use svg_dom::{SvgRoot, root::filter::{Channel, TurbulenceType}};

let svg  = SvgRoot::attach("diagram")?;
let defs = svg.defs()?;
let flt  = defs.filter("organic-edge")?;

flt.turbulence(0.02, 3, 5.0, TurbulenceType::FractalNoise)?.set_attr("result", "noise")?;
flt.displacement_map("noise", 24.0, Channel::Red, Channel::Green)?
    .set_attr("in", "SourceGraphic")?;

Ok::<(), svg_dom::Error>(())
```

***⚠️ `<feTurbulence>` reads no `in` at all***

Unlike every other primitive on this page, `turbulence`/`turbulence_xy` create their noise image, so there is no upstream input to chain from.
Consequently, `in` on the returned node has no effect.
Name the noise via `result` (as in the example above) so a later primitive's `in`/`in2` (typically `displacement_map`'s `in2`) can reference it.

***⚠️ `displacement_map(Channel::Alpha, Channel::Alpha)` constrains displacement to the `y = x` diagonal***

Passing the *same* channel for both `x_channel_selector` and `y_channel_selector` means `dx` and `dy` are computed from the identical `0.0`–`1.0` value at every pixel, so every displacement vector lands on the `y = x` line rather than pointing freely in two dimensions.

`Alpha`/`Alpha` is the SVG default and can certainly be a valid choice when diagonal displacement is exactly what is wanted, but this is not the general case.
The example above instead selects two *different* channels — `Channel::Red` for `x_channel_selector`, `Channel::Green` for `y_channel_selector` — since `turbulence` and `turbulence_xy` generate each colour channel separately, giving the displacement two distinct dimensions.

This is also the choice the SVG specification's own explanatory `feDisplacementMap` example makes.

`morphology` takes a component-wise minimum or maximum, over a fixed radius, across the input's premultiplied R/G/B/A channels — independently of every other primitive on this page.
Against `SourceAlpha`, where alpha is the only channel with anything to shrink or grow, this reads as growing or shrinking a shape's silhouette:

```rust,no_run
use svg_dom::{SvgRoot, root::filter::MorphologyOperator};

let svg  = SvgRoot::attach("diagram")?;
let defs = svg.defs()?;
let flt  = defs.filter("bold-outline")?;

flt.morphology(2.5, MorphologyOperator::Dilate)?
    .set_attrs([("in", "SourceAlpha"), ("result", "thickened")])?;
flt.merge(&["thickened", "SourceGraphic"])?;

Ok::<(), svg_dom::Error>(())
```

Dilating `SourceAlpha` then merging it underneath the original graphic leaves only the grown-outward fringe visible — a bolder outline without otherwise changing the shape's own fill.
`Erode` (the SVG default) does the reverse: shrinking a mask inward, useful before reusing it elsewhere (for example, insetting a mask so a subsequent blur does not visibly extend past the original edge).

The example explicitly sets `in` to `SourceAlpha` for exactly this reason: `morphology`'s *implicit* input (when it is the filter's first primitive, as it would be without the explicit `set_attr` above) is `SourceGraphic`, not `SourceAlpha`.
Against `SourceGraphic`, the same component-wise minimum/maximum applies to the colour channels too, which can shift or bleed colours at edges where they differ between neighbouring pixels — not purely a silhouette-size transform in that case.

***⚠️ `morphology_xy` with a zero (or negative) component disables the whole primitive, not just that axis***

Unlike `gaussian_blur_xy`, where `gaussian_blur_xy(0.0, 6.0)` is an explicitly supported one-dimensional blur, the SVG spec defines `feMorphology` differently: a `radius` component that is zero or negative disables the *entire* primitive and causes `in` to be passed through completely unchanged.

`morphology_xy(3.0, 0.0, MorphologyOperator::Dilate)` is therefore a no-op, not a horizontal-only dilation — both `radius_x` and `radius_y` must be positive for this primitive to have any effect.

`image` brings external image content into a filter graph, where it can be combined with any other primitive on this page.

Filtering a plain `<image>` element already lets *that* image be colour-transformed, blended, or composited on its own: the image becomes the filter's `SourceGraphic`, and any primitive that reads `SourceGraphic` (the implicit input of a filter's first primitive) operates on it directly.
`image`'s value is supplying a *second*, independent source — content unrelated to whatever element the filter is applied to — that can be combined with the filtered element's own `SourceGraphic`/`SourceAlpha` within the same filter graph, without a second layered display element.

The simplest chain just imports and transforms the image itself, the same way any other primitive's output can be chained:

```rust,no_run
use svg_dom::{SvgRoot, root::filter::ColorMatrixType};

let svg  = SvgRoot::attach("diagram")?;
let defs = svg.defs()?;
let flt  = defs.filter("greyscale-image")?;

flt.image("photo.jpg")?;
flt.color_matrix(ColorMatrixType::Saturate(0.0))?;

Ok::<(), svg_dom::Error>(())
```

Like `turbulence` and `turbulence_xy`, `image` does not read from the `in` argument.
Its content comes from `href`, not from an upstream primitive's result, so `color_matrix`'s implicit input above (being the filter's second primitive) is `image`'s own output, not `SourceGraphic`.

The distinguishing use case combines `image`'s output with the *filtered element's own* graphic — something a filtered `<image>` cannot do on its own, since it has no second, independent source to combine with:

```rust,no_run
use svg_dom::{SvgRoot, root::filter::{BlendMode, CompositeOperator}};

let svg  = SvgRoot::attach("diagram")?;
let defs = svg.defs()?;
let flt  = defs.filter("textured-text")?;

flt.image("texture.jpg")?.set_attr("result", "texture")?;
flt.composite("SourceAlpha", CompositeOperator::In)?
    .set_attrs([("in", "texture"), ("result", "clipped-texture")])?;
flt.blend("clipped-texture", BlendMode::Multiply)?
    .set_attr("in", "SourceGraphic")?;

Ok::<(), svg_dom::Error>(())
```

`composite(..., In)` against `SourceAlpha` clips the texture to the filtered element's own silhouette; `blend(..., Multiply)` against `SourceGraphic` then multiplies it over the element's own colours — a texture applied to text or a shape, with no second layered display element required.

***⚠️ `href` is written verbatim, with no validation***

As with `SvgRoot::image`, do not pass a `javascript:` URL or any other attacker-controlled string as `href` without validating it first.

***⚠️ `href` loads asynchronously, and a failed resource fails silently***

A successful `image(href)` return means only that the `<feImage>` DOM node was constructed — it says nothing about whether the resource identified by `href` has finished loading, or ever will.
A resource that is either missing, unsupported, zero-sized, or fails to download will be rendered as transparent black across the primitive's subregion, per the SVG specification - without any error being reported through the API.

***⚠️ Taint and `displacement_map`***

Per the Filter Effects specification, an `<feImage>` result is *tainted* when `href` references an SVG element (a same-document `"#id"` reference) or when an image resource is fetched in no-CORS mode.
A tainted result used as `in2` for `displacement_map` makes that displacement a silent pass-through (`in` is returned unmodified, with no error to signal the mistake) rather than an actual displacement.

For a *fetched image resource* used as a displacement map, set `crossorigin` (not wrapped by a named parameter, so use `set_attr`) — typically `"anonymous"` — and ensure the server hosting `href` sends matching CORS headers, so the fetch succeeds in CORS mode rather than no-CORS mode:

```rust,no_run
use svg_dom::SvgRoot;

let svg  = SvgRoot::attach("diagram")?;
let defs = svg.defs()?;
let flt  = defs.filter("cross-origin-displacement")?;

flt.image("https://example.com/map.png")?
    .set_attrs([("crossorigin", "anonymous"), ("result", "displacement-map")])?;

Ok::<(), svg_dom::Error>(())
```

`crossorigin` has no effect on a same-document element reference (such as `"#texture"` — see above) — those are unconditionally tainted by definition, so they remain unusable as a displacement map regardless.
They are perfectly usable everywhere else (as input to `color_matrix`, `blend`, `composite`, and so on) — taint only forecloses the `displacement_map` case.

`tile` repeats its input across this primitive's own subregion, behaving as the filter-graph counterpart to `SvgDefs::pattern` and `SvgDefs::build_pattern`, which repeats a paint server (applied via `fill` or `stroke`) rather than a filter-generated tile:

```rust,no_run
use svg_dom::{SvgRoot, root::filter::TurbulenceType};

let svg  = SvgRoot::attach("diagram")?;
let defs = svg.defs()?;
let flt  = defs.filter("tiled-noise")?;

flt.turbulence(0.2, 2, 4.0, TurbulenceType::FractalNoise)?
    .set_attrs([("x", "0"), ("y", "0"), ("width", "20"), ("height", "20")])?;
flt.tile()?;

Ok::<(), svg_dom::Error>(())
```

***⚠️ The reference tile must be smaller than `tile`'s own output subregion, or tiling has no visible effect***

Two rectangles are in play: the selected input's *effective* subregion is the reference tile that gets repeated; `tile`'s own subregion is the *destination* rectangle the repetitions fill, which the specification defaults to the whole filter region (unlike an ordinary primitive) when left unset, as in the example above.

`tile` does not choose the reference tile itself: it is whatever the selected input's effective subregion resolves to.
An ordinary primitive's default subregion is the union of its own referenced inputs' subregions — so the immediate input to `tile` need not itself carry explicit `x`/`y`/`width`/`height`; it inherits a narrower default from an earlier primitive in the chain that was itself explicitly narrowed.
A generator with no referenced input, such as `feTurbulence` in the example above, has no earlier primitive to inherit from, so its own default subregion is the whole filter region instead, which is why the example narrows it directly.

Whichever primitive in the chain actually needs narrowing, its effective subregion must be smaller than `tile`'s own output subregion, or there is nothing smaller than the destination rectangle to repeat, and `tile`'s output is indistinguishable from its input passed through unchanged.

`convolve_matrix` applies an arbitrary matrix convolution, independent of every other primitive on this page — the same general-purpose operation behind sharpen, blur, emboss, and edge-detect kernels in any image editor:

```rust,no_run
use svg_dom::{SvgRoot, root::filter::EdgeMode};

let svg  = SvgRoot::attach("diagram")?;
let defs = svg.defs()?;
let flt  = defs.filter("sharpen")?;

#[rustfmt::skip]
let kernel = [
     0.0, -1.0,  0.0,
    -1.0,  5.0, -1.0,
     0.0, -1.0,  0.0,
];
flt.convolve_matrix(3, &kernel, 1.0, EdgeMode::Duplicate, false)?;

Ok::<(), svg_dom::Error>(())
```

The kernel above sums to `1.0`, so `divisor: 1.0` preserves overall brightness and the default `bias` of `0.0` needs no adjustment.
A kernel that sums to `0.0` instead (e.g. most edge-detect and emboss kernels) convolves a flat region of input to `0.0`, which clamps to black unless `bias` is set via the generic escape hatch (since it is not one of `convolve_matrix`'s own parameters) to shift that midpoint back into the visible range; `0.5` is the standard choice for a classic embossed-grey look:

```rust,no_run
use svg_dom::{SvgRoot, root::filter::EdgeMode};

let svg  = SvgRoot::attach("diagram")?;
let defs = svg.defs()?;
let flt  = defs.filter("emboss")?;

#[rustfmt::skip]
let kernel = [
    -2.0, -1.0, 0.0,
    -1.0,  1.0, 1.0,
     0.0,  1.0, 2.0,
];
flt.convolve_matrix(3, &kernel, 1.0, EdgeMode::Duplicate, true)?
    .set_attr("bias", "0.5")?;

Ok::<(), svg_dom::Error>(())
```

***⚠️ A `kernel_matrix` whose length does not equal `order * order` (or `order_x * order_y`) is not rejected — instead, no visible effect is produced***

Per the SVG spec, `<feConvolveMatrix>` *acts as a pass through filter* when `kernelMatrix`'s length does not match `order`'s — `in` renders unchanged, with no error raised by this crate or by the browser.

This is a defined, well-formed (albeit inert) rendering outcome, the same category of behaviour as `morphology_xy`'s zero-or-negative-component case above, so it is documented rather than validated — see `convolve_matrix`'s own doc comment for the full explanation of why.

Double-check `kernel_matrix.len()` against `order * order` yourself; a silently inert filter is easy to mistake for one that simply has no visible effect on the chosen input.

***⚠️ An `order`/`order_x`/`order_y` of `0`, unlike the two cases above, *is* rejected***

`u32` already rules out negative and fractional `order` values at compile time, but `0` is still representable, and — unlike a `kernelMatrix` length mismatch or a zero `divisor`, both of which have a defined fallback — the SVG spec gives `0` none.
A zero order is simply outside the attribute's permitted range.

`convolve_matrix` and `convolve_matrix_xy` therefore both check for a zero component and return `Error::InvalidConvolveMatrixOrder` before creating any element, rather than serializing a value the specification never assigns a meaning to.

***IMPORTANT***

Prefer small kernels, normally `3`×`3` or `5`×`5`, for `order`/`order_x`/`order_y`.
Convolution cost rises with the number of kernel entries (the square of `order` for a square kernel), and the SVG specification itself recommends small values, warning that larger ones may impose very high CPU overhead without a proportionate visual benefit.
A large `order` is not rejected — it is legal SVG and occasionally necessary — but every example above deliberately uses a `3`×`3` kernel, the size the specification itself suggests.

See [`../gaps.md`](../gaps.md) for the primitives still to be added.

## Region and Coordinate-Space Attributes

`set_x`, `set_y`, `set_width` and `set_height` set the filter region.

`set_filter_units` and `set_primitive_units` set its coordinate space and the space used by primitive attributes respectively.
Both take a `FilterUnits` value, and you should decide whether to use `UserSpaceOnUse` or `ObjectBoundingBox`.

The SVG default filter region (`-10% -10% 120% 120%` of the referencing element's bounding box, i.e. `filterUnits: ObjectBoundingBox`) can clip a wide blur; widen it explicitly for large `stdDeviation` values, e.g. `filter.set_x(-0.5)?; filter.set_y(-0.5)?; filter.set_width(2.0)?; filter.set_height(2.0)?;`.

***IMPORTANT***

Only expand the region wide enough to contain the intended effect.
The filter region is a hard clip on every intermediate offscreen buffer the browser rasterises while evaluating the filter's primitives, not just the final painted area, so an unnecessarily large region can increase both rasterisation work and temporary memory use.

***IMPORTANT***

* All filter ids must match the pattern `[A-Za-z_][A-Za-z0-9_-]*`.
* Ids are document-scoped, so they must be globally unique across all `<svg>` elements on the page.
