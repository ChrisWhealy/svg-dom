use crate::{
    SvgRoot,
    error::Error,
    node::SvgNode,
    root::{factory::SvgFactory, utils::Point},
};

impl SvgRoot {
    /// Creates a `<polyline>` element through the given points, appends it to the root, and returns its [`SvgNode`]
    /// handle.
    ///
    /// A polyline draws connected straight segments through every point. Unlike [`polygon`](Self::polygon) it is not
    /// closed (the last point is not joined back to the first). It is still filled by default, so set `fill` to
    /// `"none"` for an open multi-segment line.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use svg_dom::{SvgRoot, root::utils::Point};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let line = svg.polyline(&[Point::new(0.0, 0.0), Point::new(20.0, 40.0), Point::new(40.0, 0.0)])?;
    /// line.set_fill("none")?;
    /// line.set_stroke("teal")?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn polyline(&self, points: &[Point]) -> Result<SvgNode, Error> {
        self.create_polyline(points)
    }
}
