# Gap Analysis

This crate offers a working foundation for generating SVG content driven by a `requestAnimationFrame` (RAF) loop.
It supports nested groups, `<defs>`, `<marker>`, batch building, and a full set of managed event wrappers.
However, it can't yet produce anything with gradients, filters, clipping, or reusable symbols.

These gaps will be filled in time, but for now, this crate must be treated as a work-in-progress, not a general-purpose SVG library.

## Supported SVG elements

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

### `<defs>`

`<defs>` is the standard SVG container for reusable assets and can be obtained from `SvgRoot::defs()`.
All shape factory methods are available on `SvgDefs` for building inner content.

### `<marker>`

`<marker>` defines a reusable graphic (e.g. an arrowhead or a dot etc) rendered at the start, mid-point, or end of a stroked path and can be obtained from `SvgDefs::marker(id)`.

Apply it to any stroked element — `<line>`, `<path>`, `<polyline>`, `<polygon>` — via `SvgNode::set_marker_start`, `set_marker_mid`, or `set_marker_end`.
The `MarkerUnits` enum controls whether `markerWidth`/`markerHeight` are relative to `strokeWidth` (default) or user coordinates.

## Missing SVG elements

The following SVG elements still need to be implemented:

| Missing Element | Why it matters
|---|---|
| `<linearGradient>` / `<radialGradient>` | Gradient fills (the required `<defs>` container now exists)
| `<pattern>` | Tiled fill patterns
| `<clipPath>` | Masking regions
| `<image>` | Embedding raster images
| `<use>` / `<symbol>` | Reference a defined shape multiple times without duplicating DOM nodes
| `<tspan>`  | Multi-line or mixed-style text within a `<text>`
| `<textPath>` | Allows text to follow a curve
| `<filter>` and primitives | Drop shadows, blur, colour matrix, compositing etc

# Tree operations

Implemented:

- `remove()` — detach a node from the DOM
- `insert_before()` — z-order control without rebuilding
- `clear()` — remove all children of a node (e.g. to redraw a `<g>` from scratch)
- `replace_with()` — swap one node for another in place
- `parent()` — navigate up to the containing SVG element (returns an independent, non-factory handle)

Still missing:

- No downward/child navigation (`children()`, `first_child`, …)
- No way to query the tree or find a node by attribute (`query_selector` and friends)

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

Prefer `pointerenter` / `pointerleave` for hover behaviour because they do not bubble through child elements.
The legacy `mouseover` / `mouseout` wrappers remain available for compatibility reasons, but have been marked as deprecated.

# Attribute helpers

Already implemented:

- Transform helpers — `set_translate`, `set_rotate`, `set_rotate_about`, `set_scale`, `set_scale_xy`, `set_translate_scale`, and `set_transform_fmt` for arbitrary transforms (all reuse a caller-owned scratch buffer)
- Updating `<text>` content after creation — `set_text`, plus the buffer-reusing `set_text_fmt` / `set_text_display`
- Allocation-light numeric attribute writes — `set_attr_display`, and the redundant-write helpers `set_attr_if_changed` / `CachedAttr`

Still missing:

- No `matrix(...)` transform helper specifically (use `set_transform_fmt` for now)
- No `viewBox` helper (only `set_viewport`, which sets `width`/`height`)
- No `classList` / CSS class manipulation
- No `text_anchor`, `dominant_baseline`, `font_family`, `font_size` helpers

# Missing geometry access

Geometry read-back is mostly absent.
The crate exposes text advance measurement through `SvgNode::computed_text_length`, but does not currently expose broader geometry APIs:

- `getBBox()` — bounding box in local coordinates
- `getTotalLength()` / `getPointAtLength()` — path measurement
- `getCTM()` / `getScreenCTM()` — coordinate system transforms
- `getBoundingClientRect()` — position relative to viewport
