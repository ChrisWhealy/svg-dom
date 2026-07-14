# Gap Analysis

This document tracks the known functional gaps in `svg-dom`.
For a description of what the crate currently supports, see [Supported SVG Elements](elements.md).

These gaps will be filled in time, but for now, this crate must be treated as a work-in-progress, not a general-purpose SVG library.

## Missing SVG elements

`<filter>` itself and three primitives (`<feGaussianBlur>`, `<feOffset>`, `<feMerge>`/`<feMergeNode>`) are implemented — see [Supported SVG Elements](elements.md#filter).
Those three together are enough for a basic drop shadow (blur a copy of the graphic, offset it, merge it underneath the original); a true black/tinted shadow with independent opacity needs `feFlood`/`feComposite` below.
The following filter primitives still need to be implemented:

| Missing Primitive | Why it matters
|---|---|
| `<feFlood>` / `<feComposite>` | Solid colour fill and Porter-Duff compositing; needed for a proper drop shadow's colour and opacity (rather than a blurred copy of the source graphic itself) |
| `<feColorMatrix>` | Colour transforms — greyscale, saturation, hue rotation |
| `<feDropShadow>` | Shorthand combining offset + blur + flood + composite in one primitive |
| `<feBlend>`, `<feTile>`, `<feMorphology>`, `<feConvolveMatrix>`, `<feDisplacementMap>`, `<feTurbulence>`, `<feComponentTransfer>`, `<feDiffuseLighting>` / `<feSpecularLighting>`, `<feImage>` | Less commonly needed effects; lower priority |

Also missing on `SvgFilter` itself: typed setters for the filter region and coordinate-space attributes (`x`, `y`, `width`, `height`, `filterUnits`, `primitiveUnits`) — reachable today only via the generic `set_attr`/`set_attrs` escape hatch.
See `docs/design_notes.md`, "`<filter>` primitives return a plain `SvgNode`", for why a typed per-primitive wrapper was deferred rather than built now.

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
