use crate::{SvgRoot, error::Error, node::SvgNode};

impl SvgRoot {
    /// Creates a `<circle>` element, appends it to the root, and returns its [`SvgNode`] handle.
    ///
    /// # Arguments
    ///
    /// * `cx`, `cy` — centre point of the circle in pixels
    /// * `r` — radius in pixels
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use svg_dom::SvgRoot;
    /// let svg = SvgRoot::attach("diagram")?;
    /// let circle = svg.circle(100.0, 100.0, 30.0)?;
    /// circle.set_fill("steelblue")?;
    /// # Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn circle(&self, cx: f64, cy: f64, r: f64) -> Result<SvgNode, Error> {
        let n = self.append_new("circle")?;
        n.set_attr("cx", &cx.to_string())?;
        n.set_attr("cy", &cy.to_string())?;
        n.set_attr("r", &r.to_string())?;
        Ok(n)
    }
}
