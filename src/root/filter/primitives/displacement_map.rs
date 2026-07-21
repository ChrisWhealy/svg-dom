use super::super::{Channel, SvgFilter};
use crate::{Error, SvgNode, dom_err, root::create_svg_element};
use web_sys::SvgElement;

impl SvgFilter {
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Appends a `<feDisplacementMap>` primitive to this filter, warping this primitive's `in` input using pixel
    /// values sampled from `in2` as a per-pixel displacement field.
    ///
    /// For every output pixel, the `x_channel_selector`/`y_channel_selector` channel values of `in2` at that
    /// location (each `0.0`–`1.0`) are remapped to `-scale/2`..`scale/2` and used to offset which pixel of `in` is
    /// read — the standard way to distort a shape using noise from [`turbulence`](Self::turbulence)/
    /// [`turbulence_xy`](Self::turbulence_xy), producing a hand-drawn/organic edge instead of a perfectly
    /// geometric one.
    ///
    /// `scale` controls the maximum displacement, in the coordinate system established by
    /// [`primitiveUnits`](Self::set_primitive_units). `0.0` (the SVG default if this is never called with a
    /// non-zero value) produces no displacement at all — `in` passes through unchanged.
    ///
    /// `x_channel_selector`/`y_channel_selector` choose which of `in2`'s four channels drives the horizontal/
    /// vertical displacement respectively. [`Channel::Alpha`] for both is the SVG default and the usual choice
    /// when `in2` is noise from [`turbulence`](Self::turbulence)/[`turbulence_xy`](Self::turbulence_xy) with
    /// [`TurbulenceType::FractalNoise`](super::super::TurbulenceType::FractalNoise) — the RGB channels there are just as
    /// usable, but alpha needs no colour-space reasoning to interpret as a plain displacement magnitude.
    ///
    /// `in2` is written directly.
    ///
    /// ***IMPORTANT*** The value of `in2` is not validated. It is typically another primitive's `result` name —
    /// most often [`turbulence`](Self::turbulence)/[`turbulence_xy`](Self::turbulence_xy)'s output — or one of the
    /// SVG keyword inputs (`"SourceGraphic"`/`"SourceAlpha"`).
    ///
    /// `in` is not set by this method: if this is the filter's first primitive, its implicit input is
    /// `SourceGraphic`, otherwise use the returned [`SvgNode`]'s [`set_attr`](crate::SvgNode::set_attr) to set `in`
    /// explicitly, the same as every other primitive here.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if the browser refuses to create or append the `<feDisplacementMap>` element.
    ///
    /// # Example
    ///
    /// Distort a shape's edge with fractal noise — the standard `feTurbulence` + `feDisplacementMap` pairing:
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::filter::{Channel, TurbulenceType}};
    ///
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    /// let flt  = defs.filter("organic-edge")?;
    /// flt.turbulence(0.02, 3, 5.0, TurbulenceType::FractalNoise)?.set_attr("result", "noise")?;
    /// flt.displacement_map("noise", 24.0, Channel::Alpha, Channel::Alpha)?
    ///     .set_attr("in", "SourceGraphic")?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn displacement_map(
        &self,
        in2: &str,
        scale: f64,
        x_channel_selector: Channel,
        y_channel_selector: Channel,
    ) -> Result<SvgNode, Error> {
        let el = create_svg_element::<SvgElement>(&self.document, "feDisplacementMap", "SvgElement")?;
        el.set_attribute("in2", in2).map_err(dom_err)?;
        self.attrs.borrow_mut().display_element(&el, "scale", scale)?;
        el.set_attribute("xChannelSelector", x_channel_selector.selector_str())
            .map_err(dom_err)?;
        el.set_attribute("yChannelSelector", y_channel_selector.selector_str())
            .map_err(dom_err)?;
        self.element.append_child(&el).map_err(dom_err)?;
        Ok(SvgNode::new(el))
    }
}
