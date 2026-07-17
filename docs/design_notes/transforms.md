# Transform API design: `Matrix2D`, `set_matrix`, `set_matrix_precise`

[← Back to design notes](README.md)

See [Performance patterns](performance.md#transform-setters-reuse-a-caller-owned-buffer) for why the transform
setters take a caller-owned scratch buffer in the first place; this file covers the design of `set_matrix` and
`set_matrix_precise` specifically.

## `set_matrix` takes a `Matrix2D` struct with role-named fields, not `[f64; 6]`, positional parameters, or even `a`/`b`/`c`/`d`/`e`/`f`

`Matrix2D`'s fields are named for what they *do* (`h_scale`, `v_scale`, `h_skew`, `v_skew`, `h_trans`, `v_trans`) rather than for their position in the SVG grammar.

`set_matrix(&mut buf, Matrix2D { h_scale: 1.0, v_scale: 1.0, h_skew: 0.3, v_skew: 0.0, h_trans: 0.0, v_trans: 0.0 })` is now readable without needing to remember that `a` is horizontal scale and `f` is vertical translate as would be the case it were simply defined as `Matrix2D { a, b, c, d, e, f }`.

`SvgNode::set_matrix` still has to reassemble the fields into the SVG function's own `a, b, c, d, e, f` order to build the transform string, so `Matrix2D`'s doc comment spells out the mapping (`h_scale`→`a`, `v_skew`→`b`, `h_skew`→`c`, `v_scale`→`d`, `h_trans`→`e`, `v_trans`→`f`) once, in the one place a reader would need to go from the crate's names back to the spec's.

A `Matrix2D::new(...)` constructor has deliberately not been provided as adding one would just reopen the same positional-argument confusion the struct exists to close off, for a type with three times as many fields as `Point`.

The six fields use two different formatting precisions:

- `h_scale`, `v_scale`, `h_skew`, `v_skew` (the linear part — rotation and scale) are written to three decimal places, matching `set_scale`, since they are typically small dimensionless ratios where that precision is visible;
- `h_trans`, `v_trans` (the translation part) are written to one decimal place, matching `set_translate`, since they are typically pixel-scale coordinates.

This mirrors each field's *role* rather than treating the six numbers as a single, undifferentiated list.

## `set_matrix_precise` exists in addition to `set_matrix` due to the possibility of introducing visible quantisation artefacts

The named transform helpers each deliberately use a fixed precision appropriate for common interactive SVG updates: `set_translate` to `0.1` user unit, `set_rotate`/`set_rotate_about` to `0.1` degree, `set_scale`/`set_scale_xy` to `0.001`.
These are sensible defaults, not a guarantee that no caller can ever notice the rounding — a slowly animated rotation, for instance, can visibly stay put across several frames until it crosses the next tenth-of-a-degree boundary.
A caller who genuinely needs different precision for a translation, rotation, or scale has [`set_transform_fmt`](crate::SvgNode::set_transform_fmt) as the escape hatch, the same way it covers any other shape these named helpers don't.

`set_matrix` needs its own, more detailed treatment here rather than folding into that same "it's a sensible default" note, because an arbitrary affine matrix has a failure mode the other helpers structurally cannot: rounding errors in the linear coefficients (`h_scale`, `v_scale`, `h_skew`, `v_skew` — the SVG matrix's `a`, `b`, `c`, `d`) are multiplied by whatever coordinate the matrix transforms, so their effect scales with the geometry rather than staying fixed the way a rounded translation or rotation angle does.
For example:

- A rotation's sine term rounds to `0.000` below about `0.0286°` (`sin(0.0286°) ≈ 0.0005`, the rounding threshold at three decimal places). A `0.01°` rotation, for example, serialises as the exact identity matrix — the rotation does not just lose precision, it disappears completely. A slow matrix-driven rotation animation can therefore visibly stick at each frame's rounded value and then jump, rather than moving smoothly.

- Each linear coefficient's rounding error (up to `0.0005`) is applied to whatever coordinate the matrix acts on, so the resulting positional error scales with the coordinate rather than staying fixed: at `x = y = 10,000`, the error can exceed 10 user units, even though the same rounding is invisible at typical UI scales.

`set_matrix_precise` is the same function with the fixed-precision `write!` calls replaced by plain `{}` (`Display`) formatting — Rust's shortest round-trip representation, the same default-precision choice `write_d`/`build_d` already make for path data (see [Path data](path_data.md), "Formatting matches the existing `write_points` convention"). It exists alongside `set_matrix` rather than replacing it, mirroring that same `_fixed`-suffix pairing in spirit (though named the other way around here, since `set_matrix` was the already-shipped name by the time this was raised, and renaming it would have been a needless breaking change for existing callers): pick `set_matrix` when its quantisation is acceptable and limiting coefficient precision is itself desirable, and `set_matrix_precise` when the original `f64` values must survive serialisation exactly.

The choice is about precision, not an assumed size advantage — an earlier revision of this note framed `set_matrix` as the better hot-path choice because its output is "typically shorter," which does not hold in general. `Matrix2D` is a plain data struct with no matrix-composition API of its own (nothing here builds or combines matrices; that would be a separate, larger feature), so "the matrix came from this crate's own composition" was also never a meaningful test to pick by. For round-number coefficients `set_matrix_precise` is often the *shorter* string, since `set_matrix` always writes three or one decimal places even for a bare `0`: an identity matrix is `matrix(1, 0, 0, 1, 0, 0)` (24 characters) via `set_matrix_precise` but `matrix(1.000, 0.000, 0.000, 1.000, 0.0, 0.0)` (44 characters) via `set_matrix`. A computed rotation more often favours `set_matrix`'s fixed precision, since shortest-round-trip formatting of an irrational sine/cosine value can run to fifteen-plus digits — but this, too, is a property of the specific coefficients, not a rule either setter can claim in general.

See [Geometry read-back](geometry.md) ("`ctm`/`screen_ctm` are accumulated matrices...") for why a `Matrix2D` read back from `ctm()`/`screen_ctm()` generally cannot be written straight back through `set_matrix`/`set_matrix_precise` unmodified, and for the formula to recover a writable local matrix when you do need to.
