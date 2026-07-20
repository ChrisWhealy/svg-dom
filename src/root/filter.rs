use crate::{
    Error, SvgNode, dom_err,
    root::{attrs::SvgAttrs, create_svg_element, defs::URL_PREFIX},
};
use std::{cell::RefCell, fmt};
use web_sys::{Document, SvgElement};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// The Porter-Duff (or Photoshop-style) compositing operator for [`SvgFilter::composite`], controlling how the
/// `in` and `in2` inputs combine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompositeOperator {
    /// `in` painted over `in2` (SVG default). Neither input is clipped to the other's shape.
    Over,
    /// Only the part of `in` that overlaps `in2` is kept — the standard way to tint a mask, e.g. compositing a
    /// flood colour "in" a blurred alpha silhouette to colour a drop shadow.
    In,
    /// Only the part of `in` that does *not* overlap `in2` is kept.
    Out,
    /// The part of `in` that overlaps `in2`, painted on top of `in2`.
    Atop,
    /// The non-overlapping parts of both inputs; the overlap is removed from both.
    Xor,
    /// The arithmetic sum of both inputs, clamped to fully opaque — brightens rather than blends.
    Lighter,
    /// A per-pixel weighted sum `k1*i1*i2 + k2*i1 + k3*i2 + k4`, controlled by the `k1`–`k4` attributes (not
    /// wrapped by a named parameter here; set them via the returned [`SvgNode`]'s
    /// [`set_attr`](crate::SvgNode::set_attr) — this is the one operator [`composite`](SvgFilter::composite)
    /// does not fully configure on its own).
    ///
    /// ***⚠️ `k1`–`k4` arguments all default to `0`*** — [`composite`](SvgFilter::composite) does not write them, and
    /// the SVG initial value for each is `0`. Selecting `Arithmetic` and stopping there evaluates to
    /// `0*i1*i2 + 0*i1 + 0*i2 + 0` for every pixel, i.e. **transparent black**, not a blend of the two inputs.
    ///
    /// Always set all four coefficients you need immediately after calling `composite` with this operator:
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::filter::CompositeOperator};
    ///
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    /// let flt  = defs.filter("blend")?;
    /// flt.gaussian_blur(6.0)?.set_attrs([("in", "SourceGraphic"), ("result", "blur")])?;
    ///
    /// // A straightforward 50/50 blend of the sharp source and its blurred copy: k2 = k3 = 0.5, k1 = k4 = 0.
    /// flt.composite("blur", CompositeOperator::Arithmetic)?.set_attrs([
    ///     ("in", "SourceGraphic"),
    ///     ("k1", "0"), ("k2", "0.5"), ("k3", "0.5"), ("k4", "0"),
    /// ])?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    Arithmetic,
}

impl CompositeOperator {
    fn as_str(self) -> &'static str {
        match self {
            Self::Over => "over",
            Self::In => "in",
            Self::Out => "out",
            Self::Atop => "atop",
            Self::Xor => "xor",
            Self::Lighter => "lighter",
            Self::Arithmetic => "arithmetic",
        }
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// The colour transform applied by [`SvgFilter::color_matrix`], selecting both the SVG `type` attribute and the
/// shape of the `values` attribute that goes with it.
///
/// This enum deliberately does not implement `Copy` (unlike [`CompositeOperator`], which is small enough that copying
/// is free): the [`Matrix`](Self::Matrix) variant carries 160 bytes of `f64`s, and making that implicitly copyable
/// would encourage silent full-array copies at call sites when only a move or borrow was needed.
#[derive(Debug, Clone, PartialEq)]
pub enum ColorMatrixType {
    /// A full 4x5 colour transform matrix, applied to each pixel's `[R, G, B, A]` as `M · [R, G, B, A, 1]ᵀ`.
    ///
    /// Deliberately a fixed-size `[f64; 20]` rather than a `Vec<f64>` or `&[f64]`: the SVG `values` attribute for
    /// this type is defined as exactly 20 numbers, no more and no fewer, so a matrix with the wrong element count
    /// cannot be constructed at all, rather than failing at the DOM boundary or silently truncating/padding.
    Matrix([f64; 20]),
    /// Adjusts colour saturation. `1.0` is the identity (no change); `0.0` produces greyscale.
    Saturate(f64),
    /// Rotates hue by the given angle in degrees around the colour circle.
    HueRotate(f64),
    /// Converts to greyscale using each pixel's luminance as its resulting alpha (zeroing RGB) — derives a mask
    /// from perceived brightness rather than the alpha channel [`gaussian_blur`](SvgFilter::gaussian_blur) and
    /// friends use for a shadow silhouette.
    LuminanceToAlpha,
}

impl ColorMatrixType {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Matrix(_) => "matrix",
            Self::Saturate(_) => "saturate",
            Self::HueRotate(_) => "hueRotate",
            Self::LuminanceToAlpha => "luminanceToAlpha",
        }
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Controls which coordinate space the filter region (`x`, `y`, `width`, `height`) or a primitive's own coordinate
/// attributes are expressed in.
///
/// Used for both the `filterUnits` and `primitiveUnits` attributes.
/// Passed to [`SvgFilter::set_filter_units`] and [`SvgFilter::set_primitive_units`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterUnits {
    /// Values are expressed in the same coordinate space as the element that references the filter.
    /// SVG default for `primitiveUnits`.
    UserSpaceOnUse,
    /// Values are expressed as fractions of the referencing element's bounding box — `(0, 0)` maps to the top-left
    /// corner and `(1, 1)` maps to the bottom-right corner.
    /// SVG default for `filterUnits`.
    ObjectBoundingBox,
}

