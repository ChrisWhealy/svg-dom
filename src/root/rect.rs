use crate::{SvgRoot, error::Error, node::SvgNode, root::utils::{Point, Size}};

impl SvgRoot {
    /// Creates a `<rect>` element, appends it to the root, and returns its [`SvgNode`] handle.
    ///
    /// # Arguments
    ///
    /// * `top_left` — position of the top-left corner, in user units.
    /// * `w`, `h` — width and height, in user units.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use svg_dom::{SvgRoot, root::utils::{Point, Size}};
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let top_left = Point::new(10.0, 20.0);
    /// let size = Size::new(120.0, 60.0);
    /// let rect = svg.rect(top_left, size)?;
    /// rect.set_fill("tomato")?;
    /// rect.set_stroke("black")?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn rect(&self, top_left: Point, size: Size) -> Result<SvgNode, Error> {
        let n = self.make_node("rect")?;
        let mut scratch = String::new();
        n.set_attr_display("x", top_left.x, &mut scratch)?;
        n.set_attr_display("y", top_left.y, &mut scratch)?;
        n.set_attr_display("width", size.width, &mut scratch)?;
        n.set_attr_display("height", size.height, &mut scratch)?;
        self.append_node(&n)?;
        Ok(n)
    }
}
