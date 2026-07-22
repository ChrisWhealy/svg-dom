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
    /// Use the returned [`SvgNode`]'s [`set_attr`](crate::SvgNode::set_attr) to set `result` when this primitive's
    /// output must be referenced explicitly by a later primitive that is not immediately downstream, or when the filter
    /// graph branches. For a simple linear chain, such as the example below, the next primitive can consume
    /// `<feImage>`'s output implicitly by omitting its own `in`.
    ///
    /// # Loading is asynchronous
    ///
    /// `href` is fetched the same way a plain [`SvgRoot::image`](crate::SvgRoot::image) element's is: asynchronously,
    /// after this method returns. A successful [`Ok`] here means nothing more than the fact that the `<feImage>` DOM
    /// node was constructed — it says nothing about whether the resource identified by the `href` has finished loading,
    /// or ever will.
    ///
    /// A resource that is missing, unsupported, zero-sized, or fails to download will be rendered as transparent black
    /// across the primitive's subregion, as per the SVG specification.  The API will not report any error, resulting in
    /// a broken `href` that has failed silently in the rendered output.
    ///
    /// # Taint and `feDisplacementMap`
    ///
    /// Per the Filter Effects specification, an `<feImage>` result is *tainted* when `href` references an SVG
    /// element (a same-document `"#id"` reference — see the example below) or when an image resource is fetched in
    /// no-CORS mode. A tainted result used as `in2` for [`displacement_map`](Self::displacement_map) makes that
    /// displacement a pass-through: `in` is returned unmodified, with no error to signal the mistake.
    ///
    /// For a *fetched image resource* used as a displacement map, set `crossorigin` (not wrapped as a named
    /// parameter) — typically `"anonymous"` — and ensure the server hosting `href` sends matching CORS headers, so
    /// the fetch succeeds in CORS mode rather than no-CORS mode:
    ///
    /// ```rust,no_run
    /// use svg_dom::SvgRoot;
    ///
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    /// let flt  = defs.filter("cross-origin-displacement")?;
    ///
    /// flt.image("https://example.com/map.png")?
    ///     .set_attrs([("crossorigin", "anonymous"), ("result", "displacement-map")])?;
    ///
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    ///
    /// `crossorigin` has no effect on a same-document element reference — those are unconditionally tainted by
    /// definition, so they remain unusable as a displacement map regardless. They are perfectly usable everywhere
    /// else (as input to [`color_matrix`](Self::color_matrix), [`blend`](Self::blend),
    /// [`composite`](Self::composite), and so on) — taint only forecloses the `feDisplacementMap` case.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if the browser refuses to create the element, set its `href`, or append it to the
    /// filter. This is unrelated to whether `href` itself loads successfully — see "Loading is asynchronous" above.
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
