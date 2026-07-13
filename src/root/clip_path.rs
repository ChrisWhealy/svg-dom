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

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Controls which coordinate space the shapes inside a [`SvgClipPath`] are expressed in.
///
/// Passed to [`SvgClipPath::set_units`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipPathUnits {
    /// Clip shapes use the same user-coordinate space as the element that references the `<clipPath>` (SVG default).
    UserSpaceOnUse,
    /// Clip shapes use a normalised coordinate space tied to the referencing element's bounding box:
    /// `(0, 0)` maps to the element's top-left corner and `(1, 1)` maps to its bottom-right corner.
    ObjectBoundingBox,
}

impl ClipPathUnits {
    fn as_str(self) -> &'static str {
        match self {
            Self::UserSpaceOnUse => "userSpaceOnUse",
            Self::ObjectBoundingBox => "objectBoundingBox",
        }
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// A `<clipPath>` element that restricts the rendered region of any element that references it.
///
/// `<clipPath>` defines a clipping region by combining the shapes placed inside it.
/// The browser paints only the parts of the referencing element that fall inside the union of those shapes;
/// everything outside is invisible.
///
/// Obtain one from [`SvgDefs::clip_path`](crate::SvgDefs::clip_path) or
/// [`SvgDefs::build_clip_path`](crate::SvgDefs::build_clip_path), and apply it to any element with
/// [`SvgNode::set_clip_path_ref`](crate::SvgNode::set_clip_path_ref) or
/// [`SvgNode::set_clip_path`](crate::SvgNode::set_clip_path).
///
/// # Coordinate spaces
///
/// By default (`clipPathUnits="userSpaceOnUse"`) the shapes inside the `<clipPath>` share the same coordinate system
/// as the element being clipped — they are positioned in SVG root coordinates.
/// Switch to [`ClipPathUnits::ObjectBoundingBox`] (via [`set_units`](Self::set_units)) when you want the clip shape
/// to scale automatically with the element's bounding box.
/// In that mode coordinates run from `0.0` (top-left) to `1.0` (bottom-right) of the element's bounding box.
///
/// # Example
///
/// ```rust,no_run
/// use svg_dom::{SvgRoot, root::utils::{Point, Size}};
///
/// let svg  = SvgRoot::attach("diagram")?;
/// let defs = svg.defs()?;
///
/// // Define a circular clipping region centred at (60, 60).
/// let clip = defs.build_clip_path("portrait", |c| {
///     c.circle(Point::new(60.0, 60.0), 55.0)?;
///     Ok(())
/// })?;
///
/// // Apply it to a rectangle — only the portion inside the circle is visible.
/// let bg = svg.rect(Point::origin(), Size::new(120.0, 120.0))?;
/// bg.set_fill("steelblue")?;
/// bg.set_clip_path_ref(&clip)?;
/// Ok::<(), svg_dom::Error>(())
/// ```
pub struct SvgClipPath {
    /// The `id` set at construction time; cached to avoid a round-trip to the DOM for [`id`](Self::id).
    id: String,
    element: SvgElement,
    document: Document,
    attrs: RefCell<SvgAttrs>,
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl SvgClipPath {
    pub(crate) fn new(id: String, element: SvgElement, document: Document) -> Self {
        Self {
            id,
            element,
            document,
            attrs: RefCell::new(SvgAttrs::new()),
        }
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Returns the cached `id` of this clip path.
    ///
    /// Pass this to [`SvgNode::set_clip_path`](crate::SvgNode::set_clip_path), or use
    /// [`SvgNode::set_clip_path_ref`](crate::SvgNode::set_clip_path_ref) with the handle to avoid touching the id.
    ///
    /// # Caveat
    ///
    /// The returned value is cached in the `SvgClipPath` struct at construction time and kept in sync by
    /// [`set_id`](Self::set_id).
    /// [`set_attr`](Self::set_attr) and [`set_attr_display`](Self::set_attr_display) reject `"id"` so they cannot
    /// desynchronise the cache through the normal API.
    /// The only remaining escape hatch is writing through [`as_element`](Self::as_element) directly, which bypasses
    /// all crate-level checks.
    /// Always use `set_id` to rename a clip path after construction.
    pub fn id(&self) -> &str {
        &self.id
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Renames the clip path by updating both the DOM `id` attribute and the cached value returned by [`id`](Self::id).
    ///
    /// This method takes `&mut self` because it mutates Rust-owned state (the cached id string), unlike the other
    /// attribute setters that write only to the DOM.
    ///
    /// The new `id` is subject to the same validation rules as the id supplied at construction time: it must match
    /// `[A-Za-z_][A-Za-z0-9_-]*` — a letter or underscore followed by letters, digits, underscores, or hyphens.
    ///
    /// **Note:** renaming a clip path does not update any `clip-path` attributes already written to referencing
    /// elements — those store a snapshot of the id at the time the reference was applied.
    /// Either rename before applying references, or reapply the reference after renaming.
    ///
    /// # Errors
    ///
    /// - [`Error::InvalidClipPathId`] — the new id failed validation.
    /// - [`Error::Dom`] — the browser refused to write the `id` attribute.
    pub fn set_id(&mut self, id: &str) -> Result<(), Error> {
        super::defs::validate_clip_path_id(id)?;
        self.element.set_attribute("id", id).map_err(dom_err)?;
        self.id = id.to_owned();
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
    /// Sets the `clipPathUnits` attribute, controlling the coordinate space of the clip shapes.
    ///
    /// The default is [`ClipPathUnits::UserSpaceOnUse`], meaning clip shapes are positioned in the same coordinate
    /// system as the element being clipped.
    /// Use [`ClipPathUnits::ObjectBoundingBox`] to express clip coordinates as fractions (0.0–1.0) of the referencing
    /// element's bounding box, so the clip scales automatically with the element.
    pub fn set_units(&self, units: ClipPathUnits) -> Result<(), Error> {
        self.element.set_attribute("clipPathUnits", units.as_str()).map_err(dom_err)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets any attribute on the `<clipPath>` element by name and string value.
    ///
    /// This is the generic escape hatch for attributes not covered by the named setters above (e.g. `class`, `style`,
    /// `transform`).
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
    /// Uses the same `SvgAttrs` scratch buffer that the shape factories use internally, so no extra allocation is made.
    /// Passing `"id"` (matched case-insensitively) returns [`Error::ReservedAttribute`];
    /// use [`set_id`](Self::set_id) instead.
    pub fn set_attr_display<T: std::fmt::Display>(&self, name: &str, value: T) -> Result<(), Error> {
        if name.eq_ignore_ascii_case("id") {
            return Err(Error::ReservedAttribute("id"));
        }
        self.attrs.borrow_mut().display_element(&self.element, name, value)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<rect>` clip shape inside this `<clipPath>`.
    pub fn rect(&self, top_left: Point, size: Size) -> Result<SvgNode, Error> {
        self.create_rect(top_left, size)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<circle>` clip shape inside this `<clipPath>`.
    pub fn circle(&self, centre: Point, radius: f64) -> Result<SvgNode, Error> {
        self.create_circle(centre, radius)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates an `<ellipse>` clip shape inside this `<clipPath>`.
    pub fn ellipse(&self, centre: Point, radii: Size) -> Result<SvgNode, Error> {
        self.create_ellipse(centre, radii)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<line>` clip shape inside this `<clipPath>`.
    ///
    /// A `<line>` has no area, so it clips nothing by default unless the referenced element has a non-zero
    /// `stroke-width` on the line itself — this combination is uncommon.
    /// Prefer area shapes (`<rect>`, `<circle>`, `<path>`, `<polygon>`) when defining clip regions.
    pub fn line(&self, start: Point, end: Point) -> Result<SvgNode, Error> {
        self.create_line(start, end)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<path>` clip shape inside this `<clipPath>`.
    pub fn path(&self, d: &str) -> Result<SvgNode, Error> {
        self.create_path(d)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<polyline>` clip shape inside this `<clipPath>`.
    ///
    /// An open polyline has no area by default; it only clips if its `fill-rule` produces a filled region.
    /// Prefer `<polygon>` when you want a closed, filled clip boundary.
    pub fn polyline(&self, points: &[Point]) -> Result<SvgNode, Error> {
        self.create_polyline(points)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<polygon>` clip shape inside this `<clipPath>`.
    pub fn polygon(&self, points: &[Point]) -> Result<SvgNode, Error> {
        self.create_polygon(points)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<text>` clip shape inside this `<clipPath>`.
    ///
    /// Text used as a clip shape reveals the referencing element through the glyph outlines.
    /// The resulting glyphs act as a stencil: only pixels inside the rendered glyph areas are painted.
    pub fn text(&self, anchored_at: Point, content: &str) -> Result<SvgNode, Error> {
        self.create_text(anchored_at, content)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<g>` group clip shape inside this `<clipPath>`.
    ///
    /// All shapes inside the group contribute to the clipping region, letting you combine several primitives.
    pub fn group(&self) -> Result<SvgNode, Error> {
        self.create_group()
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl SvgFactory for SvgClipPath {
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
