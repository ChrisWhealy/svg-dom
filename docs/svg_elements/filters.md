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
| `gaussian_blur(std_deviation)` | `<feGaussianBlur>` | Blurs the input equally on both axes; larger `std_deviation` blurs more. Returns an `SvgNode`, so `in`/`result` (not yet wrapped by a named setter) can be set via `set_attr`. |
| `gaussian_blur_xy(std_deviation_x, std_deviation_y)` | `<feGaussianBlur>` | Blurs the input independently along the horizontal and vertical axes.  You must supply the SVG two-number `stdDeviation="x y"` in a single attribute write. Pass `0.0` for one axis to blur only along the other. |
| `offset(dx, dy)` | `<feOffset>` | Shifts the input by `(dx, dy)` user units. Returns an `SvgNode` for `in`/`result`, as above. |
| `merge(inputs)` | `<feMerge>` (with `<feMergeNode>` children) | Stacks each `&str` in `inputs` as one `<feMergeNode in="...">` child, in order (later entries painted on top). The standard way to layer a shadow underneath the original graphic. |
| `flood(color, opacity)` | `<feFlood>` | Fills the filter region with a solid `flood-color`/`flood-opacity`. Combined with `composite`, gives a shadow an independent colour rather than a blurred copy of the source graphic's own fill. |
| `composite(in2, operator)` | `<feComposite>` | Combines this primitive's `in` input with `in2` using a [Porter-Duff](https://en.wikipedia.org/wiki/Alpha_compositing) `CompositeOperator` (`Over`/`In`/`Out`/`Atop`/`Xor`/`Lighter`/`Arithmetic`). `operator: In` against a blurred alpha mask is the standard way to tint a shadow. |
| `drop_shadow(std_deviation, dx, dy, color, opacity)` | `<feDropShadow>` | Implements the browser-native shorthand for the entire `gaussian_blur` → `flood` → `composite` → `offset` → `merge` chain described below. Its result already has the original graphic merged on top: a `<filter>` containing only `drop_shadow` is a complete effect, so there is no need to call `merge` after it. |
| `color_matrix(matrix_type)` | `<feColorMatrix>` | Transforms colours via a `ColorMatrixType`: `Saturate(amount)`, `HueRotate(degrees)`, `LuminanceToAlpha`, or a full custom `Matrix([f64; 20])` (the fixed-size array rules out a wrong element count at compile time). Independent of the shadow primitives above — greyscale, saturation, and hue effects, not compositing. |

A drop-shadow can be constructed by chaining `gaussian_blur` + `flood` + `composite` + `offset` + `merge` together: blur the source alpha, composite a flood colour into the blurred mask, offset it, then merge it underneath the original — see the `<filter>` demo panel or `SvgFilter::composite`'s doc example.

For the common case (where the shadow is the same as the graphic it decorates), the above chaining is not needed; simply call `drop_shadow`.
The manual chain is only useful in situations where you need to name an intermediate result (`result="blur"`) for reuse by another primitive, or shadow one graphic while merging a different one on top.

A "poor man's" drop-shadow can be constructed by chaining `gaussian_blur` + `offset` + `merge`; however using these effects alone, you cannot have an independent shadow colour.

See [`../gaps.md`](../gaps.md) for the primitives (`feBlend`, `feTile`, and others) still to be added.

## Region and Coordinate-Space Attributes**

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
