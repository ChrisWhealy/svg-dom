# Gap Analysis

This document tracks the known functional gaps in `svg-dom`.
For a description of what the crate currently supports, see [Supported SVG Elements](svg_elements/README.md).

These gaps will be filled in time, but for now, this crate must be treated as a work-in-progress, not a general-purpose SVG library.

## Missing filter effect primitives

The following filter effect primitives still need to be implemented, each with its own cost (implementation effort) and benefit (how commonly it is needed in real SVG work):

| Missing Primitive | Cost | Benefit |
|---|---|---|
| `<feConvolveMatrix>` | High — `order`, `kernelMatrix` (variable-length, needs validation), `divisor`, `bias`, `targetX`/`targetY`, `edgeMode`, `preserveAlpha`. | Low — sharpen/emboss/edge-detect, rare outside specialty tooling. |
| `<feDiffuseLighting>` / `<feSpecularLighting>` | High — each needs light-source child elements (`<feDistantLight>`/`<fePointLight>`/`<feSpotLight>`), each with its own attribute set, plus `surfaceScale`/`specularExponent`/`lighting-color`. | Low — embossed/3D lighting effects, rarely used on the web. |

Every filter primitive reuses the same `SvgFilter` pattern already proven multiple times (`impl SvgFilter { pub fn x(&self, ...) -> Result<SvgNode, Error> }`), so none of them require a new architectural decision — only attribute-surface work that scales with each primitive's own complexity.

Each individual primitive's own `in`/`result` attributes, and any primitive-specific attribute not yet wrapped by a named parameter, remain reachable only via `SvgNode::set_attr` on the node the primitive method returns.

See [`design_notes/filters.md`](design_notes/filters.md), "`<filter>` primitives return a plain `SvgNode`", for why a typed per-primitive wrapper was deferred rather than built now.

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

`feConvolveMatrix` and `feDiffuseLighting`/`feSpecularLighting` remain the most expensive items on either list for the narrowest payoff, and not considered a high priority.
