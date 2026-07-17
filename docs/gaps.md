# Gap Analysis

This document tracks the known functional gaps in `svg-dom`.
For a description of what the crate currently supports, see [Supported SVG Elements](elements.md).

These gaps will be filled in time, but for now, this crate must be treated as a work-in-progress, not a general-purpose SVG library.

## Missing filter effect primitives

The following filter effect primitives still need to be implemented, each with its own cost (implementation effort) and benefit (how commonly it is needed in real SVG work):

| Missing Primitive | Cost | Benefit |
|---|---|---|
| `<feBlend>` | Low-medium — one `mode` enum (`Normal`/`Multiply`/`Screen`/`Darken`/`Lighten`/…), the same shape as `composite`'s `CompositeOperator`. | Moderate-high — blend modes are common in realistic shadow/lighting effects. |
| `<feComponentTransfer>` (with `<feFuncR>`/`<feFuncG>`/`<feFuncB>`/`<feFuncA>`) | Medium-high — four child elements, each with `type`/`tableValues`/`slope`/`intercept`/`amplitude`/`exponent`/`offset`; follows the `<feMergeNode>` child-element precedent already established by `merge`. | Moderate — gamma/contrast/duotone adjustments. |
| `<feTurbulence>` | Medium — `baseFrequency`/`numOctaves`/`seed`/`stitchTiles`/`type`. | Moderate-high — paired with `feDisplacementMap`/`feComposite`, the standard route to noise/hand-drawn/organic textures; genuinely popular in creative SVG work. |
| `<feDisplacementMap>` | Medium — `in2` + `scale` + two channel-selector enums. | Moderate, mostly as `feTurbulence`'s partner. |
| `<feMorphology>` | Low-medium — single `radius` plus an `operator: erode/dilate` enum. | Moderate — outline thickening/thinning. |
| `<feTile>` | Low — no meaningful attributes beyond `in`. | Low — rarely used standalone. |
| `<feImage>` | Medium — `href` + `preserveAspectRatio`, but reuses patterns already built for `image()`. | Low-moderate, niche. |
| `<feConvolveMatrix>` | High — `order`, `kernelMatrix` (variable-length, needs validation), `divisor`, `bias`, `targetX`/`targetY`, `edgeMode`, `preserveAlpha`. | Low — sharpen/emboss/edge-detect, rare outside specialty tooling. |
| `<feDiffuseLighting>` / `<feSpecularLighting>` | High — each needs light-source child elements (`<feDistantLight>`/`<fePointLight>`/`<feSpotLight>`), each with its own attribute set, plus `surfaceScale`/`specularExponent`/`lighting-color`. | Low — embossed/3D lighting effects, rarely used on the web. |

Every filter primitive reuses the same `SvgFilter` pattern already proven seven times over (`impl SvgFilter { pub fn x(&self, ...) -> Result<SvgNode, Error> }`), so none of them require a new architectural decision — only attribute-surface work that scales with each primitive's own complexity.

Each individual primitive's own `in`/`result` attributes, and any primitive-specific attribute not yet wrapped by a named parameter, remain reachable only via `SvgNode::set_attr` on the node the primitive method returns.

See [`design_notes/filters.md`](design_notes/filters.md), "`<filter>` primitives return a plain `SvgNode`", for why a typed per-primitive wrapper was deferred rather than built now.

## Other unimplemented elements

These elements have no wrapper anywhere in `src/` and, unlike the filter primitives above, were never previously tracked in this document or evaluated in the [design notes](design_notes/README.md) or [rejected ideas](design_notes/rejected_ideas/README.md) — they are genuinely unaudited rather than deliberately deferred.

| Missing element | Cost | Benefit |
|---|---|---|
| `<mask>` | Low-medium — a near-exact structural clone of `clip_path.rs` (`SvgMask` struct, `build_mask`/`mask()` factory, `set_mask_ref` setter): the `clipPath` template with `mask`/`mask-type` swapped for `clip-path`/`clipPathUnits`. | High — masks are used constantly alongside clipPaths in real work (gradient fades, vignettes, reveal effects); a foundational compositing primitive, not a specialty effect, and sits right next to a gap this crate already chose to close (`clipPath`). |
| `<desc>` / `<title>` | Very low — create the element, `set_text_content`, done; smaller than one filter primitive. | Real but narrow — the only accessibility gap on this list; nothing else gives SVG content an accessible name/description. |
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

  Whilst this features comes at a low implementation cost, it creates the possibility to write an arbitrary script into a live DOM tree.
  This is a genuinely ***bad*** idea since the security cost significantly outweighs any benefit it might bring.

### Priority

Cost/benefit favours **`<mask>`** as the most productive gap to close next: its implementation cost is comparable to one of the pricier filter primitives above, since it follows the same proven structural pattern as `clipPath`, but its benefit is broader, since it is a foundational compositing primitive rather than a specialty visual effect.

Within the filter primitives, the best value-per-effort ordering is `feBlend` → `feComponentTransfer` → `feTurbulence`/`feDisplacementMap`; `feConvolveMatrix` and `feDiffuseLighting`/`feSpecularLighting` are the most expensive items on either list for the narrowest payoff, and are deprioritised accordingly.
