# Gap Analysis

This crate offers a working foundation for generating simple, flat SVG diagrams driven by a `requestAnimationFrame` (RAF) loop.
However, it can't yet produce anything with gradients, filters, clipping, or reusable symbols.

These gaps will be filled in time, but for now, this crate must be treated as an MVP, not a general-purpose SVG library.

# Missing SVG elements

The six SVG elements currently supported (`rect`, `circle`, `line`, `path`, `text` and `g`) are the most commonly used ones.
The following elements all need to be implemented:

| Missing Element | Why it matters
|---|---|
| `<ellipse>` | Offers independent `x,y` radii.  Something `<circle>` can't substitute
| `<polyline>` / `<polygon>` | Efficient multi-segment lines and filled shapes without using the `path` syntax
| `<defs>` | Container for reusable assets. Gradients, patterns, clip-paths all live here
| `<linearGradient>` / `<radialGradient>` | Gradient fills that are not possible without `<defs>`
| `<pattern>` | Tiled fill patterns
| `<clipPath>` | Masking regions
| `<marker>` | Arrowheads on lines and paths
| `<image>` | Embedding raster images
| `<use>` / `<symbol>` | Reference a defined shape multiple times without duplicating DOM nodes
| `<tspan>`  | Multi-line or mixed-style text within a `<text>`
| `<textPath>` | Allows text to follow a curve
| `<filter>` and primitives | Drop shadows, blur, colour matrix, compositing etc

# Missing tree operations

- `remove()` — detach a node from the DOM
- `insert_before()` — z-order control without rebuilding
- No way to query children or find a node by attribute

# Event coverage

Managed wrappers now cover the SVG interaction events expected by ordinary application code: click/double-click/context menu, mouse movement and button state, pointer lifecycle, wheel, touch, keyboard, focus/blur, drag-and-drop, and a generic `on_event` escape hatch.

Prefer `pointerenter` / `pointerleave` for hover behaviour because they do not bubble through child elements. The legacy `mouseover` / `mouseout` wrappers remain for compatibility.

Potential future event work is now mostly about ergonomics rather than coverage: typed helpers for less common browser events can be added when real SVG use-cases appear.

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

Read-back from the browser's layout engine is entirely absent:

- `getBBox()` — bounding box in local coordinates
- `getTotalLength()` / `getPointAtLength()` — path measurement
- `getCTM()` / `getScreenCTM()` — coordinate system transforms
- `getBoundingClientRect()` — position relative to viewport
