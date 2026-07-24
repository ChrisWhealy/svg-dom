use super::super::{LightSource, SvgFilter};
use crate::{Error, SvgNode, dom_err, root::create_svg_element};
use web_sys::SvgElement;

impl SvgFilter {
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Appends a `<feSpecularLighting>` primitive to this filter, treating `in`'s alpha channel as a bump map and
    /// lighting the resulting surface with `light_source` using the Blinn–Phong specular reflection model (the SVG
    /// spec computes the surface normal's alignment against a halfway vector `H = normalize(L + E)` between the
    /// light and eye directions, the defining trait of Blinn–Phong rather than the plain Phong model, which instead
    /// compares the eye direction against a reflection vector) — a shiny highlight, the counterpart to
    /// [`diffuse_lighting`](Self::diffuse_lighting)'s matte lighting, and the other half of the classic bevel/emboss
    /// lighting recipe.
    ///
    /// `surface_scale` is the same bump-map height multiplier [`diffuse_lighting`](Self::diffuse_lighting)'s own
    /// parameter of the same name is — see its doc comment for what larger/smaller values do.
    ///
    /// `specular_constant` scales the highlight's overall brightness — `1.0` is the SVG default.
    ///
    /// `specular_exponent` is the Phong shininess exponent: larger values narrow and sharpen the highlight
    /// (a harder, more mirror-like surface), smaller values spread it into a softer, broader sheen. SVG 1.1 gave
    /// this a `1.0`–`128.0` clamped range, though the current Filter Effects specification has an open question
    /// over whether that range still applies; this crate does not clamp or reject a value outside it, since the
    /// specification's own position is unsettled.
    ///
    /// ***⚠️ Do not confuse this with [`LightSource::Spot`]'s own `specular_exponent` field*** — the two share an
    /// SVG attribute name and a `1.0` default, but shape entirely different things: this parameter controls how
    /// sharp the *surface's* highlight looks, [`LightSource::Spot`]'s controls how tightly the *spotlight's own
    /// beam* concentrates. Setting one does not affect the other, and an SVG example using `specularExponent` on
    /// both `<feSpecularLighting>` and `<feSpotLight>` is configuring two unrelated things that merely look alike.
    ///
    /// `lighting_color` sets the colour of the light itself (the SVG `lighting-color` property/presentation
    /// attribute) — `"white"` is the SVG default, and every example below uses it. The value is written verbatim: an
    /// invalid CSS colour does not cause a crate error, but the browser will not use it as a valid `lighting-color`
    /// value.
    ///
    /// `light_source` selects and configures the filter's one required light-source child — see [`LightSource`]
    /// for the three available kinds and what each looks like in practice.
    ///
    /// ***⚠️ Unlike [`diffuse_lighting`](Self::diffuse_lighting), the result is *not* opaque*** — per the SVG spec,
    /// `feSpecularLighting`'s alpha is the maximum of its own lit R/G/B channels, so it is fully transparent
    /// wherever the highlight itself is zero, and only as opaque as the highlight is bright elsewhere. This makes
    /// it safe to add straight back on top of the original graphic — the standard recombination is
    /// `composite(in2, CompositeOperator::Arithmetic)` with `k2: 1.0`, `k3: 1.0`, `k1`/`k4: 0.0` (a plain sum of the
    /// two inputs), rather than the multiply `diffuse_lighting` needs. See the example below.
    ///
    /// If this is the filter's first primitive, its implicit input is `SourceGraphic`. Use the returned
    /// [`SvgNode`]'s [`set_attr`](crate::SvgNode::set_attr) to set `in` or `result` (neither has a dedicated
    /// setter), and likewise for `kernelUnitLength` — see the warning below before using it.
    ///
    /// ***⚠️ `kernelUnitLength` is a deprecated legacy attribute — but, unusually, not (yet) marked as such for this
    /// specific element*** — the current Filter Effects specification marks `kernelUnitLength` deprecated for
    /// `<feConvolveMatrix>` and `<feDiffuseLighting>`, but an open specification question
    /// ([w3c/fxtf-drafts#615](https://github.com/w3c/fxtf-drafts/issues/615)) asks whether omitting
    /// `<feSpecularLighting>` from that deprecation was intentional or an oversight. Given the same
    /// platform-independence problem applies here too, this crate does not recommend relying on it regardless of
    /// its formal deprecation status on this particular element. It remains reachable through `set_attr`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if the browser refuses to create or append the `<feSpecularLighting>` element or its
    /// light-source child.
    ///
    /// # Example
    ///
    /// A specular highlight from the same upper-left light as [`diffuse_lighting`](Self::diffuse_lighting)'s own
    /// example, added on top of the original graphic:
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::filter::{CompositeOperator, LightSource}};
    ///
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    /// let flt  = defs.filter("shiny-bevel")?;
    /// flt.specular_lighting(6.0, 1.0, 20.0, "white", LightSource::Distant { azimuth: 235.0, elevation: 55.0 })?
    ///     .set_attrs([("in", "SourceAlpha"), ("result", "highlight")])?;
    /// flt.composite("highlight", CompositeOperator::Arithmetic)?.set_attrs([
    ///     ("in", "SourceGraphic"),
    ///     ("k1", "0"), ("k2", "1"), ("k3", "1"), ("k4", "0"),
    /// ])?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn specular_lighting(
        &self,
        surface_scale: f64,
        specular_constant: f64,
        specular_exponent: f64,
        lighting_color: &str,
        light_source: LightSource,
    ) -> Result<SvgNode, Error> {
        let el = create_svg_element::<SvgElement>(&self.document, "feSpecularLighting", "SvgElement")?;
        el.set_attribute("lighting-color", lighting_color).map_err(dom_err)?;
        {
            let mut attrs = self.attrs.borrow_mut();
            attrs.display_element(&el, "surfaceScale", surface_scale)?;
            attrs.display_element(&el, "specularConstant", specular_constant)?;
            attrs.display_element(&el, "specularExponent", specular_exponent)?;
        }
        self.append_light_source(&el, light_source)?;
        self.element.append_child(&el).map_err(dom_err)?;
        Ok(SvgNode::new(el))
    }
}
