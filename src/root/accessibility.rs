use crate::{SvgNode, SvgRoot, error::Error};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Forwards SvgNode's <title>/<desc> accessibility helpers onto SvgRoot, since the root <svg> — the natural place to
// give a whole document/diagram its accessible name — is a separate wrapper type, not an SvgNode. Naming the root is
// one of the principal accessibility use cases for these helpers, not an edge case, so it belongs here rather than
// being left as something only reachable via `SvgNode::new` internals the caller cannot construct.
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl SvgRoot {
    /// # `<title>` child element on the root `<svg>`
    ///
    /// Forwards to [`SvgNode::set_title`] on the root `<svg>` element — see that method for the full explanation of
    /// ARIA precedence, the `# Use judiciously` and `# Scope` sections, and the errors this can return. All of it
    /// applies equally here.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::SvgRoot;
    /// let svg = SvgRoot::attach("diagram")?;
    /// svg.set_title("Quarterly sales chart")?;
    /// svg.set_desc("A bar chart comparing sales across four regions")?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn set_title(&self, text: &str) -> Result<(), Error> {
        self.as_node().set_title(text)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Returns the text of the root `<svg>`'s first direct `<title>` child, or `None` if it has none. Forwards to
    /// [`SvgNode::title`] — see that method for why this is not necessarily the document's computed accessible name.
    pub fn title(&self) -> Option<String> {
        self.as_node().title()
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Removes the root `<svg>`'s first direct `<title>` child, if one exists. Forwards to [`SvgNode::remove_title`].
    pub fn remove_title(&self) {
        self.as_node().remove_title();
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// # `<desc>` child element on the root `<svg>`
    ///
    /// Forwards to [`SvgNode::set_desc`] on the root `<svg>` element — see that method (and
    /// [`SvgNode::set_title`]'s `# Use judiciously`/`# Scope` sections, which it shares) for the full explanation.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::SvgRoot;
    /// let svg = SvgRoot::attach("diagram")?;
    /// svg.set_desc("A bar chart comparing sales across four regions")?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn set_desc(&self, text: &str) -> Result<(), Error> {
        self.as_node().set_desc(text)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Returns the text of the root `<svg>`'s first direct `<desc>` child, or `None` if it has none. Forwards to
    /// [`SvgNode::desc`] — see that method for why this is not necessarily the document's computed accessible
    /// description.
    pub fn desc(&self) -> Option<String> {
        self.as_node().desc()
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Removes the root `<svg>`'s first direct `<desc>` child, if one exists. Forwards to [`SvgNode::remove_desc`].
    pub fn remove_desc(&self) {
        self.as_node().remove_desc();
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // A fresh, independent SvgNode handle wrapping the same underlying root <svg> element — the same "fresh,
    // independent owner" idiom SvgNode's own tree-navigation methods (parent(), first_child(), ...) already use, not
    // a new concept. None of the six methods above register listeners or otherwise need this handle to outlive the
    // call, so a throwaway wrapper per call is fine.
    fn as_node(&self) -> SvgNode {
        SvgNode::new(self.root.clone().into())
    }
}
