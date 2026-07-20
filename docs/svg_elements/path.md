# Path data

[← Back to supported elements](README.md)

**Contents**

- [`<path>`](#path)

## `<path>`

A `<path>` is created either from a hand-written `d` string (`SvgRoot::path(d)`) or, if you prefer a type-safe approach, from a sequence of typed `PathDef` segments (`SvgRoot::path_from_defs(&[PathDef])`).

A hand-written `d` string is free text, so any typos will be silently accepted by the SVG parser, which will then simply stop rendering at the first bad token rather than reporting an error.

The use of `path_from_defs` removes that failure mode for individual commands since the `d` attribute is built internally from `PathDef` values, so a mistyped command letter, wrong argument count, or invalid arc flag can never be constructed.

That guarantee is about individual commands, yet it is still possible to create a command sequence that fails to be a valid path:

- Any non-empty path ***must*** start with a moveto (`M`/`m`) command; a browser silently renders nothing for a path that starts with anything else. 
   `path_from_defs`, `SvgNode::set_d_from_defs`, and the `SvgAttrs` / `AnimationFrame` `d_from_defs` methods all check this (requiring an O(1) look at the first command) and return `Error::InvalidPathData` if it fails.
   `build_d` / `write_d` (and their `_fixed` siblings) do **not** perform this check, since they may legitimately be used to build path-data *fragments* rather than a complete, standalone path.

- Coordinates are unconstrained `f64` values, so nothing stops `f64::NAN` or `f64::INFINITY` from being supplied — the SVG number grammar has no representation for either, so `PathDef` cannot format a valid path from one.
   No function in the path API checks for this, since doing so would mean visiting every numeric argument of every command on every call, including the buffer-reusing per-frame ones.
   Validate with `f64::is_finite()` before constructing a `PathDef` if your coordinates come from a calculation (division, trigonometry) that could produce one.

### Building Path Data

| Type | Purpose |
|---|---|
| `PathDef` | One path-data command: `Abs(PathDefAbsolute)` or `Rel(PathDefRelative)`. Absolute and relative commands can be freely mixed in the same sequence, exactly as real SVG path data allows. |
| `PathDefAbsolute` / `PathDefRelative` | The ten SVG path commands (`MoveTo`, `LineTo`, `HorizontalLineTo`, `VerticalLineTo`, `CubicBezierTo`, `SmoothCubicBezierTo`, `QuadraticBezierTo`, `SmoothQuadraticBezierTo`, `EllipticalArcTo`, `ClosePath`) in absolute or relative form respectively. |
| `EllipticalArc` | Named-field parameters for an arc segment — `radii`, `x_axis_rotation`, `size`, `sweep`, `to` — instead of a five-element tuple. |
| `ArcSize` | `Small` / `Large` — the SVG `large-arc-flag`, replacing a bare `bool` that gives no clue at the call site which arc solution it selects. |
| `ArcSweep` | `CounterClockwise` / `Clockwise` — the SVG `sweep-flag`, replacing the second bare `bool`. |

### Creating and Updating Paths

| Method | Available on | Effect |
|---|---|---|
| `path(d)` | `SvgRoot`, `SvgBatch`, `SvgDefs`, `SvgClipPath`, `SvgMarker`, `SvgPattern`, `SvgSymbol` | Creates a `<path>` from a raw `d` string. |
| `path_from_defs(&[PathDef])` | Same set of types | Creates a `<path>` from typed segments, writing `d` through the factory's own retained `SvgAttrs` buffer — no extra allocation beyond the first call. |
| `SvgNode::set_d(d)` | Any `SvgNode` | Updates an existing `<path>`'s `d` string. |
| `SvgNode::set_d_from_defs(&[PathDef])` | Any `SvgNode` | Updates an existing `<path>`'s `d` from typed segments. Allocates a fresh `String` per call; consequently, this should only be used for occasional updates. See below for the hot-path alternatives. |
| `build_d(&[PathDef])` | Free function | Builds a `d` string without creating or updating any element — useful for composing a path in pieces. |
| `write_d(&mut String, &[PathDef])` | Free function | The buffer-reusing counterpart to `build_d`, for a hot path that rebuilds a curve every frame. |

### Allocation-light Updates

These are allocation-light alternatives to `set_d_from_defs` that mirror the existing `points`/`points_fixed` pattern:

| Method | Effect |
|---|---|
| `SvgAttrs::d_from_defs(&node, &[PathDef])` | Writes `d` through `SvgAttrs`'s reusable scratch buffer. |
| `AttrWriter::d_from_defs(&[PathDef])` | The chainable-writer equivalent, via `node.attrs(&mut attrs)`. |
| `AnimationFrame::set_d_from_defs(&node, &[PathDef])` | The per-frame equivalent, for use inside `AnimationLoop::start_with_frame`. |
| `write_d_fixed(&mut String, &[PathDef], dps)` / `build_d_fixed(&[PathDef], dps)` | Like `write_d`/`build_d`, but every coordinate, length, and rotation angle is rounded to `dps` decimal places (clamped to 20). The elliptical-arc flags are never rounded — the SVG grammar requires them to stay a bare `0`/`1`. |
| `SvgAttrs::d_from_defs_fixed` / `AttrWriter::d_from_defs_fixed` / `AnimationFrame::set_d_from_defs_fixed` | The fixed-precision counterparts of the three methods above, mirroring `points_fixed`/`set_points_fixed`. Use these for path data computed during an animation, where the default shortest-round-trip formatting would otherwise carry more digits than needed. |

### Example

```rust,no_run
use svg_dom::{ArcSize, ArcSweep, EllipticalArc, PathDef, PathDefAbsolute, SvgRoot, root::utils::Point};

let svg = SvgRoot::attach("diagram")?;
let arc = svg.path_from_defs(&[
    PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(10.0, 65.0))),
    PathDef::Abs(PathDefAbsolute::EllipticalArcTo(EllipticalArc {
        radii: Point::new(60.0, 60.0),
        x_axis_rotation: 0.0,
        size: ArcSize::Large,
        sweep: ArcSweep::Clockwise,
        to: Point::new(130.0, 65.0),
    })),
])?;
arc.set_fill("none")?;
arc.set_stroke("coral")?;
Ok::<(), svg_dom::Error>(())
```
