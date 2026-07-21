use super::super::{MorphologyOperator, SvgFilter};
use crate::{Error, SvgNode, dom_err, root::create_svg_element};
use std::fmt;
use web_sys::SvgElement;

impl SvgFilter {
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Shared implementation behind [`morphology`](Self::morphology) and [`morphology_xy`](Self::morphology_xy):
    /// creates a `<feMorphology>`, writes `operator` and `radius` as its `operator`/`radius` attributes, then
    /// appends it.
    ///
    /// `radius` is a pre-built [`fmt::Arguments`] rather than a `&str` so the two public callers can pass either a
    /// single number or an `"x y"` pair through
    /// [`display_element`](crate::root::attrs::SvgAttrs::display_element)'s retained scratch buffer without first
    /// collecting into an owned `String` — the same technique the private `gaussian_blur_args`/`turbulence_args`
    /// helpers use for [`gaussian_blur`](Self::gaussian_blur)/[`turbulence`](Self::turbulence) and their `_xy`
    /// counterparts.
    fn morphology_args(&self, radius: fmt::Arguments<'_>, operator: MorphologyOperator) -> Result<SvgNode, Error> {
        let el = create_svg_element::<SvgElement>(&self.document, "feMorphology", "SvgElement")?;
        el.set_attribute("operator", operator.as_str()).map_err(dom_err)?;
        self.attrs.borrow_mut().display_element(&el, "radius", radius)?;
        self.element.append_child(&el).map_err(dom_err)?;
        Ok(SvgNode::new(el))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Appends a `<feMorphology>` primitive to this filter, applying a rectangular morphological erosion or
    /// dilation (selected by `operator`) component-wise to the input's premultiplied R/G/B/A values within
    /// `radius` of each pixel — a per-pixel minimum for [`MorphologyOperator::Erode`], maximum for
    /// [`MorphologyOperator::Dilate`].
    ///
    /// The common case — passing `SourceAlpha` as `in` — shrinks or expands the source *silhouette*, since alpha
    /// is the only non-degenerate channel there; that is what [`MorphologyOperator::Erode`]/
    /// [`MorphologyOperator::Dilate`]'s own doc comments describe, and what every example below uses. Against
    /// `SourceGraphic` (this primitive's implicit input if it is the filter's first, since `in` is not set by this
    /// method), the same min/max is taken across colour channels too, which can shift or bleed colours at edges
    /// where they differ between neighbouring pixels — worth knowing before assuming this primitive only ever
    /// touches shape, never colour.
    ///
    /// Applying `Erode` then `Dilate` with the same `radius` forms an "opening", which removes small protrusions
    /// and narrow features (a thin bridge or spike can vanish in the `Erode` pass and, unlike a solid region,
    /// cannot be reconstructed by the `Dilate` pass that follows). The reverse order forms a "closing", which
    /// fills small gaps and notches instead. Neither reconstructs the geometry the first pass removed or added,
    /// so either operation may alter the resulting silhouette rather than merely smoothing it.
    ///
    /// `radius` is interpreted in the coordinate system established by
    /// [`primitiveUnits`](Self::set_primitive_units) — user-space units under the default
    /// [`FilterUnits::UserSpaceOnUse`](super::super::FilterUnits::UserSpaceOnUse), or a fraction/percentage of the
    /// referencing element's box under [`FilterUnits::ObjectBoundingBox`](super::super::FilterUnits::ObjectBoundingBox).
    /// A `radius` of `0.0` (the SVG default if this is never called with a non-zero value) disables the effect
    /// entirely — `in` passes through unchanged. A negative value is not rejected, but has the identical effect:
    /// per the SVG spec, "a negative or zero value disables the effect ... the result is the filter input image".
    ///
    /// See [`morphology_xy`](Self::morphology_xy) for a radius with independent horizontal and vertical extent — the
    /// SVG `radius` attribute accepts either one or two numbers, and this method covers only the one-number form.
    ///
    /// If this is the filter's first primitive, its implicit input is `SourceGraphic`. Use the returned [`SvgNode`]'s
    /// [`set_attr`](crate::SvgNode::set_attr) to set `in` or `result`, neither of which has a dedicated setter yet.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if the browser refuses to create or append the `<feMorphology>` element.
    ///
    /// # Example
    ///
    /// A bolder outline: dilate the source alpha, then merge it underneath the original graphic so only the
    /// grown-outward fringe shows through:
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::filter::MorphologyOperator};
    ///
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    /// let flt  = defs.filter("bold-outline")?;
    /// flt.morphology(2.5, MorphologyOperator::Dilate)?
    ///     .set_attrs([("in", "SourceAlpha"), ("result", "thickened")])?;
    /// flt.merge(&["thickened", "SourceGraphic"])?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    ///
    /// ***⚠️ [`Dilate`](MorphologyOperator::Dilate) expands the rendered area, and may be clipped by the filter
    /// region*** — the filter region is a hard clipping rectangle, and the SVG default (`-10% -10% 120% 120%` of
    /// the referencing element's bounding box) only allows for a modest 10% margin on each side. A large enough
    /// `radius` can therefore produce a visibly clipped edge on the dilated result. Increase the filter's
    /// [`x`](Self::set_x)/[`y`](Self::set_y)/[`width`](Self::set_width)/[`height`](Self::set_height) where
    /// necessary — see [`set_width`](Self::set_width)'s own doc comment for how and why.
    pub fn morphology(&self, radius: f64, operator: MorphologyOperator) -> Result<SvgNode, Error> {
        self.morphology_args(format_args!("{radius}"), operator)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Appends a `<feMorphology>` primitive to this filter with independent horizontal and vertical radii, writing the
    /// SVG `radius="radius_x radius_y"` two-number form internally.
    ///
    /// Both values are interpreted in the same [`primitiveUnits`](Self::set_primitive_units)-dependent coordinate
    /// system as [`morphology`](Self::morphology)'s `radius`.
    ///
    /// ***⚠️ Unlike [`gaussian_blur_xy`](Self::gaussian_blur_xy), `0.0` on one axis does not give a one-dimensional
    /// effect***.  Instead, as per the SVG spec, a zero (or negative) `radius` component entirely disables the
    /// primitive, not just that axis.  Consequently, `in` is passed through completely unchanged, regardless of what
    /// the other axis's value is.
    ///
    /// `morphology_xy(3.0, 0.0, ...)` is therefore a no-op, not a horizontal-only dilation.
    /// Both `radius_x` and `radius_y` must be positive for this primitive to have any effect at all.
    ///
    /// See [`morphology`](Self::morphology) for `operator`, the `in`/`result` attributes, the negative-radius
    /// caveat, and the filter-region clipping warning for [`Dilate`](MorphologyOperator::Dilate), all of which
    /// apply identically here.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if the browser refuses to create or append the `<feMorphology>` element.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::filter::MorphologyOperator};
    ///
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    /// let flt  = defs.filter("widen")?;
    /// flt.morphology_xy(4.0, 1.0, MorphologyOperator::Dilate)?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn morphology_xy(&self, radius_x: f64, radius_y: f64, operator: MorphologyOperator) -> Result<SvgNode, Error> {
        self.morphology_args(format_args!("{radius_x} {radius_y}"), operator)
    }
}
