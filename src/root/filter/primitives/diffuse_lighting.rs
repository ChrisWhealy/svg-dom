use super::super::{LightSource, SvgFilter};
use crate::{Error, SvgNode, dom_err, root::create_svg_element};
use web_sys::SvgElement;

impl SvgFilter {
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Appends a `<feDiffuseLighting>` primitive to this filter, treating `in`'s alpha channel as a bump map and
    /// lighting the resulting surface with `light_source` — a matte, non-shiny lighting model (Lambertian
    /// reflectance), the diffuse half of the classic bevel/emboss lighting recipe.
    ///
    /// `surface_scale` multiplies the alpha-derived bump-map height before lighting is computed: `0.0` removes all
    /// alpha-derived relief, leaving a perfectly flat surface — but *not* necessarily one uniformly lit by
    /// `lighting_color` outright. A flat surface still has a single, constant normal, so its lit result still
    /// depends on `diffuse_constant` and on the light's own direction relative to that normal: uniform only for
    /// [`LightSource::Distant`], whose direction is the same everywhere by definition, but still position-dependent
    /// across the flat plane for [`LightSource::Point`]/[`LightSource::Spot`], whose direction (and, for `Spot`,
    /// beam concentration) varies from point to point even without any bump-map relief left to shade. Larger
    /// `surface_scale` values exaggerate the apparent relief instead, making edges in `in`'s alpha channel read as
    /// taller, more steeply lit ridges.
    ///
    /// `diffuse_constant` scales the lit result's overall brightness — `1.0` is the SVG default. Per the SVG spec
    /// this should be non-negative; this crate does not enforce that before reaching the DOM, since no defined
    /// fallback or error classification is given for a negative value.
    ///
    /// `lighting_color` sets the colour of the light itself (the SVG `lighting-color` property/presentation
    /// attribute) — `"white"` is the SVG default, and every example below uses it. The value is written verbatim: an
    /// invalid CSS colour does not cause a crate error, but the browser will not use it as a valid `lighting-color`
    /// value.
    ///
    /// `light_source` selects and configures the filter's one required light-source child — see [`LightSource`]
    /// for the three available kinds ([`Distant`](LightSource::Distant), [`Point`](LightSource::Point),
    /// [`Spot`](LightSource::Spot)) and what each looks like in practice.
    ///
    /// ***⚠️ The result is fully opaque — `A = 1.0` everywhere*** — per the SVG spec, `feDiffuseLighting` always
    /// produces an opaque `RGBA` image, regardless of `in`'s own alpha. Merging or blending this result directly on
    /// top of `SourceGraphic` therefore hides the original entirely, rather than tinting it. The standard way to
    /// recombine it with the original graphic is `composite(in2, CompositeOperator::Arithmetic)` with `k1: 1.0` and
    /// `k2`/`k3`/`k4: 0.0` — a pure multiply of the two inputs' colours — not `merge`, which would simply paint the
    /// opaque lit surface over everything. See the example below.
    ///
    /// If this is the filter's first primitive, its implicit input is `SourceGraphic`. Use the returned
    /// [`SvgNode`]'s [`set_attr`](crate::SvgNode::set_attr) to set `in` or `result` (neither has a dedicated
    /// setter), and likewise for `kernelUnitLength` — see the warning below before using it.
    ///
    /// ***⚠️ `kernelUnitLength` is a deprecated legacy attribute*** — it requests an explicit, device-independent
    /// kernel sampling interval, but the current Filter Effects specification marks it deprecated for
    /// `feDiffuseLighting` and slated for eventual removal, since it does not reliably achieve the
    /// platform-independent rendering it was meant to provide. It remains reachable through `set_attr` (a
    /// deprecated attribute is not a removed one), but should not be relied upon.
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
        let el = create_svg_element::<SvgElement>(&self.document, "feDiffuseLighting", "SvgElement")?;
        el.set_attribute("lighting-color", lighting_color).map_err(dom_err)?;
        {
            let mut attrs = self.attrs.borrow_mut();
            attrs.display_element(&el, "surfaceScale", surface_scale)?;
            attrs.display_element(&el, "diffuseConstant", diffuse_constant)?;
        }
        self.append_light_source(&el, light_source)?;
        self.element.append_child(&el).map_err(dom_err)?;
        Ok(SvgNode::new(el))
    }
}
