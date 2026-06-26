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
/// Formats `points` into `out` as an SVG `points` list (`"x,y x,y …"`), replacing any previous contents.
///
/// `dps` selects the per-coordinate precision: `None` uses the default shortest round-trip `Display`, while `Some(n)`
/// writes each coordinate with `n` fixed decimal places. Fixed precision yields a shorter string for large animated
/// polylines, where the full-precision text would otherwise dominate the per-frame data crossing the WASM/JS boundary.
///
/// Shared by the `points` / `points_fixed` methods on [`SvgAttrs`](crate::SvgAttrs) / [`AttrWriter`](crate::AttrWriter)
/// and [`AnimationFrame`](crate::AnimationFrame), so all of them produce identical output from one reusable buffer.
pub(crate) fn write_points(out: &mut String, points: &[Point], dps: Option<usize>) {
    use std::fmt::Write;
    out.clear();
    for (i, p) in points.iter().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        // Writing to a `String` is infallible.
        let _ = match dps {
            Some(n) => write!(out, "{:.*},{:.*}", n, p.x, n, p.y),
            None => write!(out, "{},{}", p.x, p.y),
        };
    }
}
