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
