use crate::{SvgRoot, error::Error, node::SvgNode, root::utils::Point};

impl SvgRoot {
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<text>` element with initial string content, appends it to the root and returns its [`SvgNode`]
    /// handle.
    ///
    /// # Arguments
    ///
    /// * `anchored_at` — position of the text anchor point where the `y` coordinate is the **baseline** of the first
    ///                   line of text, not the top left corner of the bounding box.
    /// * `content`     — the visible text string.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::utils::Point};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let location = Point::new(50.0, 30.0);
    /// let label = svg.text(location, "SHA-256 round")?;
    /// label.set_attr("font-size", "14")?;
    /// label.set_fill("white")?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn text(&self, anchored_at: Point, content: &str) -> Result<SvgNode, Error> {
        let n = self.make_node("text")?;
        n.set_attrs([
            ("x", anchored_at.get_x_str()),
            ("y", anchored_at.get_y_str()),
        ])?;
        n.as_element().set_text_content(Some(content));
        self.append_node(&n)?;
        Ok(n)
    }
}
