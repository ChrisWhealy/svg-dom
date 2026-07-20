# Clipping and masking

[← Back to supported elements](README.md)

**Contents**

- [`<clipPath>`](#clippath)
- [`<mask>`](#mask)

## `<clipPath>`

A `<clipPath>` restricts the rendered region of any element that references it.
The browser paints only the parts of the referencing element that fall inside the union of all shapes placed inside the `<clipPath>`; everything lying outside this boundary remains invisible.

To live-append a path to the DOM, call `SvgDefs::clip_path(id)`.

To build a path from within a closure, call `SvgDefs::build_clip_path(id, closure)`.
The path will remain detached from the DOM until the closure succeeds.

Apply it to any element with `SvgNode::set_clip_path_ref(&clip)` or `SvgNode::set_clip_path("id")`.

Remove the clip with `SvgNode::remove_clip_path()`.

### Clip Shape Factories

Available on `SvgClipPath`: `rect`, `circle`, `ellipse`, `line`, `path`, `polyline`, `polygon`, `text`, `group`

### Coordinate Spaces

Controlled by `SvgClipPath::set_units(ClipPathUnits)`:

| Variant | Default | `clipPathUnits` | Meaning |
|---|:---:|---|---|
| `UserSpaceOnUse` | ✅ | `userSpaceOnUse` | The clip shape shares the same coordinate space as the SVG root, so you position the clip shape using the same coordinates as the element being clipped. |
| `ObjectBoundingBox` | | `objectBoundingBox` | The clip shape uses normalised coordinates (0.0 – 1.0) relative to the referencing element's bounding box; the clip shape scales automatically with the element. |

### Applying and Removing Clips on `SvgNode`

| Method | Description |
|---|---|
| `set_clip_path_ref(&clip)` | Apply by handle (preferred — no typo risk). |
| `set_clip_path("id")` | Apply by bare id string; `url(#...)` is added automatically. |
| `remove_clip_path()` | Remove the `clip-path` attribute, making the full element visible. |

## IMPORTANT

* All clip-path ids must match the pattern `[A-Za-z_][A-Za-z0-9_-]*`.
* Ids are document-scoped, so they must be globally unique across all `<svg>` elements on the page.
* A `<clipPath>` defined in one `<svg>`'s `<defs>` can only be referenced by elements inside the same document; it cannot be used across `\<iframe\>`s or shadow trees.

---

## `<mask>`

A `<mask>` reveals or hides parts of any element that references it based either on both the luminance and alpha values of the mask's own rendered content, or on the alpha-only value of the mask's own rendered content.

Unlike `<clipPath>`, which defines a hard, binary boundary defined purely by shape geometry (where something is either inside or outside the boundary), `<mask>` supports gradual transparency.
Each pixel of the referencing element is scaled by a value derived from the corresponding pixel of the mask's rendered content.

Under the default luminance mode, opaque white fully reveals the underlying element and opaque black fully obscures the underlying element.
Anything shade of grey in between (including gradients, and partial *opacity* on an otherwise-bright shape) partially reveals.

Transparent content hides fully regardless of colour.

To live-append a mask directly to a DOM element, use `SvgDefs::mask(id)`.

To create a mask from a closure, use `SvgDefs::build_mask(id, closure)`.
This mask will remained detached from the DOM until the closure succeeds.

A mask is applied to an element using `SvgNode::set_mask_ref(&mask)` or `SvgNode::set_mask("id")`.

Remove the mask with `SvgNode::remove_mask()`.

### Mask Shape Factories

Available on `SvgMask`: `rect`, `circle`, `ellipse`, `line`, `path`, `polyline`, `polygon`, `text`, `group`

### Coordinate Spaces

`SvgMask::set_mask_units(MaskUnits)` uses the coordinate system of the mask region's own position/size, and `SvgMask::set_mask_content_units(MaskUnits)` uses the coordinate space of the shapes inside the mask.

| Variant | Attribute value | Meaning |
|---|---|---|
| `UserSpaceOnUse` | `userSpaceOnUse` | Same coordinate system as the element being masked.<br>SVG default for `maskContentUnits`. |
| `ObjectBoundingBox` | `objectBoundingBox` | Normalised coordinates (0.0–1.0) relative to the referencing element's bounding box.<br>SVG default for `maskUnits`. |

### `mask-type`

Controlled by `SvgMask::set_mask_type(MaskType)`:

| Variant | Default | Meaning |
|---|:-:|---|
| `Luminance` | ✅ | The masked element is revealed according to the combination of teh mask content's perceived brightness *and* its opacity.<br>Transparent content hides fully no matter how bright its colour is. |
| `Alpha` |  | The masked element is revealed according to the mask content's alpha channel only; the colour is ignored.<br>In this case, only `fill-opacity`/`opacity` is significant. |

`mask-type` expresses the `<mask>` element's own preference; the element that *references* the mask can override it with its own `mask-mode` attribute (not currently wrapped by a named setter — use `SvgNode::set_attr("mask-mode", ...)`).
`mask-mode`'s default, `match-source`, honours `mask-type`, so this override only matters if a caller sets `mask-mode` explicitly.

### Applying and Removing Masks on `SvgNode`:

| Method | Description |
|---|---|
| `set_mask_ref(&mask)` | Apply a mask using its handle.<br>This is the preferred approach since there is no typo risk. |
| `set_mask("id")` | Apply by bare id string that is then translated into `url(#...)`.<br>Use this when the mask id is known statically and does not need to be checked for validity.<br>Will fail silently if the id references an element that is not a `<mask>`. |
| `remove_mask()` | Remove the `mask` attribute, making the full element visible. |

***IMPORTANT***

* All mask ids must match the pattern `[A-Za-z_][A-Za-z0-9_-]*`.
* Ids are document-scoped, so they must be globally unique across all `<svg>` elements on the page.

The mask region defaults to `-10% -10% 120% 120%` of the referencing element's bounding box (`maskUnits: ObjectBoundingBox`).

It creates a hard clip on the mask's own content and is applied before the luminance and alpha are evaluated.

A mask shape that extends further than this (a wide gradient sweep or a large soft-edged reveal) can be silently cut off.
To widen it explicitly, use `set_x`/`set_y`/`set_width`/`set_height` when that happens.

***WARNING***<br>Keep the region only as large as required.
As with a `<filter>` region, while evaluating the mask, the browser must create an offscreen buffer, so creating an unnecessarily large region may increase rendering and memory cost.
