# Gap Analysis

This crate implements almost all of the standard SVG elements and all the filter effects defined by the SVG specification.
For a description of what the crate currently supports, see [Supported SVG Elements](svg_elements/README.md).

However, it is important to understand that this crate is not a full-featured SVG renderer for the reasons described in the non-goals section below.

## Non-goals

Synchronized Multimedia Integration Language (SMIL) animation and the `<script>` element are not, and will never be supported by this crate.
They have been intentionally excluded:

* Any use of SMIL-based animation contradicts a design decision this crate has already argued for at length.

  This would require the creation of four, completely new element types (`<animate>`, `<animateTransform>`, `<animateMotion>` and `<set>`) that then creates a large, distinct attribute surface (`dur`, `begin`, `repeatCount`, `keyTimes`, `values`, `calcMode`, additive/accumulate semantics, motion-path following).

  [`design_notes/rejected_ideas/animation.md`](design_notes/rejected_ideas/animation.md) already lays out why this crate deliberately chose a `requestAnimationFrame`-driven `AnimationLoop` over declarative timing.

  Shipping both SMIL-based and RAF-based animation would hand callers two competing (and potentially conflicting) animation models.

* The `<script>` element opens up a large security-sensitive injection surface with no real workflow fit here.

  Whilst this feature is cheap to implement, it creates the possibility to write an arbitrary script into a live DOM tree.
  This is a genuinely ***bad*** idea since the cost associated with preventing security exploits significantly outweighs any benefit the presence of such an element might bring.
