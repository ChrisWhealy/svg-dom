use crate::{Error, dom_err, root::attrs::SvgAttrs, root::factory::SvgFactory};
use std::cell::RefCell;
use web_sys::SvgElement;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// A `<view>` element that names a `viewBox`/`preserveAspectRatio` combination, activated via a `#viewId` URL
/// fragment.
///
/// Unlike a `<symbol>`, a `<view>` has no rendered graphical content of its own. However, SVG permits descriptive
/// elements, animation elements, `<script>` and `<style>` as children of `<view>`. SvgView intentionally exposes no
/// child-construction API because these children are not needed for its normal viewport-navigation role.
///
/// # Fragment navigation: three cases, only two of which apply to this crate
///
/// SVG 2 activates `#viewId` fragment navigation only when the SVG resource named by `viewId` is itself *the
/// document being navigated*. It does not activate merely because some document happens to contain a matching id.
/// Concretely:
///
/// - **A standalone SVG document, navigated directly.** This includes a document opened in its own tab, or acting as
///   the top-level document that a same-document `#viewId` link/hash change targets. The browser substitutes its
///   effective `viewBox`/`preserveAspectRatio` with this view's own, with no JavaScript needed.
/// - **An external reference into an exported SVG file** — `<img src="diagram.svg#viewId">`, an SVG `<image>` with
///   that `href`, or a plain hyperlink to `diagram.svg#viewId` — activates the same substitution for that resource.
///   [`SvgNode::set_href`](crate::SvgNode::set_href) can re-trigger it on an already-loaded reference by changing the
///   fragment.
/// - **An inline `<svg>` embedded in an HTML page — the case every [`SvgRoot::attach`](crate::SvgRoot::attach)/
///   [`SvgRoot::create_in`](crate::SvgRoot::create_in) call in this crate deals with — does *not* qualify.** The
///   embedded SVG is not itself the navigated document, so this behaviour never activates for it. A same-page
///   `<a href="#viewId">` click — whether an SVG-native [`SvgRoot::anchor`](crate::SvgRoot::anchor) inside it or a
///   plain HTML link outside it — only updates `location.hash`, with no visible effect. Use
///   [`SvgRoot::set_view_box`](crate::SvgRoot::set_view_box)/[`SvgRoot::set_viewport`](crate::SvgRoot::set_viewport)
///   directly instead. The caller already has a live handle, so no URL fragment is needed.
///
/// `<view>` is therefore useful primarily when an SVG document is exported, or embedded/navigated independently of
/// any running WASM code. It is not for switching the viewport of the very SVG a running WASM instance is attached
/// to.
///
/// Obtain one from [`SvgDefs::view`](crate::SvgDefs::view) or [`SvgDefs::build_view`](crate::SvgDefs::build_view).
///
/// # Example
///
/// ```rust,no_run
/// use svg_dom::{SvgRoot, root::utils::{Point, Size}};
///
/// let svg  = SvgRoot::attach("diagram")?;
/// let defs = svg.defs()?;
///
/// // Name a zoomed-in view of the top-left quadrant.
/// let detail = defs.view("detail")?;
/// detail.set_view_box(0.0, 0.0, 50.0, 50.0)?;
///
/// // Fragment navigation only activates for an externally referenced (or standalone) copy of this document — here,
/// // an exported "diagram.svg" loaded through an <image>. Re-setting `href` re-navigates it.
/// let preview = svg.image("diagram.svg", Point::origin(), Size::new(50.0, 50.0))?;
/// preview.set_href("diagram.svg#detail")?;
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
    // `SvgView` does not offer child-element construction. SVG permits descriptive children such as <title>/<desc>
    // on <view>, but nothing in this crate's own API needs to build them. Unlike its sibling id-cached elements
    // (`SvgSymbol`, `SvgMarker`, ...), it therefore never needs a `Document` handle to create any, so none is stored.
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
    /// Embed it in an external URL fragment (`"diagram.svg#" + id`) or a standalone SVG document's own
    /// same-document fragment link — see the [type-level docs](Self) for which cases fragment navigation actually
    /// activates for.
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
    /// element, an external `<img src>`, ...). Those store a snapshot of the id at the time the reference was
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
    /// Avoid writing the `id` attribute through this handle. Use [`set_id`](Self::set_id) instead so the cached
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
    /// or `height` is negative. A `width`/`height` of exactly `0.0` is accepted. As per the SVG spec, it is a trick
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
    /// This is the generic escape hatch for attributes not covered by the named setters above. Name and value are
    /// written verbatim, so do not pass untrusted input!
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
    /// Passing `"id"` (matched case-insensitively) returns [`Error::ReservedAttribute`].
    /// Use [`set_id`](Self::set_id) instead.
    pub fn set_attr_display<T: std::fmt::Display>(&self, name: &str, value: T) -> Result<(), Error> {
        if name.eq_ignore_ascii_case("id") {
            return Err(Error::ReservedAttribute("id"));
        }
        self.attrs.borrow_mut().display_element(&self.element, name, value)
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl super::defs::SvgDefs {
    /// Creates a `<view>` child element with the given `id`, immediately appends it to `<defs>` then returns its
    /// handle.
    ///
    /// Unlike most elements built by `SvgDefs`, `<view>` has no rendered graphical content of its own. Instead, you call
    /// [`SvgView::set_view_box`](crate::SvgView::set_view_box) on the returned handle, then reference it (prefixed
    /// with `#`) from an external or standalone-document URL fragment. See [`SvgView`]'s type-level docs for exactly
    /// which cases that fragment navigation activates for.
    ///
    /// Prefer [`build_view`](Self::build_view) when the `viewBox` is known upfront. That variant keeps the element
    /// detached from the DOM until the closure succeeds, so a rejected `viewBox` does not leave a partial `<view>`
    /// in `<defs>`.
    ///
    /// # Errors
    ///
    /// - [`Error::InvalidViewId`] — `id` failed validation.
    /// - [`Error::Dom`] — the browser refused to create or append the element.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::utils::{Point, Size}};
    ///
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    /// let view = defs.view("detail")?;
    /// view.set_view_box(0.0, 0.0, 50.0, 50.0)?;
    ///
    /// // Fragment navigation only activates for an externally referenced (or standalone) copy of this document.
    /// let preview = svg.image("diagram.svg", Point::origin(), Size::new(50.0, 50.0))?;
    /// preview.set_href("diagram.svg#detail")?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn view(&self, id: &str) -> Result<SvgView, Error> {
        super::defs::validate_view_id(id)?;
        let el = super::create_svg_element::<SvgElement>(self.document(), "view", "SvgElement")?;
        el.set_attribute("id", id).map_err(dom_err)?;
        self.as_element().append_child(&el).map_err(dom_err)?;
        Ok(SvgView::new(id.to_owned(), el))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Builds a `<view>` in one shot, appending to `<defs>` only after the closure succeeds.
    ///
    /// The closure receives a reference to the new [`SvgView`].
    /// If the closure returns `Ok(())`, the view is appended to `<defs>` and the handle is returned.
    /// If the closure returns `Err`, the element is dropped without being attached to `<defs>`.
    ///
    /// # Errors
    ///
    /// - Any error returned by `build`.
    /// - [`Error::InvalidViewId`] — `id` failed validation.
    /// - [`Error::Dom`] — the browser refused to create or append the element.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::utils::{Point, Size}};
    ///
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    /// defs.build_view("detail", |v| v.set_view_box(0.0, 0.0, 50.0, 50.0))?;
    ///
    /// // Fragment navigation only activates for an externally referenced (or standalone) copy of this document.
    /// let preview = svg.image("diagram.svg", Point::origin(), Size::new(50.0, 50.0))?;
    /// preview.set_href("diagram.svg#detail")?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn build_view<F>(&self, id: &str, build: F) -> Result<SvgView, Error>
    where
        F: FnOnce(&SvgView) -> Result<(), Error>,
    {
        super::defs::validate_view_id(id)?;
        let el = super::create_svg_element::<SvgElement>(self.document(), "view", "SvgElement")?;
        el.set_attribute("id", id).map_err(dom_err)?;
        let view = SvgView::new(id.to_owned(), el);
        build(&view)?;
        self.as_element().append_child(view.as_element()).map_err(dom_err)?;
        Ok(view)
    }
}
