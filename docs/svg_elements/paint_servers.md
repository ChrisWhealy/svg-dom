# Paint servers

[← Back to supported elements](README.md)

**Contents**

- [`<linearGradient>` / `<radialGradient>`](#lineargradient--radialgradient)
- [`<pattern>`](#pattern)

## `<linearGradient>` / `<radialGradient>`

Gradient paint servers defined inside `<defs>` and referenced by shape `fill` or `stroke` attributes.

You can obtain such a paint server either from `SvgDefs::build_linear_gradient` or `build_radial_gradient`.

The live-append variants are `linear_gradient` and `radial_gradient`.

### `<linearGradient>`

This paints a colour transition along a straight line.

- The axis runs along the line from (`x1`, `y1`) to (`x2`, `y2`).
  Under the default `gradientUnits="objectBoundingBox"`, these are fractions in the range `0.0` to `1.0` of the element's bounding box.

  If omitted, the default is a horizontal left-to-right gradient (specification defaults: `x1=0%`, `y1=0%`, `x2=100%`, `y2=0%`).
  These percentages only coincide with the bare numbers `0`/`1` once resolved under `objectBoundingBox` units.
  Under `userSpaceOnUse`, a percentage resolves against the viewport instead, so keep the spec's percentage notation in mind rather than assuming it is generally interchangeable with a unitless number.

- Use `set_gradient_transform("rotate(45, 0.5, 0.5)")` for a diagonal gradient without the need to compute trigonometric endpoint coordinates.

- A linear gradient can be applied to a shape's `fill` or `stroke` attributes using `SvgNode::set_fill_linear_gradient` or `SvgNode::set_stroke_linear_gradient`.

### `<radialGradient>`

This a gradient that radiates outward from the focal point at `fx / fy` towards an outer circle centered at `cx / cy` and having a radius of `r`.

- The specification defaults are `cx=50%`, `cy=50%`, `r=50%` (these only coincide with the bare numbers `0.5` once resolved under `objectBoundingBox` units).
  This positions the focal point at the centre of the outer circle.

- Move the focal point with `set_fx` / `set_fy` to create an asymmetric "hot spot" or spotlight effect.

- `set_fr` sets the radius of the focal/start circle (SVG 2).

  The gradient's `0%` stop is mapped to that circle's perimeter, and its interior is painted with the first stop's colour; however, `fr` does not inherently create a hole.
  A hollow-looking centre is created from the stop colours themselves, e.g. a transparent first stop.

- Apply with `SvgNode::set_fill_radial_gradient` / `SvgNode::set_stroke_radial_gradient`.

### Shared API

Applicable to both gradient types:

| Function | Description
|---|---|
| `add_stop(offset, color)` | Add a `<stop>` at `offset` (0.0–1.0) with `stop-color` and full opacity.
| `add_stop_opacity(offset, color, opacity)` | As above, but with explicit `stop-opacity`.
| `set_gradient_units(GradientUnits)` | Switch between `ObjectBoundingBox` (default) and `UserSpaceOnUse`.
| `set_spread_method(SpreadMethod)` | `Pad` (default), `Reflect`, or `Repeat` outside the stop range.
| `set_gradient_transform(transform)` | Arbitrary SVG transform applied to the gradient coordinate system.
| `set_attr` / `set_attrs` / `set_attr_display` | Generic escape hatches for other attributes.
| `id()` / `set_id()` | Cached id management; renaming does not retroactively update shape references.

***IMPORTANT***

* All gradient ids must match the pattern `[A-Za-z_][A-Za-z0-9_-]*`.
* The ids used by the SVG paint-server are document-scoped not SVG-element-scoped; therefore, they must be globally unique across all `<svg>` elements on the page (using `url(#id)`).

A fully qualified prefix such as `"my-app-sky-gradient"` is a practical guard against collisions.

---

## `<pattern>`

A `<pattern>` element defines a tiled graphic that is painted repeatedly to fill (or stroke) the region of any element that references it via `fill="url(#id)"` or `stroke="url(#id)"`.
Like `<clipPath>`, it is a shape container; but unlike the linear or radial gradients, each tile is a full rendered graphic rather than a colour interpolation.

To live-append a `<pattern>` directly to the DOM, call `SvgDefs::pattern(id)`

To generate a `<pattern>` using a closure, call `SvgDefs::build_pattern(id, closure)`.
The pattern will remain detached from the DOM until the closure succeeds.

Apply it to any element with `SvgNode::set_fill_pattern_ref(&pat)` or `SvgNode::set_fill_pattern("id")`, or their stroke equivalents.

### `SvgPattern` API Overview

| Method | Attribute | Description |
|---|---|---|
| `set_x(v)` | `x` | Horizontal offset of the tile origin |
| `set_y(v)` | `y` | Vertical offset of the tile origin |
| `set_width(v)` | `width` | Width of a single tile |
| `set_height(v)` | `height` | Height of a single tile |
| `set_pattern_units(u)` | `patternUnits` | Coordinate space for `x`/`y`/`width`/`height` |
| `set_pattern_content_units(u)` | `patternContentUnits` | Coordinate space for shapes inside the tile |
| `set_pattern_transform(t)` | `patternTransform` | SVG transform applied to the tile before tiling |
| `set_view_box(x, y, w, h)` | `viewBox` | Internal coordinate system for tile content |
| `set_id(&mut self, id)` | `id` | Renames the pattern (updates both DOM and cached id) |
| `set_attr(name, value)` | — | Generic setter for unlisted attributes |

All shape factory methods are available on `SvgPattern`.

### Applying Patterns on `SvgNode`

| Method | Description |
|---|---|
| `set_fill_pattern_ref(&pat)` | Apply by handle (preferred, as there is no risk of typos). |
| `set_fill_pattern("id")` | Apply by bare id string; `url(#...)` is added automatically. |
| `set_stroke_pattern_ref(&pat)` | Apply to stroke by handle. |
| `set_stroke_pattern("id")` | Apply to stroke by bare id string. |

### Coordinate Systems

These are controlled by `PatternUnits` and are used for both `patternUnits` and `patternContentUnits`.

| Variant | SVG value | Meaning |
|---|---|---|
| `UserSpaceOnUse` | `userSpaceOnUse` | Tile dimensions use the same coordinate space as the referencing element. |
| `ObjectBoundingBox` | `objectBoundingBox` | Tile dimensions are fractions of the referencing element's bounding box (SVG default for `patternUnits`). |

**Example**:

```rust,no_run
use svg_dom::{SvgRoot, root::{pattern::PatternUnits, utils::{Point, Size}}};

let svg = SvgRoot::attach("diagram")?;

svg.build_defs(|d| {
    d.build_pattern("checker", |p| {
        p.set_pattern_units(PatternUnits::UserSpaceOnUse)?;
        p.set_width(20.0)?;
        p.set_height(20.0)?;
        p.rect(Point::new(0.0, 0.0), Size::new(20.0, 20.0))?.set_fill("teal")?;
        p.rect(Point::new(0.0, 0.0), Size::new(10.0, 10.0))?.set_fill("white")?;
        p.rect(Point::new(10.0, 10.0), Size::new(10.0, 10.0))?.set_fill("white")?;
        Ok(())
    })?;
    Ok(())
})?;

let rect = svg.rect(Point::origin(), Size::new(300.0, 200.0))?;
rect.set_fill_pattern("checker")?;
Ok::<(), svg_dom::Error>(())
```

***IMPORTANT***

* All pattern ids must match the pattern `[A-Za-z_][A-Za-z0-9_-]*`.
* Ids are document-scoped, so they must be globally unique across all `<svg>` elements on the page.
* Always use `SvgPattern::set_id` to rename a pattern after construction; `set_attr("id", ...)` will be rejected with `Error::ReservedAttribute` to protect the cached value.
