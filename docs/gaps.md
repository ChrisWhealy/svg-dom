# Gap Analysis

This document tracks the known functional gaps in `svg-dom`.
For a description of what the crate currently supports, see [Supported SVG Elements](elements.md).

These gaps will be filled in time, but for now, this crate must be treated as a work-in-progress, not a general-purpose SVG library.

## Missing SVG elements

The following filter effect primitives still need to be implemented:

| Missing Primitive | Why it matters
|---|---|
| `<feBlend>`, `<feTile>`, `<feMorphology>`, `<feConvolveMatrix>`, `<feDisplacementMap>`, `<feTurbulence>`, `<feComponentTransfer>`, `<feDiffuseLighting>` / `<feSpecularLighting>`, `<feImage>` | Less commonly needed effects; lower priority |

Each individual primitive's own `in`/`result` attributes, and any primitive-specific attribute not yet wrapped by a named parameter, remain reachable only via `SvgNode::set_attr` on the node the primitive method returns.

See [`design_notes.md`](design_notes.md#filter-primitives-return-a-plain-svgnode), "`<filter>` primitives return a plain `SvgNode`", for why a typed per-primitive wrapper was deferred rather than built now.
