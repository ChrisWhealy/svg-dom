# Supported SVG Elements

The following SVG elements are supported:

* `rect`
* `circle`
* `ellipse`
* `line`
* `polyline`
* `polygon`
* `path`
* `text`
* `g`
* `defs`
* `marker`
* `use`
* `image`
* `linearGradient` (with `stop`)
* `radialGradient` (with `stop`)
* `clipPath`

## `<defs>`

`<defs>` is the standard SVG container for reusable assets and can be obtained from `SvgRoot::defs()`.
All shape factory methods are available on `SvgDefs` for building inner content.

## `<marker>`

`<marker>` defines a reusable graphic (e.g. an arrowhead or a dot etc) rendered at the start, mid-point, or end of a stroked path and can be obtained from `SvgDefs::marker(id)`.

Apply it to any stroked element — `<line>`, `<path>`, `<polyline>`, `<polygon>` — via `SvgNode::set_marker_start`, `set_marker_mid`, or `set_marker_end`.
The `MarkerUnits` enum controls whether `markerWidth`/`markerHeight` are relative to `strokeWidth` (default) or user coordinates.

## `<image>`

`<image>` embeds a raster image (PNG, JPEG, WebP etc) or another SVG into the current document.
Obtain a handle via `SvgRoot::image(href, top_left, size)` or `SvgBatch::image(href, top_left, size)`.

- `href` accepts any URL the browser can fetch: a relative path, an absolute URL, or a `data:` URI.
  When using `data:image/svg+xml`, use base64 encoding to avoid percent-encoding `<`, `>`, and `#`.
- `top_left` and `size` define the display rectangle.
  Both width and height must be set; omitting either makes the image invisible.
- Control aspect-ratio handling with `set_attr("preserveAspectRatio", value)`:
  - `"xMidYMid meet"` — fit the whole image inside the box, adding letterbox bars if needed (default).
  - `"none"` — stretch to fill the box exactly, ignoring the source aspect ratio.
  - `"xMidYMid slice"` — scale up to fill the box and clip any overflow.
- To swap the image source after creation, call `SvgNode::set_href`.

## `<use>`

`<use>` stamps a copy of any element — typically one defined inside `<defs>` — into the rendered tree without duplicating the DOM node.
Obtain a handle via `SvgRoot::use_node(href, at)` or `SvgBatch::use_node(href, at)`.

- `href` is a local fragment reference such as `"#my-shape"` (the `id` attribute of the target element).
- `at` is an `(x, y)` offset in the parent coordinate system; pass `Point::origin()` to control positioning entirely through `transform`.
- Each returned `SvgNode` is independent: `transform`, `opacity`, `fill`, and other presentation attributes can be set per-copy without affecting the original.
- To change the referenced element after creation, call `SvgNode::set_href("#other-shape")`.

Any change to the original definition is immediately visible in all copies.
`<symbol>` is not yet supported; for now, define reusable content directly inside `<defs>` with a shape or group that carries an `id`.

## `<linearGradient>` / `<radialGradient>`

Gradient paint servers defined inside `<defs>` and referenced by shape `fill` or `stroke` attributes.
You can obtain such a paint server either from `SvgDefs::build_linear_gradient` or `build_radial_gradient`.
The live-append variants are `linear_gradient` and `radial_gradient`.

**`<linearGradient>`** paints a colour transition along a straight line.

- The axis runs from (`x1`, `y1`) to (`x2`, `y2`).
  Under the default `gradientUnits="objectBoundingBox"` these are fractions in the range `0.0` to `1.0` of the element's bounding box.
  If omitted, the default is a horizontal left-to-right gradient (SVG defaults: `x1=0`, `y1=0`, `x2=1`, `y2=0`).
- Use `set_gradient_transform("rotate(45, 0.5, 0.5)")` for a diagonal gradient without the need to compute trigonometric endpoint coordinates.
- A linear gradient can be applied to a shape's `fill` or `stroke` attributes using `SvgNode::set_fill_linear_gradient` or `SvgNode::set_stroke_linear_gradient`.

**`<radialGradient>`** radiates outward from some focal point at `fx / fy` through an outer circle centered at `cx / cy` and having a radius of `r`.

- SVG uses the defaults `cx=0.5`, `cy=0.5`, `r=0.5`.
  This positions the focal point at the centre of the outer circle.
