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
    /// Appends a `<feMorphology>` primitive to this filter, growing or shrinking its input's opaque regions by
    /// `radius` according to `operator`.
    ///
    /// [`MorphologyOperator::Erode`] shrinks/thins opaque regions inward; [`MorphologyOperator::Dilate`] grows/
    /// thickens them outward. Applying `Erode` then `Dilate` with the same `radius` (an "opening") smooths small
    /// outward bumps off a silhouette without changing its overall size; the reverse order (a "closing") fills in small
    /// inward notches instead.
    ///
    /// `radius` is interpreted in the coordinate system established by
    /// [`primitiveUnits`](Self::set_primitive_units) — user-space units under the default
    /// [`FilterUnits::UserSpaceOnUse`](super::super::FilterUnits::UserSpaceOnUse), or a fraction/percentage of the
    /// referencing element's box under [`FilterUnits::ObjectBoundingBox`](super::super::FilterUnits::ObjectBoundingBox).
    /// A `radius` of `0.0` (the SVG default if this is never called with a non-zero value) produces no effect at all —
    /// `in` passes through unchanged. Must be non-negative; the SVG spec does not define a rendering result for a
    /// negative value, so passing one may produce an unspecified result rather than a runtime error.
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
    /// Pass `0.0` for one axis to grow/shrink only along the other: for example:
    /// `morphology_xy(3.0, 0.0, MorphologyOperator::Dilate)` thickens a shape horizontally only, useful for widening a
    /// vertical divider without also lengthening it.
    ///
    /// See [`morphology`](Self::morphology) for `operator`, the `in`/`result` attributes, and the negative-radius
    /// caveat, all of which apply identically here.
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
    /// flt.morphology_xy(4.0, 0.0, MorphologyOperator::Dilate)?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn morphology_xy(&self, radius_x: f64, radius_y: f64, operator: MorphologyOperator) -> Result<SvgNode, Error> {
        self.morphology_args(format_args!("{radius_x} {radius_y}"), operator)
    }
}
