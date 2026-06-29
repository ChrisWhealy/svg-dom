use crate::{
    SvgRoot,
    error::Error,
    node::SvgNode,
    root::{factory::SvgFactory, utils::Point},
};

impl SvgRoot {
    /// Creates a `<use>` element, appends it to the root, and returns its [`SvgNode`] handle.
    ///
    /// `<use>` stamps a copy of an existing element (typically one defined in a [`SvgDefs`](crate::SvgDefs) container),
    /// into the rendered SVG tree without duplicating the underlying DOM node.
    /// Any change to the original definition is immediately reflected in all copies.
    ///
    /// # Arguments
    ///
    /// * `href` — a local fragment reference to the element being reused, e.g. `"#my-shape"`, where `"my-shape"` is the
    ///   `id` attribute of the target element.
    ///   The referenced element is typically defined inside `<defs>` and is not rendered directly.
    /// * `at` — position in the parent coordinate system, written as `x`/`y` attributes on the `<use>` element.
    ///   These apply an additional translation on top of any coordinate already implied by the `transform` attribute.
    ///   Pass [`Point::origin`](Point::origin) when you intend to control positioning entirely via `transform`.
    ///
    /// Each `<use>` instance receives its own [`SvgNode`] handle and can have independent attributes — `transform`,
    /// `opacity`, `fill`, `stroke` — applied to it without affecting the original or any other copy.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if the browser refuses to create or append the element.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use svg_dom::{SvgRoot, root::utils::Point};
    /// let svg = SvgRoot::attach("diagram")?;
    ///
    /// // Define a reusable shape in <defs>.
    /// svg.build_defs(|d| {
    ///     let shape = d.circle(Point::origin(), 20.0)?;
    ///     shape.set_attr("id", "dot")?;
    ///     Ok(())
    /// })?;
    ///
    /// // Stamp three independent copies at different positions.
    /// svg.use_node("#dot", Point::new(50.0, 60.0))?;
    /// svg.use_node("#dot", Point::new(150.0, 60.0))?;
    /// svg.use_node("#dot", Point::new(250.0, 60.0))?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn use_node(&self, href: &str, at: Point) -> Result<SvgNode, Error> {
        self.create_use(href, at)
    }
}
