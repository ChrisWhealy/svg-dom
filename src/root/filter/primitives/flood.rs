use super::super::SvgFilter;
use crate::{Error, SvgNode, dom_err, root::create_svg_element};
use web_sys::SvgElement;

impl SvgFilter {
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Appends a `<feFlood>` primitive to this filter, producing a solid-colour rectangle covering the filter region.
    ///
    /// `color` is any valid SVG/CSS colour value (`"red"`, `"#ff0000"`, `"rgb(255,0,0)"`, ...), written as `flood-color`.
    /// `opacity` (written as `flood-opacity`) is a value in the range `0.0` to `1.0`.
    ///
    /// **IMPORTANT** values outside that range will not cause a runtime error, but may well produce an unspecified
    /// rendering result.  This is the same convention as used by
    /// [`SvgLinearGradient::add_stop_opacity`](crate::SvgLinearGradient::add_stop_opacity)
    ///
    /// On its own, a flood fills the entire filter region with one flat colour, which by itself is rarely useful, but
    /// when combined with [`composite`](Self::composite) (`operator: `
    /// [`In`](super::super::CompositeOperator::In)) against a blurred alpha mask, it is the standard way to give a
    /// drop shadow an actual colour and opacity rather than leaving it simply as a blurred copy of the source
    /// graphic's own fill; see [`composite`](Self::composite)'s example.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if the browser refuses to create or append the `<feFlood>` element.
    pub fn flood(&self, color: &str, opacity: f64) -> Result<SvgNode, Error> {
        let el = create_svg_element::<SvgElement>(&self.document, "feFlood", "SvgElement")?;
        {
            let mut attrs = self.attrs.borrow_mut();
            attrs.display_element(&el, "flood-opacity", opacity)?;
            el.set_attribute("flood-color", color).map_err(dom_err)?;
        }
        self.element.append_child(&el).map_err(dom_err)?;
        Ok(SvgNode::new(el))
    }
}
