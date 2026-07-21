# Gap Analysis

This document tracks the known functional gaps in `svg-dom`.
For a description of what the crate currently supports, see [Supported SVG Elements](svg_elements/README.md).

These gaps will be filled in time, but for now, this crate must be treated as a work-in-progress, not a general-purpose SVG library.

## Missing filter effect primitives

The following filter effect primitives still need to be implemented, each with its own cost (implementation effort) and benefit (how commonly it is needed in real SVG work):

| Missing Primitive | Cost | Benefit |
|---|---|---|
| `<feComponentTransfer>` (with `<feFuncR>`/`<feFuncG>`/`<feFuncB>`/`<feFuncA>`) | Medium-high — four child elements, each with `type`/`tableValues`/`slope`/`intercept`/`amplitude`/`exponent`/`offset`; follows the `<feMergeNode>` child-element precedent already established by `merge`. | Moderate — gamma/contrast/duotone adjustments. |
| `<feTurbulence>` | Medium — `baseFrequency`/`numOctaves`/`seed`/`stitchTiles`/`type`. | Moderate-high — paired with `feDisplacementMap`/`feComposite`, the standard route to noise/hand-drawn/organic textures; genuinely popular in creative SVG work. |
| `<feDisplacementMap>` | Medium — `in2` + `scale` + two channel-selector enums. | Moderate, mostly as `feTurbulence`'s partner. |
| `<feMorphology>` | Low-medium — single `radius` plus an `operator: erode/dilate` enum. | Moderate — outline thickening/thinning. |
| `<feTile>` | Low — no meaningful attributes beyond `in`. | Low — rarely used standalone. |
| `<feImage>` | Medium — `href` + `preserveAspectRatio`, but reuses patterns already built for `image()`. | Low-moderate, niche. |
| `<feConvolveMatrix>` | High — `order`, `kernelMatrix` (variable-length, needs validation), `divisor`, `bias`, `targetX`/`targetY`, `edgeMode`, `preserveAlpha`. | Low — sharpen/emboss/edge-detect, rare outside specialty tooling. |
| `<feDiffuseLighting>` / `<feSpecularLighting>` | High — each needs light-source child elements (`<feDistantLight>`/`<fePointLight>`/`<feSpotLight>`), each with its own attribute set, plus `surfaceScale`/`specularExponent`/`lighting-color`. | Low — embossed/3D lighting effects, rarely used on the web. |

Every filter primitive reuses the same `SvgFilter` pattern already proven eight times over (`impl SvgFilter { pub fn x(&self, ...) -> Result<SvgNode, Error> }`), so none of them require a new architectural decision — only attribute-surface work that scales with each primitive's own complexity.

`<feBlend>` (`SvgFilter::blend`, with a `BlendMode` enum the same shape as `composite`'s `CompositeOperator`) is now implemented — see [Filters](svg_elements/filters.md#primitive-factories-available-on-svgfilter).

Each individual primitive's own `in`/`result` attributes, and any primitive-specific attribute not yet wrapped by a named parameter, remain reachable only via `SvgNode::set_attr` on the node the primitive method returns.

See [`design_notes/filters.md`](design_notes/filters.md), "`<filter>` primitives return a plain `SvgNode`", for why a typed per-primitive wrapper was deferred rather than built now.

## Other unimplemented elements

These elements have no wrapper anywhere in `src/` and, unlike the filter primitives above, were never previously tracked in this document or evaluated in the [design notes](design_notes/README.md) or [rejected ideas](design_notes/rejected_ideas/README.md) — they are genuinely unaudited rather than deliberately deferred.

| Missing element | Cost | Benefit |
|---|---|---|
| `<a>` (anchor) | Low — a `<g>`-like wrapper plus `href`/`target`. | Low-moderate — useful for interactive diagrams/maps, rarely essential. |
| `<switch>` | Low structurally, but `systemLanguage`/`requiredFeatures` are already reachable as plain attributes on a `g()` via `set_attr`. | Low — a dedicated wrapper buys little over what is already reachable. |
| `<view>` | Low — attribute-only container. | Low — fragment-identifier navigation is a niche use case for a WASM-driven crate that already controls the DOM directly. |
| `<foreignObject>` | Low to build (`x`/`y`/`width`/`height` + the existing `set_inner_html`), but the crate's query methods (`first_child`, `query_selector`, etc.) already deliberately treat its contents as opaque — a constructor would be inconsistent with that stance unless the read side is revisited too. | Low-moderate, and narrow to HTML-interop scenarios. |
| `<style>` | Low to build. | Low — styling already goes through `set_attr`/CSS classes set from Rust; injecting a `<style>` text block doesn't fit how this crate expects callers to work. |

## Non-goals

SMIL animation and the `<script>` element are not, and will never be supported by this crate.

They have been intentionally excluded:

* Any use of Synchronised Multimedia Integration Language (SMIL) contradicts a design decision this crate has already argued for at length.

  This would require the creation of four, completely new element types (`<animate>`, `<animateTransform>`, `<animateMotion>` and `<set>`) that then creates a large, distinct attribute surface (`dur`, `begin`, `repeatCount`, `keyTimes`, `values`, `calcMode`, additive/accumulate semantics, motion-path following).

  [`design_notes/rejected_ideas/animation.md`](design_notes/rejected_ideas/animation.md) already lays out why this crate deliberately chose a `requestAnimationFrame`-driven `AnimationLoop` over declarative timing.

  Shipping SMIL elements alongside RAF-based animation would hand callers two competing animation models to reason about.

* The `<script>` element is a security-sensitive injection surface with no real workflow fit here.

  Whilst this feature is cheap to implement, it creates the possibility to write an arbitrary script into a live DOM tree.
  This is a genuinely ***bad*** idea since the security cost significantly outweighs any benefit it might bring.

### Priority

With `feBlend` now implemented, the cost/benefit favours **`feComponentTransfer`** as the next gap worth closing: moderate implementation cost (four child elements following the `<feMergeNode>` precedent `merge` already established) against moderate benefit for gamma/contrast/duotone adjustments.

The best value-per-effort ordering within the remaining filter primitives is `feComponentTransfer` → `feTurbulence`/`feDisplacementMap`; `feConvolveMatrix` and `feDiffuseLighting`/`feSpecularLighting` remain the most expensive items on either list for the narrowest payoff, and are deprioritised accordingly.
