use crate::{
    SvgRoot,
    error::Error,
    node::SvgNode,
    root::{
        factory::SvgFactory,
        utils::{Point, Size},
    },
};

impl SvgRoot {
    /// Creates an `<ellipse>` element, appends it to the root, and returns its [`SvgNode`] handle.
    ///
    /// Unlike [`circle`](Self::circle), an ellipse has independent horizontal and vertical radii.
    ///
    /// # Arguments
    ///
    /// * `centre` — centre point of the ellipse, in user units.
    /// * `radii` — the horizontal radius (`width` → `rx`) and vertical radius (`height` → `ry`).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use svg_dom::{SvgRoot, root::utils::{Point, Size}};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let ellipse = svg.ellipse(Point::new(100.0, 60.0), Size::new(80.0, 40.0))?;
    /// ellipse.set_fill("plum")?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn ellipse(&self, centre: Point, radii: Size) -> Result<SvgNode, Error> {
        self.create_ellipse(centre, radii)
    }
}
