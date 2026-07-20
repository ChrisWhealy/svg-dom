use crate::{
    Error, SvgNode, dom_err,
    root::{
        attrs::SvgAttrs,
        defs::URL_PREFIX,
        factory::SvgFactory,
        path::path_def::PathDef,
        utils::{Point, Size},
    },
};
use std::cell::RefCell;
use web_sys::{Document, SvgElement};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Controls which coordinate space a [`SvgMask`]'s region (`x`/`y`/`width`/`height`) or content is expressed in.
///
/// Used for both the `maskUnits` and `maskContentUnits` attributes.
/// Passed to [`SvgMask::set_mask_units`] and [`SvgMask::set_mask_content_units`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaskUnits {
    /// Values are expressed in the same coordinate space as the element that references the mask.
    /// SVG default for `maskContentUnits`.
    UserSpaceOnUse,
    /// Values are expressed as fractions of the referencing element's bounding box — `(0, 0)` maps to the top-left
    /// corner and `(1, 1)` maps to the bottom-right corner.
    /// SVG default for `maskUnits`.
    ObjectBoundingBox,
}

impl MaskUnits {
    fn as_str(self) -> &'static str {
        match self {
            Self::UserSpaceOnUse => "userSpaceOnUse",
            Self::ObjectBoundingBox => "objectBoundingBox",
        }
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Selects which channel of a [`SvgMask`]'s rendered content determines opacity, via the `mask-type` attribute.
///
/// Passed to [`SvgMask::set_mask_type`].
///
/// # `mask-type` is a preference, not a guarantee
///
/// `mask-type` expresses this mask element's own preferred interpretation.
///
/// The element that *references* the mask can override it with its own `mask-mode` **CSS property** — unlike
/// `mask-type`, `mask-mode` is not an SVG presentation attribute, so it cannot be set as a plain XML attribute.
/// A literal `mask-mode="alpha"` attribute is not the specification-defined syntax, however tolerant some browsers may
/// happen to be about it.
///
/// As a result, there is no dedicated typed setter for it; instead, write it into the element's `style` attribute
/// e.g. `SvgNode::set_attr("style", "mask-mode: alpha")` — bearing in mind that this statement alone will overwrite the
/// whole `style` attribute, so merge in any other inline declarations the element already has or might need.
///
/// `mask-mode`'s default value, `match-source`, honours whatever `mask-type` says, so the behaviour documented here
/// is what callers get unless a referencing element opts out explicitly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaskType {
    /// The referencing element is revealed in proportion to the mask content's *luminance and alpha combined* — the
    /// SVG default. Opaque white reveals fully and opaque black hides fully; intermediate brightness or alpha
    /// produces partial visibility, and transparent content (whatever its colour) hides fully regardless of
    /// brightness. A colour flood or gradient makes this behave intuitively without any extra `opacity` bookkeeping,
    /// provided the content stays fully opaque.
    Luminance,
    /// The referencing element is revealed in proportion to the mask content's *alpha* channel alone, ignoring
    /// colour/luminance entirely — a solid white shape with `fill-opacity="0.5"` reveals exactly 50%, and so does a
    /// solid black shape with the same `fill-opacity`.
    Alpha,
}

