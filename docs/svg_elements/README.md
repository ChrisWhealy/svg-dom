# Supported SVG Elements

This directory describes what `svg-dom` currently supports:

* the SVG elements wrapped so far
* their capabilities
* element-focused guides for the ones with enough surface area to warrant one

The basic shape factories (`circle`, `ellipse`, `g`, `line`, `polygon`, `polyline`, `rect`) are listed below but do not
yet have their own thematic page here — their construction is largely self-explanatory (a factory method plus the
shared presentation-attribute setters on `SvgNode`), so consult their own rustdoc for the full signature of each.

For known gaps, see [Gap Analysis](../gaps.md).

The following SVG elements are supported:

* `circle`
* `clipPath`
* `defs`
* `desc`
* `ellipse`
* `filter` (with `feGaussianBlur`, `feOffset`, `feMerge`/`feMergeNode`, `feFlood`, `feComposite`, `feBlend`, `feDropShadow`, `feColorMatrix`, `feComponentTransfer`/`feFuncR`/`feFuncG`/`feFuncB`/`feFuncA`, `feTurbulence`, `feDisplacementMap`, `feMorphology`)
* `g`
* `image`
* `line`
* `linearGradient` (with `stop`)
* `marker`
* `mask`
* `pattern`
* `rect`
* `path` (with a type-safe `PathDef` builder as an alternative to hand-written `d` strings)
* `polygon`
* `polyline`
* `radialGradient` (with `stop`)
* `symbol`
* `text` (with `tspan`, `textPath`)
* `title`
* `use`

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

- [`<defs>`, `<marker>`, `<image>`, `<symbol>` and `<use>`](structural_elements.md)

  A set of reusable SVG asset containers such as path-decoration markers, raster/SVG embedding, reusable scaled viewports and element instancing without the need for DOM duplication.

## Text

- [`<text>`, `<textPath>` and `<tspan>`](text.md)

  Elements for defining text attributes, then presenting that text either as multi-line/mixed-style inline spans, or following a curved path.

## Path data

- [`<path>`](path.md)

  Allows you to define path data either as hand-written `d` strings or using the type-safe `PathDef` builder via an allocation-light update API.
