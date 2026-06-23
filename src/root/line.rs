use crate::{SvgRoot, error::Error, node::SvgNode, root::utils::Point};

impl SvgRoot {
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<line>` element between two points, appends it to the root, and returns its [`SvgNode`] handle.
    ///
    /// A line has no fill; give it a `stroke` to make it visible.
    ///
    /// # Arguments
    ///
    /// * `x1`, `y1` — start point.
    /// * `x2`, `y2` — end point.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use svg_dom::{SvgRoot, root::utils::Point};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let start = Point::new(50.0, 100.0);
    /// let end = Point::new(250.0, 100.0);
    /// let wire = svg.line(start, end)?;
    /// wire.set_stroke("grey")?;
    /// wire.set_stroke_width(2.0)?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn line(&self, start: Point, end: Point) -> Result<SvgNode, Error> {
        let n = self.make_node("line")?;
        n.set_attrs([
            ("x1", start.get_x_str()),
            ("y1", start.get_y_str()),
            ("x2", end.get_x_str()),
            ("y2", end.get_y_str()),
        ])?;
        self.append_node(&n)?;
        Ok(n)
    }
}