impl MaskType {
    fn as_str(self) -> &'static str {
        match self {
            Self::Luminance => "luminance",
            Self::Alpha => "alpha",
        }
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// A `<mask>` element that reveals or hides parts of any element that references it, based on the luminance or alpha of
/// the mask's own rendered content.
///
/// Unlike [`SvgClipPath`](crate::SvgClipPath), which is a hard, binary (in/out) boundary defined purely by shape
/// geometry, `<mask>` supports gradual transparency: each pixel of the referencing element is scaled by a value
/// derived from the corresponding pixel of the mask's rendered content — luminance and alpha combined under the
/// default [`MaskType::Luminance`], or alpha alone under [`MaskType::Alpha`] (see [`MaskType`] for the difference).
/// Under the default, opaque white reveals fully, opaque black hides fully, and anything in between — including
/// gradients, and including partial *opacity* on an otherwise-bright shape — reveals partially.
///
/// Obtain one from [`SvgDefs::mask`](crate::SvgDefs::mask) or [`SvgDefs::build_mask`](crate::SvgDefs::build_mask),
/// and apply it to any element with [`SvgNode::set_mask_ref`](crate::SvgNode::set_mask_ref) or
/// [`SvgNode::set_mask`](crate::SvgNode::set_mask).
///
/// # The mask region defaults to `-10% -10% 120% 120%`
///
/// Unlike `<clipPath>`, `<mask>` has its own bounding region (`x`, `y`, `width`, `height`), and content painted
/// outside that region is clipped away before luminance/alpha is even evaluated.
/// The SVG default region — `-10%, -10%, 120%, 120%` of the referencing element's bounding box under the default
/// [`MaskUnits::ObjectBoundingBox`] — comfortably covers content that stays close to the referencing element's own
/// bounds, but a mask shape that extends further (a wide gradient sweep, a large soft-edged reveal) can be silently
/// clipped by this region.
/// Widen it explicitly with [`set_x`](Self::set_x)/[`set_y`](Self::set_y)/[`set_width`](Self::set_width)/
/// [`set_height`](Self::set_height) if the mask content is unexpectedly being cut off.
///
/// Keep the region only as large as required, though: as with a `<filter>` region, this is the maximum size of the
/// offscreen buffer the browser rasterises while evaluating the mask, so an unnecessarily large region may increase
/// rendering and memory cost, not just the risk of over-widening.
///
/// # Example
///
/// ```rust,no_run
/// use svg_dom::{SvgRoot, root::utils::{Point, Size}};
///
/// let svg  = SvgRoot::attach("diagram")?;
/// let defs = svg.defs()?;
///
/// // A soft-edged reveal: a white-to-black gradient fades the referencing element to transparent.
/// defs.build_linear_gradient("fade-gradient", |g| {
///     g.add_stop(0.0, "white")?;
///     g.add_stop(1.0, "black")?;
///     Ok(())
/// })?;
/// let fade = defs.build_mask("fade-right", |m| {
///     m.rect(Point::origin(), Size::new(120.0, 120.0))?.set_fill_gradient("fade-gradient")?;
///     Ok(())
/// })?;
///
/// let bg = svg.rect(Point::origin(), Size::new(120.0, 120.0))?;
/// bg.set_fill("steelblue")?;
/// bg.set_mask_ref(&fade)?;
/// Ok::<(), svg_dom::Error>(())
/// ```
pub struct SvgMask {
    /// The complete `url(#id)` reference, built once at construction and kept in sync by [`set_id`](Self::set_id).
    /// Caching the full reference (rather than the bare id) means [`SvgNode::set_mask_ref`](crate::SvgNode::set_mask_ref)
    /// can write it straight to the `mask` attribute with no per-call formatting allocation, however many elements the
    /// same mask is applied to.
    ///
    /// [`id`](Self::id) slices the bare id back out of this string rather than storing it separately.
    url_ref: String,
    element: SvgElement,
    document: Document,
    attrs: RefCell<SvgAttrs>,
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl SvgMask {
    pub(crate) fn new(id: &str, element: SvgElement, document: Document) -> Self {
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
    /// Returns the cached `id` of this mask.
    ///
    /// Pass this to [`SvgNode::set_mask`](crate::SvgNode::set_mask), or use
    /// [`SvgNode::set_mask_ref`](crate::SvgNode::set_mask_ref) with the handle to avoid touching the id.
    ///
    /// # ⚠️ Caveat ⚠️
    ///
    /// The returned value is sliced out of the cached `url(#id)` reference (see `url_ref`) built at construction time
    /// and kept in sync by [`set_id`](Self::set_id). The slice is exact because mask ids are restricted at validation
    /// time to the pattern `[A-Za-z_][A-Za-z0-9_-]*`, which is pure ASCII, so byte offsets from `URL_PREFIX`'s length
    /// and the string's end always land on the bare id exactly.
    ///
    /// [`set_attr`](Self::set_attr) and [`set_attr_display`](Self::set_attr_display) reject `"id"` so they cannot
    /// desynchronise the cache through the normal API.
    ///
    /// The only remaining escape hatch is writing through [`as_element`](Self::as_element) directly, which bypasses all
    /// crate-level checks.
    ///
    /// Always use `set_id` to rename a mask after construction.
    pub fn id(&self) -> &str {
        &self.url_ref[URL_PREFIX.len()..self.url_ref.len() - 1]
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Returns the cached `url(#id)` reference, ready to write directly to a `mask` attribute.
    ///
    /// Visibility need only be `pub(crate)` since [`SvgNode::set_mask_ref`](crate::SvgNode::set_mask_ref) is the only
    /// function that needs it; external callers use [`id`](Self::id) instead.
    pub(crate) fn url_ref(&self) -> &str {
        &self.url_ref
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Renames the mask by updating both the DOM `id` attribute and the cached value returned by [`id`](Self::id).
    ///
    /// This method takes `&mut self` because it mutates Rust-owned state (the cached reference string), unlike the
    /// other attribute setters that write only to the DOM.
    ///
    /// The new `id` is subject to the same validation rules as the id supplied at construction time: it must match the
    /// pattern `[A-Za-z_][A-Za-z0-9_-]*` — a letter or underscore followed by letters, digits, underscores, or hyphens.
    ///
    /// ⚠️ Caveat ⚠️
    ///
    /// Renaming a mask does not update any `mask` attributes already written to referencing elements — those store a
    /// snapshot of the reference at the time it was applied.
    ///
    /// Either rename before applying references, or reapply the reference after renaming.
    ///
    /// # Errors
    ///
    /// - [`Error::InvalidMaskId`] — the new id failed validation.
    /// - [`Error::Dom`] — the browser refused to write the `id` attribute.
    pub fn set_id(&mut self, id: &str) -> Result<(), Error> {
        super::defs::validate_mask_id(id)?;
        self.element.set_attribute("id", id).map_err(dom_err)?;
        self.url_ref.clear();
        self.url_ref.push_str(URL_PREFIX);
        self.url_ref.push_str(id);
        self.url_ref.push(')');
        Ok(())
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Returns a reference to the underlying `web-sys` `SvgElement`.
    ///
    /// This provides a direct escape hatch to the DOM.
    ///
    /// Avoid writing the `id` attribute through this handle; use [`set_id`](Self::set_id) instead so the cached value
    /// stays in sync.
    pub fn as_element(&self) -> &SvgElement {
        &self.element
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the horizontal offset of the mask region.
    ///
    /// Interpreted according to [`maskUnits`](Self::set_mask_units) — a fraction of the referencing element's bounding
    /// box under the SVG default ([`MaskUnits::ObjectBoundingBox`]), or a user-space coordinate under
    /// [`MaskUnits::UserSpaceOnUse`].
    ///
    /// See the [type-level docs](Self) for why this often needs widening beyond the SVG default.
    pub fn set_x(&self, v: f64) -> Result<(), Error> {
        self.attrs.borrow_mut().display_element(&self.element, "x", v)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the vertical offset of the mask region.
    ///
    /// See [`set_x`](Self::set_x) for the coordinate space this value is interpreted in.
    pub fn set_y(&self, v: f64) -> Result<(), Error> {
        self.attrs.borrow_mut().display_element(&self.element, "y", v)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the width of the mask region.
    ///
    /// See the [type-level docs](Self) for why this often needs widening beyond the SVG default — and, in the same
    /// breath, why it should not be widened further than the mask content actually needs, since the region bounds
    /// the offscreen buffer the browser rasterises while evaluating the mask.
    /// See [`set_x`](Self::set_x) for the coordinate space this value is interpreted in.
    pub fn set_width(&self, v: f64) -> Result<(), Error> {
        self.attrs.borrow_mut().display_element(&self.element, "width", v)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the height of the mask region.
    ///
    /// See [`set_width`](Self::set_width) for why this often needs widening beyond the SVG default, and
    /// [`set_x`](Self::set_x) for the coordinate space this value is interpreted in.
    pub fn set_height(&self, v: f64) -> Result<(), Error> {
        self.attrs.borrow_mut().display_element(&self.element, "height", v)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `maskUnits` attribute, controlling the coordinate space for the mask region's position and size.
    ///
    /// The default is [`MaskUnits::ObjectBoundingBox`], meaning [`set_x`](Self::set_x), [`set_y`](Self::set_y),
    /// [`set_width`](Self::set_width), and [`set_height`](Self::set_height) are fractions of the referencing
    /// element's bounding box.
    ///
    /// Use [`MaskUnits::UserSpaceOnUse`] to express the mask region in the referencing element's user coordinate
    /// system instead.
    pub fn set_mask_units(&self, u: MaskUnits) -> Result<(), Error> {
        self.element.set_attribute("maskUnits", u.as_str()).map_err(dom_err)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `maskContentUnits` attribute, controlling the coordinate space used by the shapes *inside* the mask.
    ///
    /// The default is [`MaskUnits::UserSpaceOnUse`], meaning the shapes inside the mask use the same coordinate system
    /// as the element that references the mask.
    ///
    /// Use [`MaskUnits::ObjectBoundingBox`] to express mask content coordinates as fractions (0.0–1.0) of the
    /// referencing element's bounding box instead.
    pub fn set_mask_content_units(&self, u: MaskUnits) -> Result<(), Error> {
        self.element.set_attribute("maskContentUnits", u.as_str()).map_err(dom_err)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `mask-type` attribute, selecting whether luminance or alpha determines how much of the referencing
    /// element each pixel of this mask reveals.
    ///
    /// The default is [`MaskType::Luminance`]. See [`MaskType`] for the difference.
    pub fn set_mask_type(&self, t: MaskType) -> Result<(), Error> {
        self.element.set_attribute("mask-type", t.as_str()).map_err(dom_err)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets any attribute on the `<mask>` element by name and string value.
    ///
    /// This is the generic escape hatch for attributes not covered by the named setters above (e.g. `class`, `style`,
    /// `transform`).
    ///
    /// ⚠️ Caveat ⚠️
    ///
    /// Name and value are written verbatim; so do not pass untrusted input!
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
    /// Creates a `<rect>` mask shape inside this `<mask>`.
    pub fn rect(&self, top_left: Point, size: Size) -> Result<SvgNode, Error> {
        self.create_rect(top_left, size)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<circle>` mask shape inside this `<mask>`.
    pub fn circle(&self, centre: Point, radius: f64) -> Result<SvgNode, Error> {
        self.create_circle(centre, radius)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates an `<ellipse>` mask shape inside this `<mask>`.
    pub fn ellipse(&self, centre: Point, radii: Size) -> Result<SvgNode, Error> {
        self.create_ellipse(centre, radii)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<line>` mask shape inside this `<mask>`.
    ///
    /// A `<line>` has no fillable area. To contribute to the mask, this line itself needs both a visible stroke
    /// (the default `stroke` is `none`) and a non-zero `stroke-width` — this combination is uncommon.
    ///
    /// Prefer area shapes (`<rect>`, `<circle>`, `<path>`, `<polygon>`) when defining mask content.
    pub fn line(&self, start: Point, end: Point) -> Result<SvgNode, Error> {
        self.create_line(start, end)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<path>` mask shape inside this `<mask>`.
    pub fn path(&self, d: &str) -> Result<SvgNode, Error> {
        self.create_path(d)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<path>` mask shape inside this `<mask>` from a sequence of typed [`PathDef`] segments.
    ///
    /// The type-safe alternative to [`path`](Self::path); see [`SvgRoot::path_from_defs`](crate::SvgRoot::path_from_defs)
    /// for the full rationale.
    pub fn path_from_defs(&self, defs: &[PathDef]) -> Result<SvgNode, Error> {
        self.create_path_from_defs(defs)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<polyline>` mask shape inside this `<mask>`.
    ///
    /// A polyline remains open for stroking, but SVG implicitly closes it for filling — a straight edge is treated
    /// as though it ran from the last point back to the first, purely for the purpose of computing the filled
    /// region. Consequently, a non-degenerate (three-or-more-point) polyline already contributes a filled region
    /// without needing to be turned into a polygon; `fill-rule` only selects how that implicitly closed area is
    /// classified (nonzero vs even-odd), not whether it exists. Use [`polygon`](Self::polygon) when an explicitly
    /// closed shape better expresses the intent.
    pub fn polyline(&self, points: &[Point]) -> Result<SvgNode, Error> {
        self.create_polyline(points)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<polygon>` mask shape inside this `<mask>`.
    pub fn polygon(&self, points: &[Point]) -> Result<SvgNode, Error> {
        self.create_polygon(points)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<text>` mask shape inside this `<mask>`.
    ///
    /// Text used as mask content reveals the referencing element through the glyph outlines — white (or any non-black)
    /// fill text is the standard way to cut a legible hole through a solid shape.
    pub fn text(&self, anchored_at: Point, content: &str) -> Result<SvgNode, Error> {
        self.create_text(anchored_at, content)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<g>` group mask shape inside this `<mask>`.
    ///
    /// All shapes inside the group contribute to the mask content, letting you combine several primitives.
    pub fn group(&self) -> Result<SvgNode, Error> {
        self.create_group()
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl SvgFactory for SvgMask {
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
