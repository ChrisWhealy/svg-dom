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
| `turbulence(base_frequency, num_octaves, seed, turbulence_type)` | `<feTurbulence>` | Generates Perlin-noise output — the only primitive here with no meaningful `in`, since it fabricates its own image rather than reading an upstream one. `turbulence_type` is a `TurbulenceType` (`Turbulence`, higher-contrast/marbled; `FractalNoise`, softer/cloud-like). `num_octaves` is a `u32`, ruling out the SVG spec's "negative is an error" case at compile time. Almost always paired with `displacement_map` — see the warning below. |
| `turbulence_xy(base_frequency_x, base_frequency_y, num_octaves, seed, turbulence_type)` | `<feTurbulence>` | As `turbulence`, but with independent horizontal/vertical base frequencies, writing the SVG two-number `baseFrequency="x y"` form. Unequal frequencies stretch the noise along whichever axis has the lower one — useful for wood-grain/brushed-metal textures that should read as directional rather than isotropic. |
| `displacement_map(in2, scale, x_channel_selector, y_channel_selector)` | `<feDisplacementMap>` | Warps this primitive's `in` input using `in2`'s `x_channel_selector`/`y_channel_selector` channel values (each a `Channel`) as a per-pixel displacement field, scaled by `scale`. `in2` is typically `turbulence`/`turbulence_xy`'s `result`. `Channel::Alpha` for both selectors is the SVG default and the usual choice when `in2` is noise. |

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
flt.displacement_map("noise", 24.0, Channel::Alpha, Channel::Alpha)?
    .set_attr("in", "SourceGraphic")?;

Ok::<(), svg_dom::Error>(())
```

***⚠️ `<feTurbulence>` reads no `in` at all***

Unlike every other primitive on this page, `turbulence`/`turbulence_xy` create their noise image, so there is no upstream input to chain from.
Consequently, `in` on the returned node has no effect.
Name the noise via `result` (as in the example above) so a later primitive's `in`/`in2` (typically `displacement_map`'s `in2`) can reference it.

See [`../gaps.md`](../gaps.md) for the primitives (`feMorphology`, `feTile`, and others) still to be added.

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