- Move the focal point with `set_fx` / `set_fy` to create asymmetric "hot spot" or spotlight effects.
- `set_fr` sets the focal circle radius (SVG 2) for a hollow centre.
- Apply with `SvgNode::set_fill_radial_gradient` / `SvgNode::set_stroke_radial_gradient`.

**Shared API** for both gradient types:

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

## `<clipPath>`

A `<clipPath>` restricts the rendered region of any element that references it.
The browser paints only the parts of the referencing element that fall inside the union of all shapes placed inside the `<clipPath>`; everything outside is invisible.

Obtain one from `SvgDefs::clip_path(id)` (live-append) or `SvgDefs::build_clip_path(id, closure)` (detached until the closure succeeds).
Apply it to any element with `SvgNode::set_clip_path_ref(&clip)` or `SvgNode::set_clip_path("id")`.
Remove the clip with `SvgNode::remove_clip_path()`.

**Clip shape factories** available on `SvgClipPath`:
`rect`, `circle`, `ellipse`, `line`, `path`, `polyline`, `polygon`, `text`, `group`

**Coordinate spaces** — controlled by `SvgClipPath::set_units(ClipPathUnits)`:

| Variant | `clipPathUnits` | Meaning |
|---|---|---|
| `UserSpaceOnUse` (default) | `userSpaceOnUse` | Clip shapes use SVG root coordinates; position them at the same coordinates as the element being clipped. |
| `ObjectBoundingBox` | `objectBoundingBox` | Clip shapes use normalised coordinates (0.0–1.0) relative to the referencing element's bounding box; the clip scales automatically with the element. |

**Applying and removing clips** on `SvgNode`:

| Method | Description |
|---|---|
| `set_clip_path_ref(&clip)` | Apply by handle (preferred — no typo risk). |
| `set_clip_path("id")` | Apply by bare id string; `url(#...)` is added automatically. |
| `remove_clip_path()` | Remove the `clip-path` attribute, making the full element visible. |

***IMPORTANT***

* All clip-path ids must match the pattern `[A-Za-z_][A-Za-z0-9_-]*`.
* Ids are document-scoped, so they must be globally unique across all `<svg>` elements on the page.
* A `<clipPath>` defined in one `<svg>`'s `<defs>` can only be referenced by elements inside the same document; it cannot be used across iframes or shadow trees.

# Implemented Tree operations

- `remove()` — detach a node from the DOM
- `insert_before()` — z-order control without rebuilding
- `clear()` — remove all children of a node (e.g. to redraw a `<g>` from scratch)
- `replace_with()` — swap one node for another in place
- `parent()` — navigate up to the containing SVG element (returns an independent, non-factory handle)

# Event coverage

Managed wrappers now cover the SVG interaction events expected by ordinary application code:

* click/double-click/context menu,
* mouse movement and button state,
* pointer lifecycle,
* wheel,
* touch,
* keyboard,
* focus/blur,
* drag-and-drop,
* a generic `on_event` escape hatch for event types not covered by a named wrapper, and
* `on_event_once` — a generic one-shot variant; accepts any event type `E` via an `instanceof` cast at runtime.
* Typed one-shot wrappers for every named event: `on_click_once`, `on_pointerdown_once`, `on_pointerenter_once`, `on_pointerleave_once`, and equivalents for all other named events.
  These bake in the correct event type so the `instanceof` mismatch footgun cannot occur.
* Passive variants for the three high-frequency scroll events — `on_wheel_passive`, `on_touchstart_passive`, and `on_touchmove_passive` — registered with `{ passive: true }` so the compositor thread is never blocked.
  Any `prevent_default()` call made inside a passive handler is silently ignored by the browser.
  If you do need to suppress the default scroll or touch behaviour, then use the non-passive sibling instead.

Prefer `pointerenter` / `pointerleave` for hover behaviour because they do not bubble through child elements.
The legacy `mouseover` / `mouseout` wrappers remain available for compatibility reasons, but have been marked as deprecated.

# Implemented Attribute helpers

- Transform helpers — `set_translate`, `set_rotate`, `set_rotate_about`, `set_scale`, `set_scale_xy`, `set_translate_scale`, and `set_transform_fmt` for arbitrary transforms (all reuse a caller-owned scratch buffer)
- Updating `<text>` content after creation — `set_text`, plus the buffer-reusing `set_text_fmt` / `set_text_display`
- Allocation-light numeric attribute writes — `set_attr_display`, and the redundant-write helpers `set_attr_if_changed` / `CachedAttr`
