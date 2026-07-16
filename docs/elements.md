# Supported SVG Elements

The following SVG elements are supported:

* `circle`
* `clipPath`
* `defs`
* `ellipse`
* `filter` (with `feGaussianBlur`, `feOffset`, `feMerge`/`feMergeNode`, `feFlood`, `feComposite`, `feDropShadow`, `feColorMatrix`)
* `g`
* `image`
* `line`
* `linearGradient` (with `stop`)
* `marker`
* `pattern`
* `rect`
* `path` (with a type-safe `PathDef` builder — see below — as an alternative to hand-written `d` strings)
* `polygon`
* `polyline`
* `radialGradient` (with `stop`)
* `symbol`
* `text` (with `tspan`, `textPath`)
* `use`

# Implemented Tree operations

- `remove()` — detach a node from the DOM
- `insert_before()` — z-order control without rebuilding
- `clear()` — remove all children of a node (e.g. to redraw a `<g>` from scratch)
- `replace_with()` — swap one node for another in place
- `parent()` — navigate up to the containing SVG element (returns an independent, non-factory handle)
- `first_child()` / `last_child()` / `next_sibling()` / `previous_sibling()` — navigate down/across without having kept a handle to the target (returns independent, non-factory handles, like `parent()`)
- `children()` — every SVG child element, in document order (independent, non-factory handles)
- `query_selector()` / `query_selector_all()` — find descendant(s) anywhere in the subtree by CSS selector, including by attribute (independent, non-factory handles)

***IMPORTANT***

Every handle returned by the tree navigation and query methods above is a **fresh, independent** owner of its element, not a reference to whatever handle originally created it.
This is the same caveat that applies to the use of `parent()`.

In particular, You should not register event listeners through one of these handles; see `SvgNode::parent`'s doc comment for the full explanation.

All non-SVG matches (for example HTML content inside a `<foreignObject>`) are silently skipped rather than returned.

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

