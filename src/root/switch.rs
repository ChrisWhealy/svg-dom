use crate::{SvgRoot, error::Error, node::SvgNode, root::factory::SvgFactory};

impl SvgRoot {
    /// Creates a `<switch>` element, appends it to the root, and returns its [`SvgNode`] handle.
    ///
    /// `<switch>` renders at most one of its direct children: the first one, in document order, whose conditional
    /// processing attributes all evaluate to true, rather than rendering every child as `<g>` would.
    ///
    /// As per the SVG 2 specification, if none match, it renders **nothing**. A child with none of those attributes set
    /// always passes, so by appending an attribute-free element last (in document order), you create a fallback that
    /// guarantees something renders even when every other conditional child fails.
    ///
    /// Add children with [`SvgNode::append`], the same way as [`group`](Self::group). The conditional attributes
    /// themselves — `systemLanguage`, `requiredExtensions` (`requiredFeatures` existed in earlier SVG versions but
    /// was removed from SVG 2 because it proved unreliable as a feature-support test) — are not wrapped by named
    /// parameters; set them directly on each child via
    /// [`SvgNode::set_attr`](crate::SvgNode::set_attr)/[`set_attrs`](crate::SvgNode::set_attrs). This crate performs
    /// no validation of its own on them: the browser evaluates each child's test attributes and picks the first
    /// match at render time.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if the browser refuses to create or append the `<switch>` element.
    ///
    /// # Example
    ///
    /// Show a localised label, falling back to English if neither `systemLanguage` matches:
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::utils::Point};
    ///
    /// let svg = SvgRoot::attach("diagram")?;
    /// let switch = svg.switch()?;
    ///
    /// let french = svg.text(Point::new(10.0, 30.0), "Bonjour")?;
    /// french.set_attr("systemLanguage", "fr")?;
    /// let german = svg.text(Point::new(10.0, 30.0), "Hallo")?;
    /// german.set_attr("systemLanguage", "de")?;
    /// let fallback = svg.text(Point::new(10.0, 30.0), "Hello")?;
    ///
    /// switch.append(&french)?;
    /// switch.append(&german)?;
    /// switch.append(&fallback)?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn switch(&self) -> Result<SvgNode, Error> {
        self.create_switch()
    }
}
