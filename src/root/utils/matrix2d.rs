// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// A 2D affine transform matrix, passed to [`SvgNode::set_matrix`](crate::SvgNode::set_matrix) and
/// [`SvgNode::set_matrix_precise`](crate::SvgNode::set_matrix_precise).
///
/// Field names describe each component's geometric role rather than its position in the SVG `matrix(a, b, c, d, e,
/// f)` transform function, applied to a point as:
///
/// ```text
/// | h_scale  h_skew   h_trans |
/// | v_skew   v_scale  v_trans |
/// | 0        0        1       |
/// ```
///
/// [`SvgNode::set_matrix`](crate::SvgNode::set_matrix) writes these out in the SVG function's own `a, b, c, d, e, f`
/// order (`h_scale, v_skew, h_skew, v_scale, h_trans, v_trans`) to build the `matrix(...)` string — the field names
/// exist for the call site, not because the underlying attribute grammar groups them this way.
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix2D {
    /// Horizontal scaling — the SVG matrix's `a` component, combined with `v_skew` for rotation.
    pub h_scale: f64,
    /// Vertical scaling — the SVG matrix's `d` component, combined with `h_skew` for rotation.
    pub v_scale: f64,
    /// Horizontal skewing — the SVG matrix's `c` component, combined with `v_scale` for rotation.
    pub h_skew: f64,
    /// Vertical skewing — the SVG matrix's `b` component, combined with `h_scale` for rotation.
    pub v_skew: f64,
    /// Horizontal translation, in user units (usually pixels) — the SVG matrix's `e` component.
    pub h_trans: f64,
    /// Vertical translation, in user units (usually pixels) — the SVG matrix's `f` component.
    pub v_trans: f64,
}
