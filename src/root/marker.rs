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

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Controls the coordinate space in which `markerWidth` and `markerHeight` are expressed.
///
/// Passed to [`SvgMarker::set_units`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkerUnits {
    /// Marker dimensions are multiples of the element's `stroke-width` (the SVG default).
    StrokeWidth,
    /// Marker dimensions are in the same user-coordinate space as the element the marker is applied to.
    UserSpaceOnUse,
}

impl MarkerUnits {
    fn as_str(self) -> &'static str {
        match self {
            Self::StrokeWidth => "strokeWidth",
            Self::UserSpaceOnUse => "userSpaceOnUse",
        }
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// A `<marker>` element that can be referenced by `marker-start`, `marker-mid`, or `marker-end` on lines and paths.
///
/// A marker defines a reusable graphic — typically an arrowhead or dot — that the browser renders at a specified
/// position along the stroked path.
/// The shapes inside it are added with the same factory methods available on [`SvgRoot`](crate::SvgRoot).
///
/// Obtain one from [`SvgDefs::marker`](crate::SvgDefs::marker); attach it to any stroked element (`<line>`,
/// `<path>`, `<polyline>`, `<polygon>`) with [`SvgNode::set_marker_end`](crate::SvgNode::set_marker_end) and its
/// `_start` / `_mid` siblings.
///
/// # Example
///
/// ```rust,no_run
/// use svg_dom::{SvgRoot, root::utils::{Point, Size}};
///
/// let svg  = SvgRoot::attach("diagram")?;
/// let defs = svg.defs()?;
///
/// let marker = defs.marker("arrow")?;
/// marker.set_ref_x(10.0)?;
/// marker.set_ref_y(3.5)?;
/// marker.set_marker_width(10.0)?;
/// marker.set_marker_height(7.0)?;
/// marker.set_orient("auto")?;
/// marker.polygon(&[Point::new(0.0, 0.0), Point::new(10.0, 3.5), Point::new(0.0, 7.0)])?;
///
/// let line = svg.line(Point::new(20.0, 50.0), Point::new(180.0, 50.0))?;
/// line.set_stroke("black")?;
/// line.set_marker_end("arrow")?;
/// Ok::<(), svg_dom::Error>(())
/// ```
pub struct SvgMarker {
    /// The `id` set at construction time; cached to avoid a round-trip to the DOM for [`id`](Self::id).
    id: String,
    element: SvgElement,
    document: Document,
    attrs: RefCell<SvgAttrs>,
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl SvgMarker {
    pub(crate) fn new(id: String, element: SvgElement, document: Document) -> Self {
        Self {
            id,
            element,
            document,
            attrs: RefCell::new(SvgAttrs::new()),
        }
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Returns the cached `id` of this marker.
    ///
    /// Pass this to [`SvgNode::set_marker_end_ref`](crate::SvgNode::set_marker_end_ref) and its siblings, or use
    /// those methods directly with the marker handle to avoid touching the id at all.
    ///
    /// # Caveat
    ///
    /// The returned value is cached in the `SvgMarker` struct at construction time and kept in sync by
    /// [`set_id`](Self::set_id).
    /// [`set_attr`](Self::set_attr) and [`set_attr_display`](Self::set_attr_display) reject `"id"` so they cannot
    /// desynchronise the cache through the normal API.
    /// The only remaining escape hatch is writing through [`as_element`](Self::as_element) directly, which bypasses
    /// all crate-level checks.
    /// Always use `set_id` to rename a marker after construction.
    pub fn id(&self) -> &str {
        &self.id
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Renames the marker by updating both the DOM `id` attribute and the cached value returned by [`id`](Self::id).
    ///
    /// This method takes `&mut self` because it mutates Rust-owned state (the cached id string), unlike the other
    /// attribute setters that write only to the DOM.
    ///
    /// The new `id` is subject to the same validation rules as the id supplied at construction time: it must
    /// match `[A-Za-z_][A-Za-z0-9_-]*` — a letter or underscore followed by letters, digits, underscores, or hyphens.
    ///
    /// **Note:** renaming a marker does not update any `marker-start`, `marker-mid`, or `marker-end` attributes
    /// already written to referencing elements — those store a snapshot of the id at the time the reference was
    /// applied.
    /// Either rename the marker before applying references, or reapply the reference after renaming.
    ///
    /// # Errors
    ///
    /// - [`Error::InvalidMarkerId`] — the new id failed validation.
    /// - [`Error::Dom`] — the browser refused to write the `id` attribute.
    pub fn set_id(&mut self, id: &str) -> Result<(), Error> {
        super::defs::validate_marker_id(id)?;
        self.element.set_attribute("id", id).map_err(dom_err)?;
        self.id.clear();
        self.id.push_str(id);
        Ok(())
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Returns a reference to the underlying `web-sys` `SvgElement`.
    ///
    /// This provides a direct escape hatch to the DOM.
    /// Avoid writing the `id` attribute through this handle; use [`set_id`](Self::set_id) instead so the cached value
    /// stays in sync.
    pub fn as_element(&self) -> &SvgElement {
        &self.element
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `refX` attribute — the x-coordinate within the marker that aligns with the path endpoint.
    pub fn set_ref_x(&self, x: f64) -> Result<(), Error> {
        self.attrs.borrow_mut().display_element(&self.element, "refX", x)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `refY` attribute — the y-coordinate within the marker that aligns with the path endpoint.
    pub fn set_ref_y(&self, y: f64) -> Result<(), Error> {
        self.attrs.borrow_mut().display_element(&self.element, "refY", y)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `markerWidth` attribute — the width of the marker's viewport.
    pub fn set_marker_width(&self, w: f64) -> Result<(), Error> {
        self.attrs.borrow_mut().display_element(&self.element, "markerWidth", w)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `markerHeight` attribute — the height of the marker's viewport.
    pub fn set_marker_height(&self, h: f64) -> Result<(), Error> {
        self.attrs.borrow_mut().display_element(&self.element, "markerHeight", h)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `orient` attribute.
    ///
    /// Common values: `"auto"` (rotates to match the path tangent), `"auto-start-reverse"` (same but flips at
    /// `marker-start`), or a fixed angle such as `"45deg"`.
    pub fn set_orient(&self, orient: &str) -> Result<(), Error> {
        self.element.set_attribute("orient", orient).map_err(dom_err)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `markerUnits` attribute, controlling the coordinate space of `markerWidth`/`markerHeight`.
    pub fn set_units(&self, units: MarkerUnits) -> Result<(), Error> {
        self.element.set_attribute("markerUnits", units.as_str()).map_err(dom_err)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets any attribute on the `<marker>` element by name and string value.
    ///
    /// This is the generic escape hatch for attributes not covered by the named setters above (e.g. `viewBox`,
    /// `preserveAspectRatio`, `overflow`, `class`, `style`).
    /// Name and value are written verbatim; do not pass untrusted input.
    ///
    /// # Reserved attributes
    ///
    /// Passing `"id"` (matched case-insensitively) returns [`Error::ReservedAttribute`].
    /// Use [`set_id`](Self::set_id) instead so the cached id stays in sync with the DOM.
    pub fn set_attr(&self, name: &str, value: &str) -> Result<(), Error> {
        if name.eq_ignore_ascii_case("id") {
            return Err(Error::ReservedAttribute("id"));
        }
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
    /// Uses the same `SvgAttrs` scratch buffer that the named numeric setters (`set_ref_x`, `set_marker_width`, ...)
    /// use internally, so no extra allocation is made.
    /// Passing `"id"` (matched case-insensitively) returns [`Error::ReservedAttribute`];
    /// use [`set_id`](Self::set_id) instead.
    pub fn set_attr_display<T: std::fmt::Display>(&self, name: &str, value: T) -> Result<(), Error> {
        if name.eq_ignore_ascii_case("id") {
            return Err(Error::ReservedAttribute("id"));
        }
        self.attrs.borrow_mut().display_element(&self.element, name, value)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<rect>` child inside the marker.
    pub fn rect(&self, top_left: Point, size: Size) -> Result<SvgNode, Error> {
        self.create_rect(top_left, size)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<circle>` child inside the marker.
    pub fn circle(&self, centre: Point, radius: f64) -> Result<SvgNode, Error> {
        self.create_circle(centre, radius)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates an `<ellipse>` child inside the marker.
    pub fn ellipse(&self, centre: Point, radii: Size) -> Result<SvgNode, Error> {
        self.create_ellipse(centre, radii)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<line>` child inside the marker.
    pub fn line(&self, start: Point, end: Point) -> Result<SvgNode, Error> {
        self.create_line(start, end)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<path>` child inside the marker.
    pub fn path(&self, d: &str) -> Result<SvgNode, Error> {
        self.create_path(d)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<path>` child inside the marker from a sequence of typed [`PathDef`] segments.
    ///
    /// The type-safe alternative to [`path`](Self::path); see [`SvgRoot::path_from_defs`](crate::SvgRoot::path_from_defs)
    /// for the full rationale.
    pub fn path_from_defs(&self, defs: &[PathDef]) -> Result<SvgNode, Error> {
        self.create_path_from_defs(defs)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<polyline>` child inside the marker.
    pub fn polyline(&self, points: &[Point]) -> Result<SvgNode, Error> {
        self.create_polyline(points)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<polygon>` child inside the marker from a slice of [`Point`]s.
    pub fn polygon(&self, points: &[Point]) -> Result<SvgNode, Error> {
        self.create_polygon(points)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<text>` child inside the marker.
    pub fn text(&self, anchored_at: Point, content: &str) -> Result<SvgNode, Error> {
        self.create_text(anchored_at, content)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<g>` group child inside the marker.
    pub fn group(&self) -> Result<SvgNode, Error> {
        self.create_group()
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl SvgFactory for SvgMarker {
    fn document(&self) -> &Document {
        &self.document
    }

    fn attrs(&self) -> &RefCell<SvgAttrs> {
        &self.attrs
    }

    fn append_node(&self, node: &SvgNode) -> Result<(), Error> {
        self.element.append_child(node.as_element()).map(|_| ()).map_err(dom_err)
    }
}
