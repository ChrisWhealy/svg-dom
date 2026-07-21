use super::super::SvgFilter;
use crate::{Error, SvgNode, dom_err, root::create_svg_element};
use web_sys::SvgElement;

impl SvgFilter {
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Appends a `<feOffset>` primitive to this filter, shifting its input by `(dx, dy)` in the coordinate system
    /// established by [`primitiveUnits`](Self::set_primitive_units) — user-space units under the default
    /// [`FilterUnits::UserSpaceOnUse`](super::super::FilterUnits::UserSpaceOnUse), or a fraction of the referencing
    /// element's bounding box under
    /// [`FilterUnits::ObjectBoundingBox`](super::super::FilterUnits::ObjectBoundingBox) — for example, `0.1`
    /// represents 10% of the relevant dimension. (The SVG `dx`/`dy` grammar is a plain `<number>`; there is no
    /// percentage token to write here, just a fraction expressed as a plain `f64`.)
    ///
    /// The most common use is shifting a blurred alpha silhouette to build a drop shadow — see [`merge`](Self::merge)
    /// for combining the result back with the original graphic.
    ///
    /// If this is the filter's first primitive, its implicit input is `SourceGraphic`. Use the returned
    /// [`SvgNode`]'s [`set_attr`](crate::SvgNode::set_attr) to set `in` or `result`, neither of which has a dedicated
    /// setter yet.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if the browser refuses to create or append the `<feOffset>` element.
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
    /// shadow.offset(4.0, 4.0)?.set_attr("in", "blur")?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn offset(&self, dx: f64, dy: f64) -> Result<SvgNode, Error> {
        let el = create_svg_element::<SvgElement>(&self.document, "feOffset", "SvgElement")?;
        {
            let mut attrs = self.attrs.borrow_mut();
            attrs.display_element(&el, "dx", dx)?;
            attrs.display_element(&el, "dy", dy)?;
        }
        self.element.append_child(&el).map_err(dom_err)?;
        Ok(SvgNode::new(el))
    }
}
