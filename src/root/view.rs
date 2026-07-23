use crate::{Error, dom_err, root::attrs::SvgAttrs};
use std::cell::RefCell;
use web_sys::SvgElement;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// A `<view>` element that names a `viewBox`/`preserveAspectRatio` combination for external, JavaScript-free navigation
/// via a URL fragment.
///
/// Unlike a `<symbol>`, a `<view>` has no rendered graphical content of its own. However, SVG does permit descriptive
/// child elements (`<title>`, `<desc>`, `<metadata>`), but does not currently offer a way to add them — it models only
/// the viewport and fragment-navigation attributes below, since that covers `<view>`'s actual purpose.
///
/// A browser resolves a URL ending in `#viewId` (where `viewId` is this element's [`id`](Self::id)) by temporarily
/// substituting the document's effective `viewBox`/`preserveAspectRatio` with this element's own, without the need for
/// any JavaScript. This works for:
///
/// - a same-document fragment link, e.g. [`SvgRoot::anchor`](crate::SvgRoot::anchor)`("#viewId")`;
/// - an external reference into an exported SVG file, e.g. `<img src="diagram.svg#viewId">` or a plain hyperlink to
///   `diagram.svg#viewId`.
///
/// For a live, WASM-attached SVG, for which the caller already has a handle, the same effect is just a direct
/// [`SvgRoot::set_view_box`](crate::SvgRoot::set_view_box) or [`SvgRoot::set_viewport`](crate::SvgRoot::set_viewport)
/// call.
///
/// `<view>` is useful primarily in cases where an SVG document is exported or embedded/navigated independently of any
/// running WASM code.
///
/// Obtain one from [`SvgDefs::view`](crate::SvgDefs::view) or [`SvgDefs::build_view`](crate::SvgDefs::build_view).
///
/// # Example
///
/// ```rust,no_run
/// use svg_dom::SvgRoot;
///
/// let svg  = SvgRoot::attach("diagram")?;
/// let defs = svg.defs()?;
///
/// // Name a zoomed-in view of the top-left quadrant.
/// let detail = defs.view("detail")?;
/// detail.set_view_box(0.0, 0.0, 50.0, 50.0)?;
///
/// // A same-document link that switches to it with no JavaScript at all.
/// svg.anchor("#detail")?;
/// Ok::<(), svg_dom::Error>(())
/// ```
pub struct SvgView {
    /// The `id` set at construction time; cached to avoid a round-trip to the DOM for [`id`](Self::id).
    id: String,
    element: SvgElement,
    attrs: RefCell<SvgAttrs>,
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl SvgView {
    // `SvgView` does not offer child-element construction (SVG permits descriptive children such as <title>/<desc>
    // on <view>, but nothing in this crate's own API needs to build), so unlike its sibling id-cached elements
    // (`SvgSymbol`, `SvgMarker`, ...) it never needs a `Document` handle to create any — none is stored.
    pub(crate) fn new(id: String, element: SvgElement) -> Self {
        Self {
            id,
            element,
            attrs: RefCell::new(SvgAttrs::new()),
        }
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Returns the cached `id` of this view.
    ///
    /// Pass this (prefixed with `#`) to [`SvgRoot::anchor`](crate::SvgRoot::anchor), or embed it directly in an
    /// external URL fragment (`"diagram.svg#" + id`).
    ///
    /// # ⚠️ Caveat ⚠️
    ///
    /// The returned value is cached in the `SvgView` struct at construction time and kept in sync by
    /// [`set_id`](Self::set_id).
    /// [`set_attr`](Self::set_attr) and [`set_attr_display`](Self::set_attr_display) reject `"id"` so they cannot
    /// desynchronise the cache through the normal API.
    /// Always use `set_id` to rename a view after construction.
    pub fn id(&self) -> &str {
        &self.id
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Renames the view by updating both the DOM `id` attribute and the cached value returned by [`id`](Self::id).
    ///
    /// **Note:** renaming a view does not update any `href`/URL fragment already written elsewhere (an `<a>`
    /// element, an external `<img src>`, ...) — those store a snapshot of the id at the time the reference was
    /// written.
    ///
    /// # Errors
    ///
    /// - [`Error::InvalidViewId`] — the new id failed validation.
    /// - [`Error::Dom`] — the browser refused to write the `id` attribute.
    pub fn set_id(&mut self, id: &str) -> Result<(), Error> {
        super::defs::validate_view_id(id)?;
        self.element.set_attribute("id", id).map_err(dom_err)?;
        self.id.clear();
        self.id.push_str(id);
        Ok(())
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Returns a reference to the underlying `web-sys` `SvgElement`.
    ///
    /// Avoid writing the `id` attribute through this handle; use [`set_id`](Self::set_id) instead so the cached
    /// value stays in sync.
    pub fn as_element(&self) -> &SvgElement {
        &self.element
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `viewBox` attribute, declaring the coordinate region this view switches the document to.
    ///
    /// This is the entire reason `<view>` exists: without a `viewBox`, navigating to this view's `#id` has no
    /// visible effect.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidViewBox`] if any component is not finite (`NaN`/`±infinity`), or if either of `width`
    /// or `height` is negative. A `width`/`height` of exactly `0.0` is accepted; as per the SVG spec, it is a trick
    /// to disable rendering.
    pub fn set_view_box(&self, x: f64, y: f64, width: f64, height: f64) -> Result<(), Error> {
        super::utils::validate_view_box(x, y, width, height)?;
        self.attrs
            .borrow_mut()
            .display_element(&self.element, "viewBox", format_args!("{x} {y} {width} {height}"))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `preserveAspectRatio` attribute, controlling alignment and clipping when the viewport this view is
    /// navigated into has a different aspect ratio than `viewBox`.
    ///
    /// The default value (`"xMidYMid meet"`) centres the view's region and scales it to fit without clipping.
    /// Use `"none"` to stretch it to exactly fill the viewport.
    ///
    /// See the [MDN reference](https://developer.mozilla.org/en-US/docs/Web/SVG/Attribute/preserveAspectRatio) for the
    /// full list of alignment keywords.
    pub fn set_preserve_aspect_ratio(&self, value: &str) -> Result<(), Error> {
        self.element.set_attribute("preserveAspectRatio", value).map_err(dom_err)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets any attribute on the `<view>` element by name and string value.
    ///
    /// This is the generic escape hatch for attributes not covered by the named setters above.  Name and value are
    /// written verbatim; so do not pass untrusted input!
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
}
