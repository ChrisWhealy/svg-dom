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
    /// `<foreignObject>` defines a rectangular containing block in SVG user space within which foreign (typically
    /// HTML) content is laid out by the browser's own engine — CSS text flow/wrapping, form controls, and other HTML
    /// features that SVG's own text and shape model does not provide.
    ///
    /// `top_left` and `size` define that rectangle in the current user-space coordinate system, exactly like
    /// [`SvgRoot::rect`](crate::SvgRoot::rect)/[`SvgRoot::image`](crate::SvgRoot::image).
    ///
    /// # This is a containing block, not an unconditional clip
    ///
    /// The rectangle establishes the viewport and CSS containing block for the foreign content. Browsers clip to it
    /// by default — `<foreignObject>` gets `overflow: hidden` from the UA stylesheet, the same as `<svg>`/`<symbol>`/
    /// `<marker>`/`<pattern>`, the other elements that establish a new viewport — but that is an ordinary,
    /// overridable CSS property, not a structural guarantee this crate or SVG's rendering model enforces. Content
    /// set to `overflow: visible` (via [`SvgNode::set_attr`](crate::SvgNode::set_attr)) can still paint outside the
    /// rectangle.
    ///
    /// # No content-setting method — by design
    ///
    /// This factory returns an *empty* `<foreignObject>`; there is no `set_inner_html` or `set_content` method to fill
    /// it. This is not a missing feature, rather it is a deliberate design decision to limit the crate's public API
    /// surface.
    ///
    /// A string-based HTML convenience method would need to parse caller-supplied markup (typically via `innerHTML` or
    /// an equivalent browser parsing API). However, parsing arbitrary markup means this crate must takw on sanitisation
    /// and trust concerns that it has no business maintaining. No part of this crate's public API parses a string as
    /// markup anywhere (this crate's top-level documentation states that guarantee explicitly), and this factory does
    /// not make an exception for `<foreignObject>`.
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
    /// `<foreignObject>` is universally supported in every browser this crate targets.
    ///
    /// `requiredExtensions` is still part of SVG 2's conditional processing and is applicable to `<foreignObject>`
    /// (among other elements) for declaring that a particular foreign-language extension is required.
    ///
    /// It is a general conditional-processing attribute but has not been modelled specially here.
    ///
    /// It remains available through [`SvgNode::set_attr`](crate::SvgNode::set_attr), the same as on
    /// [`SvgRoot::switch`](crate::SvgRoot::switch)'s children.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if the browser refuses to create or append the `<foreignObject>` element.
    pub fn foreign_object(&self, top_left: Point, size: Size) -> Result<SvgNode, Error> {
        self.create_foreign_object(top_left, size)
    }
}
