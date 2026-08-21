# Supported SVG Elements

This directory describes what `svg-dom` currently supports:

* the SVG elements wrapped so far
* their capabilities
* element-focused guides for the ones with enough surface area to warrant one

The basic element factories (`circle`, `ellipse`, `g`, `line`, `polygon`, `polyline`, `rect`) are listed below, but do not yet have their own thematic page here.
Their construction is largely self-explanatory — a factory method, plus the shared presentation-attribute setters on `SvgNode`.
Consult their own rustdoc for the full signature of each.

Note that `<g>` is a structural/container element (SVG 2 classifies it as such), not a shape, despite sharing this factory grouping with the others for construction purposes.

`svg-dom` provides typed support for the SVG elements listed below.
`<script>` and the SMIL-based animation elements — `<animate>`, `<animateTransform>`, `<animateMotion>`, `<set>`, `<discard>` and `<mpath>` — are intentional non-goals (see [Implementation Non-goals](../non-goals.md) for details):

- `<a>` (anchor)
- `<circle>`
- `<clipPath>`
- `<defs>`
- `<desc>`
- `<ellipse>`
- `<filter>` and filter effects
  - `<feBlend>`
  - `<feColorMatrix>`
  - `<feComponentTransfer>` with `<feFuncR>`, `<feFuncG>`, `<feFuncB>`, `<feFuncA>`
  - `<feComposite>`
  - `<feConvolveMatrix>`
  - `<feDiffuseLighting>` with `<feDistantLight>`, `<fePointLight>`, or `<feSpotLight>`
  - `<feDisplacementMap>`
  - `<feDropShadow>`
  - `<feFlood>`
  - `<feGaussianBlur>`
  - `<feImage>`
  - `<feMerge>`, `<feMergeNode>`
  - `<feMorphology>`
  - `<feOffset>`
  - `<feSpecularLighting>` with `<feDistantLight>`, `<fePointLight>`, or `<feSpotLight>`
  - `<feTile>`
  - `<feTurbulence>`
- `<foreignObject>` — no content-setting method, by design — see [Structural Elements](structural_elements.md#foreignobject) for the raw-DOM escape hatch
- `<g>`
- `<image>`
- `<line>`
- `<linearGradient>` — with `<stop>`
- `<marker>`
- `<mask>`
- `<metadata>` — plain-text/JSON content — see [Core Operations](core_operations.md#metadata) for the escape hatch to structured foreign-namespace children
- `<path>` — with a type-safe `PathDef` builder as an alternative to hand-written `d` strings
- `<pattern>`
- `<polygon>`
- `<polyline>`
- `<radialGradient>` — with `<stop>`
- `<rect>`
- `<style>`
- `<svg>` — the root element itself (`SvgRoot`), either attached to an existing element or created programmatically
- `<switch>`
- `<symbol>`
- `<text>` — with `<tspan>` and `<textPath>`
- `<title>`
- `<tspan>`
- `<use>`
- `<view>`

## SVG Elements Not Supported

At the moment, there is no support for the following SVG elements:

- Nested `<svg>`
- `<audio>`
- `<video>`
- `<iframe>`
- `<canvas>`

These might be implemented in future if a legitimate use case is presented.

`<unknown>` is also absent from the list above, but for a different reason: SVG 2 defines it as the browser's own fallback rendering for unrecognised markup, not an element a typed factory API would ever construct.

## Core Operations

- [Tree operations, events, attribute, geometry, and accessibility helpers](core_operations.md)

  The common set of capabilities that apply to every `SvgNode` regardless of element type: DOM tree navigation, the managed event-listener API, generic transform and text attribute helpers, read-only geometry queries such as `bounding_box`, current transformation matrix (`ctm` `screen_ctm`), `total_length` and `point_at_length`, and accessible name/description via `set_title`/`set_desc`.

## Clipping and Masking

- [`<clipPath>` and `<mask>`](clipping_and_masking.md)

  Restrict or fade the rendered region of any element either by shape geometry or by luminance/alpha.

## Filters

- [`<filter>`](filters.md)

  Use the filter-primitive builder methods on `SvgFilter` to apply raster effects such as blur, colour manipulation, compositing or drop shadows.

## Paint Servers

- [`<linearGradient>`, `<radialGradient>` and `<pattern>`](paint_servers.md)

  Different paint servers (defined in `<defs>`) that allow you to apply gradient and tiled-patterned fill or stroke effects to SVG elements.

## Structural and Reusable elements

- [`<defs>`, `<marker>`, `<image>`, `<symbol>`, `<use>`, `<a>`, `<switch>`, `<view>` and `<foreignObject>`](structural_elements.md)

  A set of reusable SVG asset containers such as path-decoration markers, raster/SVG embedding, reusable scaled viewports and element instancing without the need for DOM duplication, plus the hyperlink and conditional-rendering wrappers, plus fragment-addressable named viewports, plus a browser-HTML-laid-out rectangular region.

## Text

- [`<text>`, `<textPath>` and `<tspan>`](text.md)

  Elements for defining text attributes, then presenting that text either as multi-line/mixed-style inline spans, or following a curved path.

## Path data

- [`<path>`](path.md)

  Allows you to define path data either as hand-written `d` strings or using the type-safe `PathDef` builder via an allocation-light update API.
