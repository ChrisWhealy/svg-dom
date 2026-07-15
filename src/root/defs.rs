use crate::{
    Error, SvgNode, dom_err,
    root::{
        attrs::SvgAttrs,
        factory::SvgFactory,
        path::path_def::PathDef,
        utils::{Point, Size},
    },
};
use std::cell::RefCell;
use web_sys::{Document, SvgElement};

use super::{
    clip_path::SvgClipPath,
    filter::SvgFilter,
    gradient::{linear::SvgLinearGradient, radial::SvgRadialGradient},
    marker::SvgMarker,
    pattern::SvgPattern,
    svg_root::SvgRoot,
    symbol::SvgSymbol,
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// The fixed prefix of a `url(#...)` reference.
///
/// This value is used by every type below that caches a complete reference string (`SvgMarker`, `SvgClipPath`,
/// `SvgPattern`, `SvgFilter`, `GradientInner`) rather than just its bare id, so the `url(#id)` value can be written to
/// a `fill`/`stroke`/`clip-path`/`marker-*`/`filter` attribute without allocating a fresh `String` on every reference.
pub(crate) const URL_PREFIX: &str = "url(#";

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Checks whether `id` is safe to embed in a `url(#...)` CSS/SVG paint-server reference.
///
/// A valid id must match `[A-Za-z_][A-Za-z0-9_-]*`: it must begin with an ASCII letter or underscore,
/// followed by zero or more ASCII letters, digits, underscores, or hyphens.
fn is_valid_svg_id(id: &str) -> bool {
    let mut chars = id.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {
            chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        },
        _ => false,
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Rejects marker ids that would produce broken or ambiguous `url(#...)` references.
///
/// A valid id must match `[A-Za-z_][A-Za-z0-9_-]*`: it must begin with an ASCII letter or underscore,
/// followed by zero or more ASCII letters, digits, underscores, or hyphens.
/// This conservative allow-list ensures that any accepted id can be safely embedded in the generated
/// `url(#id)` CSS/SVG paint-server reference without quoting, escaping, or browser-specific interpretation.
pub(crate) fn validate_marker_id(id: &str) -> Result<(), Error> {
    if is_valid_svg_id(id) {
        Ok(())
    } else {
        Err(Error::InvalidMarkerId(id.to_owned()))
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Rejects gradient ids that would produce broken or ambiguous `url(#...)` references.
///
/// Applies the same allow-list as [`validate_marker_id`]: the id must match `[A-Za-z_][A-Za-z0-9_-]*`.
pub(crate) fn validate_gradient_id(id: &str) -> Result<(), Error> {
    if is_valid_svg_id(id) {
        Ok(())
    } else {
        Err(Error::InvalidGradientId(id.to_owned()))
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Rejects clip-path ids that would produce broken or ambiguous `url(#...)` references.
///
/// Applies the same allow-list as [`validate_marker_id`]: the id must match `[A-Za-z_][A-Za-z0-9_-]*`.
pub(crate) fn validate_clip_path_id(id: &str) -> Result<(), Error> {
    if is_valid_svg_id(id) {
        Ok(())
    } else {
        Err(Error::InvalidClipPathId(id.to_owned()))
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Rejects filter ids that would produce broken or ambiguous `url(#...)` references.
///
/// Applies the same allow-list as [`validate_marker_id`]: the id must match `[A-Za-z_][A-Za-z0-9_-]*`.
pub(crate) fn validate_filter_id(id: &str) -> Result<(), Error> {
    if is_valid_svg_id(id) {
        Ok(())
    } else {
        Err(Error::InvalidFilterId(id.to_owned()))
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Rejects symbol ids that would produce broken `#id` fragment references.
///
/// Applies the same allow-list as [`validate_marker_id`]: the id must match `[A-Za-z_][A-Za-z0-9_-]*`.
pub(crate) fn validate_symbol_id(id: &str) -> Result<(), Error> {
    if is_valid_svg_id(id) {
        Ok(())
    } else {
        Err(Error::InvalidSymbolId(id.to_owned()))
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Rejects pattern ids that would produce broken or ambiguous `url(#...)` references.
///
/// Applies the same allow-list as [`validate_marker_id`]: the id must match `[A-Za-z_][A-Za-z0-9_-]*`.
pub(crate) fn validate_pattern_id(id: &str) -> Result<(), Error> {
    if is_valid_svg_id(id) {
        Ok(())
    } else {
        Err(Error::InvalidPatternId(id.to_owned()))
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// A `<defs>` element that holds reusable SVG assets such as markers and gradients.
///
/// Elements created inside `<defs>` are not rendered directly; they are referenced by other elements via an `id`.
/// All the usual shape factory methods are available for building inner content of markers, but the primary purpose of
/// `SvgDefs` is to serve as the container for named paint servers:
///
/// | Asset | Factory | Eager variant |
/// |---|---|---|
/// | [`SvgMarker`] | [`marker`](Self::marker) | [`build_marker`](Self::build_marker) |
/// | [`SvgLinearGradient`] | [`linear_gradient`](Self::linear_gradient) | [`build_linear_gradient`](Self::build_linear_gradient) |
/// | [`SvgRadialGradient`] | [`radial_gradient`](Self::radial_gradient) | [`build_radial_gradient`](Self::build_radial_gradient) |
/// | [`SvgClipPath`] | [`clip_path`](Self::clip_path) | [`build_clip_path`](Self::build_clip_path) |
/// | [`SvgSymbol`] | [`symbol`](Self::symbol) | [`build_symbol`](Self::build_symbol) |
/// | [`SvgPattern`] | [`pattern`](Self::pattern) | [`build_pattern`](Self::build_pattern) |
/// | [`SvgFilter`] | [`filter`](Self::filter) | [`build_filter`](Self::build_filter) |
///
/// Obtain one from [`SvgRoot::defs`].
///
/// # Example
///
/// ```rust,no_run
/// use svg_dom::{SvgRoot, root::utils::{Point, Size}};
///
/// let svg  = SvgRoot::attach("diagram")?;
/// let defs = svg.defs()?;
///
/// // A filled-triangle arrowhead marker.
/// let marker = defs.marker("arrow")?;
/// marker.set_ref_x(10.0)?;
/// marker.set_ref_y(3.5)?;
/// marker.set_marker_width(10.0)?;
/// marker.set_marker_height(7.0)?;
/// marker.set_orient("auto")?;
/// marker.polygon(&[Point::new(0.0, 0.0), Point::new(10.0, 3.5), Point::new(0.0, 7.0)])?;
///
/// // Apply the marker to a line.
/// let line = svg.line(Point::new(20.0, 50.0), Point::new(180.0, 50.0))?;
/// line.set_stroke("black")?;
/// line.set_marker_end("arrow")?;
/// Ok::<(), svg_dom::Error>(())
/// ```
pub struct SvgDefs {
    element: SvgElement,
    document: Document,
    attrs: RefCell<SvgAttrs>,
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl SvgDefs {
    pub(crate) fn new(element: SvgElement, document: Document) -> Self {
        Self {
            element,
            document,
            attrs: RefCell::new(SvgAttrs::new()),
        }
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Returns a reference to the underlying `web-sys` `SvgElement`.
    pub fn as_element(&self) -> &SvgElement {
        &self.element
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<marker>` child element with the given `id`, appends it to `<defs>` immediately and returns its
    /// handle.
    ///
    /// The `id` is used to reference the marker from a line or path via
    /// [`set_marker_end`](crate::SvgNode::set_marker_end) and its `_start` / `_mid` siblings.
    ///
    /// Each shape added to the returned [`SvgMarker`] is appended to the live marker element one at a time.
    /// Use this when you need to add shapes to a marker dynamically after initial construction.
    ///
    /// Prefer [`build_marker`](Self::build_marker) when all marker contents are known upfront: that variant holds the
    /// `<marker>` element detached until the closure succeeds, so a mid-build error leaves no partial marker in
    /// `<defs>`.
    /// With this method, if a shape or attribute setter fails after `marker()` returns, the partial `<marker>` element
    /// remains in `<defs>` (though an incomplete marker is harmless — it renders nothing unless referenced).
    ///
    /// # Errors
    ///
    /// - [`Error::Dom`] — the browser refused to create or append the element.
    pub fn marker(&self, id: &str) -> Result<SvgMarker, Error> {
        validate_marker_id(id)?;
        let el = super::create_svg_element::<SvgElement>(&self.document, "marker", "SvgElement")?;
        el.set_attribute("id", id).map_err(dom_err)?;
        self.element.append_child(&el).map_err(dom_err)?;
        Ok(SvgMarker::new(id.to_owned(), el, self.document.clone()))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Builds a `<marker>` and all its child shapes in one shot, appending to `<defs>` only after the closure succeeds.
    ///
    /// The closure receives a reference to the new [`SvgMarker`].
    /// All shapes added inside the closure are appended to a detached marker element.
    ///
    /// If the closure returns `Ok(())`, the marker is appended to `<defs>` and the handle is returned.
    /// If the closure returns `Err`, the marker element is dropped without being attached to `<defs>`.
    ///
    /// This is the preferred way to populate a marker when you know all its children up-front.
    /// For dynamically adding shapes to a marker over time, use [`marker`](Self::marker) instead.
    ///
    /// # Errors
    ///
    /// - Any error returned by `build`.
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
    /// let marker = defs.build_marker("arrow", |m| {
    ///     m.set_ref_x(10.0)?;
    ///     m.set_ref_y(3.5)?;
    ///     m.set_marker_width(10.0)?;
    ///     m.set_marker_height(7.0)?;
    ///     m.set_orient("auto")?;
    ///     m.polygon(&[Point::new(0.0, 0.0), Point::new(10.0, 3.5), Point::new(0.0, 7.0)])?;
    ///     Ok(())
    /// })?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn build_marker<F>(&self, id: &str, build: F) -> Result<SvgMarker, Error>
    where
        F: FnOnce(&SvgMarker) -> Result<(), Error>,
    {
        validate_marker_id(id)?;
        let el = super::create_svg_element::<SvgElement>(&self.document, "marker", "SvgElement")?;
        el.set_attribute("id", id).map_err(dom_err)?;
        let marker = SvgMarker::new(id.to_owned(), el, self.document.clone());
        build(&marker)?;
        self.element.append_child(marker.as_element()).map_err(dom_err)?;
        Ok(marker)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<clipPath>` child element with the given `id`, appends it to `<defs>` immediately and returns its
    /// handle.
    ///
    /// The `id` is used to reference the clip path from any element via
    /// [`set_clip_path_ref`](crate::SvgNode::set_clip_path_ref) or
    /// [`set_clip_path`](crate::SvgNode::set_clip_path).
    ///
    /// Each shape added to the returned [`SvgClipPath`] is appended to the live element one at a time.
    /// Use this when you need to add clip shapes dynamically after initial construction.
    ///
    /// Prefer [`build_clip_path`](Self::build_clip_path) when all clip shapes are known upfront: that variant holds the
    /// `<clipPath>` element detached until the closure succeeds, so a mid-build error leaves no partial element in
    /// `<defs>`.
    /// With this method, if a shape or attribute setter fails after `clip_path()` returns, the partial `<clipPath>`
    /// remains in `<defs>` (though an incomplete clip path is harmless — it clips nothing unless referenced).
    ///
    /// # Errors
    ///
    /// - [`Error::InvalidClipPathId`] — `id` failed validation.
    /// - [`Error::Dom`] — the browser refused to create or append the element.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::utils::{Point, Size}};
    ///
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    /// let clip = defs.clip_path("round-viewport")?;
    /// clip.circle(Point::new(60.0, 60.0), 55.0)?;
    ///
    /// let bg = svg.rect(Point::origin(), Size::new(120.0, 120.0))?;
    /// bg.set_fill("steelblue")?;
    /// bg.set_clip_path_ref(&clip)?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn clip_path(&self, id: &str) -> Result<SvgClipPath, Error> {
        validate_clip_path_id(id)?;
        let el = super::create_svg_element::<SvgElement>(&self.document, "clipPath", "SvgElement")?;
        el.set_attribute("id", id).map_err(dom_err)?;
        self.element.append_child(&el).map_err(dom_err)?;
        Ok(SvgClipPath::new(id.to_owned(), el, self.document.clone()))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Builds a `<clipPath>` and all its clip shapes in one shot, appending to `<defs>` only after the closure
    /// succeeds.
    ///
    /// The closure receives a reference to the new [`SvgClipPath`].
    /// All shapes added inside the closure are appended to a detached element.
    ///
    /// If the closure returns `Ok(())`, the clip path is appended to `<defs>` and the handle is returned.
    /// If the closure returns `Err`, the element is dropped without being attached to `<defs>`.
    ///
    /// This is the preferred way to build a clip path when all its shapes are known upfront.
    /// For dynamically adding shapes over time, use [`clip_path`](Self::clip_path) instead.
    ///
    /// # Errors
    ///
    /// - Any error returned by `build`.
    /// - [`Error::InvalidClipPathId`] — `id` failed validation.
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
    /// // Clip a gradient rectangle to a hexagon.
    /// let clip = defs.build_clip_path("hex-frame", |c| {
    ///     c.polygon(&[
    ///         Point::new(60.0,  5.0), Point::new(115.0, 35.0), Point::new(115.0, 85.0),
    ///         Point::new(60.0, 115.0), Point::new( 5.0, 85.0), Point::new(  5.0, 35.0),
    ///     ])?;
    ///     Ok(())
    /// })?;
    ///
    /// let rect = svg.rect(Point::origin(), Size::new(120.0, 120.0))?;
    /// rect.set_fill("steelblue")?;
    /// rect.set_clip_path_ref(&clip)?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn build_clip_path<F>(&self, id: &str, build: F) -> Result<SvgClipPath, Error>
    where
        F: FnOnce(&SvgClipPath) -> Result<(), Error>,
    {
        validate_clip_path_id(id)?;
        let el = super::create_svg_element::<SvgElement>(&self.document, "clipPath", "SvgElement")?;
        el.set_attribute("id", id).map_err(dom_err)?;
        let clip = SvgClipPath::new(id.to_owned(), el, self.document.clone());
        build(&clip)?;
        self.element.append_child(clip.as_element()).map_err(dom_err)?;
        Ok(clip)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<filter>` child element with the given `id`, appends it to `<defs>` immediately and returns its
    /// handle.
    ///
    /// The `id` is used to reference the filter from any element via
    /// [`set_filter_ref`](crate::SvgNode::set_filter_ref) or [`set_filter`](crate::SvgNode::set_filter).
    ///
    /// Each primitive added to the returned [`SvgFilter`] is appended to the live element one at a time.
    /// Use this when you need to add primitives dynamically after initial construction.
    ///
    /// Prefer [`build_filter`](Self::build_filter) when all primitives are known upfront: that variant holds the
    /// `<filter>` element detached until the closure succeeds, so a mid-build error leaves no partial element in
    /// `<defs>`.
    /// With this method, if a primitive or attribute setter fails after `filter()` returns, the partial `<filter>`
    /// remains in `<defs>` (though an incomplete filter is harmless — it applies no effect unless referenced).
    ///
    /// # Errors
    ///
    /// - [`Error::InvalidFilterId`] — `id` failed validation.
    /// - [`Error::Dom`] — the browser refused to create or append the element.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::utils::{Point, Size}};
    ///
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    /// let blur = defs.filter("soft-blur")?;
    /// blur.gaussian_blur(4.0)?;
    ///
    /// let rect = svg.rect(Point::origin(), Size::new(120.0, 80.0))?;
    /// rect.set_fill("steelblue")?;
    /// rect.set_filter_ref(&blur)?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn filter(&self, id: &str) -> Result<SvgFilter, Error> {
        validate_filter_id(id)?;
        let el = super::create_svg_element::<SvgElement>(&self.document, "filter", "SvgElement")?;
        el.set_attribute("id", id).map_err(dom_err)?;
        self.element.append_child(&el).map_err(dom_err)?;
        Ok(SvgFilter::new(id.to_owned(), el, self.document.clone()))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Builds a `<filter>` and all its primitives in one shot, appending to `<defs>` only after the closure succeeds.
    ///
    /// The closure receives a reference to the new [`SvgFilter`].
    /// All primitives added inside the closure are appended to a detached element.
    ///
    /// If the closure returns `Ok(())`, the filter is appended to `<defs>` and the handle is returned.
    /// If the closure returns `Err`, the element is dropped without being attached to `<defs>`.
    ///
    /// This is the preferred way to build a filter when all its primitives are known upfront.
    /// For dynamically adding primitives over time, use [`filter`](Self::filter) instead.
    ///
    /// # Errors
    ///
    /// - Any error returned by `build`.
    /// - [`Error::InvalidFilterId`] — `id` failed validation.
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
    /// let blur = defs.build_filter("soft-blur", |f| {
    ///     f.gaussian_blur(4.0)?;
    ///     Ok(())
    /// })?;
    ///
    /// let rect = svg.rect(Point::origin(), Size::new(120.0, 80.0))?;
    /// rect.set_fill("steelblue")?;
    /// rect.set_filter_ref(&blur)?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn build_filter<F>(&self, id: &str, build: F) -> Result<SvgFilter, Error>
    where
        F: FnOnce(&SvgFilter) -> Result<(), Error>,
    {
        validate_filter_id(id)?;
        let el = super::create_svg_element::<SvgElement>(&self.document, "filter", "SvgElement")?;
        el.set_attribute("id", id).map_err(dom_err)?;
        let filter = SvgFilter::new(id.to_owned(), el, self.document.clone());
        build(&filter)?;
        self.element.append_child(filter.as_element()).map_err(dom_err)?;
        Ok(filter)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<symbol>` child element with the given `id`, appends it to `<defs>` immediately and returns its
    /// handle.
    ///
    /// A `<symbol>` defines a reusable viewport: unlike a plain `<g>` in `<defs>`, it can carry a `viewBox` so
    /// that each `<use>` instance scales the content to its own `width` and `height`.
    ///
    /// Each shape added to the returned [`SvgSymbol`] is appended to the live element one at a time.
    /// Use this when you need to add shapes to a symbol dynamically after initial construction.
    ///
    /// Prefer [`build_symbol`](Self::build_symbol) when all symbol contents are known upfront: that variant holds the
    /// element detached until the closure succeeds, so a mid-build error leaves no partial symbol in `<defs>`.
    ///
    /// # Errors
    ///
    /// - [`Error::InvalidSymbolId`] — `id` failed validation.
    /// - [`Error::Dom`] — the browser refused to create or append the element.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::utils::{Point, Size}};
    ///
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    /// let sym  = defs.symbol("icon")?;
    /// sym.set_view_box(0.0, 0.0, 40.0, 40.0)?;
    /// sym.circle(Point::new(20.0, 20.0), 18.0)?.set_fill("steelblue")?;
    ///
    /// svg.use_node("#icon", Point::new(10.0, 10.0))?.set_attr("width", "40")?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn symbol(&self, id: &str) -> Result<SvgSymbol, Error> {
        validate_symbol_id(id)?;
        let el = super::create_svg_element::<SvgElement>(&self.document, "symbol", "SvgElement")?;
        el.set_attribute("id", id).map_err(dom_err)?;
        self.element.append_child(&el).map_err(dom_err)?;
        Ok(SvgSymbol::new(id.to_owned(), el, self.document.clone()))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Builds a `<symbol>` and all its child shapes in one shot, appending to `<defs>` only after the closure
    /// succeeds.
    ///
    /// The closure receives a reference to the new [`SvgSymbol`].
    /// All shapes added inside the closure are appended to a detached element.
    ///
    /// If the closure returns `Ok(())`, the symbol is appended to `<defs>` and the handle is returned.
    /// If the closure returns `Err`, the element is dropped without being attached to `<defs>`.
    ///
    /// This is the preferred way to build a symbol when all its content is known upfront.
    /// For dynamically adding shapes over time, use [`symbol`](Self::symbol) instead.
    ///
    /// # Errors
    ///
    /// - Any error returned by `build`.
    /// - [`Error::InvalidSymbolId`] — `id` failed validation.
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
    /// // Build a badge icon; the viewBox scales it automatically at any <use> size.
    /// defs.build_symbol("badge", |s| {
    ///     s.set_view_box(0.0, 0.0, 40.0, 40.0)?;
    ///     s.circle(Point::new(20.0, 20.0), 18.0)?.set_fill("steelblue")?;
    ///     Ok(())
    /// })?;
    ///
    /// svg.use_node("#badge", Point::new(10.0, 10.0))?.set_attr("width", "80")?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn build_symbol<F>(&self, id: &str, build: F) -> Result<SvgSymbol, Error>
    where
        F: FnOnce(&SvgSymbol) -> Result<(), Error>,
    {
        validate_symbol_id(id)?;
        let el = super::create_svg_element::<SvgElement>(&self.document, "symbol", "SvgElement")?;
        el.set_attribute("id", id).map_err(dom_err)?;
        let sym = SvgSymbol::new(id.to_owned(), el, self.document.clone());
        build(&sym)?;
        self.element.append_child(sym.as_element()).map_err(dom_err)?;
        Ok(sym)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<pattern>` child element with the given `id`, immediately appends it to `<defs>` then returns its
    /// handle.
    ///
    /// The `id` is used to reference the pattern from shapes via
    /// [`set_fill_pattern`](crate::SvgNode::set_fill_pattern) and its stroke sibling.
    ///
    /// Each shape added to the returned [`SvgPattern`] is appended to the live element one at a time. Use this when you
    /// need to add tile shapes dynamically after initial construction.
    ///
    /// Prefer [`build_pattern`](Self::build_pattern) when all tile contents are known upfront: that variant holds the
    /// `<pattern>` element detached until the closure succeeds, so a mid-build error leaves no partial element in
    /// `<defs>`.
    ///
    /// # Errors
    ///
    /// - [`Error::InvalidPatternId`] — `id` failed validation.
    /// - [`Error::Dom`] — the browser refused to create or append the element.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::{pattern::PatternUnits, utils::{Point, Size}}};
    ///
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    /// let pat  = defs.pattern("dots")?;
    /// pat.set_pattern_units(PatternUnits::UserSpaceOnUse)?;
    /// pat.set_width(20.0)?;
    /// pat.set_height(20.0)?;
    /// pat.circle(Point::new(10.0, 10.0), 6.0)?.set_fill("white")?;
    ///
    /// let rect = svg.rect(Point::origin(), Size::new(300.0, 200.0))?;
    /// rect.set_fill_pattern("dots")?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn pattern(&self, id: &str) -> Result<SvgPattern, Error> {
        validate_pattern_id(id)?;
        let el = super::create_svg_element::<SvgElement>(&self.document, "pattern", "SvgElement")?;
        el.set_attribute("id", id).map_err(dom_err)?;
        self.element.append_child(&el).map_err(dom_err)?;
        Ok(SvgPattern::new(id.to_owned(), el, self.document.clone()))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Builds a `<pattern>` and all its tile shapes in one shot, appending to `<defs>` only after the closure
    /// succeeds.
    ///
    /// The closure receives a reference to the new [`SvgPattern`]. All shapes added inside the closure are appended to
    /// a detached element.
    ///
    /// If the closure returns `Ok(())`, the pattern is appended to `<defs>` and the handle is returned.
    /// If the closure returns `Err`, the element is dropped without being attached to `<defs>`.
    ///
    /// This is the preferred way to build a pattern when all its tile content is known upfront.
    /// For dynamically adding shapes over time, use [`pattern`](Self::pattern) instead.
    ///
    /// # Errors
    ///
    /// - Any error returned by `build`.
    /// - [`Error::InvalidPatternId`] — `id` failed validation.
    /// - [`Error::Dom`] — the browser refused to create or append the element.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::{pattern::PatternUnits, utils::{Point, Size}}};
    ///
    /// let svg = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    ///
    /// defs.build_pattern("checker", |p| {
    ///     p.set_pattern_units(PatternUnits::UserSpaceOnUse)?;
    ///     p.set_width(20.0)?;
    ///     p.set_height(20.0)?;
    ///     p.rect(Point::new(0.0, 0.0), Size::new(20.0, 20.0))?.set_fill("teal")?;
    ///     p.rect(Point::new(0.0, 0.0), Size::new(10.0, 10.0))?.set_fill("white")?;
    ///     p.rect(Point::new(10.0, 10.0), Size::new(10.0, 10.0))?.set_fill("white")?;
    ///     Ok(())
    /// })?;
    ///
    /// let rect = svg.rect(Point::origin(), Size::new(300.0, 200.0))?;
    /// rect.set_fill_pattern("checker")?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn build_pattern<F>(&self, id: &str, build: F) -> Result<SvgPattern, Error>
    where
        F: FnOnce(&SvgPattern) -> Result<(), Error>,
    {
        validate_pattern_id(id)?;
        let el = super::create_svg_element::<SvgElement>(&self.document, "pattern", "SvgElement")?;
        el.set_attribute("id", id).map_err(dom_err)?;
        let pat = SvgPattern::new(id.to_owned(), el, self.document.clone());
        build(&pat)?;
        self.element.append_child(pat.as_element()).map_err(dom_err)?;
        Ok(pat)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets any attribute on the `<defs>` element by name and string value.
    ///
    /// This is the generic escape hatch for attributes not covered by a named setter (e.g. `class`, `style`).
    /// Name and value are written verbatim; do not pass untrusted input.
    pub fn set_attr(&self, name: &str, value: &str) -> Result<(), Error> {
        self.element.set_attribute(name, value).map_err(dom_err)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets several attributes in one call.
    ///
    /// Equivalent to calling [`set_attr`](Self::set_attr) for each pair.
    /// Returns the first error encountered; attributes written before the error are left in place.
    pub fn set_attrs<I, K, V>(&self, attrs: I) -> Result<(), Error>
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
    /// Formats `value` through the element's internal scratch buffer and writes it as `name`.
    ///
    /// Uses the same `SvgAttrs` scratch buffer that the shape factories use internally, so no extra allocation is made.
    pub fn set_attr_display<T: std::fmt::Display>(&self, name: &str, value: T) -> Result<(), Error> {
        self.attrs.borrow_mut().display_element(&self.element, name, value)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<rect>` child inside `<defs>`.
    pub fn rect(&self, top_left: Point, size: Size) -> Result<SvgNode, Error> {
        self.create_rect(top_left, size)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<circle>` child inside `<defs>`.
    pub fn circle(&self, centre: Point, radius: f64) -> Result<SvgNode, Error> {
        self.create_circle(centre, radius)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates an `<ellipse>` child inside `<defs>`.
    pub fn ellipse(&self, centre: Point, radii: Size) -> Result<SvgNode, Error> {
        self.create_ellipse(centre, radii)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<line>` child inside `<defs>`.
    pub fn line(&self, start: Point, end: Point) -> Result<SvgNode, Error> {
        self.create_line(start, end)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<path>` child inside `<defs>`.
    pub fn path(&self, d: &str) -> Result<SvgNode, Error> {
        self.create_path(d)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<path>` child inside `<defs>` from a sequence of typed [`PathDef`] segments.
    ///
    /// The type-safe alternative to [`path`](Self::path); see [`SvgRoot::path_from_defs`](crate::SvgRoot::path_from_defs)
    /// for the full rationale.
    pub fn path_from_defs(&self, defs: &[PathDef]) -> Result<SvgNode, Error> {
        self.create_path_from_defs(defs)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<polyline>` child inside `<defs>`.
    pub fn polyline(&self, points: &[Point]) -> Result<SvgNode, Error> {
        self.create_polyline(points)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<polygon>` child inside `<defs>`.
    pub fn polygon(&self, points: &[Point]) -> Result<SvgNode, Error> {
        self.create_polygon(points)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<text>` child inside `<defs>`.
    pub fn text(&self, anchored_at: Point, content: &str) -> Result<SvgNode, Error> {
        self.create_text(anchored_at, content)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<g>` group child inside `<defs>`.
    pub fn group(&self) -> Result<SvgNode, Error> {
        self.create_group()
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<linearGradient>` child element with the given `id`, appends it to `<defs>` immediately and returns
    /// its handle.
    ///
    /// The `id` is used to reference the gradient from shapes via
    /// [`set_fill_linear_gradient`](crate::SvgNode::set_fill_linear_gradient) and its stroke sibling.
    ///
    /// Each stop added to the returned [`SvgLinearGradient`] is appended to the live gradient element one at a time.
    /// Use this when you need to add stops dynamically after initial construction.
    ///
    /// Prefer [`build_linear_gradient`](Self::build_linear_gradient) when all stops are known upfront: that variant
    /// keeps the `<linearGradient>` detached until the closure succeeds, so a mid-build error will not leave a partial
    /// gradient in `<defs>`.
    ///
    /// # Errors
    ///
    /// - [`Error::InvalidGradientId`] — `id` failed validation.
    /// - [`Error::Dom`] — the browser refused to create or append the element.
    pub fn linear_gradient(&self, id: &str) -> Result<SvgLinearGradient, Error> {
        validate_gradient_id(id)?;
        let el = super::create_svg_element::<SvgElement>(&self.document, "linearGradient", "SvgElement")?;
        el.set_attribute("id", id).map_err(dom_err)?;
        self.element.append_child(&el).map_err(dom_err)?;
        Ok(SvgLinearGradient::new(id, el, self.document.clone()))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Builds a `<linearGradient>` and all its stops in one go, appending it to `<defs>` only after the closure
    /// succeeds.
    ///
    /// The closure receives a reference to the new [`SvgLinearGradient`].
    /// All stops added inside the closure are appended to a detached gradient element.
    ///
    /// If the closure returns `Ok(())`, the gradient is appended to `<defs>` and the handle is returned.
    /// If the closure returns `Err`, the gradient element is dropped without being attached to `<defs>`.
    ///
    /// This is the preferred way to build a gradient when all stops are known upfront.
    /// For dynamically adding stops over time, use [`linear_gradient`](Self::linear_gradient) instead.
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
    /// let grad = defs.build_linear_gradient("sky", |g| {
    ///     g.add_stop(0.0, "deepskyblue")?;
    ///     g.add_stop(1.0, "white")?;
    ///     Ok(())
    /// })?;
    ///
    /// let rect = svg.rect(Point::origin(), Size::new(300.0, 150.0))?;
    /// rect.set_fill_linear_gradient(&grad)?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn build_linear_gradient<F>(&self, id: &str, build: F) -> Result<SvgLinearGradient, Error>
    where
        F: FnOnce(&SvgLinearGradient) -> Result<(), Error>,
    {
        validate_gradient_id(id)?;
        let el = super::create_svg_element::<SvgElement>(&self.document, "linearGradient", "SvgElement")?;
        el.set_attribute("id", id).map_err(dom_err)?;
        let grad = SvgLinearGradient::new(id, el, self.document.clone());
        build(&grad)?;
        self.element.append_child(grad.as_element()).map_err(dom_err)?;
        Ok(grad)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
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
        let el = super::create_svg_element::<SvgElement>(&self.document, "radialGradient", "SvgElement")?;
        el.set_attribute("id", id).map_err(dom_err)?;
        self.element.append_child(&el).map_err(dom_err)?;
        Ok(SvgRadialGradient::new(id, el, self.document.clone()))
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
        let el = super::create_svg_element::<SvgElement>(&self.document, "radialGradient", "SvgElement")?;
        el.set_attribute("id", id).map_err(dom_err)?;
        let grad = SvgRadialGradient::new(id, el, self.document.clone());
        build(&grad)?;
        self.element.append_child(grad.as_element()).map_err(dom_err)?;
        Ok(grad)
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl SvgFactory for SvgDefs {
    fn document(&self) -> &Document {
        &self.document
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    fn attrs(&self) -> &RefCell<SvgAttrs> {
        &self.attrs
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    fn append_node(&self, node: &SvgNode) -> Result<(), Error> {
        self.element.append_child(node.as_element()).map(|_| ()).map_err(dom_err)
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl SvgRoot {
    /// Creates a `<defs>` element, appends it to the root `<svg>` immediately, and returns its handle.
    ///
    /// Each marker or shape added through the returned [`SvgDefs`] is appended to the live `<defs>` element one at a
    /// time.
    /// Use this when you need to extend `<defs>` dynamically — for example, adding markers in response to user
    /// actions after the initial build.
    ///
    /// Prefer [`build_defs`](Self::build_defs) when all the contents are known upfront: that variant holds the
    /// `<defs>` element detached until the closure succeeds, so a mid-build error leaves no partial element in the
    /// live tree.
    /// With this method, if a subsequent call fails after `defs()` returns, the empty `<defs>` remains in the DOM.
    ///
    /// # Errors
    ///
    /// - [`Error::Dom`] — the browser refused to create or append the element.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::SvgRoot;
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    /// let marker = defs.marker("dot")?;
    /// marker.circle(svg_dom::root::utils::Point::origin(), 4.0)?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn defs(&self) -> Result<SvgDefs, Error> {
        let element = super::create_svg_element::<SvgElement>(&self.document, "defs", "SvgElement")?;
        self.root.append_child(&element).map_err(dom_err)?;
        Ok(SvgDefs::new(element, self.document.clone()))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Builds a `<defs>` subtree and all its contents in one shot, appending to the root `<svg>` only after the
    /// closure succeeds.
    ///
    /// The closure receives a reference to the new [`SvgDefs`].
    /// Markers and shapes added inside the closure are appended to a detached `<defs>` element.
    /// If the closure returns `Ok(())`, `<defs>` is appended to the root `<svg>` and the handle is returned.
    /// If the closure returns `Err`, the element is dropped without being attached to the live tree.
    ///
    /// This is the preferred way to populate `<defs>` when you know all its contents up-front.
    /// For dynamically extending `<defs>` after initial construction, use [`defs`](Self::defs) instead.
    ///
    /// # Errors
    ///
    /// - Any error returned by `build`.
    /// - [`Error::Dom`] — the browser refused to create or append the element.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::utils::{Point, Size}};
    ///
    /// let svg = SvgRoot::attach("diagram")?;
    /// let defs = svg.build_defs(|defs| {
    ///     defs.build_marker("arrow", |m| {
    ///         m.set_ref_x(10.0)?;
    ///         m.set_ref_y(3.5)?;
    ///         m.set_marker_width(10.0)?;
    ///         m.set_marker_height(7.0)?;
    ///         m.set_orient("auto")?;
    ///         m.polygon(&[Point::new(0.0, 0.0), Point::new(10.0, 3.5), Point::new(0.0, 7.0)])?;
    ///         Ok(())
    ///     })?;
    ///     Ok(())
    /// })?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn build_defs<F>(&self, build: F) -> Result<SvgDefs, Error>
    where
        F: FnOnce(&SvgDefs) -> Result<(), Error>,
    {
        let element = super::create_svg_element::<SvgElement>(&self.document, "defs", "SvgElement")?;
        let defs = SvgDefs::new(element, self.document.clone());
        build(&defs)?;
        self.root.append_child(defs.as_element()).map_err(dom_err)?;
        Ok(defs)
    }
}
