use crate::{SvgRoot, error::Error, node::SvgNode, root::factory::SvgFactory};

impl SvgRoot {
    /// Creates a `<metadata>` element containing `content`, appends it to the root, and returns its [`SvgNode`]
    /// handle.
    ///
    /// `<metadata>` holds machine-readable information about the document — conventionally an RDF/Dublin Core
    /// description, though SVG permits any content there. It is never rendered: browsers skip it entirely when
    /// painting, and unlike [`SvgRoot::set_title`](crate::SvgRoot::set_title)/
    /// [`SvgRoot::set_desc`](crate::SvgRoot::set_desc), it has no accessibility role either. It is not consumed
    /// automatically by the browser's rendering or accessibility pipelines, but it remains an ordinary part of the
    /// DOM — reachable via `textContent`, selectors, or tree traversal like any other element — and stays present in
    /// the serialized document for external tooling to read.
    ///
    /// `content` is written as the element's text content via [`SvgNode::set_text`](crate::SvgNode::set_text) — a
    /// genuine DOM text node, not parsed markup, so no HTML entity-escaping is needed for `<`/`>`/`&`. `metadata`
    /// never parses `content` as markup: a string that looks like XML is stored and later serialized as literal
    /// escaped text, not parsed into child nodes.
    ///
    /// The returned [`SvgNode`] is otherwise ordinary and can still be built out afterwards with this crate's
    /// generic tree APIs (`append`, `insert_before`, `clear`, ...) — those work on any element, `<metadata>`
    /// included. What this crate does not provide is a namespace-aware *factory* for foreign-namespace elements such
    /// as `rdf:RDF` or Dublin Core terms; those still require the raw DOM.
    ///
    /// SVG 2 illustrates structured metadata using an RDF/Dublin Core graph built from namespaced `<rdf:RDF>`/
    /// Dublin Core child elements. This is one common foreign-namespace representation, but SVG 2 does not
    /// prescribe any particular metadata vocabulary or structure. If you need one, reach for the raw DOM via
    /// [`SvgNode::as_element`](crate::SvgNode::as_element) — already a first-class, intentional escape hatch in this
    /// crate, not a fallback of last resort:
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
    /// Writing `content` as a text node means it cannot execute script or affect rendering in this browser session —
    /// unlike [`SvgRoot::style`](crate::SvgRoot::style)'s `css`, nothing here interprets it live. The residual risk
    /// is downstream: if this SVG is later exported and opened by a different tool (another renderer, an RDF
    /// processor, a search indexer, ...), that tool may parse and act on `<metadata>` content in ways this crate
    /// cannot anticipate. Do not embed attacker-controlled content without considering how it might be interpreted
    /// wherever the exported file ends up.
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
