use crate::{
    error::Error,
    node::SvgNode,
    root::{factory::SvgFactory, utils::Point},
    SvgRoot,
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
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
        self.create_circle(centre, radius)
    }
}
