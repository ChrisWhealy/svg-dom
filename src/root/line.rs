use crate::{SvgRoot, error::Error, node::SvgNode};

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
    /// let wire = svg.line(50.0, 100.0, 250.0, 100.0)?;
    /// wire.set_stroke("grey")?;
    /// wire.set_stroke_width(2.0)?;
    /// # Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn line(&self, x1: f64, y1: f64, x2: f64, y2: f64) -> Result<SvgNode, Error> {
        let n = self.append_new("line")?;
        n.set_attr("x1", &x1.to_string())?;
        n.set_attr("y1", &y1.to_string())?;
        n.set_attr("x2", &x2.to_string())?;
        n.set_attr("y2", &y2.to_string())?;
        Ok(n)
    }
}
