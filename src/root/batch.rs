use crate::{
    Error, SvgNode, SvgRoot, dom_err,
    root::{
        attrs::SvgAttrs,
        factory::SvgFactory,
        path::path_def::PathDef,
        utils::{Point, Size},
    },
};
use std::cell::RefCell;
use web_sys::{Document, DocumentFragment, Node};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Builds several SVG elements in a [`DocumentFragment`] and appends them to a single parent in one DOM operation.
///
/// Create a batch with [`SvgRoot::batch`] (to build into the `<svg>` root) or [`SvgRoot::batch_into`] (to build into
/// any existing element, such as a `<g>`), call the same element factory methods you would normally call on
/// [`SvgRoot`], then call [`commit`](Self::commit).  Each factory returns a live [`SvgNode`] handle immediately, but
/// the element is not attached to the rendered SVG tree until the batch is committed.
///
/// This is useful when constructing many elements at once: attributes and text content are set while each element is
/// detached, and the whole fragment is appended to the target once. Building straight into a `<g>` this way also
/// avoids the append-to-root-then-move round-trip you would otherwise incur by creating elements on the root and
/// re-parenting them into the group with [`SvgNode::append`](crate::SvgNode::append).
#[must_use = "an SvgBatch builds elements in a detached fragment; call commit() to add them to the document"]
pub struct SvgBatch {
    target: Node,
    document: Document,
    fragment: DocumentFragment,
    attrs: RefCell<SvgAttrs>,
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl SvgBatch {
    pub(crate) fn new(target: Node, document: Document, fragment: DocumentFragment) -> Self {
        Self {
            target,
            document,
            fragment,
            attrs: RefCell::new(SvgAttrs::new()),
        }
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Appends the whole batch to its target parent in a single DOM operation.
    pub fn commit(self) -> Result<(), Error> {
        self.target.append_child(&self.fragment).map(|_| ()).map_err(dom_err)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a detached `<rect>` element in this batch and returns its [`SvgNode`] handle.
    pub fn rect(&self, top_left: Point, size: Size) -> Result<SvgNode, Error> {
        self.create_rect(top_left, size)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a detached `<circle>` element in this batch and returns its [`SvgNode`] handle.
    pub fn circle(&self, centre: Point, radius: f64) -> Result<SvgNode, Error> {
        self.create_circle(centre, radius)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a detached `<ellipse>` element in this batch and returns its [`SvgNode`] handle.
    pub fn ellipse(&self, centre: Point, radii: Size) -> Result<SvgNode, Error> {
        self.create_ellipse(centre, radii)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a detached `<line>` element in this batch and returns its [`SvgNode`] handle.
    pub fn line(&self, start: Point, end: Point) -> Result<SvgNode, Error> {
        self.create_line(start, end)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a detached `<path>` element in this batch and returns its [`SvgNode`] handle.
    pub fn path(&self, d: &str) -> Result<SvgNode, Error> {
        self.create_path(d)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a detached `<path>` element in this batch from a sequence of typed [`PathDef`] segments, and returns its
    /// [`SvgNode`] handle.
    ///
    /// The type-safe alternative to [`path`](Self::path); see [`SvgRoot::path_from_defs`](crate::SvgRoot::path_from_defs)
    /// for the full rationale.
    pub fn path_from_defs(&self, defs: &[PathDef]) -> Result<SvgNode, Error> {
        self.create_path_from_defs(defs)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a detached `<polyline>` element in this batch and returns its [`SvgNode`] handle.
    pub fn polyline(&self, points: &[Point]) -> Result<SvgNode, Error> {
        self.create_polyline(points)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a detached `<polygon>` element in this batch and returns its [`SvgNode`] handle.
    pub fn polygon(&self, points: &[Point]) -> Result<SvgNode, Error> {
        self.create_polygon(points)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a detached `<text>` element in this batch and returns its [`SvgNode`] handle.
    pub fn text(&self, anchored_at: Point, content: &str) -> Result<SvgNode, Error> {
        self.create_text(anchored_at, content)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a detached `<g>` element in this batch and returns its [`SvgNode`] handle.
    pub fn group(&self) -> Result<SvgNode, Error> {
        self.create_group()
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a detached `<image>` element in this batch and returns its [`SvgNode`] handle.
    ///
    /// See [`SvgRoot::image`](crate::SvgRoot::image) for full documentation.
    pub fn image(&self, href: &str, top_left: Point, size: Size) -> Result<SvgNode, Error> {
        self.create_image(href, top_left, size)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a detached `<use>` element in this batch and returns its [`SvgNode`] handle.
    ///
    /// See [`SvgRoot::use_node`](crate::SvgRoot::use_node) for full documentation.
    pub fn use_node(&self, href: &str, at: Point) -> Result<SvgNode, Error> {
        self.create_use(href, at)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a detached `<a>` element in this batch and returns its [`SvgNode`] handle.
    ///
    /// See [`SvgRoot::anchor`](crate::SvgRoot::anchor) for full documentation.
    pub fn anchor(&self, href: &str) -> Result<SvgNode, Error> {
        self.create_anchor(href)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a detached `<switch>` element in this batch and returns its [`SvgNode`] handle.
    ///
    /// See [`SvgRoot::switch`](crate::SvgRoot::switch) for full documentation.
    pub fn switch(&self) -> Result<SvgNode, Error> {
        self.create_switch()
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl SvgFactory for SvgBatch {
    fn document(&self) -> &Document {
        &self.document
    }

    fn attrs(&self) -> &RefCell<SvgAttrs> {
        &self.attrs
    }

    fn append_node(&self, node: &SvgNode) -> Result<(), Error> {
        self.fragment.append_child(node.as_element()).map(|_| ()).map_err(dom_err)
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
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
            self.root.clone().into(),
            self.document.clone(),
            self.document.create_document_fragment(),
        )
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Builds and commits a [`SvgBatch`] into the root in one call.
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

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a batch builder that commits into `parent` (any existing element, typically a `<g>`) rather than the
    /// root.
    ///
    /// Children created through the returned [`SvgBatch`] are appended to a detached [`DocumentFragment`], and
    /// [`SvgBatch::commit`] appends that fragment to `parent` in a single DOM operation. Compared with creating
    /// elements on the root and re-parenting them with [`SvgNode::append`](crate::SvgNode::append), this avoids the
    /// extra "append to root, then move into the group" DOM mutation for every child.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::utils::{Point, Size}};
    ///
    /// let svg = SvgRoot::attach("diagram")?;
    /// let group = svg.group()?;
    /// let batch = svg.batch_into(&group);
    /// batch.rect(Point::origin(), Size::new(80.0, 40.0))?;
    /// batch.text(Point::new(8.0, 26.0), "XOR")?;
    /// batch.commit()?; // both children land directly inside <g>, never on the root
    /// # Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn batch_into(&self, parent: &SvgNode) -> SvgBatch {
        SvgBatch::new(
            parent.as_element().clone().into(),
            self.document.clone(),
            self.document.create_document_fragment(),
        )
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Builds and commits a [`SvgBatch`] into `parent` in one call.
    ///
    /// The closure-based counterpart to [`batch_into`](Self::batch_into); see also [`build_batch`](Self::build_batch).
    /// If the closure returns an error, the fragment is dropped without being appended to `parent`.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::utils::{Point, Size}};
    ///
    /// let svg = SvgRoot::attach("diagram")?;
    /// let group = svg.group()?;
    /// svg.build_batch_into(&group, |b| {
    ///     b.rect(Point::origin(), Size::new(80.0, 40.0))?;
    ///     b.text(Point::new(8.0, 26.0), "XOR")?;
    ///     Ok(())
    /// })?;
    /// # Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn build_batch_into<F>(&self, parent: &SvgNode, build: F) -> Result<(), Error>
    where
        F: FnOnce(&SvgBatch) -> Result<(), Error>,
    {
        let batch = self.batch_into(parent);
        build(&batch)?;
        batch.commit()
    }
}
