use super::super::SvgFilter;
use crate::{Error, SvgNode, dom_err, root::create_svg_element};
use web_sys::SvgElement;

impl SvgFilter {
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Appends a `<feMerge>` primitive to this filter, stacking `inputs` on top of one another in the given order
    /// (later entries painted last, i.e. on top).
    ///
    /// Each entry in `inputs` becomes one `<feMergeNode in="...">` child, in order — the standard way to layer, for
    /// example, an offset blurred shadow underneath the original graphic: `merge(&["offset-blur", "SourceGraphic"])`.
    ///
    /// Unlike [`gaussian_blur`](Self::gaussian_blur) and [`offset`](Self::offset), `<feMerge>` has no attributes of its
    /// own to set beyond the generic `result` — its content is entirely the ordered list of `<feMergeNode>` children
    /// this method builds, so there is nothing for the returned [`SvgNode`]'s [`set_attr`](crate::SvgNode::set_attr) to
    /// configure except `result`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if the browser refuses to create or append the `<feMerge>` element or any of its
    /// `<feMergeNode>` children.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::SvgRoot;
    ///
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    /// let shadow = defs.filter("shadow")?;
    /// shadow.gaussian_blur(4.0)?.set_attrs([("in", "SourceAlpha"), ("result", "blur")])?;
    /// shadow.offset(4.0, 4.0)?.set_attrs([("in", "blur"), ("result", "offset-blur")])?;
    /// shadow.merge(&["offset-blur", "SourceGraphic"])?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn merge(&self, inputs: &[&str]) -> Result<SvgNode, Error> {
        let el = create_svg_element::<SvgElement>(&self.document, "feMerge", "SvgElement")?;
        for input in inputs {
            let node = create_svg_element::<SvgElement>(&self.document, "feMergeNode", "SvgElement")?;
            node.set_attribute("in", input).map_err(dom_err)?;
            el.append_child(&node).map_err(dom_err)?;
        }
        self.element.append_child(&el).map_err(dom_err)?;
        Ok(SvgNode::new(el))
    }
}
