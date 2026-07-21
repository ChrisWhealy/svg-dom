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
    /// vertical displacement respectively.
    ///
    /// ***⚠️ [`Channel::Alpha`] for both selectors constrains every displacement vector to one diagonal*** — it is
    /// the SVG default, and a valid choice when that constraint is exactly what is wanted, but passing the *same*
    /// channel for both selectors means `dx` and `dy` are always equal at every pixel (both computed from the same
    /// `0.0`–`1.0` value), so displacement only ever points along the `y = x` line rather than freely in two
    /// dimensions. For the general, more natural-looking "organic edge" case, select two *different* channels —
    /// [`Channel::Red`] for `x_channel_selector` and [`Channel::Green`] for `y_channel_selector`, as in the example
    /// below — since [`turbulence`](Self::turbulence)/[`turbulence_xy`](Self::turbulence_xy) generate each colour
    /// channel independently (the same choice the SVG specification's own explanatory `feDisplacementMap` example
    /// makes).
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
    /// ***⚠️ Cross-browser rendering is not guaranteed to match exactly*** — the Filter Effects specification
    /// itself notes interoperability differences between implementations of `feDisplacementMap`, including
    /// disagreement on exactly how the displaced sample is interpolated. This crate's own tests only assert on DOM
    /// structure and attribute values — none of them render this primitive and inspect pixels — so a filter that
    /// passes those tests is verified to be *well-formed*, not verified to *look* identical in every browser.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if the browser refuses to create or append the `<feDisplacementMap>` element.
    ///
    /// # Example
    ///
    /// Distort a shape's edge with fractal noise — the standard `feTurbulence` + `feDisplacementMap` pairing.
    /// `Channel::Red`/`Channel::Green` give the displacement two free, uncorrelated dimensions rather than
    /// constraining it to a single diagonal (see the warning above):
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::filter::{Channel, TurbulenceType}};
    ///
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    /// let flt  = defs.filter("organic-edge")?;
    /// flt.turbulence(0.02, 3, 5.0, TurbulenceType::FractalNoise)?.set_attr("result", "noise")?;
    /// flt.displacement_map("noise", 24.0, Channel::Red, Channel::Green)?
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
