use crate::{
    Error, SvgNode, dom_err,
    root::{attrs::SvgAttrs, create_svg_element, defs::URL_PREFIX},
};
use std::{cell::RefCell, fmt};
use web_sys::{Document, SvgElement};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// A `<filter>` element that applies raster effects (blur, colour manipulation, compositing, ...) to any element that
/// references it.
///
/// A `<filter>` is a container of one or more filter-primitive elements (`<feGaussianBlur>`, `<feOffset>`, etc.); the
/// browser evaluates them in document order and paints the final result in place of the referencing element.
///
/// Obtain one from [`SvgDefs::filter`](crate::SvgDefs::filter) or
/// [`SvgDefs::build_filter`](crate::SvgDefs::build_filter), and apply it to any element with
/// [`SvgNode::set_filter_ref`](crate::SvgNode::set_filter_ref) or [`SvgNode::set_filter`](crate::SvgNode::set_filter).
///
/// # Primitive coverage
///
/// [`gaussian_blur`](Self::gaussian_blur) / [`gaussian_blur_xy`](Self::gaussian_blur_xy) (`<feGaussianBlur>`),
/// [`offset`](Self::offset) (`<feOffset>`), and [`merge`](Self::merge) (`<feMerge>`/`<feMergeNode>`) are
/// implemented — together enough to build a drop shadow (blur the source alpha, offset it, then merge it
/// underneath the original graphic; see [`merge`](Self::merge)'s example). The SVG filter specification defines
/// around fifteen primitives in total (`feColorMatrix`,
/// `feComposite`, `feFlood`, `feBlend`, and others), each with its own attribute grammar. See `docs/gaps.md` for
/// the primitives still to be added.
///
/// In the meantime, [`set_attr`](Self::set_attr) / [`set_attr_display`](Self::set_attr_display) on the `SvgFilter`
/// itself cover region attributes (`x`, `y`, `width`, `height`, `filterUnits`, `primitiveUnits`) not yet wrapped by a
/// named setter, and [`SvgNode::set_attr`](crate::SvgNode::set_attr) on any node returned by a primitive method
/// covers that primitive's own attributes not yet wrapped by a named parameter (`in`, `result`, and so on).
///
/// # Example
///
/// ```rust,no_run
/// use svg_dom::{SvgRoot, root::utils::{Point, Size}};
///
/// let svg  = SvgRoot::attach("diagram")?;
/// let defs = svg.defs()?;
///
/// let blur = defs.filter("soft-blur")?;
/// blur.gaussian_blur(4.0)?;
///
/// let rect = svg.rect(Point::origin(), Size::new(120.0, 80.0))?;
/// rect.set_fill("steelblue")?;
/// rect.set_filter_ref(&blur)?;
/// Ok::<(), svg_dom::Error>(())
/// ```
pub struct SvgFilter {
    /// The complete `url(#id)` reference, built once at construction and kept in sync by [`set_id`](Self::set_id).
    /// Caching the full reference (rather than the bare id) means that
    /// [`SvgNode::set_filter_ref`](crate::SvgNode::set_filter_ref) can write it straight to the `filter` attribute with
    /// no per-call formatting allocation, however many elements the same filter is applied to.
    ///
    /// [`id`](Self::id) slices the bare id back out of this string rather than storing it separately.
    url_ref: String,
    element: SvgElement,
    document: Document,
    attrs: RefCell<SvgAttrs>,
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl SvgFilter {
    pub(crate) fn new(id: String, element: SvgElement, document: Document) -> Self {
        let mut url_ref = String::with_capacity(URL_PREFIX.len() + id.len() + 1);
        url_ref.push_str(URL_PREFIX);
        url_ref.push_str(&id);
        url_ref.push(')');
        Self {
            url_ref,
            element,
            document,
            attrs: RefCell::new(SvgAttrs::new()),
        }
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Returns the cached `id` of this filter.
    ///
    /// Pass this to [`SvgNode::set_filter`](crate::SvgNode::set_filter), or use
    /// [`SvgNode::set_filter_ref`](crate::SvgNode::set_filter_ref) with the handle to avoid touching the id.
    ///
    /// # ⚠️ Caveat ⚠️
    ///
    /// The returned value is sliced out of the cached `url(#id)` reference (see `url_ref`) built at construction time
    /// and kept in sync by [`set_id`](Self::set_id). The slice is exact because filter ids are restricted at validation
    /// time to match the pattern `[A-Za-z_][A-Za-z0-9_-]*`, which is pure ASCII, so byte offsets from `URL_PREFIX`'s
    /// length and the string's end always land on the bare id exactly.
    ///
    /// [`set_attr`](Self::set_attr) and [`set_attr_display`](Self::set_attr_display) reject `"id"` so they cannot
    /// desynchronise the cache through the normal API.
    ///
    /// The only remaining escape hatch is writing through [`as_element`](Self::as_element) directly, which bypasses
    /// all crate-level checks.
    ///
    /// Always use `set_id` to rename a filter after construction.
    pub fn id(&self) -> &str {
        &self.url_ref[URL_PREFIX.len()..self.url_ref.len() - 1]
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Returns the cached `url(#id)` reference, ready to write directly to a `filter` attribute.
    ///
    /// Visbility need only be `pub(crate)` since [`SvgNode::set_filter_ref`](crate::SvgNode::set_filter_ref) is the
    /// only caller that needs it; external callers use [`id`](Self::id) instead.
    pub(crate) fn url_ref(&self) -> &str {
        &self.url_ref
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Renames the filter by updating both the DOM `id` attribute and the cached value returned by [`id`](Self::id).
    ///
    /// This method takes `&mut self` because it mutates Rust-owned state (the cached reference string), unlike the
    /// other attribute setters that only write to the DOM.
    ///
    /// The new `id` is subject to the same validation rules as the id supplied at construction time: it must match the
    /// pattern `[A-Za-z_][A-Za-z0-9_-]*` — a letter or underscore followed by letters, digits, underscores, or hyphens.
    ///
    /// ⚠️ Caveat ⚠️
    ///
    /// Renaming a filter does not update any `filter` attributes already written to referencing elements — those store
    /// a snapshot of the reference at the time it was applied.
    ///
    /// Either rename before applying references, or reapply the reference after renaming it.
    ///
    /// # Errors
    ///
    /// - [`Error::InvalidFilterId`] — the new id failed validation.
    /// - [`Error::Dom`] — the browser refused to write the `id` attribute.
    pub fn set_id(&mut self, id: &str) -> Result<(), Error> {
        super::defs::validate_filter_id(id)?;
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
    /// Sets any attribute on the `<filter>` element by name and string value.
    ///
    /// This is the generic escape hatch for attributes not covered by a named setter — for example the filter region
    /// (`x`, `y`, `width`, `height`) or coordinate-space attributes (`filterUnits`, `primitiveUnits`).
    ///
    /// ⚠️ Caveat ⚠️
    ///
    /// Name and value are written verbatim; so be careful not pass any untrusted input!
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
    /// Shared implementation behind [`gaussian_blur`](Self::gaussian_blur) and
    /// [`gaussian_blur_xy`](Self::gaussian_blur_xy): creates a `<feGaussianBlur>`, writes `std_deviation` as its
    /// `stdDeviation` attribute, and appends it.
    ///
    /// `std_deviation` is a pre-built [`fmt::Arguments`] rather than a `&str` so the two public callers can pass either
    /// a single number or an `"x y"` pair through [`display_element`](SvgAttrs::display_element)'s retained scratch
    /// buffer without first collecting into an owned `String`.  This is the same technique used by
    /// [`SvgPattern::set_view_box`](crate::SvgPattern::set_view_box) and
    /// [`SvgSymbol::set_view_box`](crate::SvgSymbol::set_view_box) to combine several numbers into one attribute.
    fn gaussian_blur_args(&self, std_deviation: fmt::Arguments<'_>) -> Result<SvgNode, Error> {
        let el = create_svg_element::<SvgElement>(&self.document, "feGaussianBlur", "SvgElement")?;
        self.attrs.borrow_mut().display_element(&el, "stdDeviation", std_deviation)?;
        self.element.append_child(&el).map_err(dom_err)?;
        Ok(SvgNode::new(el))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Appends a `<feGaussianBlur>` primitive to this filter, blurring its input equally on both axes by
    /// `std_deviation`.
    ///
    /// `std_deviation` is the standard deviation of the Gaussian blur kernel, in user units; larger values blur more.
    /// A `std_deviation` of `0.0` produces no blur (the input passes through unchanged).
    ///
    /// See [`gaussian_blur_xy`](Self::gaussian_blur_xy) for a blur with independent horizontal and vertical
    /// deviations — the SVG `stdDeviation` attribute accepts either one or two numbers, and this method covers only
    /// the one-number form.
    ///
    /// If this is the filter's first primitive, its implicit input is `SourceGraphic` (the referencing element as
    /// normally rendered). Use the returned [`SvgNode`]'s [`set_attr`](crate::SvgNode::set_attr) to set `in` (e.g.
    /// `"SourceAlpha"`, or the `result` name of an earlier primitive) or `result` (to name this primitive's output for
    /// a later primitive's `in`/`in2` to reference) — neither has a dedicated setter yet.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if the browser refuses to create or append the `<feGaussianBlur>` element.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::SvgRoot;
    ///
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    /// let blur = defs.filter("soft")?;
    /// blur.gaussian_blur(4.0)?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn gaussian_blur(&self, std_deviation: f64) -> Result<SvgNode, Error> {
        self.gaussian_blur_args(format_args!("{std_deviation}"))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Appends a `<feGaussianBlur>` primitive to this filter with independent horizontal and vertical standard
    /// deviations, writing the SVG `stdDeviation="std_deviation_x std_deviation_y"` two-number form.
    ///
    /// Pass `0.0` for one axis to blur only along the other — for example `gaussian_blur_xy(0.0, 6.0)` blurs
    /// vertically only, useful for a horizontal motion-blur effect.
    ///
    /// For an equal blur on both axes, prefer [`gaussian_blur`](Self::gaussian_blur): passing the same value twice
    /// here writes the same two-number attribute the one-number form already implies, at no benefit.
    ///
    /// See [`gaussian_blur`](Self::gaussian_blur) for the `in`/`result` attributes, which apply identically here.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if the browser refuses to create or append the `<feGaussianBlur>` element.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::SvgRoot;
    ///
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    /// let blur = defs.filter("streak")?;
    /// blur.gaussian_blur_xy(12.0, 0.0)?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn gaussian_blur_xy(&self, std_deviation_x: f64, std_deviation_y: f64) -> Result<SvgNode, Error> {
        self.gaussian_blur_args(format_args!("{std_deviation_x} {std_deviation_y}"))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Appends a `<feOffset>` primitive to this filter, shifting its input by `(dx, dy)` user units.
    ///
    /// The most common use is shifting a blurred alpha silhouette to build a drop shadow — see [`merge`](Self::merge)
    /// for combining the result back with the original graphic.
    ///
    /// If this is the filter's first primitive, its implicit input is `SourceGraphic`. Use the returned
    /// [`SvgNode`]'s [`set_attr`](crate::SvgNode::set_attr) to set `in` or `result`, neither of which has a dedicated
    /// setter yet.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if the browser refuses to create or append the `<feOffset>` element.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::SvgRoot;
    ///
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    /// let shadow = defs.filter("shadow")?;
    /// shadow.gaussian_blur(4.0)?.set_attrs([("in", "SourceAlpha"), ("result", "blur")])?;
    /// shadow.offset(4.0, 4.0)?.set_attr("in", "blur")?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn offset(&self, dx: f64, dy: f64) -> Result<SvgNode, Error> {
        let el = create_svg_element::<SvgElement>(&self.document, "feOffset", "SvgElement")?;
        {
            let mut attrs = self.attrs.borrow_mut();
            attrs.display_element(&el, "dx", dx)?;
            attrs.display_element(&el, "dy", dy)?;
        }
        self.element.append_child(&el).map_err(dom_err)?;
        Ok(SvgNode::new(el))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Appends a `<feMerge>` primitive to this filter, stacking `inputs` on top of one another in the given order
    /// (later entries painted last, i.e. on top).
    ///
    /// Each entry in `inputs` becomes one `<feMergeNode in="...">` child, in order — the standard way to layer, for
    /// example, an offset blurred shadow underneath the original graphic: `merge(&["offset-blur", "SourceGraphic"])`.
    ///
    /// Unlike [`gaussian_blur`](Self::gaussian_blur) and [`offset`](Self::offset), `<feMerge>` has no attributes of its
    /// own to set beyond the generic `result` — its content is entirely the ordered list of `<feMergeNode>` children
    /// this method builds, so there is nothing for the returned [`SvgNode`]'s [`set_attr`](crate::SvgNode::set_attr) to
    /// configure except `result`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if the browser refuses to create or append the `<feMerge>` element or any of its
    /// `<feMergeNode>` children.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::SvgRoot;
    ///
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    /// let shadow = defs.filter("shadow")?;
    /// shadow.gaussian_blur(4.0)?.set_attrs([("in", "SourceAlpha"), ("result", "blur")])?;
    /// shadow.offset(4.0, 4.0)?.set_attrs([("in", "blur"), ("result", "offset-blur")])?;
    /// shadow.merge(&["offset-blur", "SourceGraphic"])?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn merge(&self, inputs: &[&str]) -> Result<SvgNode, Error> {
        let el = create_svg_element::<SvgElement>(&self.document, "feMerge", "SvgElement")?;
        for input in inputs {
            let node = create_svg_element::<SvgElement>(&self.document, "feMergeNode", "SvgElement")?;
            node.set_attribute("in", input).map_err(dom_err)?;
            el.append_child(&node).map_err(dom_err)?;
        }
        self.element.append_child(&el).map_err(dom_err)?;
        Ok(SvgNode::new(el))
    }
}
