use crate::{
    Error, SvgNode, dom_err,
    root::{
        attrs::SvgAttrs,
        factory::SvgFactory,
        utils::{Point, Size},
    },
};
use std::cell::RefCell;
use web_sys::{Document, SvgElement};

use super::{marker::SvgMarker, svg_root::SvgRoot};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Rejects marker ids that would produce broken or ambiguous `url(#…)` references.
///
/// A valid id must match `[A-Za-z_][A-Za-z0-9_-]*`: it must begin with an ASCII letter or underscore,
/// followed by zero or more ASCII letters, digits, underscores, or hyphens.
/// This conservative allow-list ensures that any accepted id can be safely embedded in the generated
/// `url(#id)` CSS/SVG paint-server reference without quoting, escaping, or browser-specific interpretation.
pub(crate) fn validate_marker_id(id: &str) -> Result<(), Error> {
    let mut chars = id.chars();
    let valid = match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {
            chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        },
        _ => false,
    };
    if valid { Ok(()) } else { Err(Error::InvalidMarkerId(id.to_owned())) }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// A `<defs>` element that holds reusable SVG assets such as markers, gradients, and clip-paths.
///
/// Elements created inside `<defs>` are not rendered directly; they are referenced by other elements via `id`.
/// All the usual shape factory methods are available, but their primary purpose here is to build the inner content of
/// a [`SvgMarker`] or (in future) a gradient or clip-path.
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
/// marker.polygon_raw("0 0, 10 3.5, 0 7")?;
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
    /// Each shape added to the returned [`SvgMarker`] is appended to the live marker element one at a time. Prefer
    /// [`build_marker`](Self::build_marker) when building the marker contents in one shot, because that variant only
    /// appends the `<marker>` to `<defs>` once, after all child shapes have been added.
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
    ///     m.polygon_raw("0 0, 10 3.5, 0 7")?;
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
    /// Prefer [`build_defs`](Self::build_defs) when building the entire `<defs>` subtree in one shot, because that
    /// variant only appends `<defs>` to the `<svg>` root once, after all children have been added.
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
    ///         m.polygon_raw("0 0, 10 3.5, 0 7")?;
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
