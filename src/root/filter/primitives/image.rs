use super::super::SvgFilter;
use crate::{Error, SvgNode, dom_err, root::create_svg_element};
use web_sys::SvgElement;

impl SvgFilter {
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Appends a `<feImage>` primitive to this filter, using the image or SVG content at `href` as this
    /// primitive's own generated output.
    ///
    /// Like [`turbulence`](Self::turbulence)/[`turbulence_xy`](Self::turbulence_xy), `<feImage>` does not read from `in`
    /// at all — the SVG spec defines it as a generator whose content comes from `href`, not from an upstream
    /// primitive's result. The standard use is bringing external image content into a filter graph so it can be
    /// combined with other primitives (colour-transformed, blended, composited, ...) the same way generated or vector
    /// content already can be — something a plain [`SvgRoot::image`](crate::SvgRoot::image) element, filtered on its
    /// own, cannot do without a second layered element.
    ///
    /// `preserveAspectRatio` is not wrapped by a named parameter, the same choice already made for
    /// [`SvgRoot::image`](crate::SvgRoot::image): use the returned [`SvgNode`]'s
    /// [`set_attr`](crate::SvgNode::set_attr) for anything other than the SVG default (`"xMidYMid meet"`).
    ///
    /// # Security
    ///
    /// ⚠️ The `href` value is written verbatim to the DOM via `setAttribute`!
    /// Do not pass a `javascript:` URL or any other attacker-controlled string without validation.
    ///
    /// Use the returned [`SvgNode`]'s [`set_attr`](crate::SvgNode::set_attr) to set `result` — the usual way to
    /// consume this primitive's output, since there is no `in` to chain from.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if the browser refuses to create or append the `<feImage>` element.
    ///
    /// # Example
    ///
    /// Import an image into the filter graph, then greyscale it with [`color_matrix`](Self::color_matrix) — a
    /// combination a bare `<image>` element cannot express on its own:
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::filter::ColorMatrixType};
    ///
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    /// let flt  = defs.filter("greyscale-image")?;
    /// flt.image("photo.jpg")?;
    /// flt.color_matrix(ColorMatrixType::Saturate(0.0))?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn image(&self, href: &str) -> Result<SvgNode, Error> {
        let el = create_svg_element::<SvgElement>(&self.document, "feImage", "SvgElement")?;
        el.set_attribute("href", href).map_err(dom_err)?;
        self.element.append_child(&el).map_err(dom_err)?;
        Ok(SvgNode::new(el))
    }
}
