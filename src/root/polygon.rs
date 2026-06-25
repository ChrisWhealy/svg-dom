use crate::{
    SvgRoot,
    error::Error,
    node::SvgNode,
    root::{factory::SvgFactory, utils::Point},
};

impl SvgRoot {
    /// Creates a `<polygon>` element with the given vertices, appends it to the root, and returns its [`SvgNode`]
    /// handle.
    ///
    /// A polygon is like a [`polyline`](Self::polyline) except the final point is automatically joined back to the
    /// first, producing a closed, fillable shape.
    ///
    /// To update the vertices later without allocating a new string each time (e.g. an animated polygon), use
    /// [`SvgAttrs::points`](crate::SvgAttrs::points).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use svg_dom::{SvgRoot, root::utils::Point};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let tri = svg.polygon(&[Point::new(50.0, 0.0), Point::new(100.0, 80.0), Point::new(0.0, 80.0)])?;
    /// tri.set_fill("coral")?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn polygon(&self, points: &[Point]) -> Result<SvgNode, Error> {
        self.create_polygon(points)
    }
}
