use super::*;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// A `<radialGradient>` element defined inside a `<defs>` block.
///
/// A radial gradient paints a smooth colour transition that radiates outward in a circle or ellipse from a focal
/// point (`fx`, `fy`). The transition passes through an outer circle, defined by its centre (`cx`, `cy`) and radius
/// (`r`).
///
/// Under the default `gradientUnits="objectBoundingBox"`, all geometry values are fractions of the
/// painted element's bounding box in [0.0, 1.0].
///
/// The SVG-specified defaults — `cx="50%"`, `cy="50%"`, `r="50%"`, `fx`/`fy` matching `cx`/`cy` — produce a centred
/// circular gradient that fills the element. These percentages only coincide with the bare numbers `0.5` once resolved
/// under the default `objectBoundingBox` units. Under `userSpaceOnUse`, a percentage resolves against the viewport
/// instead. So writing an explicit `0.5` in that mode means "0.5 user units", not "50% of the viewport".
///
/// Apply the gradient to any shape with
/// [`SvgNode::set_fill_radial_gradient`](crate::SvgNode::set_fill_radial_gradient) (fill) or
/// [`SvgNode::set_stroke_radial_gradient`](crate::SvgNode::set_stroke_radial_gradient) (stroke).
///
/// Obtain one from [`SvgDefs::radial_gradient`](crate::SvgDefs::radial_gradient) or the transactional
/// [`SvgDefs::build_radial_gradient`](crate::SvgDefs::build_radial_gradient).
///
/// # Example
///
/// ```rust,no_run
/// use svg_dom::{SvgRoot, root::utils::{Point, Size}};
///
/// let svg  = SvgRoot::attach("diagram")?;
/// let defs = svg.defs()?;
///
/// // Centred glow: white hot core fading to transparent deep-blue.
/// let grad = defs.build_radial_gradient("glow", |g| {
///     g.add_stop_opacity(0.0, "white", 1.0)?;
///     g.add_stop_opacity(1.0, "midnightblue", 0.0)?;
///     Ok(())
/// })?;
///
/// let circle = svg.circle(Point::new(100.0, 100.0), 80.0)?;
/// circle.set_fill_radial_gradient(&grad)?;
/// Ok::<(), svg_dom::Error>(())
/// ```
pub struct SvgRadialGradient(GradientInner);

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl SvgRadialGradient {
    pub(crate) fn new(id: &str, element: SvgElement, document: Document) -> Self {
        Self(GradientInner::new(id, element, document))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Returns the cached `id` of this gradient.
    pub fn id(&self) -> &str {
        self.0.id()
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Returns the cached `url(#id)` reference, ready to write directly to a `fill`/`stroke` attribute.
    ///
    /// Visibility is `pub(crate)` because [`set_fill_radial_gradient`](crate::SvgNode::set_fill_radial_gradient)
    /// and its stroke sibling are the only functions that need it. External callers use [`id`](Self::id) instead.
    pub(crate) fn url_ref(&self) -> &str {
        self.0.url_ref()
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Renames the gradient, updating both the DOM `id` attribute and the cached value returned by [`id`](Self::id).
    ///
    /// **Note:** renaming does not update any `fill="url(#...)"` or `stroke="url(#...)"` attributes
    /// already written to referencing elements.
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
    /// Avoid writing the `id` attribute through this handle. Use [`set_id`](Self::set_id) instead.
    pub fn as_element(&self) -> &SvgElement {
        self.0.as_element()
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Appends a `<stop>` with full opacity to this gradient.
    ///
    /// `offset` is a fraction in [0.0, 1.0] giving the position along the gradient radius.
    /// `color` is any valid SVG/CSS colour value.
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
    /// `opacity` is in [0.0, 1.0]. `0.0` is fully transparent, and `1.0` is fully opaque.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if the browser refuses to create or append the `<stop>` element.
    pub fn add_stop_opacity(&self, offset: f64, color: &str, opacity: f64) -> Result<(), Error> {
        self.0.add_stop_opacity(offset, color, opacity)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `cx` attribute — the x-coordinate of the outer circle's centre.
    ///
    /// Under the default `gradientUnits="objectBoundingBox"`, this is a fraction in the range [0.0, 1.0] of the
    /// element's bounding box width.
    ///
    /// The SVG-specified default when absent is `50%` (horizontal centre of the bounding box under the default
    /// `objectBoundingBox` units).
    pub fn set_cx(&self, v: f64) -> Result<(), Error> {
        self.0.attrs.borrow_mut().display_element(self.0.as_element(), "cx", v)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `cy` attribute — the y-coordinate of the outer circle's centre.
    ///
    /// Under the default `gradientUnits="objectBoundingBox"`, this is a fraction in the range [0.0, 1.0] of the
    /// element's bounding box height.
    ///
    /// The SVG-specified default when absent is `50%` (vertical centre of the bounding box under the default
    /// `objectBoundingBox` units).
    pub fn set_cy(&self, v: f64) -> Result<(), Error> {
        self.0.attrs.borrow_mut().display_element(self.0.as_element(), "cy", v)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `r` attribute — the radius of the outer circle.
    ///
    /// Under the default `gradientUnits="objectBoundingBox"`, this is a fraction of the element's
    /// bounding box.
    ///
    /// The SVG-specified default when absent is `50%` (half the bounding box size under the default `objectBoundingBox`
    /// units).
    pub fn set_r(&self, v: f64) -> Result<(), Error> {
        self.0.attrs.borrow_mut().display_element(self.0.as_element(), "r", v)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `fx` attribute — the x-coordinate of the focal point.
    ///
    /// The gradient colour at offset `0.0` emanates from the focal point.
    /// When omitted, the focal point defaults to the outer circle's centre (`cx`).
    /// Placing it off-centre produces an asymmetric "hot spot" or directional glow effect.
    pub fn set_fx(&self, v: f64) -> Result<(), Error> {
        self.0.attrs.borrow_mut().display_element(self.0.as_element(), "fx", v)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `fy` attribute — the y-coordinate of the focal point.
    ///
    /// When omitted, defaults to the outer circle's centre (`cy`).
    pub fn set_fy(&self, v: f64) -> Result<(), Error> {
        self.0.attrs.borrow_mut().display_element(self.0.as_element(), "fy", v)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `fr` attribute — the radius of the focal/start circle (SVG 2).
    ///
    /// The gradient's `0%` stop is mapped to this circle's perimeter, and its interior is painted with the first stop's
    /// colour.
    ///
    /// `fr` does not inherently create a hole. A hollow-looking centre must be created from the stop colours themselves,
    /// for example by making the first stop transparent.
    ///
    /// When omitted, the SVG-specified default is `0%` (point focal), which is the usual behaviour.
    pub fn set_fr(&self, v: f64) -> Result<(), Error> {
        self.0.attrs.borrow_mut().display_element(self.0.as_element(), "fr", v)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `gradientUnits` attribute, controlling the coordinate space used by `cx`, `cy`, `r`, `fx`, `fy`,
    /// and `fr`.
    pub fn set_gradient_units(&self, units: GradientUnits) -> Result<(), Error> {
        self.0.set_gradient_units(units)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `spreadMethod` attribute, controlling how the gradient behaves outside [0.0, 1.0].
    pub fn set_spread_method(&self, method: SpreadMethod) -> Result<(), Error> {
        self.0.set_spread_method(method)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `gradientTransform` attribute, applying an additional transform to the gradient coordinate system.
    pub fn set_gradient_transform(&self, transform: &str) -> Result<(), Error> {
        self.0.set_gradient_transform(transform)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets any attribute on the `<radialGradient>` element by name and string value.
    ///
    /// Passing `"id"` (case-insensitively) returns [`Error::ReservedAttribute`]. Use [`set_id`](Self::set_id) instead.
    pub fn set_attr(&self, name: &str, value: &str) -> Result<(), Error> {
        self.0.set_attr(name, value)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets several attributes in one call.
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
    /// Passing `"id"` (case-insensitively) returns [`Error::ReservedAttribute`]. Use [`set_id`](Self::set_id) instead.
    pub fn set_attr_display<T: std::fmt::Display>(&self, name: &str, value: T) -> Result<(), Error> {
        self.0.set_attr_display(name, value)
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl SvgDefs {
    /// Creates a `<radialGradient>` child element with the given `id`, appends it to `<defs>` immediately and returns
    /// its handle.
    ///
    /// The `id` is used to reference the gradient from shapes via
    /// [`set_fill_radial_gradient`](crate::SvgNode::set_fill_radial_gradient) and its stroke sibling.
    ///
    /// Prefer [`build_radial_gradient`](Self::build_radial_gradient) when all stops are known upfront.
    ///
    /// # Errors
    ///
    /// - [`Error::InvalidGradientId`] — `id` failed validation.
    /// - [`Error::Dom`] — the browser refused to create or append the element.
    pub fn radial_gradient(&self, id: &str) -> Result<SvgRadialGradient, Error> {
        validate_gradient_id(id)?;
        let el = crate::root::create_svg_element::<SvgElement>(self.document(), "radialGradient", "SvgElement")?;
        el.set_attribute("id", id).map_err(dom_err)?;
        self.as_element().append_child(&el).map_err(dom_err)?;
        Ok(SvgRadialGradient::new(id, el, self.document().clone()))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Builds a `<radialGradient>` and all its stops in one go, appending it to `<defs>` only after the closure
    /// succeeds.
    ///
    /// The closure receives a reference to the new [`SvgRadialGradient`].
    ///
    /// If the closure returns `Ok(())`, the gradient is appended to `<defs>` and the handle is returned.
    /// If the closure returns `Err`, the gradient element is dropped without being attached to `<defs>`.
    ///
    /// # Errors
    ///
    /// - Any error returned by `build`.
    /// - [`Error::InvalidGradientId`] — `id` failed validation.
    /// - [`Error::Dom`] — the browser refused to create or append the element.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::utils::{Point, Size}};
    ///
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    ///
    /// let grad = defs.build_radial_gradient("glow", |g| {
    ///     g.add_stop_opacity(0.0, "white", 1.0)?;
    ///     g.add_stop_opacity(1.0, "midnightblue", 0.0)?;
    ///     Ok(())
    /// })?;
    ///
    /// let circle = svg.circle(Point::new(100.0, 100.0), 80.0)?;
    /// circle.set_fill_radial_gradient(&grad)?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn build_radial_gradient<F>(&self, id: &str, build: F) -> Result<SvgRadialGradient, Error>
    where
        F: FnOnce(&SvgRadialGradient) -> Result<(), Error>,
    {
        validate_gradient_id(id)?;
        let el = crate::root::create_svg_element::<SvgElement>(self.document(), "radialGradient", "SvgElement")?;
        el.set_attribute("id", id).map_err(dom_err)?;
        let grad = SvgRadialGradient::new(id, el, self.document().clone());
        build(&grad)?;
        self.as_element().append_child(grad.as_element()).map_err(dom_err)?;
        Ok(grad)
    }
}
