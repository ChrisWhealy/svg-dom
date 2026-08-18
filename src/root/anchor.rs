use crate::{SvgRoot, error::Error, node::SvgNode, root::factory::SvgFactory};

impl SvgRoot {
    /// Creates an anchor (`<a>`) element, appends it to the root and returns its [`SvgNode`] handle.
    ///
    /// `<a>` is similar to `<g>` in that it has no visual appearance of its own, but turns every child appended to it
    /// (via [`SvgNode::append`]) into a hyperlink.
    ///
    /// When the user clicks on such a rendered element, the browser navigates to `href`, just as it would for an HTML
    /// `<a>` wrapped around its own child elements.
    ///
    /// # Arguments
    ///
    /// * `href` — the link target. Accepts anything a browser can navigate to: a relative path, an absolute URL, or
    ///   a same-document fragment (`"#section"`).
    ///
    /// `target` uses the same vocabulary as HTML `<a target>`. It responds to `"_blank"`, `"_self"`, `"_parent"`,
    /// `"_top"`, or a named frame, but this crate does not wrap it in a named parameter. Every meaningful use of
    /// `<a>` must supply `href`, but `target` is only occasionally needed. Set it, along with any other attribute
    /// (`download`, `rel`, etc.) not covered here, via [`SvgNode::set_attr`](crate::SvgNode::set_attr).
    ///
    /// ***⚠️ Links cannot be nested***
    ///
    /// Just as in HTML, nested links are invalid. An `<a>` appended somewhere inside another `<a>` has its own `href`
    /// ignored and is inactive. [`SvgNode::append`] does not check for this, so you must take care to avoid appending
    /// the result of one [`anchor`](Self::anchor) call inside another.
    ///
    /// ***⚠️ The clickable region is each child's own hit region, not the wrapper's bounding box***
    ///
    /// Unlike wrapping children in a `<g>` to implement a shared transform, `<a>` does not make the whole rectangular
    /// area spanning its children clickable. Only points within each rendered child's `pointer-events`-defined hit
    /// region are clickable. And this is not necessarily identical to its visibly painted pixels, since `fill`,
    /// `stroke`, `visibility` and `pointer-events` itself all influence what that region actually covers.
    /// There may be empty space between or around the children within what looks like the group's bounding box,
    /// but this does not automatically become part of the clickable link.
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
