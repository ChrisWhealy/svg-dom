use super::{document, set_display, utils::Size, SVG_NS};
use crate::{SvgNode, error::Error};

use std::cell::Cell;
use wasm_bindgen::JsCast;
use web_sys::{Document, SvgElement, SvgsvgElement};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Wraps the root `<svg>` element and acts as the factory for all child SVG elements.
///
/// If the `<svg>` element already exists in the DOM, then you can [`attach`](Self::attach) to it.  Otherwise,
/// call [`create_in`](Self::create_in) to create a new DOM element and append it to the specified parent.
///
/// Every element-creation function appends a new element to the `<svg>` and returns an [`SvgNode`] handle that can be
/// used to style it, move it, or attach event listeners.
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub struct SvgRoot {
    pub root: SvgsvgElement,
    pub(crate) document: Document,
    viewport: Cell<Size>,
}

impl SvgRoot {
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Use this constructor when the `<svg>` placeholder already exists in the HTML and its dimensions have been set
    /// using CSS or HTML attributes.
    ///
    /// The element is first looked up by `id`.  If found, it is verified that the `id` really belongs to an `<svg>`
    /// element.
    ///
    /// # Errors
    ///
    /// - [`Error::ElementNotFound`] — no element with that `id` exists.
    /// - [`Error::CastFailed`] — an element with the specified `id` exists, but it is not an `<svg>`.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::SvgRoot;
    /// // index.html contains: <svg id="diagram" width="800" height="600"></svg>
    /// let svg = SvgRoot::attach("diagram")?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    pub fn attach(id: &str) -> Result<Self, Error> {
        let document = document()?;
        let element = document
            .get_element_by_id(id)
            .ok_or_else(|| Error::ElementNotFound(id.into()))?;
        let root = element
            .dyn_into::<SvgsvgElement>()
            .map_err(|_| Error::CastFailed("SvgsvgElement"))?;

        let viewport = Cell::new(read_viewport(&root));

        Ok(SvgRoot {
            root,
            document,
            viewport,
        })
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a new `<svg>` element, sizes it, and appends it to an existing HTML element.
    ///
    /// Use this constructor when the the element needs to be created programmatically, or the SVG dimensions can only
    /// be known at runtime (e.g. derived from data).
    ///
    /// # Arguments
    ///
    /// * `parent_id` — the `id` of the HTML element that will contain the new `<svg>`.
    /// * `width`, `height` — initial dimensions (in pixels) of the `<svg>` element.
    ///
    /// # Errors
    ///
    /// - [`Error::ElementNotFound`] — cannot find the element called `parent_id`.
    /// - [`Error::Dom`] — the browser refused to create or append the element.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::utils::Size};
    /// // index.html contains: <div id="app"></div>
    /// let svg = SvgRoot::create_in("app", Size::new(800.0, 600.0))?;
    /// assert_eq!(svg.width(), 800.0);
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    pub fn create_in(parent_id: &str, size: Size) -> Result<Self, Error> {
        let document = document()?;
        let parent = document
            .get_element_by_id(parent_id)
            .ok_or_else(|| Error::ElementNotFound(parent_id.into()))?;

        let svg = document
            .create_element_ns(Some(SVG_NS), "svg")
            .map_err(|e| Error::Dom(format!("{e:?}")))?
            .dyn_into::<SvgsvgElement>()
            .map_err(|_| Error::CastFailed("SvgsvgElement"))?;

        let mut scratch = String::new();
        set_display(&svg, "width", size.width, &mut scratch)?;
        set_display(&svg, "height", size.height, &mut scratch)?;

        parent
            .append_child(&svg)
            .map_err(|e| Error::Dom(format!("{e:?}")))?;

        Ok(SvgRoot {
            root: svg,
            document,
            viewport: Cell::new(size),
        })
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // Viewport
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Returns the cached `width` of the `<svg>` in pixels.
    ///
    /// The value is read from the DOM once when attaching to an existing element, then kept in memory.
    /// Returns `0.0` if the initial attribute is absent or cannot be parsed as a number.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::utils::Size};
    /// let svg = SvgRoot::create_in("app", Size::new(800.0, 600.0))?;
    /// assert_eq!(svg.width(), 800.0);
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    pub fn width(&self) -> f64 {
        self.viewport.get().width
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Returns the cached `height` of the `<svg>` in pixels.
    ///
    /// The value is read from the DOM once when attaching to an existing element, then kept in memory.
    /// Returns `0.0` if the initial attribute is absent or cannot be parsed as a number.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::utils::Size};
    /// let svg = SvgRoot::create_in("app", Size::new(800.0, 600.0))?;
    /// assert_eq!(svg.height(), 600.0);
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    pub fn height(&self) -> f64 {
        self.viewport.get().height
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Updates the cached viewport and the `width` and `height` attributes on the root `<svg>` element.
    ///
    /// Call this when the available viewport changes — for example in response to the browser window being resized (the
    /// `resize` event).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::utils::Size};
    /// let svg = SvgRoot::attach("diagram")?;
    /// svg.set_viewport(Size::new(1024.0, 768.0))?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    pub fn set_viewport(&self, size: Size) -> Result<(), Error> {
        let mut scratch = String::new();
        set_display(&self.root, "width", size.width, &mut scratch)?;
        set_display(&self.root, "height", size.height, &mut scratch)?;
        self.viewport.set(size);
        Ok(())
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // Local helpers
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    pub(crate) fn make_element(&self, tag: &str) -> Result<SvgElement, Error> {
        self.document
            .create_element_ns(Some(SVG_NS), tag)
            .map_err(|e| Error::Dom(format!("{e:?}")))?
            .dyn_into::<SvgElement>()
            .map_err(|_| Error::CastFailed("SvgElement"))
    }

    pub(crate) fn make_node(&self, tag: &str) -> Result<SvgNode, Error> {
        self.make_element(tag).map(SvgNode::new)
    }

    pub(crate) fn append_node(&self, node: &SvgNode) -> Result<(), Error> {
        self.root
            .append_child(node.as_element())
            .map(|_| ())
            .map_err(|e| Error::Dom(format!("{e:?}")))
    }

    pub(crate) fn append_new(&self, tag: &str) -> Result<SvgNode, Error> {
        let node = self.make_node(tag)?;
        self.append_node(&node)?;
        Ok(node)
    }
}

fn read_viewport(root: &SvgsvgElement) -> Size {
    Size::new(read_number_attr(root, "width"), read_number_attr(root, "height"))
}

fn read_number_attr(root: &SvgsvgElement, name: &str) -> f64 {
    root.get_attribute(name)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0)
}
