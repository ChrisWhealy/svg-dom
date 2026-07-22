use crate::{SvgRoot, error::Error, node::SvgNode, root::factory::SvgFactory};

impl SvgRoot {
    /// Creates an `<a>` element, appends it to the root, and returns its [`SvgNode`] handle.
    ///
    /// `<a>` is a `<g>`-like wrapper: it has no visual appearance of its own, but turns every child appended to it
    /// (via [`SvgNode::append`]) into a hyperlink — clicking any rendered child navigates to `href`, the same way an
    /// HTML `<a>` around several elements would.
    ///
    /// # Arguments
    ///
    /// * `href` — the link target. Accepts anything a browser can navigate to: a relative path, an absolute URL, or
    ///   a same-document fragment (`"#section"`).
    ///
    /// `target` (`"_blank"`, `"_self"`, `"_parent"`, `"_top"`, or a named frame — the same vocabulary as HTML
    /// `<a target>`) is not wrapped by a named parameter: every meaningful use of `<a>` supplies `href`, but `target`
    /// is only occasionally needed, so it is left to [`SvgNode::set_attr`](crate::SvgNode::set_attr) alongside any
    /// other attribute (`download`, `rel`, ...) not covered here.
    ///
    /// # Security
    ///
    /// ⚠️ The `href` value is written verbatim to the DOM via `setAttribute`!
    /// Do not pass a `javascript:` URL or any other attacker-controlled string without validation.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if the browser refuses to create or append the `<a>` element.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::utils::Point};
    ///
    /// let svg = SvgRoot::attach("diagram")?;
    /// let link = svg.anchor("https://example.com")?;
    /// link.set_attr("target", "_blank")?;
    ///
    /// // Both the icon and its label become part of the same hyperlink.
    /// let icon = svg.circle(Point::new(30.0, 30.0), 18.0)?;
    /// let label = svg.text(Point::new(56.0, 35.0), "Learn more")?;
    /// link.append(&icon)?;
    /// link.append(&label)?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn anchor(&self, href: &str) -> Result<SvgNode, Error> {
        self.create_anchor(href)
    }
}
