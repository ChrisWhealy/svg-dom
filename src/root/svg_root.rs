use super::{SVG_NS, document, set};
use crate::{SvgNode, error::Error};

use wasm_bindgen::JsCast;
use web_sys::{Document, SvgElement, SvgsvgElement};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Wraps the root `<svg>` element and acts as the factory for all child SVG elements.
///
/// If the `<svg>` element already exists in the DOM, then you can [`attach`](Self::attach) to it.
///
/// If you need to create a new DOM element, then call [`create_in`](Self::create_in).
///
/// Every element-creation method appends the new element to the `<svg>` and returns an [`SvgNode`] handle that can be
/// used to style it, move it, or attach event listeners.
pub struct SvgRoot {
    pub root: SvgsvgElement,
    document: Document,
}

impl SvgRoot {
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // Constructors

    /// Attaches to an existing `<svg>` element already present in the DOM.
    ///
    /// Looks up the element by `id` and verifies that it really is an `<svg>`.
    ///
    /// Use this constructor when the `<svg>` placeholder exists directly in HTML and its dimensions have been set using
    /// CSS or HTML attributes.
    ///
    /// # Errors
    ///
    /// - [`Error::ElementNotFound`] — no element with that `id` exists.
    /// - [`Error::CastFailed`] — an element with the specified `id` exists, but it is not an `<svg>`.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use svg_dom::SvgRoot;
    /// // index.html contains: <svg id="diagram" width="800" height="600"></svg>
    /// let svg = SvgRoot::attach("diagram")?;
    /// # Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn attach(id: &str) -> Result<Self, Error> {
        let document = document()?;
        let element = document
            .get_element_by_id(id)
            .ok_or_else(|| Error::ElementNotFound(id.into()))?;
        let root = element
            .dyn_into::<SvgsvgElement>()
            .map_err(|_| Error::CastFailed("SvgsvgElement"))?;

        Ok(SvgRoot { root, document })
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a new `<svg>` element, sizes it, and appends it to an existing HTML element.
    ///
    /// Use this constructor when the SVG dimensions are only known at runtime (e.g. derived from data) or when you want
    /// the element to be created programmatically rather than declared in HTML.
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
    /// # use svg_dom::SvgRoot;
    /// // index.html contains: <div id="app"></div>
    /// let svg = SvgRoot::create_in("app", 800.0, 600.0)?;
    /// assert_eq!(svg.width(), 800.0);
    /// # Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn create_in(parent_id: &str, width: f64, height: f64) -> Result<Self, Error> {
        let document = document()?;
        let parent = document
            .get_element_by_id(parent_id)
            .ok_or_else(|| Error::ElementNotFound(parent_id.into()))?;

        let svg = document
            .create_element_ns(Some(SVG_NS), "svg")
            .map_err(|e| Error::Dom(format!("{e:?}")))?
            .dyn_into::<SvgsvgElement>()
            .map_err(|_| Error::CastFailed("SvgsvgElement"))?;

        set(&svg, "width", &width.to_string())?;
        set(&svg, "height", &height.to_string())?;

        parent
            .append_child(&svg)
            .map_err(|e| Error::Dom(format!("{e:?}")))?;

        Ok(SvgRoot {
            root: svg,
            document,
        })
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // Viewport
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

    /// Returns the current `width` of the `<svg>` in pixels.
    ///
    /// Returns `0.0` if the attribute is absent or cannot be parsed as a number.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use svg_dom::SvgRoot;
    /// let svg = SvgRoot::create_in("app", 800.0, 600.0)?;
    /// assert_eq!(svg.width(), 800.0);
    /// # Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn width(&self) -> f64 {
        self.root
            .get_attribute("width")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Returns the current `height` of the `<svg>` in pixels.
    ///
    /// Returns `0.0` if the attribute is absent or cannot be parsed as a number.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use svg_dom::SvgRoot;
    /// let svg = SvgRoot::create_in("app", 800.0, 600.0)?;
    /// assert_eq!(svg.height(), 600.0);
    /// # Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn height(&self) -> f64 {
        self.root
            .get_attribute("height")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Updates the `width` and `height` attributes on the root `<svg>` element.
    ///
    /// Call this when the available viewport changes — for example in response to the browser window being resized (the
    /// `resize` event).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use svg_dom::SvgRoot;
    /// let svg = SvgRoot::attach("diagram")?;
    /// svg.set_viewport(1024.0, 768.0)?;
    /// # Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn set_viewport(&self, width: f64, height: f64) -> Result<(), Error> {
        set(&self.root, "width", &width.to_string())?;
        set(&self.root, "height", &height.to_string())
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

    pub(crate) fn append_new(&self, tag: &str) -> Result<SvgNode, Error> {
        let el = self.make_element(tag)?;
        self.root
            .append_child(&el)
            .map_err(|e| Error::Dom(format!("{e:?}")))?;
        Ok(SvgNode::new(el))
    }
}
