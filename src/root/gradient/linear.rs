use super::*;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// A `<linearGradient>` element defined inside a `<defs>` block.
///
/// A linear gradient paints a smooth colour transition along a straight line from (`x1`, `y1`) to (`x2`, `y2`).
/// Colour stops (which can be added with [`add_stop`](Self::add_stop)) define the colours at specific positions along
/// that line.
///
/// Under the default `gradientUnits="objectBoundingBox"`, the geometry coordinates are fractions of the painted
/// element's bounding box: `0.0` means the left/top edge and `1.0` means the right/bottom edge.
///
/// The SVG defaults for the axis endpoints are `x1="0"`, `y1="0"`, `x2="1"`, `y2="0"`, which gives a horizontal
/// left-to-right gradient.  This is the most common case, so for that you only need to add stops.
///
/// Apply the gradient to any shape with
/// [`SvgNode::set_fill_linear_gradient`](crate::SvgNode::set_fill_linear_gradient) (fill) or
/// [`SvgNode::set_stroke_linear_gradient`](crate::SvgNode::set_stroke_linear_gradient) (stroke).
///
/// Obtain one from [`SvgDefs::linear_gradient`](crate::SvgDefs::linear_gradient) or the transactional
/// [`SvgDefs::build_linear_gradient`](crate::SvgDefs::build_linear_gradient).
///
/// # Example
///
/// ```rust,no_run
/// use svg_dom::{SvgRoot, root::utils::{Point, Size}};
///
/// let svg  = SvgRoot::attach("diagram")?;
/// let defs = svg.defs()?;
///
/// // Horizontal steelblue-to-coral gradient (default left-to-right direction).
/// let grad = defs.build_linear_gradient("sunset", |g| {
///     g.add_stop(0.0, "steelblue")?;
///     g.add_stop(1.0, "coral")?;
///     Ok(())
/// })?;
///
/// let rect = svg.rect(Point::origin(), Size::new(200.0, 100.0))?;
/// rect.set_fill_linear_gradient(&grad)?;
/// Ok::<(), svg_dom::Error>(())
/// ```
pub struct SvgLinearGradient(GradientInner);

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl SvgLinearGradient {
    pub(crate) fn new(id: &str, element: SvgElement, document: Document) -> Self {
        Self(GradientInner::new(id, element, document))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Returns the cached `id` of this gradient.
    ///
    /// Pass this to [`SvgNode::set_fill_gradient`](crate::SvgNode::set_fill_gradient) and its stroke sibling, or use
    /// the typed [`set_fill_linear_gradient`](crate::SvgNode::set_fill_linear_gradient) with the gradient handle
    /// directly to avoid touching the id at all.
    pub fn id(&self) -> &str {
        self.0.id()
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Returns the cached `url(#id)` reference, ready to write directly to a `fill`/`stroke` attribute.
    ///
    /// Visibility need only be `pub(crate)` since [`set_fill_linear_gradient`](crate::SvgNode::set_fill_linear_gradient)
    /// and its stroke sibling are the only functions that need it; external callers use [`id`](Self::id) instead.
    pub(crate) fn url_ref(&self) -> &str {
        self.0.url_ref()
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Renames the gradient, updating both the DOM `id` attribute and the cached value returned by [`id`](Self::id).
    ///
    /// The new `id` is subject to the same validation rules as the id supplied at construction time.
    ///
    /// **IMPORTANT**<br>
    /// Renaming an id does not update any `fill="url(#...)"` or `stroke="url(#...)"` attributes that already reference
    /// the old gradient name. Either rename the id before referencing it, or reapply the new id reference after
    /// renaming.
    ///
    /// # Errors
    ///
    /// - [`Error::InvalidGradientId`] — the new id failed validation.
    /// - [`Error::Dom`] — the browser refused to write the `id` attribute.
    pub fn set_id(&mut self, id: &str) -> Result<(), Error> {
        self.0.set_id(id)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Returns a reference to the underlying `web-sys` `SvgElement`.
    ///
    /// Avoid writing the `id` attribute through this handle; use [`set_id`](Self::set_id) instead so the cached value
    /// stays in sync.
    pub fn as_element(&self) -> &SvgElement {
        self.0.as_element()
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Appends a `<stop>` with full opacity to this gradient.
    ///
    /// `offset` is a fraction in the range `[0.0, 1.0]` giving the position along the gradient axis.
    /// `color` is any valid SVG/CSS colour value (`"red"`, `"#ff0000"`, `"rgb(255,0,0)"`, etc.).
    ///
    /// Stops are rendered in document order; therefore, for more predictable results, they should be added in ascending
    /// offset order.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if the browser refuses to create or append the `<stop>` element.
    pub fn add_stop(&self, offset: f64, color: &str) -> Result<(), Error> {
        self.0.add_stop(offset, color)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Appends a `<stop>` with an explicit `stop-opacity` to this gradient.
    ///
    /// `opacity` is in the range `[0.0, 1.0]` where `0.0` is fully transparent and `1.0` is fully opaque.
    /// Values outside that range are accepted but produce unspecified rendering.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if the browser refuses to create or append the `<stop>` element.
    pub fn add_stop_opacity(&self, offset: f64, color: &str, opacity: f64) -> Result<(), Error> {
        self.0.add_stop_opacity(offset, color, opacity)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the x-coordinate of the gradient's start point.
    ///
    /// If you are using `gradientUnits="objectBoundingBox"`, this is a fraction in the range `[0.0, 1.0]` of the
    /// element's bounding box width (`0.0` = left edge, `1.0` = right edge).
    ///
    /// If you are using `"userSpaceOnUse"`, it is an absolute coordinate in the user coordinate system.
    ///
    /// If absent, the SVG default is `0.0` (left edge of the bounding box).
    pub fn set_x1(&self, v: f64) -> Result<(), Error> {
        self.0.attrs.borrow_mut().display_element(self.0.as_element(), "x1", v)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the y-coordinate of the gradient's start point.
    ///
    /// If absent, the SVG default is `0.0` (top edge of the bounding box).
    pub fn set_y1(&self, v: f64) -> Result<(), Error> {
        self.0.attrs.borrow_mut().display_element(self.0.as_element(), "y1", v)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the x-coordinate of the gradient's end point.
    ///
    /// If absent, the SVG default is `1.0` (right edge of the bounding box), which together with the `y2` default of
    /// `0.0` gives the standard horizontal left-to-right gradient.
    pub fn set_x2(&self, v: f64) -> Result<(), Error> {
        self.0.attrs.borrow_mut().display_element(self.0.as_element(), "x2", v)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the y-coordinate of the gradient's end point.
    ///
    /// If absent, the SVG default is `0.0` (top edge of the bounding box).
    pub fn set_y2(&self, v: f64) -> Result<(), Error> {
        self.0.attrs.borrow_mut().display_element(self.0.as_element(), "y2", v)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `gradientUnits` attribute, controlling the coordinate space used by `x1`, `y1`, `x2`, and `y2`.
    pub fn set_gradient_units(&self, units: GradientUnits) -> Result<(), Error> {
        self.0.set_gradient_units(units)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `spreadMethod` attribute, controlling how the gradient behaves outside the `[0.0, 1.0]` range.
    ///
    /// - [`Pad`](SpreadMethod::Pad) — extend the nearest end colour (default).
    /// - [`Reflect`](SpreadMethod::Reflect) — mirror the pattern alternately.
    /// - [`Repeat`](SpreadMethod::Repeat) — tile the pattern in the same direction.
    pub fn set_spread_method(&self, method: SpreadMethod) -> Result<(), Error> {
        self.0.set_spread_method(method)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `gradientTransform` attribute, applying an additional transform to the gradient coordinate system.
    ///
    /// Accepts the same transform syntax as the SVG `transform` attribute:
    /// `"rotate(45)"`, `"translate(10, 5) scale(2)"`, etc.
    ///
    /// This is the most flexible way to create a diagonal gradient: keep the default `x1`/`x2` endpoints and rotate the
    /// gradient by 45º with `set_gradient_transform("rotate(45)")`.
    pub fn set_gradient_transform(&self, transform: &str) -> Result<(), Error> {
        self.0.set_gradient_transform(transform)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets any attribute on the `<linearGradient>` element by name and string value.
    ///
    /// This is a generic escape hatch for attributes not covered by a named setter.
    ///
    /// ***WARNING***
    /// Name and value are written verbatim; so do not pass any untrusted input.
    ///
    /// Passing `"id"` (case-insensitively) returns [`Error::ReservedAttribute`]; use [`set_id`](Self::set_id) instead.
    pub fn set_attr(&self, name: &str, value: &str) -> Result<(), Error> {
        self.0.set_attr(name, value)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets multiple attributes in a single call.
    ///
    /// Equivalent to calling [`set_attr`](Self::set_attr) for each pair.
    /// Returns the first error encountered and applies no further attributes.
    /// Attributes written before the error are left in place.
    pub fn set_attrs<I, K, V>(&self, attrs: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.0.set_attrs(attrs)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Formats `value` through the element's internal scratch buffer and writes it as `name`.
    ///
    /// Passing `"id"` (case-insensitively) returns [`Error::ReservedAttribute`]; use [`set_id`](Self::set_id) instead.
    pub fn set_attr_display<T: std::fmt::Display>(&self, name: &str, value: T) -> Result<(), Error> {
        self.0.set_attr_display(name, value)
    }
}
