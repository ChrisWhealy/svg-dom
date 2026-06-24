use crate::{SvgRoot, error::Error, node::SvgNode, root::utils::Point};

impl SvgRoot {
    /// Creates a `<circle>` element, appends it to the root, and returns its [`SvgNode`] handle.
    ///
    /// # Arguments
    ///
    /// * `centre` — centre point of the circle in pixels
    /// * `r` — radius in pixels
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::utils::Point};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let centre = Point::new(100.0, 100.0);
    /// let circle = svg.circle(centre, 30.0)?;
    /// circle.set_fill("steelblue")?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn circle(&self, centre: Point, radius: f64) -> Result<SvgNode, Error> {
        let n = self.make_node("circle")?;
        let mut scratch = String::new();
        n.set_attr_display("cx", centre.x, &mut scratch)?;
        n.set_attr_display("cy", centre.y, &mut scratch)?;
        n.set_attr_display("r", radius, &mut scratch)?;
        self.append_node(&n)?;
        Ok(n)
    }
}
