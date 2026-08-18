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

use super::svg_root::SvgRoot;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// The fixed prefix of a `url(#...)` reference.
///
/// Every type below that caches a complete reference string uses this value, rather than just its bare id
/// (`SvgMarker`, `SvgClipPath`, `SvgMask`, `SvgPattern`, `SvgFilter`, `GradientInner`).
/// This lets the `url(#id)` value be written to a `fill`/`stroke`/`clip-path`/`mask`/`marker-*`/`filter` attribute
/// without allocating a fresh `String` on every reference.
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
/// Rejects mask ids that would produce broken or ambiguous `url(#...)` references.
///
/// Applies the same allow-list as [`validate_marker_id`]: the id must match `[A-Za-z_][A-Za-z0-9_-]*`.
pub(crate) fn validate_mask_id(id: &str) -> Result<(), Error> {
    if is_valid_svg_id(id) {
        Ok(())
    } else {
        Err(Error::InvalidMaskId(id.to_owned()))
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
/// Rejects view ids outside the crate-imposed subset that would produce broken `#id` fragment references.
///
/// Applies the same allow-list as [`validate_marker_id`]: the id must match `[A-Za-z_][A-Za-z0-9_-]*`. This is
/// narrower than SVG/XML's own id grammar — it is a restriction this crate chooses, not a claim about what SVG
/// itself permits.
/// Unlike most of the other id-validated elements in this crate, a `<view>` is never wrapped in a `url(#id)` form.
/// It is only ever referenced as a plain `#id` fragment, the same way [`validate_symbol_id`] already is.
pub(crate) fn validate_view_id(id: &str) -> Result<(), Error> {
    if is_valid_svg_id(id) {
        Ok(())
    } else {
        Err(Error::InvalidViewId(id.to_owned()))
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
/// Elements created inside `<defs>` are not rendered directly.
/// Other elements reference them via an `id`.
/// All the usual shape factory methods are available for building inner content of markers, but the primary purpose of
/// `SvgDefs` is to serve as the container for named paint servers:
///
/// | Asset | Factory | Eager variant |
/// |---|---|---|
/// | [`SvgMarker`](crate::SvgMarker) | [`marker`](Self::marker) | [`build_marker`](Self::build_marker) |
/// | [`SvgLinearGradient`](crate::SvgLinearGradient) | [`linear_gradient`](Self::linear_gradient) | [`build_linear_gradient`](Self::build_linear_gradient) |
/// | [`SvgRadialGradient`](crate::SvgRadialGradient) | [`radial_gradient`](Self::radial_gradient) | [`build_radial_gradient`](Self::build_radial_gradient) |
/// | [`SvgClipPath`](crate::SvgClipPath) | [`clip_path`](Self::clip_path) | [`build_clip_path`](Self::build_clip_path) |
/// | [`SvgMask`](crate::SvgMask) | [`mask`](Self::mask) | [`build_mask`](Self::build_mask) |
/// | [`SvgSymbol`](crate::SvgSymbol) | [`symbol`](Self::symbol) | [`build_symbol`](Self::build_symbol) |
/// | [`SvgPattern`](crate::SvgPattern) | [`pattern`](Self::pattern) | [`build_pattern`](Self::build_pattern) |
/// | [`SvgFilter`](crate::SvgFilter) | [`filter`](Self::filter) | [`build_filter`](Self::build_filter) |
/// | [`SvgView`](crate::SvgView) | [`view`](Self::view) | [`build_view`](Self::build_view) |
///
/// Obtain one from [`SvgRoot::defs`].
///
/// Each asset type's `SvgDefs` constructor methods live alongside that type's own definition (e.g. `marker` or
/// `build_marker` are defined in `root::marker`, not here).
/// This avoids excessive growth in this file.
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
    /// Sets any attribute on the `<defs>` element by name and string value.
    ///
    /// This is the generic escape hatch for attributes not covered by a named setter (e.g. `class`, `style`).
    /// Name and value are written verbatim.
    /// Do not pass untrusted input.
    pub fn set_attr(&self, name: &str, value: &str) -> Result<(), Error> {
        self.element.set_attribute(name, value).map_err(dom_err)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets several attributes in one call.
    ///
    /// Equivalent to calling [`set_attr`](Self::set_attr) for each pair.
    /// Returns the first error encountered.
    /// Attributes written before the error are left in place.
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
    /// Creates a `<style>` child inside `<defs>` — the conventional placement for a document-wide stylesheet,
    /// though it applies the same wherever it sits in the tree.
    ///
    /// See [`SvgRoot::style`](crate::SvgRoot::style) for full documentation.
    pub fn style(&self, css: &str) -> Result<SvgNode, Error> {
        self.create_style(css)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<metadata>` child inside `<defs>`.  Whilst this is syntactically valid, it is not the conventional
    /// placement for `<metadata>`.  That said, its meaning is unaffected by where this element sits in the tree.
    ///
    /// Document-level metadata is more commonly placed directly beneath the root `<svg>`, as in SVG 2's own metadata
    /// example.
    /// The spec describes `<defs>` primarily as a container for objects defined for later reference, not for
    /// metadata.
    /// Use [`SvgRoot::metadata`](crate::SvgRoot::metadata) for that placement.
    ///
    /// See [`SvgRoot::metadata`](crate::SvgRoot::metadata) for full documentation.
    pub fn metadata(&self, content: &str) -> Result<SvgNode, Error> {
        self.create_metadata(content)
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
    /// Prefer [`build_defs`](Self::build_defs) when all the contents are known upfront.
    /// That variant holds the `<defs>` element detached until the closure succeeds.
    /// A mid-build error therefore leaves no partial element in the live tree.
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