- Transform helpers — `set_translate`, `set_rotate`, `set_rotate_about`, `set_scale`, `set_scale_xy`, `set_translate_scale`, `set_matrix`/`set_matrix_precise` (2D affine matrix via a role-named `Matrix2D { h_scale, v_scale, h_skew, v_skew, h_trans, v_trans }`, for skew/shear or any transform not expressible via the named helpers above — `set_matrix` is quantised to 3/1 decimal places for compact hot-path output, `set_matrix_precise` uses exact shortest-round-trip formatting for matrices computed elsewhere), and `set_transform_fmt` for anything else (all reuse a caller-owned scratch buffer)
- Updating `<text>` content after creation — `set_text`, plus the buffer-reusing `set_text_fmt` / `set_text_display`
- Allocation-light numeric attribute writes — `set_attr_display`, and the redundant-write helpers `set_attr_if_changed` / `CachedAttr`
- `SvgRoot::set_view_box(x, y, width, height)` — the root `<svg>`'s internal coordinate system, independent of `set_viewport`'s `width`/`height`. `SvgSymbol`, `SvgPattern`, and `SvgMarker` have the same method for their own `viewBox`; see [`<symbol>`](#symbol), [`<pattern>`](#pattern), and [`<marker>`](#marker) below.
- CSS class manipulation on `SvgNode` — `add_class`, `remove_class`, `toggle_class`, `set_class_enabled` (deterministic set/clear via `classList.toggle(token, force)`), `has_class`, backed by the DOM `classList` API.

---

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

---

## `<defs>`

`<defs>` is the standard SVG container for reusable assets and can be obtained from `SvgRoot::defs()`.
All shape factory methods are available on `SvgDefs` for building inner content.

---

## `<filter>`

A `<filter>` applies raster effects (blur, colour manipulation, compositing, ...) to any element that references it.
The browser evaluates the filter's primitive children in document order and paints the result in place of the referencing element.

Obtain one from `SvgDefs::filter(id)` (live-append) or `SvgDefs::build_filter(id, closure)` (detached until the closure succeeds).
Apply it to any element with `SvgNode::set_filter_ref(&filter)` or `SvgNode::set_filter("id")`.
Remove the filter with `SvgNode::remove_filter()`.

**Primitive factories** available on `SvgFilter`:

| Method | Element | Description |
|---|---|---|
| `gaussian_blur(std_deviation)` | `<feGaussianBlur>` | Blurs the input equally on both axes; larger `std_deviation` blurs more. Returns an `SvgNode`, so `in`/`result` (not yet wrapped by a named setter) can be set via `set_attr`. |
| `gaussian_blur_xy(std_deviation_x, std_deviation_y)` | `<feGaussianBlur>` | Independent horizontal/vertical deviation, writing the SVG two-number `stdDeviation="x y"` form in a single attribute write. Pass `0.0` for one axis to blur only along the other. |
| `offset(dx, dy)` | `<feOffset>` | Shifts the input by `(dx, dy)` user units. Returns an `SvgNode` for `in`/`result`, as above. |
| `merge(inputs)` | `<feMerge>` (with `<feMergeNode>` children) | Stacks each `&str` in `inputs` as one `<feMergeNode in="...">` child, in order (later entries painted on top). The standard way to layer a shadow underneath the original graphic. |
| `flood(color, opacity)` | `<feFlood>` | Fills the filter region with a solid `flood-color`/`flood-opacity`. Combined with `composite`, gives a shadow an independent colour rather than a blurred copy of the source graphic's own fill. |
| `composite(in2, operator)` | `<feComposite>` | Combines this primitive's `in` input with `in2` using a Porter-Duff `CompositeOperator` (`Over`/`In`/`Out`/`Atop`/`Xor`/`Lighter`/`Arithmetic`). `operator: In` against a blurred alpha mask is the standard way to tint a shadow. |
| `drop_shadow(std_deviation, dx, dy, color, opacity)` | `<feDropShadow>` | Implements the browser-native shorthand for the entire `gaussian_blur` → `flood` → `composite` → `offset` → `merge` chain described below. Its result already has the original graphic merged on top: a `<filter>` containing only `drop_shadow` is a complete effect, so there is no need to call `merge` after it. |
| `color_matrix(matrix_type)` | `<feColorMatrix>` | Transforms colours via a `ColorMatrixType`: `Saturate(amount)`, `HueRotate(degrees)`, `LuminanceToAlpha`, or a full custom `Matrix([f64; 20])` (the fixed-size array rules out a wrong element count at compile time). Independent of the shadow primitives above — greyscale, saturation, and hue effects, not compositing. |

`gaussian_blur` + `flood` + `composite` + `offset` + `merge` together build a *true* tinted, opacity-controlled drop shadow: blur the source alpha, composite a flood colour into the blurred mask, offset it, then merge it underneath the original — see the `<filter>` demo panel or `SvgFilter::composite`'s doc example.
For the common case (shadow of the same graphic it decorates), `drop_shadow` produces the identical effect in one call; reach for the manual chain instead when you need to name intermediate results (`result="blur"`) for another primitive to reuse, or shadow one graphic while merging a different one on top.

`gaussian_blur` + `offset` + `merge` alone still work for the simpler case of a blurred *copy* of the graphic to produce an "almost drop shadow", but using these effects alone, you cannot have an independent shadow colour.

See `docs/gaps.md` for the primitives (`feBlend`, `feTile`, and others) still to be added.

**Region and coordinate-space attributes**: `set_x`/`set_y`/`set_width`/`set_height` set the filter region, and `set_filter_units`/`set_primitive_units` (both taking a `FilterUnits`, `UserSpaceOnUse`/`ObjectBoundingBox`) set its coordinate space and the space used by primitive attributes respectively.
The SVG default filter region (`-10% -10% 120% 120%` of the referencing element's bounding box, i.e. `filterUnits: ObjectBoundingBox`) can clip a wide blur; widen it explicitly for large `stdDeviation` values, e.g. `filter.set_x(-0.5)?; filter.set_y(-0.5)?; filter.set_width(2.0)?; filter.set_height(2.0)?;`.

***IMPORTANT***

Expand the region only enough to contain the intended effect.
The filter region is a hard clip on every intermediate offscreen buffer the browser rasterises while evaluating the filter's primitives, not just the final painted area, so an unnecessarily large region can increase both rasterisation work and temporary memory use.

***IMPORTANT***

* All filter ids must match the pattern `[A-Za-z_][A-Za-z0-9_-]*`.
* Ids are document-scoped, so they must be globally unique across all `<svg>` elements on the page.

---

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

---

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

---

## `<marker>`

`<marker>` defines a reusable graphic (e.g. an arrowhead or a dot etc) rendered at the start, mid-point, or end of a stroked path and can be obtained from `SvgDefs::marker(id)`.

Apply it to any stroked element — `<line>`, `<path>`, `<polyline>`, `<polygon>` — via `SvgNode::set_marker_start`, `set_marker_mid`, or `set_marker_end`.
The `MarkerUnits` enum controls whether `markerWidth`/`markerHeight` are relative to `strokeWidth` (default) or user coordinates.

`set_view_box(x, y, width, height)` establishes an internal coordinate system for the marker's content, mapped onto the `markerWidth`/`markerHeight` viewport — the same `viewBox` relationship `<symbol>`/`<use>` has, validated the same way (`Error::InvalidViewBox` on a non-finite component or a negative `width`/`height`). `preserveAspectRatio` has no dedicated setter for `<marker>`; use `set_attr("preserveAspectRatio", value)`.

---

## `<path>`

A `<path>` is created either from a hand-written `d` string (`SvgRoot::path(d)`) or, type-safely, from a sequence of typed `PathDef` segments (`SvgRoot::path_from_defs(&[PathDef])`).

A hand-written `d` string is free text: any typos will be silently accepted by the SVG parser, which will then just stop rendering at the first bad token rather than reporting an error.
`path_from_defs` removes that failure mode for individual commands — the `d` attribute is built internally from `PathDef` values, so a mistyped command letter, wrong argument count, or invalid arc flag can never be constructed.

That guarantee is about individual commands, yet it is still possible to create a sequence of them that fails to be a valid path:

- A non-empty path must start with a moveto (`M`/`m`); a browser silently renders nothing for a path that starts with anything else. 
   `path_from_defs`, `SvgNode::set_d_from_defs`, and the `SvgAttrs` / `AnimationFrame` `d_from_defs` methods all check this (requiring an O(1) look at the first command) and return `Error::InvalidPathData` if it fails.
   `build_d` / `write_d` (and their `_fixed` siblings) do **not** check this, since they may legitimately be used to build path-data *fragments* rather than a complete, standalone path.

- Coordinates are unconstrained `f64` values, so nothing stops `f64::NAN` or `f64::INFINITY` from being supplied — the SVG number grammar has no representation for either, so `PathDef` cannot format a valid path from one.
   No function in the path API checks for this, since doing so would mean visiting every numeric argument of every command on every call, including the buffer-reusing per-frame ones.
   Validate with `f64::is_finite()` before constructing a `PathDef` if your coordinates come from a calculation (division, trigonometry) that could produce one.

**Building path data:**

| Type | Purpose |
|---|---|
| `PathDef` | One path-data command: `Abs(PathDefAbsolute)` or `Rel(PathDefRelative)`. Absolute and relative commands can be freely mixed in the same sequence, exactly as real SVG path data allows. |
| `PathDefAbsolute` / `PathDefRelative` | The ten SVG path commands (`MoveTo`, `LineTo`, `HorizontalLineTo`, `VerticalLineTo`, `CubicBezierTo`, `SmoothCubicBezierTo`, `QuadraticBezierTo`, `SmoothQuadraticBezierTo`, `EllipticalArcTo`, `ClosePath`) in absolute or relative form respectively. |
| `EllipticalArc` | Named-field parameters for an arc segment — `radii`, `x_axis_rotation`, `size`, `sweep`, `to` — instead of a five-element tuple. |
| `ArcSize` | `Small` / `Large` — the SVG `large-arc-flag`, replacing a bare `bool` that gives no clue at the call site which arc solution it selects. |
| `ArcSweep` | `CounterClockwise` / `Clockwise` — the SVG `sweep-flag`, replacing the second bare `bool`. |

**Creating and updating paths:**

| Method | Available on | Effect |
|---|---|---|
| `path(d)` | `SvgRoot`, `SvgBatch`, `SvgDefs`, `SvgClipPath`, `SvgMarker`, `SvgPattern`, `SvgSymbol` | Creates a `<path>` from a raw `d` string. |
| `path_from_defs(&[PathDef])` | Same set of types | Creates a `<path>` from typed segments, writing `d` through the factory's own retained `SvgAttrs` buffer — no extra allocation beyond the first call. |
| `SvgNode::set_d(d)` | Any `SvgNode` | Updates an existing `<path>`'s `d` string. |
| `SvgNode::set_d_from_defs(&[PathDef])` | Any `SvgNode` | Updates an existing `<path>`'s `d` from typed segments. Allocates a fresh `String` per call; consequently, this should only be used for occasional updates. See below for the hot-path alternatives. |
| `build_d(&[PathDef])` | Free function | Builds a `d` string without creating or updating any element — useful for composing a path in pieces. |
| `write_d(&mut String, &[PathDef])` | Free function | The buffer-reusing counterpart to `build_d`, for a hot path that rebuilds a curve every frame. |

**Allocation-light updates**, mirroring the existing `points`/`points_fixed` pattern:

| Method | Effect |
|---|---|
| `SvgAttrs::d_from_defs(&node, &[PathDef])` | Writes `d` through `SvgAttrs`'s reusable scratch buffer. |
| `AttrWriter::d_from_defs(&[PathDef])` | The chainable-writer equivalent, via `node.attrs(&mut attrs)`. |
| `AnimationFrame::set_d_from_defs(&node, &[PathDef])` | The per-frame equivalent, for use inside `AnimationLoop::start_with_frame`. |
| `write_d_fixed(&mut String, &[PathDef], dps)` / `build_d_fixed(&[PathDef], dps)` | Like `write_d`/`build_d`, but every coordinate, length, and rotation angle is rounded to `dps` decimal places (clamped to 20). The elliptical-arc flags are never rounded — the SVG grammar requires them to stay a bare `0`/`1`. |
| `SvgAttrs::d_from_defs_fixed` / `AttrWriter::d_from_defs_fixed` / `AnimationFrame::set_d_from_defs_fixed` | The fixed-precision counterparts of the three methods above, mirroring `points_fixed`/`set_points_fixed`. Use these for path data computed during an animation, where the default shortest-round-trip formatting would otherwise carry more digits than needed. |

### Example

```rust,no_run
use svg_dom::{ArcSize, ArcSweep, EllipticalArc, PathDef, PathDefAbsolute, SvgRoot, root::utils::Point};

let svg = SvgRoot::attach("diagram")?;
let arc = svg.path_from_defs(&[
    PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(10.0, 65.0))),
    PathDef::Abs(PathDefAbsolute::EllipticalArcTo(EllipticalArc {
        radii: Point::new(60.0, 60.0),
        x_axis_rotation: 0.0,
        size: ArcSize::Large,
        sweep: ArcSweep::Clockwise,
        to: Point::new(130.0, 65.0),
    })),
])?;
arc.set_fill("none")?;
arc.set_stroke("coral")?;
Ok::<(), svg_dom::Error>(())
```

---

## `<pattern>`

A `<pattern>` element defines a tiled graphic that is painted repeatedly to fill (or stroke) the region of any element that references it via `fill="url(#id)"` or `stroke="url(#id)"`.
Like `<clipPath>`, it is a shape container; unlike gradients, each tile is a full rendered graphic rather than a colour interpolation.

Obtain one from `SvgDefs::pattern(id)` (live-append) or `SvgDefs::build_pattern(id, closure)` (detached until the closure succeeds).
Apply it to any element with `SvgNode::set_fill_pattern_ref(&pat)`, `SvgNode::set_fill_pattern("id")`, or their stroke equivalents.

**API** overview on `SvgPattern`:

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

All shape factory methods (`rect`, `circle`, `ellipse`, `line`, `path`, `polyline`, `polygon`, `text`, `group`) are available on `SvgPattern`.

**Applying patterns** on `SvgNode`:

| Method | Description |
|---|---|
| `set_fill_pattern_ref(&pat)` | Apply by handle (preferred — no typo risk). |
| `set_fill_pattern("id")` | Apply by bare id string; `url(#...)` is added automatically. |
| `set_stroke_pattern_ref(&pat)` | Apply to stroke by handle. |
| `set_stroke_pattern("id")` | Apply to stroke by bare id string. |

**Coordinate systems** — controlled by `PatternUnits` (used for both `patternUnits` and `patternContentUnits`):

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

---

## `<symbol>`

A `<symbol>` defines a reusable viewport.
Unlike a plain `<g>` in `<defs>`, it can carry its own `viewBox` and `preserveAspectRatio`.
The browser scales the symbol's content to fit the `<use>` element's `width` and `height`, exactly as it would an embedded `<svg>` &mdash; so the same definition renders correctly at any size with no manual rescaling.

**API** — obtain a handle via `SvgDefs::symbol(id)` or the transactional `SvgDefs::build_symbol(id, closure)`:

| Method | Description |
|---|---|
| `set_view_box(x, y, w, h)` | Establishes the symbol's internal coordinate space |
| `set_preserve_aspect_ratio(value)` | Controls alignment / clipping when the `<use>` dimensions differ from the `viewBox` aspect ratio |
| `set_id(&mut self, id)` | Renames the symbol (updates both the DOM and the cached id) |
| `set_attr(name, value)` | Generic setter for unlisted attributes (`class`, `style`, `overflow` …) |

All shape factory methods (`rect`, `circle`, `ellipse`, `line`, `path`, `polyline`, `polygon`, `text`, `group`) are available on `SvgSymbol`.

**Stamping copies** — pass the symbol's id (prefixed with `#`) to `SvgRoot::use_node`:

```rust,no_run
defs.build_symbol("badge", |s| {
    s.set_view_box(0.0, 0.0, 40.0, 40.0)?;
    s.circle(Point::new(20.0, 20.0), 18.0)?.set_fill("steelblue")?;
    Ok(())
})?;

// Each <use> can have its own width/height; the viewBox scales the content automatically.
svg.use_node("#badge", Point::new(10.0, 10.0))?.set_attr("width", "40")?;
svg.use_node("#badge", Point::new(60.0, 10.0))?.set_attr("width", "80")?;
```

**id rules** — symbol ids follow the same allow-list as markers and gradients: `[A-Za-z_][A-Za-z0-9_-]*`.
A non-conforming id causes `Error::InvalidSymbolId` to be raised before any DOM call is made.

Always use `SvgSymbol::set_id` to rename a symbol after construction; `set_attr("id", ...)` will be rejected with `Error::ReservedAttribute` to protect the cached value.

---

## `<text>` presentation attributes

The `<text>` factory (`SvgRoot::text`, `SvgBatch::text`) returns a plain `SvgNode`.
Four typed helpers are available on any `SvgNode` for styling text:

| Method | Attribute | Type |
|---|---|---|
| `set_font_family(family)` | `font-family` | Any CSS font-family string |
| `set_font_size(size)` | `font-size` | `f64` in user units |
| `set_text_anchor(TextAnchor)` | `text-anchor` | `TextAnchor::{Start, Middle, End}` |
| `set_dominant_baseline(DominantBaseline)` | `dominant-baseline` | `DominantBaseline::{Auto, Alphabetic, Middle, …}` |

**`TextAnchor`** controls which part of the string aligns with the `x` coordinate.
`Start` (default) places the beginning of the text at `x`; `Middle` centres it; `End` places the end.

**`DominantBaseline`** controls which font baseline aligns with the `y` coordinate.
The default (`Auto`/`Alphabetic`) places the alphabetic baseline on `y`, so ascenders rise above it.
Use `Middle` or `Central` to vertically centre text on a coordinate.
Use `Hanging` for scripts (Devanagari, Tibetan, etc.) whose bodies hang from the top of the line box.

---

## `<textPath>`

`<textPath>` glues a `<text>` string to the outline of a `<path>` (or, per SVG2, a basic shape).
In other words, the baseline of the letters follow the outline defined by the path instead of a straight line.

Obtain a handle by calling `text_path(href, content)` on any `SvgNode` that wraps a `<text>` element (or another `<tspan>`/`<textPath>`).

| Method | Effect |
|---|---|
| `node.text_path(href, content)` | Appends a `<textPath>` with `content`, following the path referenced by `href`. |
| `node.set_start_offset(offset)` | Sets `startOffset` — the distance in user units along the path where the text begins. |
| `node.set_text_path_method(TextPathMethod)` | Sets `method` — `Align` (default) rotates whole glyphs onto the path; `Stretch` distorts glyph outlines to match its curvature. |
| `node.set_text_path_spacing(TextPathSpacing)` | Sets `spacing` — `Auto` (default) compensates spacing for curvature; `Exact` uses the font's natural advance widths. |
| `node.set_text_path_side(TextPathSide)` | Sets the SVG2 `side` attribute — `Left` (default) or `Right` of the path. |

- `href` is a local fragment reference such as `"#wave"` (the `id` attribute of the target `<path>`).
- The referenced path is typically defined inside `<defs>`, or given no fill/stroke, so only the text is visible rather than the guide geometry.
- All text styling helpers (`set_fill`, `set_font_size`, `set_font_family`) work on the returned `SvgNode` exactly as they do for `<tspan>`.
- To offset by a percentage of the path length instead of an absolute distance, call `set_attr("startOffset", "50%")` directly.

**Browser support:** `side` is an SVG2 addition; verify it renders as expected on every browser you target before relying on `TextPathSide::Right` in production.

---

## `<tspan>`

`<tspan>` is an inline text span that lives inside a `<text>` element (or another `<tspan>`).
Each span can override any text presentation attribute inherited from its parent, making it the standard mechanism for multi-line text and mixed-style inline text in SVG.

Obtain a span by calling `tspan` or `tspan_dy` on any `SvgNode` that wraps a `<text>` or `<tspan>` element:

| Method | Effect |
|---|---|
| `node.tspan(content)` | Appends a `<tspan>` with `content`; inherits position from the parent. |
| `node.tspan_dy(dy, content)` | Same but also sets `dy` — advances the text position `dy` user units downward before rendering. |
| `node.set_dy(dy)` | Sets the `dy` attribute on an existing node. |
| `node.set_dx(dx)` | Sets the `dx` attribute on an existing node (horizontal offset). |

All text styling helpers (`set_fill`, `set_font_size`, `set_font_family`, `set_text_anchor`, `set_dominant_baseline`) work on the returned `SvgNode` and override the inherited value for that span only.

**Multi-line text:** create a `<text>` with an empty content string (`""`), add the first line as a `tspan`, then add subsequent lines with `tspan_dy` and a consistent `dy` value equal to the desired line height.

**Mixed-style inline text:** create a `<text>`, then add each word or phrase as a `tspan`, setting fill/size per span.
When any `<tspan>` children are present the `<text>` element's own text content should be empty.

---

## `<use>`

`<use>` stamps a copy of any element — typically one defined inside `<defs>` — into the rendered tree without duplicating the DOM node.
Obtain a handle via `SvgRoot::use_node(href, at)` or `SvgBatch::use_node(href, at)`.

- `href` is a local fragment reference such as `"#my-shape"` (the `id` attribute of the target element).
- `at` is an `(x, y)` offset in the parent coordinate system; pass `Point::origin()` to control positioning entirely through `transform`.
- Each returned `SvgNode` is independent: `transform`, `opacity`, `fill`, and other presentation attributes can be set per-copy without affecting the original.
- To change the referenced element after creation, call `SvgNode::set_href("#other-shape")`.

Any change to the original definition is immediately visible in all copies.
A `<use>` element can reference any element by id, including a `<symbol>` (see the `<symbol>` section above).
