use crate::{
    SvgRoot,
    error::Error,
    node::SvgNode,
    root::{factory::SvgFactory, utils::Point},
};

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
        self.create_line(start, end)
    }
}
