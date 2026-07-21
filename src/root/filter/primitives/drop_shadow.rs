use super::super::SvgFilter;
use crate::{Error, SvgNode, dom_err, root::create_svg_element};
use web_sys::SvgElement;

impl SvgFilter {
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Appends a `<feDropShadow>` primitive to this filter, which is SVG shorthand for the entire chain shown in
    /// [`composite`](Self::composite)'s example:
    ///
    /// [`gaussian_blur`](Self::gaussian_blur) → [`flood`](Self::flood) → [`composite`](Self::composite) →
    /// [`offset`](Self::offset) → [`merge`](Self::merge),
    ///
    /// - `std_deviation` is the blur radius (as [`gaussian_blur`](Self::gaussian_blur))
    /// - `dx`/`dy` are the shadow offset (as [`offset`](Self::offset))
    /// - `color`/`opacity` are the shadow's `flood-color`/`flood-opacity` (as [`flood`](Self::flood))
    ///
    /// # This primitive already merges the original graphic on top, so there is no need to call [`merge`](Self::merge)
    /// after it
    ///
    /// As per the SVG specification, `<feDropShadow>`'s result already includes its `in` input painted over the shadow,
    /// exactly as the final `merge(&[shadow, "SourceGraphic"])` step does in the manual chain.
    ///
    /// A `<filter>` containing only `drop_shadow(...)` is already a complete, ready-to-use shadow effect; adding a
    /// further `merge` call would paint the original graphic on top a second time.
    ///
    /// If this is the filter's first (and only) primitive, its implicit `in` is `SourceGraphic`, and that is also what
    /// gets composited back on top; which is the common case this shorthand exists for.
    ///
    /// Use the returned [`SvgNode`]'s [`set_attr`](crate::SvgNode::set_attr) to set `in` or `result` explicitly for
    /// anything else.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if the browser refuses to create or append the `<feDropShadow>` element.
    ///
    /// # Example
    ///
    /// The five-primitive chain from [`composite`](Self::composite)'s example, collapsed to one call:
    ///
    /// ```rust,no_run
    /// use svg_dom::SvgRoot;
    ///
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    /// let shadow = defs.filter("shadow")?;
    /// shadow.drop_shadow(4.0, 4.0, 4.0, "black", 0.5)?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn drop_shadow(
        &self,
        std_deviation: f64,
        dx: f64,
        dy: f64,
        color: &str,
        opacity: f64,
    ) -> Result<SvgNode, Error> {
        let el = create_svg_element::<SvgElement>(&self.document, "feDropShadow", "SvgElement")?;
        {
            let mut attrs = self.attrs.borrow_mut();
            attrs.display_element(&el, "stdDeviation", std_deviation)?;
            attrs.display_element(&el, "dx", dx)?;
            attrs.display_element(&el, "dy", dy)?;
            attrs.display_element(&el, "flood-opacity", opacity)?;
        }
        el.set_attribute("flood-color", color).map_err(dom_err)?;
        self.element.append_child(&el).map_err(dom_err)?;
        Ok(SvgNode::new(el))
    }
}
