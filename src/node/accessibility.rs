use crate::{SvgNode, dom_err, error::Error, root::SVG_NS};
use wasm_bindgen::JsCast;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl SvgNode {
    /// # `<title>` child element
    ///
    /// Creates or updates this element's direct `<title>` child.
    ///
    /// This `<title>` participates in the element's accessible-name computation, but does not unconditionally determine
    /// it: as per the accessible-name-and-description computation algorithm, `aria-labelledby` and `aria-label` — when
    /// present on this element — take precedence over a `<title>` child.
    ///
    /// When neither ARIA attribute is present, the first `<title>` child (in document order) supplies the accessible
    /// name. Separately, a `<title>` child is also what most browsers render as a native tooltip on hover.
    ///
    /// This method is idempotent: calling it again updates the existing `<title>` child's text rather than accumulating
    /// duplicates, and a brand-new `<title>` is inserted as this element's first child, so it is unambiguously the one
    /// [`title`](Self::title) finds, regardless of what other content this element already has.
    ///
    /// See [`title`](Self::title) to read the current child text back, and [`remove_title`](Self::remove_title) to
    /// remove it entirely.
    ///
    /// # Errors
    ///
    /// - [`Error::InvalidAccessibleText`] — `text` is empty or contains only whitespace. SVG 2 requires authoring
    ///   tools not to produce an empty or whitespace-only `<title>`, since it can suppress an otherwise-usable
    ///   accessible name derived from other content.
    /// - [`Error::Dom`] — the browser refused to create, append, or insert the `<title>` element.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::utils::{Point, Size}};
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let icon = svg.rect(Point::origin(), Size::new(24.0, 24.0))?;
    /// icon.set_title("Close dialog")?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn set_title(&self, text: &str) -> Result<(), Error> {
        self.set_accessible_child("title", text)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Returns the text of this element's first direct `<title>` child, or `None` if it has none.
    ///
    /// This is **not necessarily the element's computed accessible name** — if `aria-labelledby` or `aria-label` are
    /// present on this element, they take precedence over a `<title>` child in accessible-name computation.
    ///
    /// This method reads the `<title>` child directly; it does not run that computation or consult ARIA attributes.
    ///
    /// See [`set_title`](Self::set_title) for how that child is created and kept singular.
    pub fn title(&self) -> Option<String> {
        self.accessible_child_text("title")
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Removes this element's `<title>` child, if one exists.
    ///
    /// This method is idempotent: calling it has no effect if no `<title>` child is present.
    pub fn remove_title(&self) {
        self.remove_accessible_child("title");
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// # `<desc>` child element
    ///
    /// Creates or updates this element's direct `<desc>` child.
    ///
    /// This `<desc>` participates in the element's accessible-description computation, but does not unconditionally
    /// determine it: if present on the element, `aria-describedby` takes precedence over a `<desc>` child.
    ///
    /// When `aria-describedby` is not present, the first `<desc>` child (in document order) supplies the accessible
    /// description. Unlike [`set_title`](Self::set_title), browsers do not render `<desc>` as a tooltip.
    ///
    /// The same singular-child, first-position behaviour applies: calling this again updates the existing `<desc>` in
    /// place rather than accumulating duplicates.
    ///
    /// See [`desc`](Self::desc) to read the current child text back, and [`remove_desc`](Self::remove_desc) to remove
    /// it.
    ///
    /// # Errors
    ///
    /// - [`Error::InvalidAccessibleText`] — `text` is empty or contains only whitespace. SVG 2 requires authoring
    ///   tools not to produce an empty or whitespace-only `<desc>`.
    /// - [`Error::Dom`] — the browser refused to create, append, or insert the `<desc>` element.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::utils::{Point, Size}};
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let icon = svg.rect(Point::origin(), Size::new(24.0, 24.0))?;
    /// icon.set_title("Close dialog")?;
    /// icon.set_desc("Discards unsaved changes and closes this dialog.")?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn set_desc(&self, text: &str) -> Result<(), Error> {
        self.set_accessible_child("desc", text)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Returns the text of this element's first direct `<desc>` child, or `None` if it has none.
    ///
    /// This is **not necessarily the element's computed accessible description** — `aria-describedby`, when present
    /// on this element, takes precedence over a `<desc>` child in accessible-description computation. This method
    /// reads the `<desc>` child directly; it does not run that computation or consult ARIA attributes.
    ///
    /// See [`set_desc`](Self::set_desc) for how that child is created and kept singular.
    pub fn desc(&self) -> Option<String> {
        self.accessible_child_text("desc")
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Removes this element's `<desc>` child, if one exists.
    ///
    /// Has no effect if no `<desc>` child is present. Idempotent, like the rest of this crate's `remove_*` helpers.
    pub fn remove_desc(&self) {
        self.remove_accessible_child("desc");
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // Shared implementation behind set_title/set_desc, title/desc, and remove_title/remove_desc. `tag` is always the
    // literal `"title"` or `"desc"` from one of the six public methods above, never caller-supplied.
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

    fn set_accessible_child(&self, tag: &'static str, text: &str) -> Result<(), Error> {
        // SVG 2 requires authoring tools not to produce an empty or whitespace-only <title>/<desc>, since either can
        // expose an apparently nameless object to assistive technology (or, for <title>, suppress an otherwise-usable
        // accessible name derived from other content). Reject outright rather than silently creating a blank element
        // or reinterpreting this as a removal request.
        if text.trim().is_empty() {
            return Err(Error::InvalidAccessibleText(tag));
        }

        if let Some(existing) = self.accessible_child(tag)? {
            existing.set_text(text);
            return Ok(());
        }

        let doc = self
            .inner
            .element
            .owner_document()
            .ok_or_else(|| Error::Dom(format!("set_{tag}: element has no owner document")))?;
        let el = doc
            .create_element_ns(Some(SVG_NS), tag)
            .map_err(dom_err)?
            .dyn_into::<web_sys::SvgElement>()
            .map_err(|_| Error::CastFailed("SvgElement"))?;
        let node = SvgNode::new(el);
        node.set_text(text);

        // <title> always becomes this element's first child, so it is unambiguously the one title()/accessible-name
        // computation finds (when not superseded by aria-label/aria-labelledby), regardless of what content this
        // element already has.
        //
        // <desc> is inserted immediately after an existing <title> when there is one — matching the SVG
        // specification's own examples, and keeping the two in the conventional order when both are set, whichever
        // order the caller calls set_title/set_desc in. With no <title> present, <desc> falls back to becoming the
        // first child itself, exactly as <title> would.
        let reference = if tag == "desc" {
            match self.accessible_child("title")? {
                Some(title) => title.next_sibling(),
                None => self.first_child(),
            }
        } else {
            self.first_child()
        };

        match reference {
            Some(reference) => self.insert_before(&node, &reference)?,
            None => {
                self.inner.element.append_child(node.as_element()).map_err(dom_err)?;
            },
        }
        Ok(())
    }

    fn accessible_child(&self, tag: &'static str) -> Result<Option<SvgNode>, Error> {
        self.query_selector(&format!(":scope > {tag}"))
    }

    fn accessible_child_text(&self, tag: &'static str) -> Option<String> {
        self.accessible_child(tag)
            .ok()
            .flatten()
            .map(|node| node.as_element().text_content().unwrap_or_default())
    }

    fn remove_accessible_child(&self, tag: &'static str) {
        if let Ok(Some(existing)) = self.accessible_child(tag) {
            existing.remove();
        }
    }
}
