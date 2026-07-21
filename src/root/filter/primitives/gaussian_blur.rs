use super::super::SvgFilter;
use crate::{Error, SvgNode, dom_err, root::create_svg_element};
use std::fmt;
use web_sys::SvgElement;

impl SvgFilter {
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Shared implementation behind [`gaussian_blur`](Self::gaussian_blur) and
    /// [`gaussian_blur_xy`](Self::gaussian_blur_xy): creates a `<feGaussianBlur>`, writes `std_deviation` as its
    /// `stdDeviation` attribute, and appends it.
    ///
    /// `std_deviation` is a pre-built [`fmt::Arguments`] rather than a `&str` so the two public callers can pass either
    /// a single number or an `"x y"` pair through [`display_element`](crate::root::attrs::SvgAttrs::display_element)'s
    /// retained scratch buffer without first collecting into an owned `String`.  This is the same technique used by
    /// [`SvgPattern::set_view_box`](crate::SvgPattern::set_view_box) and
    /// [`SvgSymbol::set_view_box`](crate::SvgSymbol::set_view_box) to combine several numbers into one attribute.
    fn gaussian_blur_args(&self, std_deviation: fmt::Arguments<'_>) -> Result<SvgNode, Error> {
        let el = create_svg_element::<SvgElement>(&self.document, "feGaussianBlur", "SvgElement")?;
        self.attrs.borrow_mut().display_element(&el, "stdDeviation", std_deviation)?;
        self.element.append_child(&el).map_err(dom_err)?;
        Ok(SvgNode::new(el))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Appends a `<feGaussianBlur>` primitive to this filter, blurring its input equally on both axes by
    /// `std_deviation`.
    ///
    /// `std_deviation` is the standard deviation of the Gaussian blur kernel where larger values blur more and is
    /// interpreted in the coordinate system established by [`primitiveUnits`](Self::set_primitive_units) — user-space
    /// units under the default [`FilterUnits::UserSpaceOnUse`](super::super::FilterUnits::UserSpaceOnUse), or a
    /// fraction of the referencing element's bounding box under
    /// [`FilterUnits::ObjectBoundingBox`](super::super::FilterUnits::ObjectBoundingBox) — for example, `0.1`
    /// represents 10% of the relevant dimension. (The SVG `stdDeviation` grammar is one or two plain `<number>`
    /// values; there is no percentage token to write here, just a fraction expressed as a plain `f64`.)
    ///
    /// A `std_deviation` of `0.0` produces no blur (the input passes through unchanged).
    ///
    /// See [`gaussian_blur_xy`](Self::gaussian_blur_xy) for a blur with independent horizontal and vertical
    /// deviations — the SVG `stdDeviation` attribute accepts either one or two numbers, and this method covers only
    /// the one-number form.
    ///
    /// If this is the filter's first primitive, its implicit input is `SourceGraphic` (the referencing element as
    /// normally rendered). Use the returned [`SvgNode`]'s [`set_attr`](crate::SvgNode::set_attr) to set `in` (e.g.
    /// `"SourceAlpha"`, or the `result` name of an earlier primitive) or `result` (to name this primitive's output for
    /// a later primitive's `in`/`in2` to reference) — neither has a dedicated setter yet.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if the browser refuses to create or append the `<feGaussianBlur>` element.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::SvgRoot;
    ///
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    /// let blur = defs.filter("soft")?;
    /// blur.gaussian_blur(4.0)?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn gaussian_blur(&self, std_deviation: f64) -> Result<SvgNode, Error> {
        self.gaussian_blur_args(format_args!("{std_deviation}"))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Appends a `<feGaussianBlur>` primitive to this filter with independent horizontal and vertical standard
    /// deviations, writing the SVG `stdDeviation="std_deviation_x std_deviation_y"` two-number form internally.
    ///
    /// Both values are interpreted in the same [`primitiveUnits`](Self::set_primitive_units)-dependent coordinate
    /// system as [`gaussian_blur`](Self::gaussian_blur)'s `std_deviation`.
    ///
    /// Pass `0.0` for one axis to blur only along the other — for example `gaussian_blur_xy(0.0, 6.0)` blurs
    /// vertically only, useful for a horizontal motion-blur effect.
    ///
    /// For an equal blur on both axes, prefer [`gaussian_blur`](Self::gaussian_blur): passing the same value twice
    /// here writes the same two-number attribute the one-number form already implies, at no benefit.
    ///
    /// See [`gaussian_blur`](Self::gaussian_blur) for the `in`/`result` attributes, which apply identically here.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if the browser refuses to create or append the `<feGaussianBlur>` element.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::SvgRoot;
    ///
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    /// let blur = defs.filter("streak")?;
    /// blur.gaussian_blur_xy(12.0, 0.0)?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn gaussian_blur_xy(&self, std_deviation_x: f64, std_deviation_y: f64) -> Result<SvgNode, Error> {
        self.gaussian_blur_args(format_args!("{std_deviation_x} {std_deviation_y}"))
    }
}