impl FilterUnits {
    fn as_str(self) -> &'static str {
        match self {
            Self::UserSpaceOnUse => "userSpaceOnUse",
            Self::ObjectBoundingBox => "objectBoundingBox",
        }
    }
}

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
/// Implemented filter effects:
///
/// - [`gaussian_blur`](Self::gaussian_blur)
/// - [`gaussian_blur_xy`](Self::gaussian_blur_xy) (`<feGaussianBlur>`),
/// - [`offset`](Self::offset) (`<feOffset>`),
/// - [`merge`](Self::merge) (`<feMerge>`/`<feMergeNode>`),
/// - [`flood`](Self::flood) (`<feFlood>`),
/// - [`composite`](Self::composite) (`<feComposite>`),
/// - [`drop_shadow`](Self::drop_shadow) (`<feDropShadow>`),
/// - [`color_matrix`](Self::color_matrix) (`<feColorMatrix>`)
///
/// The first five, taken together, can be used to build a *true* tinted, opacity-controlled drop shadow (blur the
/// source alpha, composite a flood colour into the blurred mask, offset it, then merge it underneath the original
/// graphic; see [`composite`](Self::composite)'s example) rather than just a blurred copy of the source graphic's
/// own colour.
///
/// [`drop_shadow`](Self::drop_shadow) achieves the same effect using a single primitive, since the SVG specification
/// defines it as a browser-native shorthand for exactly that chain.
///
/// [`color_matrix`](Self::color_matrix) is independent of the shadow primitives — greyscale, saturation, hue
/// rotation, or an arbitrary linear colour transform via [`ColorMatrixType`].
///
/// The SVG filter specification defines around fifteen effect primitives in total (`feBlend`, `feTile`, and others),
/// each with its own attribute grammar. See `docs/gaps.md` for the primitives still to be added.
///
/// The filter region ([`set_x`](Self::set_x), [`set_y`](Self::set_y), [`set_width`](Self::set_width),
/// [`set_height`](Self::set_height)) and coordinate-space ([`set_filter_units`](Self::set_filter_units),
/// [`set_primitive_units`](Self::set_primitive_units)) attributes each have a named setter.
/// [`SvgNode::set_attr`](crate::SvgNode::set_attr) on any node returned by a primitive method covers that primitive's
/// own attributes not yet wrapped by a named parameter (`in`, `result`, and so on).
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
    /// This is the generic escape hatch for attributes not having a named setter: e.g. [`set_x`](Self::set_x),
    /// [`set_y`](Self::set_y), [`set_width`](Self::set_width) and [`set_height`](Self::set_height) only accept a bare
    /// `f64`, so pass an explicit SVG length or percentage (e.g. `"-20%"`) here when the filter region needs that
    /// syntax instead of a plain number.
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
    /// Sets the horizontal offset of the filter region.
    ///
    /// Interpreted according to [`filterUnits`](Self::set_filter_units) — a fraction of the referencing element's
    /// bounding box under the SVG default ([`FilterUnits::ObjectBoundingBox`]), or a user-space coordinate under
    /// [`FilterUnits::UserSpaceOnUse`].
    pub fn set_x(&self, v: f64) -> Result<(), Error> {
        self.attrs.borrow_mut().display_element(&self.element, "x", v)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the vertical offset of the filter region.
    ///
    /// See [`set_x`](Self::set_x) for the coordinate space this value is interpreted in.
    pub fn set_y(&self, v: f64) -> Result<(), Error> {
        self.attrs.borrow_mut().display_element(&self.element, "y", v)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the width of the filter region.
    ///
    /// The SVG default filter region is `-10% -10% 120% 120%` of the referencing element's bounding box, which can clip
    /// a wide blur; widen `width`/`height` explicitly for large [`gaussian_blur`](Self::gaussian_blur) `std_deviation`
    /// values.
    ///
    /// ⚠️ Performance ⚠️
    ///
    /// Expand the region only enough to contain the intended effect. Per the SVG filter specification, the filter
    /// region is a hard clip: every intermediate offscreen buffer the browser rasterises while evaluating this filter's
    /// primitives is bounded by it, so an unnecessarily large region can inflate both rasterisation work and temporary
    /// memory use, not just the final painted area.
    ///
    /// See [`set_x`](Self::set_x) for the coordinate space this value is interpreted in.
    pub fn set_width(&self, v: f64) -> Result<(), Error> {
        self.attrs.borrow_mut().display_element(&self.element, "width", v)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the height of the filter region.
    ///
    /// See [`set_width`](Self::set_width) for why this often needs widening beyond the SVG default, why it should
    /// not be widened further than the effect needs, and for the coordinate space this value is interpreted in.
    pub fn set_height(&self, v: f64) -> Result<(), Error> {
        self.attrs.borrow_mut().display_element(&self.element, "height", v)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `filterUnits` attribute, controlling the coordinate space for the filter region's position and size.
    ///
    /// The default is [`FilterUnits::ObjectBoundingBox`], meaning [`set_x`](Self::set_x), [`set_y`](Self::set_y),
    /// [`set_width`](Self::set_width), and [`set_height`](Self::set_height) are fractions of the referencing element's
    /// bounding box.
    ///
    /// Use [`FilterUnits::UserSpaceOnUse`] to express the filter region in the referencing element's user coordinate
    /// system instead.
    pub fn set_filter_units(&self, u: FilterUnits) -> Result<(), Error> {
        self.element.set_attribute("filterUnits", u.as_str()).map_err(dom_err)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `primitiveUnits` attribute, controlling the coordinate space used by length-valued attributes on the
    /// filter's own primitives (for example [`gaussian_blur`](Self::gaussian_blur)'s `std_deviation` or
    /// [`offset`](Self::offset)'s `dx`/`dy`).
    ///
    /// The default is [`FilterUnits::UserSpaceOnUse`], meaning primitive attributes use the same coordinate system as
    /// the element that references the filter.
    ///
    /// Use [`FilterUnits::ObjectBoundingBox`] to express them as fractions of the referencing element's bounding box
    /// instead.
    pub fn set_primitive_units(&self, u: FilterUnits) -> Result<(), Error> {
        self.element.set_attribute("primitiveUnits", u.as_str()).map_err(dom_err)
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

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Appends a `<feFlood>` primitive to this filter, producing a solid-colour rectangle covering the filter region.
    ///
    /// `color` is any valid SVG/CSS colour value (`"red"`, `"#ff0000"`, `"rgb(255,0,0)"`, ...), written as `flood-color`.
    /// `opacity` (written as `flood-opacity`) is a value in the range `0.0` to `1.0`.
    ///
    /// **IMPORTANT** values outside that range will not cause a runtime error, but may well produce an unspecified
    /// rendering results.  This is the same convention as used by
    /// [`SvgLinearGradient::add_stop_opacity`](crate::SvgLinearGradient::add_stop_opacity)
    ///
    /// On its own, a flood fills the entire filter region with one flat colour, which by itself is rarely useful, but
    /// when combined with [`composite`](Self::composite) (`operator: `[`In`](CompositeOperator::In)) against a blurred
    /// alpha mask, it is the standard way to give a drop shadow an actual colour and opacity rather than leaving it
    /// simply as a blurred copy of the source graphic's own fill; see [`composite`](Self::composite)'s example.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if the browser refuses to create or append the `<feFlood>` element.
    pub fn flood(&self, color: &str, opacity: f64) -> Result<SvgNode, Error> {
        let el = create_svg_element::<SvgElement>(&self.document, "feFlood", "SvgElement")?;
        {
            let mut attrs = self.attrs.borrow_mut();
            attrs.display_element(&el, "flood-opacity", opacity)?;
            el.set_attribute("flood-color", color).map_err(dom_err)?;
        }
        self.element.append_child(&el).map_err(dom_err)?;
        Ok(SvgNode::new(el))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Appends a `<feComposite>` primitive to this filter, combining this primitive's `in` input with `in2` using
    /// the given Porter-Duff [`operator`](CompositeOperator).
    ///
    /// ***⚠️ [`CompositeOperator::Arithmetic`] needs `k1`–`k4` to be set manually*** — see that variant's own doc for
    /// why skipping them silently produces transparent black rather than an error.
    ///
    /// `in2` is written directly.
    ///
    /// ***IMPORTANT*** The value of `in2` is not validated.  It is typically another primitive's `result` name, or one
    /// of the SVG keyword inputs `"SourceGraphic"`/`"SourceAlpha"`).
    ///
    /// `in` is not set by this method: if this is the filter's first primitive, its implicit input is `SourceGraphic`,
    /// otherwise use the returned [`SvgNode`]'s [`set_attr`](crate::SvgNode::set_attr) to set `in` explicitly, the same
    /// as every other primitive here.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if the browser refuses to create or append the `<feComposite>` element.
    ///
    /// # Example
    ///
    /// A true tinted, semi-transparent drop shadow (as opposed to the blurred-copy approximation produced by
    /// [`merge`](Self::merge)) by flooding a colour and compositing it into the blurred alpha mask before offsetting
    /// and merging it underneath the original graphic:
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::filter::CompositeOperator};
    ///
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    /// let shadow = defs.filter("shadow")?;
    /// shadow.gaussian_blur(4.0)?.set_attrs([("in", "SourceAlpha"), ("result", "blur")])?;
    /// shadow.flood("black", 0.5)?.set_attr("result", "colour")?;
    /// shadow.composite("blur", CompositeOperator::In)?.set_attrs([("in", "colour"), ("result", "tinted")])?;
    /// shadow.offset(4.0, 4.0)?.set_attrs([("in", "tinted"), ("result", "offset-shadow")])?;
    /// shadow.merge(&["offset-shadow", "SourceGraphic"])?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn composite(&self, in2: &str, operator: CompositeOperator) -> Result<SvgNode, Error> {
        let el = create_svg_element::<SvgElement>(&self.document, "feComposite", "SvgElement")?;
        el.set_attribute("in2", in2).map_err(dom_err)?;
        el.set_attribute("operator", operator.as_str()).map_err(dom_err)?;
        self.element.append_child(&el).map_err(dom_err)?;
        Ok(SvgNode::new(el))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Appends a `<feDropShadow>` primitive to this filter, which is SVG shorthand for the entire chain shown in
    /// [`composite`](Self::composite)'s example:
    ///
    /// [`gaussian_blur`](Self::gaussian_blur) → [`flood`](Self::flood) → [`composite`](Self::composite) →
    /// [`offset`](Self::offset) → [`merge`](Self::merge),
    ///
    /// - `std_deviation` is the blur radius (as [`gaussian_blur`](Self::gaussian_blur))
    /// - `dx`/`dy` are the shadow offset (as [`offset`](Self::offset))
    /// - `color`/`opacity` are the shadow's `flood-color`/`flood-opacity` (as [`flood`](Self::flood))
    ///
    /// # This primitive already merges the original graphic on top, so there is no need to call [`merge`](Self::merge)
    /// after it
    ///
    /// As per the SVG specification, `<feDropShadow>`'s result already includes its `in` input painted over the shadow,
    /// exactly as the final `merge(&[shadow, "SourceGraphic"])` step does in the manual chain.
    ///
    /// A `<filter>` containing only `drop_shadow(...)` is already a complete, ready-to-use shadow effect; adding a
    /// further `merge` call would paint the original graphic on top a second time.
    ///
    /// If this is the filter's first (and only) primitive, its implicit `in` is `SourceGraphic`, and that is also what
    /// gets composited back on top; which is the common case this shorthand exists for.
    ///
    /// Use the returned [`SvgNode`]'s [`set_attr`](crate::SvgNode::set_attr) to set `in` or `result` explicitly for
    /// anything else.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if the browser refuses to create or append the `<feDropShadow>` element.
    ///
    /// # Example
    ///
    /// The five-primitive chain from [`composite`](Self::composite)'s example, collapsed to one call:
    ///
    /// ```rust,no_run
    /// use svg_dom::SvgRoot;
    ///
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    /// let shadow = defs.filter("shadow")?;
    /// shadow.drop_shadow(4.0, 4.0, 4.0, "black", 0.5)?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn drop_shadow(
        &self,
        std_deviation: f64,
        dx: f64,
        dy: f64,
        color: &str,
        opacity: f64,
    ) -> Result<SvgNode, Error> {
        let el = create_svg_element::<SvgElement>(&self.document, "feDropShadow", "SvgElement")?;
        {
            let mut attrs = self.attrs.borrow_mut();
            attrs.display_element(&el, "stdDeviation", std_deviation)?;
            attrs.display_element(&el, "dx", dx)?;
            attrs.display_element(&el, "dy", dy)?;
            attrs.display_element(&el, "flood-opacity", opacity)?;
        }
        el.set_attribute("flood-color", color).map_err(dom_err)?;
        self.element.append_child(&el).map_err(dom_err)?;
        Ok(SvgNode::new(el))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Appends a `<feColorMatrix>` primitive to this filter, transforming colours via [`matrix_type`](ColorMatrixType).
    ///
    /// Writes the SVG `type` attribute from `matrix_type`'s variant, and — for every variant except
    /// [`LuminanceToAlpha`](ColorMatrixType::LuminanceToAlpha), which needs none — the matching `values` attribute:
    /// twenty space-separated numbers for [`Matrix`](ColorMatrixType::Matrix), or a single number for
    /// [`Saturate`](ColorMatrixType::Saturate)/[`HueRotate`](ColorMatrixType::HueRotate).
    ///
    /// If this is the filter's first primitive, its implicit input is `SourceGraphic`. Use the returned [`SvgNode`]'s
    /// [`set_attr`](crate::SvgNode::set_attr) to set `in` or `result`, neither of which has a dedicated setter yet.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if the browser refuses to create or append the `<feColorMatrix>` element.
    ///
    /// # Example
    ///
    /// A fully desaturated (greyscale) copy of the source graphic:
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::filter::ColorMatrixType};
    ///
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    /// let grey = defs.filter("greyscale")?;
    /// grey.color_matrix(ColorMatrixType::Saturate(0.0))?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn color_matrix(&self, matrix_type: ColorMatrixType) -> Result<SvgNode, Error> {
        let el = create_svg_element::<SvgElement>(&self.document, "feColorMatrix", "SvgElement")?;
        el.set_attribute("type", matrix_type.as_str()).map_err(dom_err)?;
        match matrix_type {
            ColorMatrixType::Matrix(m) => {
                self.attrs.borrow_mut().display_element(
                    &el,
                    "values",
                    format_args!(
                        "{} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
                        m[0],
                        m[1],
                        m[2],
                        m[3],
                        m[4],
                        m[5],
                        m[6],
                        m[7],
                        m[8],
                        m[9],
                        m[10],
                        m[11],
                        m[12],
                        m[13],
                        m[14],
                        m[15],
                        m[16],
                        m[17],
                        m[18],
                        m[19],
                    ),
                )?;
            },
            ColorMatrixType::Saturate(v) | ColorMatrixType::HueRotate(v) => {
                self.attrs.borrow_mut().display_element(&el, "values", v)?;
            },
            ColorMatrixType::LuminanceToAlpha => {},
        }
        self.element.append_child(&el).map_err(dom_err)?;
        Ok(SvgNode::new(el))
    }
}
