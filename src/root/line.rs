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
        let mut scratch = String::new();
        n.set_attr_display("x1", start.x, &mut scratch)?;
        n.set_attr_display("y1", start.y, &mut scratch)?;
        n.set_attr_display("x2", end.x, &mut scratch)?;
        n.set_attr_display("y2", end.y, &mut scratch)?;
        self.append_node(&n)?;
        Ok(n)
    }
}
