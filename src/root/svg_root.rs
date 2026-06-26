use super::{attrs::SvgAttrs, document, factory::SvgFactory, utils::Size};
use crate::{SvgNode, dom_err, error::Error};

use std::cell::{Cell, RefCell};
use wasm_bindgen::JsCast;
use web_sys::{Document, SvgsvgElement};

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
    /// The underlying `<svg>` element wrapped by this root.
    ///
    /// This is exposed as an escape hatch for the occasional attribute or property this crate does not wrap directly —
    /// a `viewBox`, `preserveAspectRatio`, a CSS class, and so on.
    ///
    /// Note, however, that `width` and `height` are tracked by a cached viewport.
    /// Writing them directly on this element (for example `root.set_attribute("width", …)`) desynchronises
    /// [`width`](Self::width) and [`height`](Self::height) from what the DOM actually shows.
    /// To resize the root, use [`set_viewport`](Self::set_viewport), which is the cache-aware path.
    pub root: SvgsvgElement,
    pub(crate) document: Document,
    viewport: Cell<Size>,
    pub(crate) attrs: RefCell<SvgAttrs>,
}

impl SvgRoot {
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Use this constructor when the `<svg>` placeholder already exists in the HTML and its size has been set with the
    /// `width` and `height` attributes.
    ///
    /// The element is first looked up by `id`.  If found, it is verified that the `id` really belongs to an `<svg>`
    /// element.
    ///
    /// Only the `width` and `height` attributes are read to seed the cached viewport (see [`width`](Self::width) and
    /// [`height`](Self::height)); the rendered size is **not** measured.
    ///
    /// If the units are omitted (e.g. `width="800"`) or explicitly stated in pixels (`width="800px"`), then both are
    /// parsed correctly. Other relative units (such as `%`, `em`, `cm`, etc) and elements sized purely with CSS (for
    /// example `style="width: 100%"`) produce a cached size of `0 × 0`.
    ///
    /// Call [`set_viewport`](Self::set_viewport) after attaching to establish the actual size in those cases.
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
            attrs: RefCell::new(SvgAttrs::new()),
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

        let svg: SvgsvgElement = super::create_svg_element(&document, "svg", "SvgsvgElement")?;

        let mut attrs = SvgAttrs::new();
        attrs.display_element(&svg, "width", size.width)?;
        attrs.display_element(&svg, "height", size.height)?;

        parent.append_child(&svg).map_err(dom_err)?;

        Ok(SvgRoot {
            root: svg,
            document,
            viewport: Cell::new(size),
            // Reuse the buffer just used for width/height rather than discarding it for a fresh empty one; each
            // write clears the scratch first, so its leftover contents do not matter.
            attrs: RefCell::new(attrs),
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
        // The cached viewport is authoritative (so are `width()`/`height()`), so use it to skip redundant DOM writes:
        // a duplicate resize notification with the same size writes nothing, and a one-axis change writes only the axis
        // that moved.
        let old = self.viewport.get();
        if old == size {
            return Ok(());
        }

        let mut attrs = self.attrs.borrow_mut();
        let mut current = old;
        if current.width != size.width {
            attrs.display_element(&self.root, "width", size.width)?;
            current.width = size.width;
            self.viewport.set(current);
        }
        if current.height != size.height {
            attrs.display_element(&self.root, "height", size.height)?;
            current.height = size.height;
            self.viewport.set(current);
        }
        Ok(())
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl SvgFactory for SvgRoot {
    fn document(&self) -> &Document {
        &self.document
    }

    fn attrs(&self) -> &RefCell<SvgAttrs> {
        &self.attrs
    }

    fn append_node(&self, node: &SvgNode) -> Result<(), Error> {
        self.root.append_child(node.as_element()).map(|_| ()).map_err(dom_err)
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn read_viewport(root: &SvgsvgElement) -> Size {
    Size::new(read_number_attr(root, "width"), read_number_attr(root, "height"))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn read_number_attr(root: &SvgsvgElement, name: &str) -> f64 {
    root.get_attribute(name).and_then(|s| parse_svg_length(&s)).unwrap_or(0.0)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Parses a unitless number (`"800"`) or a `px`-suffixed value (`"800px"`) from an SVG length attribute.
///
/// Other units (`%`, `em`, `cm`, etc.) and CSS-only sizing return `None`.
fn parse_svg_length(s: &str) -> Option<f64> {
    let s = s.trim();

    if let Ok(v) = s.parse() {
        return Some(v);
    }

    s.strip_suffix("px")?.trim_end().parse().ok()
}
