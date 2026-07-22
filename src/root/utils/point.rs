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
