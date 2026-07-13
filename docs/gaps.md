# Gap Analysis

This document tracks the known functional gaps in `svg-dom`.
For a description of what the crate currently supports, see [Supported SVG Elements](elements.md).

These gaps will be filled in time, but for now, this crate must be treated as a work-in-progress, not a general-purpose SVG library.

## Missing SVG elements

The following SVG elements still need to be implemented:

| Missing Element | Why it matters
|---|---|
| `<textPath>` | Allows text to follow a curve
| `<filter>` and primitives | Drop shadows, blur, colour matrix, compositing etc

# Missing Tree operations

- No downward/child navigation (`children()`, `first_child`, ...)
- No way to query the tree or find a node by attribute (`query_selector` and friends)

# Missing Attribute helpers

- No `matrix(...)` transform helper specifically (use `set_transform_fmt` for now)
- No `viewBox` helper (only `set_viewport`, which sets `width`/`height`)
- No `classList` / CSS class manipulation

# Missing geometry access

Geometry read-back is mostly absent.
The crate exposes text advance measurement through `SvgNode::computed_text_length`, but does not currently expose broader geometry APIs:

- `getBBox()` — bounding box in local coordinates
- `getTotalLength()` / `getPointAtLength()` — path measurement
- `getCTM()` / `getScreenCTM()` — coordinate system transforms
- `getBoundingClientRect()` — position relative to viewport
