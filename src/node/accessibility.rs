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
    /// When neither ARIA naming attribute is present, the user agent selects an appropriate direct `<title>` child
    /// according to the SVG language-selection rules — see [`title`](Self::title) and this method's `# Scope`
    /// section for why that is not simply "the first one" once multilingual siblings are involved. In the common
    /// single-title case, that child supplies the accessible name. Separately, a `<title>` child is also what most
    /// browsers render as a native tooltip on hover.
    ///
    /// Calling this again updates the *first* `<title>` child's text (see the scope note below) rather than always
    /// creating a new one, and when there was no `<title>` child at all, the brand-new one is inserted as this
    /// element's first child, so it is unambiguously the one [`title`](Self::title) finds next time.
    ///
    /// See [`title`](Self::title) to read the current child text back, and [`remove_title`](Self::remove_title) to
    /// remove it.
    ///
    /// # Use judiciously: not every element needs a name or description!
    ///
    /// Adding a non-empty `<title>` or `<desc>` can cause an otherwise purely decorative or presentational element
    /// to be exposed to assistive technology as its own separate object in the accessibility tree. That is exactly
    /// the point for meaningful icons, controls, diagrams, and diagram components — but naming every individual
    /// decorative path or primitive produces a noisy, cumbersome accessibility tree that works against the users
    /// it is meant to help.
    ///
    /// Because [`set_title`](Self::set_title)/[`set_desc`](Self::set_desc) are generic on [`SvgNode`], they are
    /// callable on almost any element this crate hands back, which makes it easy to over-apply them. As a rule of
    /// thumb: attach `<title>`/`<desc>` to elements that are meaningful on their own — icons, controls, whole
    /// diagrams, or a `<g>` representing one logical idea — and leave purely decorative geometry (the individual
    /// paths/shapes that only exist to render a larger meaningful group) unnamed, so it is not individually exposed.
    ///
    /// A `<title>`/`<desc>` also does not, by itself, make an element interactive: it makes a graphic
    /// *describable*, not a control. If an icon is meant to be operable (clickable, focusable, activatable from the
    /// keyboard), that behaviour has to be built independently — a suitable `role`, a `tabindex`, and keyboard event
    /// handling — none of which these two methods provide.
    ///
    /// # Scope: a first-direct-child convenience, not a language-aware manager
    ///
    /// This method (and [`title`](Self::title)/[`remove_title`](Self::remove_title)) is a simple, single-value
    /// convenience for the common case: read, write, or remove whichever `<title>` happens to be this element's
    /// *first* direct `<title>` child.
    ///
    /// SVG 2 deliberately permits multiple `<title>`/`<desc>` siblings on one element, one per language, with the
    /// user agent selecting the most appropriate one via `lang`/`xml:lang`. This crate does not implement that
    /// selection, and these methods make **no attempt to keep a `<title>` singular** on an element whose DOM they
    /// did not build from scratch: if this element already has more than one `<title>` child — for example, one
    /// attached from externally authored markup, or a multilingual set built by hand — `set_title` updates only the
    /// first one, `title()` reads only the first one, and `remove_title` removes only the first one. Every other
    /// `<title>` sibling is left completely untouched by any of these three methods.
    ///
    /// Build or manage multilingual `<title>`/`<desc>` sets through the underlying DOM directly (via
    /// [`as_element`](Self::as_element), or this crate's generic tree/attribute methods) — this convenience API
    /// intentionally does not grow a `lang`-aware surface; that would be a distinct, future capability rather than
    /// an extension of what these methods already promise.
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
    /// This method reads the *first* direct `<title>` child; it does not run that computation, consult ARIA
    /// attributes, or account for language-tagged sibling `<title>`s. See [`set_title`](Self::set_title)'s
    /// `# Scope` section for the full explanation of that limitation.
    pub fn title(&self) -> Option<String> {
        self.accessible_child_text("title")
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Removes this element's *first* direct `<title>` child, if one exists.
    ///
    /// This method is idempotent: calling it has no effect if no `<title>` child is present. See
    /// [`set_title`](Self::set_title)'s `# Scope` section — if this element has more than one `<title>` sibling,
    /// only the first is removed; the rest are left untouched.
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
    /// When `aria-describedby` is not present, the user agent selects an appropriate direct `<desc>` child according
    /// to the same SVG language-selection rules noted on [`set_title`](Self::set_title). In the common single-desc
    /// case, that child supplies the accessible description. Unlike [`set_title`](Self::set_title), browsers do not
    /// render `<desc>` as a tooltip.
    ///
    /// Calling this again updates the *first* `<desc>` child's text rather than always creating a new one, and when
    /// there was no `<desc>` child at all, the brand-new one is inserted immediately after an existing `<title>` (or
    /// as the first child if there is no `<title>` either).
    ///
    /// See [`desc`](Self::desc) to read the current child text back, and [`remove_desc`](Self::remove_desc) to remove
    /// it. This method has exactly the same first-direct-child scope, and the same non-goal of language-aware
    /// management, as [`set_title`](Self::set_title) — see that method's `# Scope` section for the full explanation:
    /// if this element already has more than one `<desc>` sibling, `set_desc`/`desc`/`remove_desc` only ever act on
    /// the first one.
    ///
    /// The same "use judiciously" caution applies here too — see [`set_title`](Self::set_title)'s
    /// `# Use judiciously` section: a `<desc>` on every decorative primitive is just as noisy for assistive
    /// technology as a `<title>` on every one.
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
    /// reads the *first* direct `<desc>` child; it does not run that computation, consult ARIA attributes, or
    /// account for language-tagged sibling `<desc>`s. See [`set_title`](Self::set_title)'s `# Scope` section (which
    /// [`set_desc`](Self::set_desc) shares) for the full explanation of that limitation.
    pub fn desc(&self) -> Option<String> {
        self.accessible_child_text("desc")
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Removes this element's *first* direct `<desc>` child, if one exists.
    ///
    /// Has no effect if no `<desc>` child is present. Idempotent, like the rest of this crate's `remove_*` helpers.
    /// See [`set_title`](Self::set_title)'s `# Scope` section — if this element has more than one `<desc>` sibling,
    /// only the first is removed; the rest are left untouched.
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
