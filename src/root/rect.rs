use crate::{SvgRoot, error::Error, node::SvgNode};

impl SvgRoot {
    /// Creates a `<rect>` element, appends it to the root, and returns its [`SvgNode`] handle.
    ///
    /// # Arguments
    ///
    /// * `x`, `y` — position of the top-left corner, in user units.
    /// * `w`, `h` — width and height, in user units.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use svg_dom::SvgRoot;
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let rect = svg.rect(10.0, 20.0, 120.0, 60.0)?;
    /// rect.set_fill("tomato")?;
    /// rect.set_stroke("black")?;
    /// # Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn rect(&self, x: f64, y: f64, w: f64, h: f64) -> Result<SvgNode, Error> {
        let n = self.append_new("rect")?;
        n.set_attr("x", &x.to_string())?;
        n.set_attr("y", &y.to_string())?;
        n.set_attr("width", &w.to_string())?;
        n.set_attr("height", &h.to_string())?;
        Ok(n)
    }
}
