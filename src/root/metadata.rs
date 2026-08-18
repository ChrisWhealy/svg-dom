use crate::{SvgRoot, error::Error, node::SvgNode, root::factory::SvgFactory};

impl SvgRoot {
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<metadata>` element containing `content`, appends it to the root and returns its [`SvgNode`]
    /// handle.
    ///
    /// `<metadata>` holds machine-readable information about the document. Conventionally, this is an RDF/Dublin Core
    /// description, though SVG permits any content there.
    ///
    /// This content is never rendered: browsers skip it entirely when painting, and unlike
    /// [`SvgRoot::set_title`](crate::SvgRoot::set_title) or [`SvgRoot::set_desc`](crate::SvgRoot::set_desc), it plays
    /// no role in accessibility either. It is not consumed automatically by the browser's rendering or accessibility
    /// pipelines. It remains part of the DOM content, though, reachable via `textContent`, selectors, or tree
    /// traversal like any other element. It also stays present in the serialized document for external tooling to
    /// read.
    ///
    /// `metadata` never parses its `content` as markup, so a string that looks like XML is simply stored and later
    /// serialized as literal escaped text, not parsed into child nodes.
    ///
    /// `content` is written as the element's text content via [`SvgNode::set_text`](crate::SvgNode::set_text). This is
    /// a genuine DOM text node, not parsed markup, so no HTML entity-escaping is needed for characters such as `<`, `>`
    /// or `&` etc.
    ///
    /// The returned [`SvgNode`] is an entirely ordinary node and can be built with this crate's generic tree APIs
    /// (`append`, `insert_before`, `clear` etc) that work on any element, including `<metadata>`. What this crate does
    /// not provide is a namespace-aware *factory* for foreign-namespace elements such as `rdf:RDF` or Dublin Core terms.
    /// Building those still requires the raw DOM via [`SvgNode::as_element`](crate::SvgNode::as_element), which is
    /// already a first-class, intentional escape hatch in this crate, not a fallback of last resort.
    ///
    /// SVG 2 illustrates structured metadata using an RDF/Dublin Core graph built from namespaced `<rdf:RDF>` or Dublin
    /// Core child elements. This is one of several common foreign-namespace representations. However, SVG 2 does not
    /// prescribe any particular metadata vocabulary or structure. If you need a specific metadata vocabulary, use the
    /// raw DOM escape hatch described above.
    ///
    /// ```rust,no_run
    /// use svg_dom::SvgRoot;
    ///
    /// let svg = SvgRoot::attach("diagram")?;
    ///
    /// // An empty <metadata> (set_text("") leaves it childless) placed at the call site, then built out by hand.
    /// let metadata = svg.metadata("")?;
    /// let document = metadata.as_element().owner_document().expect("metadata element has no owner document");
    /// let rdf = document
    ///     .create_element_ns(Some("http://www.w3.org/1999/02/22-rdf-syntax-ns#"), "rdf:RDF")
    ///     .expect("createElementNS failed");
    /// metadata.as_element().append_child(&rdf).expect("appendChild failed");
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    ///
    /// # Security
    ///
    /// Unlike [`SvgRoot::style`](crate::SvgRoot::style)'s `css`, writing `content` as a text node means it cannot
    /// execute a script or affect rendering in this browser session. Nothing here interprets the contents live.
    ///
    /// There is, however, a residual downstream risk. This SVG might later be exported and opened by a different
    /// tool — another renderer, an RDF processor, or a search indexer, for example. That tool *may* parse and act
    /// on the `<metadata>` content in ways this crate cannot anticipate.
    ///
    /// Therefore, do not embed attacker-controlled content without considering how it might be interpreted after the
    /// content has been exported.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if the browser refuses to create or append the `<metadata>` element.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::SvgRoot;
    ///
    /// let svg = SvgRoot::attach("diagram")?;
    /// svg.metadata(r#"{"source": "quarterly-sales.csv", "generated": "2026-07-23"}"#)?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn metadata(&self, content: &str) -> Result<SvgNode, Error> {
        self.create_metadata(content)
    }
}
