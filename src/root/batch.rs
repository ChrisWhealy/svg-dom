use crate::{root::{utils::{Point, Size}, SVG_NS}, Error, SvgNode, SvgRoot};

use wasm_bindgen::JsCast;
use web_sys::{Document, DocumentFragment, SvgElement, SvgsvgElement};

/// Builds several SVG elements in a [`DocumentFragment`] and appends them to the root in one DOM operation.
///
/// Create a batch with [`SvgRoot::batch`], call the same element factory methods you would normally call on
/// [`SvgRoot`], then call [`commit`](Self::commit).  Each factory returns a live [`SvgNode`] handle immediately, but
/// the element is not attached to the rendered SVG tree until the batch is committed.
///
/// This is useful when constructing many elements at once: attributes and text content are set while each element is
/// detached, and the whole fragment is appended to the `<svg>` root once.
pub struct SvgBatch {
    root: SvgsvgElement,
    document: Document,
    fragment: DocumentFragment,
}

impl SvgBatch {
    pub(crate) fn new(root: SvgsvgElement, document: Document, fragment: DocumentFragment) -> Self {
        Self {
            root,
            document,
            fragment,
        }
    }

    pub(crate) fn make_element(&self, tag: &str) -> Result<SvgElement, Error> {
        self.document
            .create_element_ns(Some(SVG_NS), tag)
            .map_err(|e| Error::Dom(format!("{e:?}")))?
            .dyn_into::<SvgElement>()
            .map_err(|_| Error::CastFailed("SvgElement"))
    }

    fn append_node(&self, node: &SvgNode) -> Result<(), Error> {
        self.fragment
            .append_child(node.as_element())
            .map(|_| ())
            .map_err(|e| Error::Dom(format!("{e:?}")))
    }

    /// Appends the whole batch to the SVG root in a single DOM operation.
    pub fn commit(self) -> Result<(), Error> {
        self.root
            .append_child(&self.fragment)
            .map(|_| ())
            .map_err(|e| Error::Dom(format!("{e:?}")))
    }

    /// Creates a detached `<rect>` element in this batch and returns its [`SvgNode`] handle.
    pub fn rect(&self, top_left: Point, size: Size) -> Result<SvgNode, Error> {
        let n = self.make_node("rect")?;
        n.set_attr("x", &top_left.get_x_str())?;
        n.set_attr("y", &top_left.get_y_str())?;
        n.set_attr("width", &size.get_width_str())?;
        n.set_attr("height", &size.get_height_str())?;
        self.append_node(&n)?;
        Ok(n)
    }

    /// Creates a detached `<circle>` element in this batch and returns its [`SvgNode`] handle.
    pub fn circle(&self, centre: Point, radius: f64) -> Result<SvgNode, Error> {
        let n = self.make_node("circle")?;
        n.set_attr("cx", &centre.get_x_str())?;
        n.set_attr("cy", &centre.get_y_str())?;
        n.set_attr("r", &radius.to_string())?;
        self.append_node(&n)?;
        Ok(n)
    }

    /// Creates a detached `<line>` element in this batch and returns its [`SvgNode`] handle.
    pub fn line(&self, start: Point, end: Point) -> Result<SvgNode, Error> {
        let n = self.make_node("line")?;
        n.set_attr("x1", &start.get_x_str())?;
        n.set_attr("y1", &start.get_y_str())?;
        n.set_attr("x2", &end.get_x_str())?;
        n.set_attr("y2", &end.get_y_str())?;
        self.append_node(&n)?;
        Ok(n)
    }

    /// Creates a detached `<path>` element in this batch and returns its [`SvgNode`] handle.
    pub fn path(&self, d: &str) -> Result<SvgNode, Error> {
        let n = self.make_node("path")?;
        n.set_attr("d", d)?;
        self.append_node(&n)?;
        Ok(n)
    }

    /// Creates a detached `<text>` element in this batch and returns its [`SvgNode`] handle.
    pub fn text(&self, anchored_at: Point, content: &str) -> Result<SvgNode, Error> {
        let n = self.make_node("text")?;
        n.set_attr("x", &anchored_at.get_x_str())?;
        n.set_attr("y", &anchored_at.get_y_str())?;
        n.as_element().set_text_content(Some(content));
        self.append_node(&n)?;
        Ok(n)
    }

    /// Creates a detached `<g>` element in this batch and returns its [`SvgNode`] handle.
    pub fn group(&self) -> Result<SvgNode, Error> {
        let n = self.make_node("g")?;
        self.append_node(&n)?;
        Ok(n)
    }

    fn make_node(&self, tag: &str) -> Result<SvgNode, Error> {
        self.make_element(tag).map(SvgNode::new)
    }
}

impl SvgRoot {
    /// Creates a batch builder backed by a browser [`DocumentFragment`].
    ///
    /// Elements created through the returned [`SvgBatch`] are appended to the fragment first, not directly to the
    /// rendered `<svg>`.  Calling [`SvgBatch::commit`] appends the fragment to the root once, moving all batched
    /// children into the live SVG tree.
    /// 
    /// This way, if an entire tree of elements needs to be added, the browser does not see repeated DOM mutations.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::utils::{Point, Size}};
    ///
    /// let svg = SvgRoot::attach("diagram")?;
    /// let batch = svg.batch();
    /// let rect = batch.rect(Point::origin(), Size::new(80.0, 40.0))?;
    /// let text = batch.text(Point::new(8.0, 26.0), "XOR")?;
    /// rect.set_fill("steelblue")?;
    /// text.set_fill("white")?;
    /// batch.commit()?;
    /// # Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn batch(&self) -> SvgBatch {
        SvgBatch::new(
            self.root.clone(),
            self.document.clone(),
            self.document.create_document_fragment(),
        )
    }

    /// Builds and commits a [`SvgBatch`] in one call.
    ///
    /// If the closure returns an error, the fragment is dropped without being appended to the root.
    pub fn build_batch<F>(&self, build: F) -> Result<(), Error>
    where
        F: FnOnce(&SvgBatch) -> Result<(), Error>,
    {
        let batch = self.batch();
        build(&batch)?;
        batch.commit()
    }
}
