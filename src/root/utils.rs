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
    /// Convenient for one-off use, but should be avoided on hot paths (per-event or per-frame) since each call
    /// allocates and then discards a String.
    ///
    /// Instead, format through a reused buffer: see [`SvgNode::set_attr_display`](crate::SvgNode::set_attr_display)
    /// or the [transform setters](crate::SvgNode::set_translate).
    pub fn get_x_str(&self) -> String {
        self.x.to_string()
    }

    /// Returns the `y` coordinate as a freshly allocated `String`.
    ///
    /// See [`get_x_str`](Self::get_x_str) for the hot-path caveat.
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
    /// Convenient for one-off use, but should be avoided on hot paths (per-event or per-frame) since each call
    /// allocates and then discards a String.
    ///
    /// Instead, format through a reused buffer: see [`SvgNode::set_attr_display`](crate::SvgNode::set_attr_display)
    /// or the [transform setters](crate::SvgNode::set_translate).
    pub fn get_width_str(&self) -> String {
        self.width.to_string()
    }

    /// Returns the height as a freshly allocated `String`.
    ///
    /// See [`get_width_str`](Self::get_width_str) for the hot-path caveat.
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
