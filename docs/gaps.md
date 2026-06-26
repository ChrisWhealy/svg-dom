# Gap Analysis

This crate offers a working foundation for generating simple, flat SVG diagrams driven by a `requestAnimationFrame` (RAF) loop.
However, it can't yet produce anything with gradients, filters, clipping, or reusable symbols.

These gaps will be filled in time, but for now, this crate must be treated as an MVP, not a general-purpose SVG library.

# Missing SVG elements

The following SVG elements are supported: `rect`, `circle`, `ellipse`, `line`, `polyline`, `polygon`, `path`, `text` and `g`.

The following SVG elements all need to be implemented:

| Missing Element | Why it matters
|---|---|
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
* `on_event_once` — a one-shot variant that fires at most once and is automatically removed by the browser via the native `{ once: true }` `addEventListener` option.

Prefer `pointerenter` / `pointerleave` for hover behaviour because they do not bubble through child elements.
The legacy `mouseover` / `mouseout` wrappers remain available for compatibility reasons, but have been marked as deprecated.

`on_event_once` accepts a generic event type parameter `E` and uses a checked `instanceof` cast at runtime.
If the supplied `E` does not match the event the browser actually dispatches (e.g. trying to match the `KeyboardEvent` with a `"click"` listener), the cast fails silently and the handler will **not** be called.

Potential future event work is mostly about ergonomics rather than coverage: typed one-shot wrappers (`on_click_once`, `on_pointerdown_once`, ...) can be added when real use-cases appear, but the generic `on_event_once` covers the common case today.

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
