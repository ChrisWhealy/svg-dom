use super::{FilterUnits, SvgFilter};
use crate::{Error, dom_err};

impl SvgFilter {
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the horizontal offset of the filter region.
    ///
    /// Interpreted according to [`filterUnits`](Self::set_filter_units) — a fraction of the referencing element's
    /// bounding box under the SVG default ([`FilterUnits::ObjectBoundingBox`]), or a user-space coordinate under
    /// [`FilterUnits::UserSpaceOnUse`].
    pub fn set_x(&self, v: f64) -> Result<(), Error> {
        self.attrs.borrow_mut().display_element(&self.element, "x", v)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the vertical offset of the filter region.
    ///
    /// See [`set_x`](Self::set_x) for the coordinate space this value is interpreted in.
    pub fn set_y(&self, v: f64) -> Result<(), Error> {
        self.attrs.borrow_mut().display_element(&self.element, "y", v)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the width of the filter region.
    ///
    /// The SVG default filter region is `-10% -10% 120% 120%` of the referencing element's bounding box, which can clip
    /// a wide blur; widen `width`/`height` explicitly for large [`gaussian_blur`](Self::gaussian_blur) `std_deviation`
    /// values.
    ///
    /// ⚠️ Performance ⚠️
    ///
    /// Expand the region only enough to contain the intended effect. Per the SVG filter specification, the filter
    /// region is a hard clip: every intermediate offscreen buffer the browser rasterises while evaluating this filter's
    /// primitives is bounded by it, so an unnecessarily large region can inflate both rasterisation work and temporary
    /// memory use, not just the final painted area.
    ///
    /// See [`set_x`](Self::set_x) for the coordinate space this value is interpreted in.
    pub fn set_width(&self, v: f64) -> Result<(), Error> {
        self.attrs.borrow_mut().display_element(&self.element, "width", v)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the height of the filter region.
    ///
    /// See [`set_width`](Self::set_width) for why this often needs widening beyond the SVG default, why it should
    /// not be widened further than the effect needs, and for the coordinate space this value is interpreted in.
    pub fn set_height(&self, v: f64) -> Result<(), Error> {
        self.attrs.borrow_mut().display_element(&self.element, "height", v)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `filterUnits` attribute, controlling the coordinate space for the filter region's position and size.
    ///
    /// The default is [`FilterUnits::ObjectBoundingBox`], meaning [`set_x`](Self::set_x), [`set_y`](Self::set_y),
    /// [`set_width`](Self::set_width), and [`set_height`](Self::set_height) are fractions of the referencing element's
    /// bounding box.
    ///
    /// Use [`FilterUnits::UserSpaceOnUse`] to express the filter region in the referencing element's user coordinate
    /// system instead.
    pub fn set_filter_units(&self, u: FilterUnits) -> Result<(), Error> {
        self.element.set_attribute("filterUnits", u.as_str()).map_err(dom_err)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `primitiveUnits` attribute, controlling the coordinate space used by length-valued attributes on the
    /// filter's own primitives (for example [`gaussian_blur`](Self::gaussian_blur)'s `std_deviation` or
    /// [`offset`](Self::offset)'s `dx`/`dy`).
    ///
    /// The default is [`FilterUnits::UserSpaceOnUse`], meaning primitive attributes use the same coordinate system as
    /// the element that references the filter.
    ///
    /// Use [`FilterUnits::ObjectBoundingBox`] to express them as fractions of the referencing element's bounding box
    /// instead.
    pub fn set_primitive_units(&self, u: FilterUnits) -> Result<(), Error> {
        self.element.set_attribute("primitiveUnits", u.as_str()).map_err(dom_err)
    }
}
