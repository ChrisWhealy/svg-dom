use crate::{
    SvgRoot,
    error::Error,
    node::SvgNode,
    root::{
        factory::SvgFactory,
        utils::{Point, Size},
    },
};

impl SvgRoot {
    /// Creates an empty `<foreignObject>` element, appends it to the root, and returns its [`SvgNode`] handle.
    ///
    /// `<foreignObject>` carves out a rectangular region of the SVG canvas in which the browser renders foreign
    /// (typically HTML) content using its own layout engine — CSS text flow/wrapping, form controls, and other HTML
    /// features that SVG's own text and shape model does not provide.
    ///
    /// `top_left` and `size` define that rectangle in the current user-space coordinate system, exactly like
    /// [`SvgRoot::rect`](crate::SvgRoot::rect)/[`SvgRoot::image`](crate::SvgRoot::image).
    ///
    /// # No content-setting method — by design
    ///
    /// This factory returns an *empty* `<foreignObject>`; there is no `set_inner_html`/`set_content` method to fill
    /// it. That is a deliberate limit on this crate's public surface, not a missing feature. The whole point of
    /// `<foreignObject>` is to hold real HTML — flowing paragraphs, `<div>`s, form controls — and the only DOM
    /// operation that inserts markup like that is `innerHTML`, which no part of this crate's public API uses
    /// anywhere (this crate's top-level documentation states that guarantee explicitly). Adding a convenience method
    /// here would mean either quietly breaking it, or shipping an HTML sanitizer this crate has no business
    /// maintaining.
    ///
    /// To add content, reach for the raw DOM via [`SvgNode::as_element`](crate::SvgNode::as_element) — already a
    /// first-class, intentional escape hatch in this crate, not a fallback of last resort:
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::utils::{Point, Size}};
    ///
    /// let svg = SvgRoot::attach("diagram")?;
    /// let fo  = svg.foreign_object(Point::new(10.0, 10.0), Size::new(200.0, 80.0))?;
    ///
    /// let document = fo.as_element().owner_document().expect("foreignObject element has no owner document");
    /// // The HTML namespace, not SVG's. `document.create_element` (no namespace) would also land in the HTML
    /// // namespace here, but `create_element_ns` makes that explicit rather than relying on it.
    /// let div = document
    ///     .create_element_ns(Some("http://www.w3.org/1999/xhtml"), "div")
    ///     .expect("createElementNS failed");
    /// div.set_text_content(Some("Flows and wraps like ordinary HTML, unlike SVG <text>."));
    /// fo.as_element().append_child(&div).expect("appendChild failed");
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    ///
    /// # Content is opaque to this crate's tree-navigation methods
    ///
    /// [`SvgNode::first_child`](crate::SvgNode::first_child), [`children`](crate::SvgNode::children),
    /// [`query_selector`](crate::SvgNode::query_selector), and their siblings only ever return genuine SVG elements
    /// — HTML content inside a `<foreignObject>` is silently skipped, the same as any other non-SVG node they might
    /// encounter. This is existing, general behaviour that every one of those methods already documents; it is
    /// called out here because `<foreignObject>` is the element it is actually reachable on. Use
    /// [`as_element`](crate::SvgNode::as_element) and the raw `web_sys` API if you need to see that content too.
    ///
    /// # Browser support
    ///
    /// `<foreignObject>` is universally supported in every browser this crate targets. SVG 1.1's
    /// `requiredExtensions` escape hatch, for engines that could not render it at all, addresses a compatibility
    /// problem that no longer exists; this crate does not model it.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if the browser refuses to create or append the `<foreignObject>` element.
    pub fn foreign_object(&self, top_left: Point, size: Size) -> Result<SvgNode, Error> {
        self.create_foreign_object(top_left, size)
    }
}
