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
    /// primitive's result.
    ///
    /// Filtering a plain [`SvgRoot::image`](crate::SvgRoot::image) element allows that image to be colour-transformed,
    /// blended, or composited on its own: the image becomes the filter's `SourceGraphic`, and any primitive that reads
    /// `SourceGraphic` (the implicit input of a filter's first primitive) operates on it directly.
    ///
    /// What `<feImage>` adds is a *second*, independent source, provifing content unrelated to the element the filter
    /// is applied to. So a texture, logo, or displacement map can be combined with the filtered element's own
    /// `SourceGraphic` or `SourceAlpha` within the same filter graph, without a second layered display element.
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
    /// Import an image into the filter graph, then greyscale it with [`color_matrix`](Self::color_matrix) — chaining
    /// `<feImage>`'s own output into another primitive, the same way any other primitive's output can be chained:
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
