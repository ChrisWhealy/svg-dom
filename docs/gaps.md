# Gap Analysis

This crate offers a working foundation for generating simple, flat SVG diagrams driven by a `requestAnimationFrame` (RAF) loop.
However, it can't yet produce anything with gradients, filters, clipping, reusable symbols, touch input, or dynamic text.

These gaps will be filled in time, but for now, this crate must be treated as a PoC, not a general-purpose SVG library.

# Missing SVG elements

The six SVG elements currently supported (`rect`, `circle`, `line`, `path`, `text` and `g`) are the most commonly used ones; however, the following elements have not yet been implemented:

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

# Missing events

Only three mouse events are wrapped.
Wrappers for these events are missing:

- `mousedown`, `mouseup`, `mousemove`
- `mouseenter` / `mouseleave` (the non-bubbling equivalents we discussed)
- `wheel` (scroll/zoom)
- Touch events (`touchstart`, `touchmove`, `touchend`) — important for mobile
- No `remove_event_listener` — once registered, a handler cannot be detached

# Missing attribute helpers

- No transform helpers (translate, rotate, scale, matrix) — currently just `set_attr("transform", ...)`
- No `viewBox` helper
- No `classList` / CSS class manipulation
- No way to update `<text>` content after creation
- No `text_anchor`, `dominant_baseline`, `font_family`, `font_size` helpers

# Missing geometry access

Read-back from the browser's layout engine is entirely absent:

- `getBBox()` — bounding box in local coordinates
- `getTotalLength()` / `getPointAtLength()` — path measurement
- `getCTM()` / `getScreenCTM()` — coordinate system transforms
- `getBoundingClientRect()` — position relative to viewport
