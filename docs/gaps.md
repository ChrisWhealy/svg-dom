# Missing SVG elements

The six elements currently supported (`rect`, `circle`, `line`, `path`, `text` and `g`) are the most common ones, but the following nodes have not yet been implemented:


| Missing | Why it matters
|---|---|
| `<ellipse>`                             | Independent x/y radii — `<circle>` can't substitute                           |
| `<polyline>` / `<polygon>`              | Efficient multi-segment lines and filled shapes without path syntax           |
| `<defs>`                                | Container for reusable assets — gradients, patterns, clip-paths all live here |
| `<linearGradient>` / `<radialGradient>` | Gradient fills; not possible without `<defs>`                                 |
| `<pattern>`                             | Tiled fill patterns                                                           |
| `<clipPath>`                            | Masking regions                                                               |
| `<marker>`                              | Arrowheads on lines and paths                                                 |
| `<image>`                               | Embedding raster images                                                       |
| `<use>` / `<symbol>`                    | Instance a defined shape multiple times without duplicating DOM nodes         |
| `<tspan>`                               | Multi-line or mixed-style text within a `<text>`                              |
| `<textPath>`                            | Text following a curve                                                        |
| `<filter>` and primitives               | Drop shadows, blur, colour matrix, compositing                                |

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

# Summary

The crate is a working foundation for simple, flat SVG diagrams driven by a RAF loop, but it can't yet produce anything with gradients, filters, clipping, reusable symbols, touch input, or dynamic text.
Those gaps will be filled it time, but so far, this is a PoC crate, not a general-purpose SVG library.
