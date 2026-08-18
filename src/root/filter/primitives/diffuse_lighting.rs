use super::super::{LightSource, LightingNodes, SvgFilter};
use crate::{Error, SvgNode, dom_err, root::create_svg_element};
use web_sys::SvgElement;

impl SvgFilter {
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Appends a `<feDiffuseLighting>` primitive to this filter, treating `in`'s alpha channel as a bump map and
    /// lighting the resulting surface with `light_source`.
    /// This is a matte, non-shiny lighting model (Lambertian reflectance), the diffuse half of the classic
    /// bevel/emboss lighting recipe.
    ///
    /// `surface_scale` multiplies the alpha-derived bump-map height before lighting is computed.
    /// `0.0` removes all alpha-derived relief, leaving a perfectly flat surface.
    /// This is *not* necessarily one uniformly lit by `lighting_color` outright.
    /// A flat surface still has a single, constant normal.
    /// Its lit result still depends on `diffuse_constant` and on the light's own direction relative to that normal.
    /// This is uniform only for [`LightSource::Distant`], whose direction is the same everywhere by definition.
    /// For [`LightSource::Point`]/[`LightSource::Spot`], the result is still position-dependent across the flat
    /// plane.
    /// Their direction (and, for `Spot`, beam concentration) varies from point to point, even without any bump-map
    /// relief left to shade.
    /// Larger `surface_scale` values exaggerate the apparent relief instead, making edges in `in`'s alpha channel
    /// read as taller, more steeply lit ridges.
    ///
    /// `diffuse_constant` scales the lit result's overall brightness — `1.0` is the SVG default.
    /// Per the SVG spec this should be non-negative.
    /// This crate does not enforce that before reaching the DOM, since no defined fallback or error classification
    /// is given for a negative value.
    ///
    /// `lighting_color` sets the colour of the light itself (the SVG `lighting-color` property/presentation
    /// attribute).
    /// `"white"` is the SVG default, and every example below uses it.
    /// The value is written verbatim: an invalid CSS colour does not cause a crate error, but the browser will not
    /// use it as a valid `lighting-color` value.
    ///
    /// `light_source` selects and configures the filter's one required light-source child.
    /// See [`LightSource`] for the three available kinds ([`Distant`](LightSource::Distant), [`Point`](LightSource::Point),
    /// [`Spot`](LightSource::Spot)), and what each looks like in practice.
    ///
    /// ***⚠️ The result is fully opaque: `A = 1.0` everywhere***.
    /// Per the SVG spec, `feDiffuseLighting` always produces an opaque `RGBA` image, regardless of `in`'s own alpha.
    /// Merging or blending this result directly on top of `SourceGraphic` therefore hides the original entirely,
    /// rather than tinting it.
    /// The standard way to recombine it with the original graphic is `composite(in2, CompositeOperator::Arithmetic)`
    /// with `k1: 1.0` and `k2`/`k3`/`k4: 0.0`, a pure multiply of the two inputs' colours.
    /// Do not use `merge`, which would simply paint the opaque lit surface over everything.
    /// See the example below.
    ///
    /// If this is the filter's first primitive, its implicit input is `SourceGraphic`.
    /// Use the returned [`SvgNode`]'s [`set_attr`](crate::SvgNode::set_attr) to set `in` or `result`, since neither
    /// has a dedicated setter.
    /// Do the same for `kernelUnitLength` — see the warning below before using it.
    ///
    /// ***⚠️ `kernelUnitLength` is a deprecated legacy attribute***.
    /// It requests an explicit, device-independent kernel sampling interval.
    /// The current Filter Effects specification marks it deprecated for `feDiffuseLighting` and slated for eventual
    /// removal.
    /// It does not reliably achieve the platform-independent rendering it was meant to provide.
    /// It remains reachable through `set_attr` (a deprecated attribute is not a removed one), but should not be
    /// relied upon.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if the browser refuses to create or append the `<feDiffuseLighting>` element or its
    /// light-source child.
    ///
    /// # Example
    ///
    /// A beveled, sunlit-looking surface: `SourceAlpha`'s edges are lit from upper-left, then multiplied back over
    /// the original colours so the bevel tints the source rather than replacing it:
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::filter::{CompositeOperator, LightSource}};
    ///
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    /// let flt  = defs.filter("bevel")?;
    /// flt.diffuse_lighting(6.0, 1.0, "white", LightSource::Distant { azimuth: 235.0, elevation: 55.0 })?
    ///     .set_attrs([("in", "SourceAlpha"), ("result", "lit")])?;
    /// flt.composite("lit", CompositeOperator::Arithmetic)?.set_attrs([
    ///     ("in", "SourceGraphic"),
    ///     ("k1", "1"), ("k2", "0"), ("k3", "0"), ("k4", "0"),
    /// ])?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn diffuse_lighting(
        &self,
        surface_scale: f64,
        diffuse_constant: f64,
        lighting_color: &str,
        light_source: LightSource,
    ) -> Result<SvgNode, Error> {
        Ok(self
            .diffuse_lighting_impl(surface_scale, diffuse_constant, lighting_color, light_source)?
            .primitive)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Identical to [`diffuse_lighting`](Self::diffuse_lighting), except it also returns a retainable handle to the
    /// appended light-source child as a [`LightingNodes`], not just the `<feDiffuseLighting>` primitive itself.
    ///
    /// Use this instead of [`diffuse_lighting`](Self::diffuse_lighting) when an interactive application needs to change
    /// the light itself after construction, say sweeping a [`LightSource::Distant`]'s own `azimuth` from a slider, for
    /// example.
    /// [`diffuse_lighting`](Self::diffuse_lighting) alone gives no way to reach that child element again.
    /// The light source is appended internally, and nothing else in this crate returns a handle to it.
    /// Without this method, the only way to reach it is a raw CSS-selector query outside `svg-dom`'s own typed API.
    ///
    /// Every parameter is identical to [`diffuse_lighting`](Self::diffuse_lighting)'s own — see its doc comment for
    /// what each one does. This differs only in its return type.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] under the same conditions as [`diffuse_lighting`](Self::diffuse_lighting).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::filter::LightSource};
    ///
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    /// let flt  = defs.filter("bevel")?;
    /// let lit = flt.diffuse_lighting_with_light(
    ///     6.0, 1.0, "white",
    ///     LightSource::Distant { azimuth: 235.0, elevation: 55.0 },
    /// )?;
    /// lit.primitive.set_attr("in", "SourceAlpha")?;
    ///
    /// // Later, e.g. from a slider's own input handler:
    /// lit.light.set_attr("azimuth", "90")?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn diffuse_lighting_with_light(
        &self,
        surface_scale: f64,
        diffuse_constant: f64,
        lighting_color: &str,
        light_source: LightSource,
    ) -> Result<LightingNodes, Error> {
        self.diffuse_lighting_impl(surface_scale, diffuse_constant, lighting_color, light_source)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Shared construction path for [`diffuse_lighting`](Self::diffuse_lighting)/
    /// [`diffuse_lighting_with_light`](Self::diffuse_lighting_with_light), which differ only in whether the
    /// light-source child's own node is discarded or returned alongside the primitive.
    fn diffuse_lighting_impl(
        &self,
        surface_scale: f64,
        diffuse_constant: f64,
        lighting_color: &str,
        light_source: LightSource,
    ) -> Result<LightingNodes, Error> {
        let el = create_svg_element::<SvgElement>(&self.document, "feDiffuseLighting", "SvgElement")?;
        el.set_attribute("lighting-color", lighting_color).map_err(dom_err)?;
        {
            let mut attrs = self.attrs.borrow_mut();
            attrs.display_element(&el, "surfaceScale", surface_scale)?;
            attrs.display_element(&el, "diffuseConstant", diffuse_constant)?;
        }
        let light = self.append_light_source(&el, light_source)?;
        self.element.append_child(&el).map_err(dom_err)?;
        Ok(LightingNodes {
            primitive: SvgNode::new(el),
            light,
        })
    }
}
