use crate::{
    Error, SvgNode, dom_err,
    root::{
        attrs::SvgAttrs,
        factory::SvgFactory,
        path::path_def::PathDef,
        utils::{Point, Size},
    },
};
use std::{cell::RefCell, fmt::Display};
use web_sys::{Document, SvgElement};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Controls which coordinate space the tile dimensions and position of a [`SvgPattern`] are expressed in.
///
/// Used for both the `patternUnits` and `patternContentUnits` attributes.
/// Passed to [`SvgPattern::set_pattern_units`] and [`SvgPattern::set_pattern_content_units`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternUnits {
    /// Tile dimensions and position are expressed in the same coordinate space as the element that references
    /// the pattern (SVG default for `patternUnits`).
    UserSpaceOnUse,
    /// Tile dimensions and position are expressed as fractions of the referencing element's bounding box.
    /// `(0, 0)` maps to the top-left corner and `(1, 1)` maps to the bottom-right corner.
    ObjectBoundingBox,
}

impl PatternUnits {
    fn as_str(self) -> &'static str {
        match self {
            Self::UserSpaceOnUse => "userSpaceOnUse",
            Self::ObjectBoundingBox => "objectBoundingBox",
        }
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// A `<pattern>` element that defines a tiled fill or stroke paint server.
///
/// `<pattern>` tiles its content across any element that references it via `fill="url(#id)"` or `stroke="url(#id)"`.
/// Like [`SvgClipPath`](crate::SvgClipPath), it acts as a shape container; unlike gradients, each tile is a full
/// rendered graphic rather than a colour interpolation.
///
/// Obtain one from [`SvgDefs::pattern`](crate::SvgDefs::pattern) or
/// [`SvgDefs::build_pattern`](crate::SvgDefs::build_pattern), and apply it to any element with
/// [`SvgNode::set_fill_pattern_ref`](crate::SvgNode::set_fill_pattern_ref) or
/// [`SvgNode::set_fill_pattern`](crate::SvgNode::set_fill_pattern).
///
/// # Coordinate systems
///
/// `<pattern>` has two independent coordinate-space controls:
///
/// - `patternUnits` (set via [`set_pattern_units`](Self::set_pattern_units)) — controls where the tile is positioned
///   and how large it is.
///   The default is `objectBoundingBox`, meaning `x`, `y`, `width`, and `height` are fractions of the referencing
///   element's bounding box.
///   Switch to [`PatternUnits::UserSpaceOnUse`] to express tile dimensions in pixel coordinates.
///
/// - `patternContentUnits` (set via [`set_pattern_content_units`](Self::set_pattern_content_units)) — controls the
///   coordinate space used by the shapes *inside* the tile.
///   The default is `userSpaceOnUse`, meaning the shapes inside the pattern use the same coordinate system as the
///   element that references the pattern.
///
/// # Example
///
/// ```rust,no_run
/// use svg_dom::{SvgRoot, root::{pattern::PatternUnits, utils::{Point, Size}}};
///
/// let svg  = SvgRoot::attach("diagram")?;
///
/// let pat = svg.build_defs(|d| {
///     d.build_pattern("dots", |p| {
///         p.set_pattern_units(PatternUnits::UserSpaceOnUse)?;
///         p.set_width(20.0)?;
///         p.set_height(20.0)?;
///         p.rect(Point::new(0.0, 0.0), Size::new(20.0, 20.0))?.set_fill("steelblue")?;
///         p.circle(Point::new(10.0, 10.0), 6.0)?.set_fill("white")?;
///         Ok(())
///     })?;
///     Ok(())
/// })?;
/// let _ = pat;
///
/// let rect = svg.rect(Point::origin(), Size::new(300.0, 200.0))?;
/// rect.set_fill_pattern("dots")?;
/// Ok::<(), svg_dom::Error>(())
/// ```
pub struct SvgPattern {
    /// The `id` set at construction time; cached to avoid a round-trip to the DOM for [`id`](Self::id).
    id: String,
    element: SvgElement,
    document: Document,
    attrs: RefCell<SvgAttrs>,
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl SvgPattern {
    pub(crate) fn new(id: String, element: SvgElement, document: Document) -> Self {
        Self {
            id,
            element,
            document,
            attrs: RefCell::new(SvgAttrs::new()),
        }
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Returns the cached `id` of this pattern.
    ///
    /// Pass this to [`SvgNode::set_fill_pattern`](crate::SvgNode::set_fill_pattern), or use
    /// [`SvgNode::set_fill_pattern_ref`](crate::SvgNode::set_fill_pattern_ref) with the handle to avoid
    /// touching the id.
    ///
    /// # Caveat
    ///
    /// The returned value is cached in the `SvgPattern` struct at construction time and kept in sync by
    /// [`set_id`](Self::set_id).
    /// [`set_attr`](Self::set_attr) and [`set_attr_display`](Self::set_attr_display) reject `"id"` so they
    /// cannot desynchronise the cache through the normal API.
    /// Always use `set_id` to rename a pattern after construction.
    pub fn id(&self) -> &str {
        &self.id
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Renames the pattern by updating both the DOM `id` attribute and the cached value returned by [`id`](Self::id).
    ///
    /// This method takes `&mut self` because it mutates Rust-owned state (the cached id string), unlike the other
    /// attribute setters that write only to the DOM.
    ///
    /// The new `id` is subject to the same validation rules as the id supplied at construction time: it must match
    /// `[A-Za-z_][A-Za-z0-9_-]*`.
    ///
    /// **Note:** renaming a pattern does not update any `fill` or `stroke` attributes already written to referencing
    /// elements.
    ///
    /// # Errors
    ///
    /// - [`Error::InvalidPatternId`] — the new id failed validation.
    /// - [`Error::Dom`] — the browser refused to write the `id` attribute.
    pub fn set_id(&mut self, id: &str) -> Result<(), Error> {
        super::defs::validate_pattern_id(id)?;
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
    /// Sets the horizontal offset of the tile origin.
    pub fn set_x(&self, v: f64) -> Result<(), Error> {
        self.attrs.borrow_mut().display_element(&self.element, "x", v)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the vertical offset of the tile origin.
    pub fn set_y(&self, v: f64) -> Result<(), Error> {
        self.attrs.borrow_mut().display_element(&self.element, "y", v)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the width of a single tile.
    pub fn set_width(&self, v: f64) -> Result<(), Error> {
        self.attrs.borrow_mut().display_element(&self.element, "width", v)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the height of a single tile.
    pub fn set_height(&self, v: f64) -> Result<(), Error> {
        self.attrs.borrow_mut().display_element(&self.element, "height", v)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `patternUnits` attribute, controlling the coordinate space for the tile's position and size.
    ///
    /// Use [`PatternUnits::UserSpaceOnUse`] to express `x`, `y`, `width`, and `height` in pixel coordinates.
    /// Use [`PatternUnits::ObjectBoundingBox`] (the SVG default) to express them as fractions of the referencing
    /// element's bounding box.
    pub fn set_pattern_units(&self, u: PatternUnits) -> Result<(), Error> {
        self.element.set_attribute("patternUnits", u.as_str()).map_err(dom_err)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `patternContentUnits` attribute, controlling the coordinate space used by the shapes inside the tile.
    ///
    /// The default is `userSpaceOnUse` — shapes inside the tile use the same coordinates as the referencing element.
    pub fn set_pattern_content_units(&self, u: PatternUnits) -> Result<(), Error> {
        self.element.set_attribute("patternContentUnits", u.as_str()).map_err(dom_err)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `patternTransform` attribute — an SVG transform applied to the pattern tile before tiling.
    ///
    /// Use this to rotate, scale, or skew the entire tile without changing individual shape coordinates.
    /// For example, `"rotate(45)"` produces diagonal stripes from horizontal-stripe content.
    pub fn set_pattern_transform(&self, t: &str) -> Result<(), Error> {
        self.element.set_attribute("patternTransform", t).map_err(dom_err)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `viewBox` attribute, establishing an internal coordinate system for the tile content.
    ///
    /// The four values are formatted as `"x y width height"`.
    /// When `viewBox` is present, the pattern's content is scaled to fit the tile dimensions.
    pub fn set_view_box(&self, x: f64, y: f64, w: f64, h: f64) -> Result<(), Error> {
        self.attrs
            .borrow_mut()
            .display_element(&self.element, "viewBox", format_args!("{x} {y} {w} {h}"))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets any attribute on the `<pattern>` element by name and string value.
    ///
    /// This is the generic escape hatch for attributes not covered by the named setters above (e.g. `class`, `style`).
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
    pub fn set_attr_display<T: Display>(&self, name: &str, value: T) -> Result<(), Error> {
        if name.eq_ignore_ascii_case("id") {
            return Err(Error::ReservedAttribute("id"));
        }
        self.attrs.borrow_mut().display_element(&self.element, name, value)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<rect>` tile shape inside this `<pattern>`.
    pub fn rect(&self, top_left: Point, size: Size) -> Result<SvgNode, Error> {
        self.create_rect(top_left, size)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<circle>` tile shape inside this `<pattern>`.
    pub fn circle(&self, centre: Point, radius: f64) -> Result<SvgNode, Error> {
        self.create_circle(centre, radius)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates an `<ellipse>` tile shape inside this `<pattern>`.
    pub fn ellipse(&self, centre: Point, radii: Size) -> Result<SvgNode, Error> {
        self.create_ellipse(centre, radii)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<line>` tile shape inside this `<pattern>`.
    pub fn line(&self, start: Point, end: Point) -> Result<SvgNode, Error> {
        self.create_line(start, end)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<path>` tile shape inside this `<pattern>`.
    pub fn path(&self, d: &str) -> Result<SvgNode, Error> {
        self.create_path(d)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<path>` tile shape inside this `<pattern>` from a sequence of typed [`PathDef`]
    /// segments.
    ///
    /// The type-safe alternative to [`path`](Self::path); see [`SvgRoot::path_from_defs`](crate::SvgRoot::path_from_defs)
    /// for the full rationale.
    pub fn path_from_defs(&self, defs: &[PathDef]) -> Result<SvgNode, Error> {
        self.create_path_from_defs(defs)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<polyline>` tile shape inside this `<pattern>`.
    pub fn polyline(&self, points: &[Point]) -> Result<SvgNode, Error> {
        self.create_polyline(points)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<polygon>` tile shape inside this `<pattern>`.
    pub fn polygon(&self, points: &[Point]) -> Result<SvgNode, Error> {
        self.create_polygon(points)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<text>` tile element inside this `<pattern>`.
    pub fn text(&self, anchored_at: Point, content: &str) -> Result<SvgNode, Error> {
        self.create_text(anchored_at, content)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<g>` group tile element inside this `<pattern>`.
    pub fn group(&self) -> Result<SvgNode, Error> {
        self.create_group()
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl SvgFactory for SvgPattern {
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
