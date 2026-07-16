use crate::{SvgNode, dom_err, error::Error};
use wasm_bindgen::JsCast;
use web_sys::SvgElement;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl SvgNode {
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Appends `child` as a DOM child of this node.
    ///
    /// Use this to build up groups: create a `<g>` with [`SvgRoot::group`](crate::SvgRoot::group), then call `append` to move individual
    /// elements inside it.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let group = svg.group()?;
    /// let rect = svg.rect(Point::new(0.0, 0.0), Size::new(80.0, 40.0))?;
    /// let label = svg.text(Point::new(8.0, 26.0), "XOR")?;
    ///
    /// group.append(&rect)?;
    /// group.append(&label)?;
    ///
    /// // Moving the group moves both children.
    /// group.set_attr("transform", "translate(100, 50)")?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn append(&self, child: &SvgNode) -> Result<(), Error> {
        self.inner
            .element
            .append_child(&child.inner.element)
            .map(|_| ())
            .map_err(dom_err)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Detaches this node from its parent in the DOM.
    ///
    /// The `SvgNode` handle remains valid after removal — it simply points at an element that is no longer part of the
    /// document tree, so it can be re-inserted later with [`append`](Self::append) or [`insert_before`](Self::insert_before).
    ///
    /// Any managed event listeners stay registered on the (now detached) element and are still removed when the last
    /// handle is dropped.
    ///
    /// Removing a node is idempotent. That is, removing an already-detached node or a node that was never attached is a
    /// harmless no-op.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let rect = svg.rect(Point::new(0.0, 0.0), Size::new(40.0, 40.0))?;
    /// rect.remove(); // the <rect> is taken out of the document
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn remove(&self) {
        self.inner.element.remove();
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Inserts the `SvgNode` called `new_child` immediately before the existing `SvgNode` called `reference`.
    ///
    /// This is the tree operation for **z-order control**: SVG paints children in document order, so inserting a node
    /// before an existing sibling places it *behind* that sibling without rebuilding the rest of the tree.
    ///
    /// To have the new child appear at the top of the visibility stack, use [`append`](Self::append) instead.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if `reference` is not a child of this node, mirroring the underlying `Node.insertBefore`
    /// DOM call.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let group = svg.group()?;
    /// let front = svg.rect(Point::new(0.0, 0.0), Size::new(40.0, 40.0))?;
    /// group.append(&front)?;
    ///
    /// // Slip a new rect behind `front` in the group's paint order.
    /// let behind = svg.rect(Point::new(10.0, 10.0), Size::new(40.0, 40.0))?;
    /// group.insert_before(&behind, &front)?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn insert_before(&self, new_child: &SvgNode, reference: &SvgNode) -> Result<(), Error> {
        let reference_node: &web_sys::Node = &reference.inner.element;
        self.inner
            .element
            .insert_before(&new_child.inner.element, Some(reference_node))
            .map(|_| ())
            .map_err(dom_err)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Removes all child nodes of this node, leaving it empty. On a `<text>` node this clears the text.
    ///
    /// This is the bulk counterpart to [`remove`](Self::remove): the idiomatic way to wipe a container such as a `<g>`
    /// before rebuilding its contents. Any `SvgNode` handles the caller still holds for the removed children remain
    /// valid but detached.
    ///
    /// Calling [`clear`](Self::clear) is idempotent. That is, calling it on a node that has no children is a harmless
    /// no-op.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let group = svg.group()?;
    /// group.append(&svg.rect(Point::origin(), Size::new(10.0, 10.0))?)?;
    /// group.append(&svg.circle(Point::new(20.0, 20.0), 5.0)?)?;
    ///
    /// group.clear(); // the <g> is now empty, ready to be rebuilt
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn clear(&self) {
        // Setting text content to nothing detaches every existing child node.
        self.inner.element.set_text_content(None);
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Remove the current node in the DOM and replace it with `replacement`, which then occupies the same sibling
    /// position as the node it replaced.
    ///
    /// Use this to swap one element for another without disturbing the surrounding paint order. After the call this
    /// node is detached (its handle remains valid) and `replacement` occupies its former place.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if this node has no parent, since a detached or root node cannot be replaced in place.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let placeholder = svg.rect(Point::origin(), Size::new(40.0, 40.0))?;
    ///
    /// // Swap the placeholder rect for a circle in the same spot.
    /// let circle = svg.circle(Point::new(20.0, 20.0), 20.0)?;
    /// placeholder.replace_with(&circle)?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn replace_with(&self, replacement: &SvgNode) -> Result<(), Error> {
        let parent = self
            .inner
            .element
            .parent_node()
            .ok_or_else(|| Error::Dom("cannot replace a node that has no parent".into()))?;
        parent
            .replace_child(&replacement.inner.element, &self.inner.element)
            .map(|_| ())
            .map_err(dom_err)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Returns a handle to this node's parent element, or `None` if it either has no parent or the parent is not an SVG
    /// element.
    ///
    /// `None` is returned when either:
    /// - the node is detached (it currently has no parent), or
    /// - the parent exists but is not an SVG element - for example the root `<svg>`, whose parent is the surrounding
    ///   HTML container, not another SVG element.
    ///
    /// # ⚠️ Caveat ⚠️
    ///
    /// The returned handle is **not** a factory handle!
    ///
    /// Every other [`SvgNode`] you hold was produced by a factory method ([`SvgRoot::rect`](crate::SvgRoot::rect) and
    /// friends) or is a [`clone`](Self::clone) of one. The handle returned here is different in kind: it wraps an
    /// element that `svg-dom` almost certainly did **not** create, so it is a brand-new, *independent owner* of that
    /// element rather than another reference to an existing owner.
    ///
    /// This fact has practical and potentially significant consequences:
    ///
    /// - **Its managed-listener storage is empty.**
    ///
    ///   Managed event listeners (the `on_*` helpers) are tracked per *handle lineage* — a handle together with its
    ///   clones — and **not** per DOM element. This handle therefore does not share listener storage with whatever
    ///   handle originally created or manages the parent, and it cannot see, remove, or otherwise interact with any
    ///   listeners that were registered through those other handles.
    ///
    /// - **If you register a listener through this handle, this handle owns it**, with the usual lifetime: the listener
    ///   is detached when the last clone of *this* handle is dropped. So, just as for a factory handle, you must keep
    ///   this handle alive (store it somewhere lasting) if you want a listener registered on it to persist.
    ///
    /// - It is otherwise an ordinary handle: it points at the same live DOM element, so reading or mutating its
    ///   attributes and text takes effect immediately and is visible through any other handle to that element.
    ///
    /// Consequently, treat `parent()` as **read-only navigation** — for example, walking up to a containing `<g>`
    /// from inside an event callback to read or modify its attributes.
    ///
    /// **IMPORTANT**
    ///
    /// Do **not** register listeners through a handle obtained from `parent()`: those listeners are invisible to,
    /// and are not cleaned up by, any factory handle for the same element.
    /// Where you can, keep and reuse the factory handles you already hold instead.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let group = svg.group()?;
    /// let rect = svg.rect(Point::origin(), Size::new(40.0, 40.0))?;
    /// group.append(&rect)?;
    ///
    /// // Walk up to the containing <g>. Note this is a fresh, independent handle to that element.
    /// if let Some(parent) = rect.parent() {
    ///     parent.set_attr("transform", "translate(10, 10)")?;
    /// }
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn parent(&self) -> Option<SvgNode> {
        self.inner
            .element
            .parent_node()
            .and_then(|n| n.dyn_into::<SvgElement>().ok())
            .map(SvgNode::new)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Returns a handle to this node's first child element, or `None` if it has none.
    ///
    /// `None` is also returned when the first child exists but is not an SVG element — for example HTML content inside
    /// a `<foreignObject>`. This mirrors [`parent`](Self::parent)'s treatment of a non-SVG parent: the method does not
    /// search further for a usable sibling, it simply reports that no suitable element exists at that position.
    ///
    /// # ⚠️ Caveat ⚠️
    ///
    /// Like [`parent`](Self::parent), the returned handle is a **fresh, independent** owner of the child element — see
    /// [`parent`](Self::parent)'s doc comment for the full consequences, in particular that you should **not** register
    /// listeners through it.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let group = svg.group()?;
    /// group.append(&svg.rect(Point::origin(), Size::new(10.0, 10.0))?)?;
    ///
    /// if let Some(first) = group.first_child() {
    ///     first.set_fill("steelblue")?;
    /// }
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn first_child(&self) -> Option<SvgNode> {
        self.inner
            .element
            .first_element_child()
            .and_then(|el| el.dyn_into::<SvgElement>().ok())
            .map(SvgNode::new)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Returns a handle to this node's last child element, or `None` if it has none.
    ///
    /// See [`first_child`](Self::first_child) for the non-SVG-child and independent-handle caveats, which apply
    /// identically here.
    pub fn last_child(&self) -> Option<SvgNode> {
        self.inner
            .element
            .last_element_child()
            .and_then(|el| el.dyn_into::<SvgElement>().ok())
            .map(SvgNode::new)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Returns a handle to this node's next sibling element, or `None` if it is the last child of its parent (or is
    /// itself detached).
    ///
    /// See [`first_child`](Self::first_child) for the non-SVG-sibling and independent-handle caveats, which apply
    /// identically here.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let group = svg.group()?;
    /// group.append(&svg.rect(Point::origin(), Size::new(10.0, 10.0))?)?;
    /// group.append(&svg.circle(Point::new(20.0, 20.0), 5.0)?)?;
    ///
    /// // Walk every child from the first, without having kept a handle to each one.
    /// let mut current = group.first_child();
    /// while let Some(node) = current {
    ///     node.set_stroke("white")?;
    ///     current = node.next_sibling();
    /// }
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn next_sibling(&self) -> Option<SvgNode> {
        self.inner
            .element
            .next_element_sibling()
            .and_then(|el| el.dyn_into::<SvgElement>().ok())
            .map(SvgNode::new)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Returns a handle to this node's previous sibling element, or `None` if it is the first child of its parent
    /// (or is itself detached).
    ///
    /// See [`first_child`](Self::first_child) for the non-SVG-sibling and independent-handle caveats, which apply
    /// identically here.
    pub fn previous_sibling(&self) -> Option<SvgNode> {
        self.inner
            .element
            .previous_element_sibling()
            .and_then(|el| el.dyn_into::<SvgElement>().ok())
            .map(SvgNode::new)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Returns a handle to every child element of this node, in document order.
    ///
    /// Only SVG element children are included; text nodes, comments, and any non-SVG element child (for example, HTML
    /// content inside a `<foreignObject>`) are all silently skipped, so the returned `Vec` can be shorter than the
    /// node's actual DOM child count.
    ///
    /// Use [`SvgNode::as_element`] and the raw `web_sys` API if you need to see those too.
    ///
    /// # ⚠️ Caveat ⚠️
    ///
    /// Like [`parent`](Self::parent), every returned handle is a **fresh, independent** owner of its element — see
    /// `parent`'s doc comment for the full consequences, in particular that you should **not** register listeners
    /// through any of them.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let group = svg.group()?;
    /// group.append(&svg.rect(Point::origin(), Size::new(10.0, 10.0))?)?;
    /// group.append(&svg.circle(Point::new(20.0, 20.0), 5.0)?)?;
    ///
    /// // Recolour every child without having kept a handle to any of them.
    /// for child in group.children() {
    ///     child.set_fill("coral")?;
    /// }
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn children(&self) -> Vec<SvgNode> {
        let collection = self.inner.element.children();
        (0..collection.length())
            .filter_map(|i| collection.item(i))
            .filter_map(|el| el.dyn_into::<SvgElement>().ok())
            .map(SvgNode::new)
            .collect()
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Returns a handle to the first descendant of this node that matches the CSS `selectors` string, or `None` if
    /// there is no match.
    ///
    /// This is a thin wrapper over the browser's own `Element.querySelector`, so the full CSS selector syntax is
    /// available — including attribute selectors (`"[data-role='target']"`), which is the crate's answer to "find a
    /// node by attribute" until (or instead of) a dedicated Rust-side query API is built.
    ///
    /// If the match exists but is not an SVG element (for example HTML content inside a `<foreignObject>`), this
    /// returns `Ok(None)` rather than the match, the same non-search-further behaviour as
    /// [`first_child`](Self::first_child).
    ///
    /// # ⚠️ Caveat ⚠️
    ///
    /// Like [`parent`](Self::parent), the returned handle is a **fresh, independent** owner of the matched element —
    /// see `parent`'s doc comment for the full consequences, in particular that you should not register listeners
    /// through it.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if `selectors` is not valid CSS selector syntax.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let group = svg.group()?;
    /// let target = svg.rect(Point::origin(), Size::new(10.0, 10.0))?;
    /// target.set_attr("data-role", "target")?;
    /// group.append(&target)?;
    ///
    /// if let Some(found) = group.query_selector("[data-role='target']")? {
    ///     found.set_stroke("gold")?;
    /// }
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn query_selector(&self, selectors: &str) -> Result<Option<SvgNode>, Error> {
        let matched = self.inner.element.query_selector(selectors).map_err(dom_err)?;
        Ok(matched.and_then(|el| el.dyn_into::<SvgElement>().ok()).map(SvgNode::new))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Returns a handle to every descendant of this node that matches the CSS `selectors` string, in document order.
    ///
    /// See [`query_selector`](Self::query_selector) for the selector syntax and error behaviour.
    /// As with [`children`](Self::children), any match that is not an SVG element is silently skipped rather than
    /// included, so the returned `Vec` can be shorter than the DOM match count.
    ///
    /// # ⚠️ Caveat ⚠️
    ///
    /// Like [`parent`](Self::parent), every returned handle is a **fresh, independent** owner of its element — see
    /// `parent`'s doc comment for the full consequences, in particular that you should not register listeners
    /// through any of them.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if `selectors` is not valid CSS selector syntax.
    pub fn query_selector_all(&self, selectors: &str) -> Result<Vec<SvgNode>, Error> {
        let matches = self.inner.element.query_selector_all(selectors).map_err(dom_err)?;
        let nodes = (0..matches.length())
            .filter_map(|i| matches.item(i))
            .filter_map(|n| n.dyn_into::<SvgElement>().ok())
            .map(SvgNode::new)
            .collect();
        Ok(nodes)
    }
}
