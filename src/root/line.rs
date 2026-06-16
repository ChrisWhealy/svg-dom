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
    /// # use svg_dom::SvgRoot;
    /// let svg = SvgRoot::attach("diagram")?;
    /// let start = Point::new(50.0, 100.0);
    /// let end = Point::new(250.0, 100.0);
    /// let wire = svg.line(start, end)?;
    /// wire.set_stroke("grey")?;
    /// wire.set_stroke_width(2.0)?;
    /// # Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn line(&self, start: Point, end: Point) -> Result<SvgNode, Error> {
        let n = self.append_new("line")?;
        n.set_attr("x1", &start.get_x_str())?;
        n.set_attr("y1", &start.get_y_str())?;
        n.set_attr("x2", &end.get_x_str())?;
        n.set_attr("y2", &end.get_y_str())?;
        Ok(n)
    }
}
