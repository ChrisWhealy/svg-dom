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
        let node = self.make_node("line")?;
        {
            let mut attrs = self.attrs.borrow_mut();
            attrs.display(&node, "x1", start.x)?;
            attrs.display(&node, "y1", start.y)?;
            attrs.display(&node, "x2", end.x)?;
            attrs.display(&node, "y2", end.y)?;
        }
        self.append_node(&node)?;
        
        Ok(node)
    }
}
