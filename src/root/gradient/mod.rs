pub(crate) mod linear;
pub(crate) mod radial;

use crate::{
    Error, dom_err,
    root::{
        attrs::SvgAttrs,
        defs::{URL_PREFIX, validate_gradient_id},
    },
};
use std::cell::RefCell;
use web_sys::{Document, SvgElement};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Controls the coordinate space used by a gradient's geometry attributes.
///
/// The default in SVG is [`ObjectBoundingBox`](GradientUnits::ObjectBoundingBox), which lets you express positions as
/// fractions of the painted element's bounding box and reuse one gradient definition across elements of different sizes.
///
/// Switch to [`UserSpaceOnUse`](GradientUnits::UserSpaceOnUse) when you need the gradient to be anchored to a fixed
/// coordinate in the SVG canvas rather than scaled to each element individually.
///
/// Passed to [`crate::SvgLinearGradient::set_gradient_units`] and [`crate::SvgRadialGradient::set_gradient_units`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GradientUnits {
    /// Geometry attributes are fractions of the element's bounding box in [0.0, 1.0].
    /// This is the SVG default.
    ObjectBoundingBox,
    /// Geometry attributes use the same user-coordinate system as the element the gradient is applied to.
    UserSpaceOnUse,
}

impl GradientUnits {
    fn as_str(self) -> &'static str {
        match self {
            Self::ObjectBoundingBox => "objectBoundingBox",
            Self::UserSpaceOnUse => "userSpaceOnUse",
        }
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Controls how a gradient is rendered outside its defined [0.0, 1.0] stop range.
///
/// Passed to [`crate::SvgLinearGradient::set_spread_method`] and [`crate::SvgRadialGradient::set_spread_method`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpreadMethod {
    /// Extend the colour of the nearest stop to fill the remaining area (default).
    Pad,
    /// Mirror the gradient pattern end-to-end, alternating directions.
    Reflect,
    /// Repeat the gradient pattern in the same direction.
    Repeat,
}

impl SpreadMethod {
    fn as_str(self) -> &'static str {
        match self {
            Self::Pad => "pad",
            Self::Reflect => "reflect",
            Self::Repeat => "repeat",
        }
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Private shared state for both gradient types.
///
/// Using a shared inner struct avoids duplicating the id/element/document/attrs fields and the many methods that operate
/// identically on both types.
///
/// `url_ref` caches the complete `url(#id)` reference (built once here, rebuilt in place by `set_id`) rather than just
/// the bare id, so `set_fill_linear_gradient`/`set_fill_radial_gradient` and their stroke siblings can write it straight
/// to the `fill`/`stroke` attribute with no per-call formatting allocation, however many elements the same gradient is
/// applied to. `id()` slices the bare id back out of this string rather than storing it separately.
struct GradientInner {
    url_ref: String,
    element: SvgElement,
    document: Document,
    attrs: RefCell<SvgAttrs>,
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl GradientInner {
    fn new(id: &str, element: SvgElement, document: Document) -> Self {
        let mut url_ref = String::with_capacity(URL_PREFIX.len() + id.len() + 1);
        url_ref.push_str(URL_PREFIX);
        url_ref.push_str(id);
        url_ref.push(')');
        Self {
            url_ref,
            element,
            document,
            attrs: RefCell::new(SvgAttrs::new()),
        }
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// The slice is exact because gradient ids are restricted at validation time to such that they always match the
    /// pattern `[A-Za-z_][A-Za-z0-9_-]*`, which is pure ASCII, so byte offsets from `URL_PREFIX`'s length and the
    /// string's end always land on the bare id exactly.
    fn id(&self) -> &str {
        &self.url_ref[URL_PREFIX.len()..self.url_ref.len() - 1]
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Returns the cached `url(#id)` reference, ready to write directly to a `fill`/`stroke` attribute.
    fn url_ref(&self) -> &str {
        &self.url_ref
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    fn as_element(&self) -> &SvgElement {
        &self.element
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    fn set_id(&mut self, id: &str) -> Result<(), Error> {
        validate_gradient_id(id)?;
        self.element.set_attribute("id", id).map_err(dom_err)?;
        self.url_ref.clear();
        self.url_ref.push_str(URL_PREFIX);
        self.url_ref.push_str(id);
        self.url_ref.push(')');
        Ok(())
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    fn add_stop(&self, offset: f64, color: &str) -> Result<(), Error> {
        let stop = super::create_svg_element::<SvgElement>(&self.document, "stop", "SvgElement")?;
        self.attrs.borrow_mut().display_element(&stop, "offset", offset)?;
        stop.set_attribute("stop-color", color).map_err(dom_err)?;
        self.element.append_child(&stop).map_err(dom_err)?;
        Ok(())
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    fn add_stop_opacity(&self, offset: f64, color: &str, opacity: f64) -> Result<(), Error> {
        let stop = super::create_svg_element::<SvgElement>(&self.document, "stop", "SvgElement")?;
        let mut attrs = self.attrs.borrow_mut();
        attrs.display_element(&stop, "offset", offset)?;
        stop.set_attribute("stop-color", color).map_err(dom_err)?;
        attrs.display_element(&stop, "stop-opacity", opacity)?;
        self.element.append_child(&stop).map_err(dom_err)?;
        Ok(())
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    fn set_gradient_units(&self, units: GradientUnits) -> Result<(), Error> {
        self.element.set_attribute("gradientUnits", units.as_str()).map_err(dom_err)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    fn set_spread_method(&self, method: SpreadMethod) -> Result<(), Error> {
        self.element.set_attribute("spreadMethod", method.as_str()).map_err(dom_err)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    fn set_gradient_transform(&self, transform: &str) -> Result<(), Error> {
        self.element.set_attribute("gradientTransform", transform).map_err(dom_err)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    fn set_attr(&self, name: &str, value: &str) -> Result<(), Error> {
        if name.eq_ignore_ascii_case("id") {
            return Err(Error::ReservedAttribute("id"));
        }
        self.element.set_attribute(name, value).map_err(dom_err)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    fn set_attrs<I, K, V>(&self, attrs: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        for (name, value) in attrs {
            self.set_attr(name.as_ref(), value.as_ref())?;
        }
        Ok(())
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    fn set_attr_display<T: std::fmt::Display>(&self, name: &str, value: T) -> Result<(), Error> {
        if name.eq_ignore_ascii_case("id") {
            return Err(Error::ReservedAttribute("id"));
        }
        self.attrs.borrow_mut().display_element(&self.element, name, value)
    }
}
