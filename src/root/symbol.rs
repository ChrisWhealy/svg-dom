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
/// A `<symbol>` element that defines a reusable, scaled viewport for stamping via `<use>`.
///
/// Unlike a plain `<g>` in `<defs>`, a `<symbol>` can carry its own `viewBox` and `preserveAspectRatio`. The browser
/// scales the symbol's content to fit the `<use>` element's `width` and `height`, exactly as it would an embedded
/// `<svg>` — so the same definition renders correctly at any size with no extra work.
///
/// Obtain one from [`SvgDefs::symbol`](crate::SvgDefs::symbol) or
/// [`SvgDefs::build_symbol`](crate::SvgDefs::build_symbol), then stamp copies with
/// [`SvgRoot::use_node`](crate::SvgRoot::use_node).
///
/// # Viewports and scaling
///
/// Call [`set_view_box`](Self::set_view_box) to declare the symbol's internal coordinate system. Each `<use>` instance
/// then maps that internal space to its own `width` and `height`, scaled according to
/// [`set_preserve_aspect_ratio`](Self::set_preserve_aspect_ratio) (default: `xMidYMid meet`).
///
/// If no `viewBox` is set, the symbol has no intrinsic size, so scaling will not occur and the content is positioned in
/// the same coordinate space as the referencing `<use>` element.
///
/// # Example
///
/// ```rust,no_run
/// use svg_dom::{SvgRoot, root::utils::{Point, Size}};
///
/// let svg  = SvgRoot::attach("diagram")?;
/// let defs = svg.defs()?;
///
/// // Define a badge icon with its own 40×40 viewport.
/// defs.build_symbol("badge", |s| {
///     s.set_view_box(0.0, 0.0, 40.0, 40.0)?;
///     s.circle(Point::new(20.0, 20.0), 18.0)?.set_fill("steelblue")?;
///     Ok(())
/// })?;
///
/// // Stamp the same definition at two different sizes — the viewBox scales automatically.
/// svg.use_node("#badge", Point::new(10.0, 10.0))?.set_attr("width", "40")?;
/// svg.use_node("#badge", Point::new(60.0, 10.0))?.set_attr("width", "80")?;
/// Ok::<(), svg_dom::Error>(())
/// ```
pub struct SvgSymbol {
    /// The `id` set at construction time; cached to avoid a round-trip to the DOM for [`id`](Self::id).
    id: String,
    element: SvgElement,
    document: Document,
    attrs: RefCell<SvgAttrs>,
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl SvgSymbol {
    pub(crate) fn new(id: String, element: SvgElement, document: Document) -> Self {
        Self {
            id,
            element,
            document,
            attrs: RefCell::new(SvgAttrs::new()),
        }
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Returns the cached `id` of this symbol.
    ///
    /// Pass this (prefixed with `#`) to [`SvgRoot::use_node`](crate::SvgRoot::use_node) to stamp a copy.
    ///
    /// # Caveat
    ///
    /// The returned value is cached in the `SvgSymbol` struct at construction time and kept in sync by
    /// [`set_id`](Self::set_id).
    /// [`set_attr`](Self::set_attr) and [`set_attr_display`](Self::set_attr_display) reject `"id"` so they cannot
    /// desynchronise the cache through the normal API.
    /// Always use `set_id` to rename a symbol after construction.
    pub fn id(&self) -> &str {
        &self.id
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Renames the symbol by updating both the DOM `id` attribute and the cached value returned by [`id`](Self::id).
    ///
    /// **Note:** renaming a symbol does not update any `href` attributes already written to referencing `<use>`
    /// elements — those store a snapshot of the id at the time the reference was applied.
    ///
    /// Either rename before stamping copies, or call [`SvgNode::set_href`](crate::SvgNode::set_href) on each `<use>`
    /// element after renaming.
    ///
    /// # Errors
    ///
    /// - [`Error::InvalidSymbolId`] — the new id failed validation.
    /// - [`Error::Dom`] — the browser refused to write the `id` attribute.
    pub fn set_id(&mut self, id: &str) -> Result<(), Error> {
        super::defs::validate_symbol_id(id)?;
        self.element.set_attribute("id", id).map_err(dom_err)?;
        self.id = id.to_owned();
        Ok(())
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Returns a reference to the underlying `web-sys` `SvgElement`.
    ///
    /// Avoid writing the `id` attribute through this handle; use [`set_id`](Self::set_id) instead so the
    /// cached value stays in sync.
    pub fn as_element(&self) -> &SvgElement {
        &self.element
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `viewBox` attribute, establishing the symbol's internal coordinate system.
    ///
    /// When a `<use>` element specifies a `width` and `height`, the browser maps the symbol's internal
    /// `(x, y, width, height)` region onto that viewport, scaling the content according to `preserveAspectRatio`
    /// (see [`set_preserve_aspect_ratio`](Self::set_preserve_aspect_ratio)).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// // viewBox="0 0 40 40" declares a 40×40 internal space.
    /// // A <use> with width="80" then renders it at 2× scale.
    /// # use svg_dom::{SvgRoot, root::utils::{Point, Size}};
    /// # let svg = SvgRoot::attach("d")?;
    /// # let defs = svg.defs()?;
    /// # let sym = defs.symbol("icon")?;
    /// sym.set_view_box(0.0, 0.0, 40.0, 40.0)?;
    /// # Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn set_view_box(&self, x: f64, y: f64, width: f64, height: f64) -> Result<(), Error> {
        self.attrs
            .borrow_mut()
            .display_element(&self.element, "viewBox", format_args!("{x} {y} {width} {height}"))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `preserveAspectRatio` attribute, controlling alignment and clipping of the scaled viewport.
    ///
    /// The default value (`"xMidYMid meet"`) centres the symbol content and scales it to fit inside the
    /// `<use>` element's box without clipping.
    ///
    /// Use `"none"` to stretch the content to exactly fill the box.
    ///
    /// See the [MDN reference](https://developer.mozilla.org/en-US/docs/Web/SVG/Attribute/preserveAspectRatio) for the
    /// full list of alignment keywords.
    pub fn set_preserve_aspect_ratio(&self, value: &str) -> Result<(), Error> {
        self.element.set_attribute("preserveAspectRatio", value).map_err(dom_err)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets any attribute on the `<symbol>` element by name and string value.
    ///
    /// This is the generic escape hatch for attributes not covered by the named setters above (e.g. `class`, `style`,
    /// `overflow`).  Name and value are written verbatim; so do not pass untrusted input!
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
    ///
    /// If an error occurs, any attributes written before the error are left in place, the first detected error is
    /// returned and no subsequent attributes are processed.
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
    ///
    /// Passing `"id"` (matched case-insensitively) returns [`Error::ReservedAttribute`];
    /// use [`set_id`](Self::set_id) instead.
    pub fn set_attr_display<T: std::fmt::Display>(&self, name: &str, value: T) -> Result<(), Error> {
        if name.eq_ignore_ascii_case("id") {
            return Err(Error::ReservedAttribute("id"));
        }
        self.attrs.borrow_mut().display_element(&self.element, name, value)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<rect>` shape inside this `<symbol>`.
    pub fn rect(&self, top_left: Point, size: Size) -> Result<SvgNode, Error> {
        self.create_rect(top_left, size)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<circle>` shape inside this `<symbol>`.
    pub fn circle(&self, centre: Point, radius: f64) -> Result<SvgNode, Error> {
        self.create_circle(centre, radius)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates an `<ellipse>` shape inside this `<symbol>`.
    pub fn ellipse(&self, centre: Point, radii: Size) -> Result<SvgNode, Error> {
        self.create_ellipse(centre, radii)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<line>` shape inside this `<symbol>`.
    pub fn line(&self, start: Point, end: Point) -> Result<SvgNode, Error> {
        self.create_line(start, end)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<path>` shape inside this `<symbol>`.
    pub fn path(&self, d: &str) -> Result<SvgNode, Error> {
        self.create_path(d)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<path>` shape inside this `<symbol>` from a sequence of typed [`PathDef`]
    /// segments.
    ///
    /// The type-safe alternative to [`path`](Self::path); see [`SvgRoot::path_from_defs`](crate::SvgRoot::path_from_defs)
    /// for the full rationale.
    pub fn path_from_defs(&self, defs: &[PathDef]) -> Result<SvgNode, Error> {
        self.create_path_from_defs(defs)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<polyline>` shape inside this `<symbol>`.
    pub fn polyline(&self, points: &[Point]) -> Result<SvgNode, Error> {
        self.create_polyline(points)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<polygon>` shape inside this `<symbol>`.
    pub fn polygon(&self, points: &[Point]) -> Result<SvgNode, Error> {
        self.create_polygon(points)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<text>` element inside this `<symbol>`.
    pub fn text(&self, anchored_at: Point, content: &str) -> Result<SvgNode, Error> {
        self.create_text(anchored_at, content)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<g>` group inside this `<symbol>`.
    pub fn group(&self) -> Result<SvgNode, Error> {
        self.create_group()
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl SvgFactory for SvgSymbol {
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
