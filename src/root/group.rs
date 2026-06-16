use crate::{SvgRoot, error::Error, node::SvgNode};

impl SvgRoot {
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<g>` group element, appends it to the root, and returns its [`SvgNode`] handle.
    ///
    /// A `<g>` element has no visual appearance of its own; it is a container used to transform, clip, or style a set
    /// of child elements together.  Add children using [`SvgNode::append`].
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::utils::{Point, Size}};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let group = svg.group()?;
    ///
    /// // Both children move with the group when its transform is updated.
    /// let box_size = Size::new(80.0, 40.0);
    /// let box_ = svg.rect(Point::origin(), box_size)?;
    /// let label_anchor = Point::new(10.0, 26.0);
    /// let label = svg.text(label_anchor, "XOR")?;
    /// group.append(&box_)?;
    /// group.append(&label)?;
    ///
    /// // Translate the whole group.
    /// group.set_attr("transform", "translate(120, 60)")?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn group(&self) -> Result<SvgNode, Error> {
        self.append_new("g")
    }
}
