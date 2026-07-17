// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// An SVG coordinate point
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    /// Horizontal coordinate, in user units (usually pixels).
    pub x: f64,
    /// Vertical coordinate, in user units (usually pixels).
    pub y: f64,
}

impl Point {
    /// Returns the origin point `(0, 0)`.
    pub fn origin() -> Self {
        Self::new(0.0, 0.0)
    }

    /// Creates a point at `(x, y)`.
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Returns the `x` coordinate as a freshly allocated `String`.
    ///
    /// Deprecated: each call allocates and discards a `String`, which sits awkwardly beside the allocation-light path
    /// the rest of the crate now uses. Format through a reused buffer instead with
    /// [`SvgNode::set_attr_display`](crate::SvgNode::set_attr_display), [`SvgAttrs::display`](crate::SvgAttrs::display),
    /// or [`AttrWriter::display`](crate::AttrWriter::display).
    #[deprecated(
        since = "0.1.39",
        note = "allocates a String per call; instead, format through a reused buffer with SvgNode::set_attr_display, SvgAttrs::display, or AttrWriter::display"
    )]
    pub fn get_x_str(&self) -> String {
        self.x.to_string()
    }

    /// Returns the `y` coordinate as a freshly allocated `String`.
    ///
    /// Deprecated for the same reason as [`get_x_str`](Self::get_x_str); use the allocation-light setters named there.
    #[deprecated(
        since = "0.1.39",
        note = "allocates a String per call; instead, format through a reused buffer with SvgNode::set_attr_display, SvgAttrs::display, or AttrWriter::display"
    )]
    pub fn get_y_str(&self) -> String {
        self.y.to_string()
    }
}

impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.x, self.y)
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// The size of an SVG element in pixels.
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    /// Width in user units (usually pixels).
    pub width: f64,
    /// Height in user units (usually pixels).
    pub height: f64,
}

impl Size {
    /// Creates a size of `width` × `height`.
    pub fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }

    /// Returns the width as a freshly allocated `String`.
    ///
    /// Deprecated: each call allocates and discards a `String`, which sits awkwardly beside the allocation-light path
    /// the rest of the crate now uses. Format through a reused buffer instead with
    /// [`SvgNode::set_attr_display`](crate::SvgNode::set_attr_display), [`SvgAttrs::display`](crate::SvgAttrs::display),
    /// or [`AttrWriter::display`](crate::AttrWriter::display).
    #[deprecated(
        since = "0.1.39",
        note = "allocates a String per call; instead, format through a reused buffer with SvgNode::set_attr_display, SvgAttrs::display, or AttrWriter::display"
    )]
    pub fn get_width_str(&self) -> String {
        self.width.to_string()
    }

    /// Returns the height as a freshly allocated `String`.
    ///
    /// Deprecated for the same reason as [`get_width_str`](Self::get_width_str); use the allocation-light setters named there.
    #[deprecated(
        since = "0.1.39",
        note = "allocates a String per call; instead, format through a reused buffer with SvgNode::set_attr_display, SvgAttrs::display, or AttrWriter::display"
    )]
    pub fn get_height_str(&self) -> String {
        self.height.to_string()
    }

    /// Returns the area (`width * height`), in square user units (usually pixels).
    pub fn get_area(&self) -> f64 {
        self.width * self.height
    }
}

impl std::fmt::Display for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.width, self.height)
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// An axis-aligned rectangle, returned by [`SvgNode::bounding_box`](crate::SvgNode::bounding_box) and
/// [`SvgNode::bounding_client_rect`](crate::SvgNode::bounding_client_rect).
///
/// # Two producers, two different coordinate spaces
///
/// These two methods both return a `Rect`, but the coordinates are **not interchangeable**:
///
/// * [`bounding_box`](crate::SvgNode::bounding_box) wraps the no-argument form of `getBBox()` and reports **local,
///   user-space** coordinates — the same coordinate system the element's own `x`/`y`/`d`/`points` attributes are
///   authored in, unaffected by any transform applied to the element or its ancestors. It is also the **object/fill**
///   bounding box only: stroke width, markers, and clipping are not included (see
///   [`bounding_box`](crate::SvgNode::bounding_box)'s own doc comment). Empirically, in Chromium at least,
///   `getBoundingClientRect()` reports this same fill-only extent for SVG shape elements too — a wide stroke does
///   not necessarily widen either box, so do not assume `bounding_client_rect` is the "include everything painted"
///   alternative to `bounding_box`; verify against the specific browsers you target if that distinction matters.
/// * [`bounding_client_rect`](crate::SvgNode::bounding_client_rect) wraps `getBoundingClientRect()` and reports
///   **rendered CSS pixels**, relative to the browser viewport, after every transform, `viewBox` scale, and CSS zoom
///   has been applied.
///
/// The two will differ whenever any transform, `viewBox`, or CSS scaling is in play. Do not feed one method's `Rect`
/// into code that assumes the other's coordinate space — see `docs/rejected_ideas.md` ("Provide a rendered-size
/// fallback...") for a worked example of exactly this mistake.
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    /// The rectangle's origin (top-left corner) — see the coordinate-space note above for which space this is in,
    /// depending on which method produced this `Rect`.
    pub origin: Point,
    /// The rectangle's size — see the coordinate-space note above.
    pub size: Size,
}

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
