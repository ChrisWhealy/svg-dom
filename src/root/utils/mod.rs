mod matrix2d;
mod point;
mod rect;
mod size;

pub use matrix2d::Matrix2D;
pub use point::Point;
pub use rect::Rect;
pub use size::Size;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Validates the four components of a `viewBox` before it reaches the DOM.
///
/// SVG defines `viewBox` as four SVG numbers; `NaN`/`±infinity` are not valid SVG numbers even though `f64` can
/// represent them and `write!`/`Display` can format them without error, so every component must be finite and numeric.
///
/// As per the SVG spec, setting `width` or `height` to negative values causes the whole attribute to be in error, so
/// both must be non-negative. Setting either `width` or `height` to be `0.0` is valid and is used to disable rendering.
///
/// Shared by [`SvgRoot::set_view_box`](crate::SvgRoot::set_view_box),
/// [`SvgSymbol::set_view_box`](crate::SvgSymbol::set_view_box), and
/// [`SvgPattern::set_view_box`](crate::SvgPattern::set_view_box), so the three setters that accept a `viewBox`
/// agree on what counts as valid rather than each silently formatting whatever `f64`s they were given.
pub(crate) fn validate_view_box(x: f64, y: f64, width: f64, height: f64) -> Result<(), crate::Error> {
    if !x.is_finite() || !y.is_finite() || !width.is_finite() || !height.is_finite() {
        return Err(crate::Error::InvalidViewBox("all viewBox components must be finite"));
    }
    if width < 0.0 || height < 0.0 {
        return Err(crate::Error::InvalidViewBox("viewBox width and height must not be negative"));
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Maximum `dps` accepted by [`write_points`] in fixed-precision mode.
///
/// `f64` carries ~17 significant decimal digits; values above this limit produce only meaningless trailing zeros
/// and can generate enormous strings. Callers that pass a higher value are clamped to this constant.
pub(crate) const MAX_DPS: usize = 20;

/// Formats `points` into `out` as an SVG `points` list (`"x,y x,y ..."`), replacing any previous contents.
///
/// `dps` selects the per-coordinate precision: `None` uses the default shortest round-trip `Display`, while `Some(n)`
/// writes each coordinate with `n` fixed decimal places (clamped to [`MAX_DPS`] = 20).
/// Fixed precision yields a shorter string for large animated polylines, where the full-precision text would
/// otherwise dominate the per-frame data crossing the WASM/JS boundary.
///
/// Shared by the `points` / `points_fixed` methods on [`SvgAttrs`](crate::SvgAttrs) / [`AttrWriter`](crate::AttrWriter)
/// and [`AnimationFrame`](crate::AnimationFrame), so all of them produce identical output from one reusable buffer.
pub(crate) fn write_points(out: &mut String, points: &[Point], dps: Option<usize>) {
    use std::fmt::Write;
    out.clear();
    if !points.is_empty() {
        // Reserve a rough lower bound so the first call on an empty/small buffer does not repeatedly reallocate as the
        // list grows.  Subsequent calls reuse retained capacity.
        //
        // For the fixed-precision path: 2*n extra bytes for fractional digits across both coordinates, plus 12 bytes
        // for integer parts, decimal points, comma, and space.
        let approx_per_point = match dps {
            Some(n) => n.min(MAX_DPS).saturating_mul(2).saturating_add(12),
            None => 24,
        };
        out.reserve(points.len().saturating_mul(approx_per_point));
    }
    for (i, p) in points.iter().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        // Writing to a `String` is infallible.
        let _ = match dps {
            Some(n) => write!(out, "{:.*},{:.*}", n.min(MAX_DPS), p.x, n.min(MAX_DPS), p.y),
            None => write!(out, "{},{}", p.x, p.y),
        };
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[cfg(test)]
mod unit_tests;
