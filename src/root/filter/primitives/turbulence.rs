use super::super::{SvgFilter, TurbulenceType};
use crate::{Error, SvgNode, dom_err, root::create_svg_element};
use std::fmt;
use web_sys::SvgElement;

impl SvgFilter {
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Shared implementation behind [`turbulence`](Self::turbulence) and [`turbulence_xy`](Self::turbulence_xy):
    /// creates a `<feTurbulence>`, writes `base_frequency` as its `baseFrequency` attribute alongside `type`,
    /// `numOctaves`, and `seed`, then appends it.
    ///
    /// `base_frequency` is a pre-built [`fmt::Arguments`] rather than a `&str` so the two public callers can pass
    /// either a single number or an `"x y"` pair through
    /// [`display_element`](crate::root::attrs::SvgAttrs::display_element)'s retained scratch buffer without first
    /// collecting into an owned `String` — the same technique the private `gaussian_blur_args` helper uses for
    /// [`gaussian_blur`](Self::gaussian_blur)/[`gaussian_blur_xy`](Self::gaussian_blur_xy)'s `stdDeviation`.
    fn turbulence_args(
        &self,
        base_frequency: fmt::Arguments<'_>,
        num_octaves: u32,
        seed: f64,
        turbulence_type: TurbulenceType,
    ) -> Result<SvgNode, Error> {
        let el = create_svg_element::<SvgElement>(&self.document, "feTurbulence", "SvgElement")?;
        el.set_attribute("type", turbulence_type.as_str()).map_err(dom_err)?;
        {
            let mut attrs = self.attrs.borrow_mut();
            attrs.display_element(&el, "baseFrequency", base_frequency)?;
            attrs.display_element(&el, "numOctaves", num_octaves)?;
            attrs.display_element(&el, "seed", seed)?;
        }
        self.element.append_child(&el).map_err(dom_err)?;
        Ok(SvgNode::new(el))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Appends a `<feTurbulence>` primitive to this filter, generating Perlin-noise output at `base_frequency`.
    ///
    /// Unlike every other primitive here, `<feTurbulence>` reads no `in` at all — the SVG spec defines it as a
    /// generator that fabricates its own image from nothing, so there is nothing upstream for it to reference.
    ///
    /// `base_frequency` controls the noise's spatial frequency (how tightly packed the pattern is): larger values
    /// produce finer, busier detail; smaller values produce broader, smoother blobs. Negative frequencies are
    /// unsupported by SVG and should not be used.
    ///
    /// `num_octaves` layers that many progressively finer copies of the noise together (each doubling the frequency and
    /// halving the amplitude of the last) — the standard "octave" technique from Perlin/fractal-noise literature
    /// generally: more octaves add finer detail, at a rendering cost that grows with `num_octaves`, up to a point
    /// (see below). `1` is a single smooth noise layer; `4`–`6` is a typical range for visibly detailed texture without
    /// excessive cost. Negative values are unsupported by SVG, hence the use of `u32` rather than a signed integer to
    /// prevent such values from being supplied through this API at all.
    ///
    /// User agents may clamp `num_octaves` once further octave contributions fall below the available colour-channel
    /// resolution — the SVG spec itself gives nine octaves at eight bits per channel as an example of the point past
    /// which this becomes likely. Very large values may therefore have no additional visual effect (and not incur the
    /// rendering cost the "grows with `num_octaves`" description above would otherwise suggest), rather than continuing
    /// to add ever-finer, imperceptible detail.
    ///
    /// `seed` selects which pseudo-random noise pattern is generated; the same `seed` always reproduces the same
    /// pattern, so vary it to get a visually different result from otherwise identical parameters. SVG truncates
    /// the seed towards zero before generating the noise pattern; fractional values with the same integer part
    /// therefore produce the same effective seed (`2.1`, `2.5`, and `2.9` are all equivalent to `2.0`).
    ///
    /// `turbulence_type` selects between [`TurbulenceType::Turbulence`] (higher-contrast, marbled noise) and
    /// [`TurbulenceType::FractalNoise`] (softer, cloud-like noise).
    ///
    /// See [`turbulence_xy`](Self::turbulence_xy) for independent horizontal/vertical frequencies, and
    /// [`displacement_map`](Self::displacement_map) for the primitive this one is most often paired with.
    ///
    /// Use the returned [`SvgNode`]'s [`set_attr`](crate::SvgNode::set_attr) to set `result` — the usual way to
    /// consume noise, since there is no `in` to chain from — or `stitchTiles` (not wrapped by a named parameter;
    /// `"stitch"` avoids a visible seam when the filter region is meant to tile seamlessly, at the cost of subtly
    /// adjusting `baseFrequency` to fit exactly).
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if the browser refuses to create or append the `<feTurbulence>` element.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::filter::TurbulenceType};
    ///
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    /// let flt  = defs.filter("clouds")?;
    /// flt.turbulence(0.015, 4, 2.0, TurbulenceType::FractalNoise)?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn turbulence(
        &self,
        base_frequency: f64,
        num_octaves: u32,
        seed: f64,
        turbulence_type: TurbulenceType,
    ) -> Result<SvgNode, Error> {
        self.turbulence_args(format_args!("{base_frequency}"), num_octaves, seed, turbulence_type)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Appends a `<feTurbulence>` primitive to this filter with independent horizontal and vertical base
    /// frequencies, writing the SVG `baseFrequency="base_frequency_x base_frequency_y"` two-number form internally.
    ///
    /// See [`turbulence`](Self::turbulence) for `num_octaves`/`seed`/`turbulence_type` and everything else, which
    /// apply identically here.
    ///
    /// Unequal frequencies stretch the noise pattern along whichever axis has the lower frequency — useful for
    /// wood-grain or brushed-metal textures that should read as directional rather than isotropic, unlike
    /// [`turbulence`](Self::turbulence)'s uniform noise.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if the browser refuses to create or append the `<feTurbulence>` element.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::filter::TurbulenceType};
    ///
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    /// let flt  = defs.filter("wood-grain")?;
    /// flt.turbulence_xy(0.05, 0.005, 3, 7.0, TurbulenceType::Turbulence)?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn turbulence_xy(
        &self,
        base_frequency_x: f64,
        base_frequency_y: f64,
        num_octaves: u32,
        seed: f64,
        turbulence_type: TurbulenceType,
    ) -> Result<SvgNode, Error> {
        self.turbulence_args(
            format_args!("{base_frequency_x} {base_frequency_y}"),
            num_octaves,
            seed,
            turbulence_type,
        )
    }
}
