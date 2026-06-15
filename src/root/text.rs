use crate::{SvgRoot, error::Error, node::SvgNode};

impl SvgRoot {
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<text>` element with initial string content, appends it to the root and returns its [`SvgNode`]
    /// handle.
    ///
    /// # Arguments
    ///
    /// * `x`, `y` — position of the text anchor point.
    ///              `y` is the **baseline** of the first line of text, not the top of the bounding box.
    /// * `content` — the visible string.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use svg_dom::SvgRoot;
    /// let svg = SvgRoot::attach("diagram")?;
    /// let label = svg.text(50.0, 30.0, "SHA-256 round")?;
    /// label.set_attr("font-size", "14")?;
    /// label.set_fill("white")?;
    /// # Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn text(&self, x: f64, y: f64, content: &str) -> Result<SvgNode, Error> {
        let el = self.make_element("text")?;
        el.set_attribute("x", &x.to_string())
            .map_err(|e| Error::Dom(format!("{e:?}")))?;
        el.set_attribute("y", &y.to_string())
            .map_err(|e| Error::Dom(format!("{e:?}")))?;
        el.set_text_content(Some(content));
        self.root
            .append_child(&el)
            .map_err(|e| Error::Dom(format!("{e:?}")))?;
        Ok(SvgNode::new(el))
    }
}
