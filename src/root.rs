use wasm_bindgen::JsCast;
use web_sys::{Document, SvgElement, SvgsvgElement};

use crate::{error::Error, node::SvgNode};

const SVG_NS: &str = "http://www.w3.org/2000/svg";

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
    root: SvgsvgElement,
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
    // Element factories
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

    /// Creates a `<rect>` element, appends it to the root, and returns its [`SvgNode`] handle.
    ///
    /// # Arguments
    ///
    /// * `x`, `y` — position of the top-left corner, in user units.
    /// * `w`, `h` — width and height, in user units.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use svg_dom::SvgRoot;
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let rect = svg.rect(10.0, 20.0, 120.0, 60.0)?;
    /// rect.set_fill("tomato")?;
    /// rect.set_stroke("black")?;
    /// # Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn rect(&self, x: f64, y: f64, w: f64, h: f64) -> Result<SvgNode, Error> {
        let n = self.append_new("rect")?;
        n.set_attr("x", &x.to_string())?;
        n.set_attr("y", &y.to_string())?;
        n.set_attr("width", &w.to_string())?;
        n.set_attr("height", &h.to_string())?;
        Ok(n)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<circle>` element, appends it to the root, and returns its [`SvgNode`] handle.
    ///
    /// # Arguments
    ///
    /// * `cx`, `cy` — centre point of the circle in pixels
    /// * `r` — radius in pixels
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use svg_dom::SvgRoot;
    /// let svg = SvgRoot::attach("diagram")?;
    /// let circle = svg.circle(100.0, 100.0, 30.0)?;
    /// circle.set_fill("steelblue")?;
    /// # Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn circle(&self, cx: f64, cy: f64, r: f64) -> Result<SvgNode, Error> {
        let n = self.append_new("circle")?;
        n.set_attr("cx", &cx.to_string())?;
        n.set_attr("cy", &cy.to_string())?;
        n.set_attr("r", &r.to_string())?;
        Ok(n)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<line>` element between two points, appends it to the root, and returns its [`SvgNode`] handle.
    ///
    /// A line has no fill; give it a `stroke` to make it visible.
    ///
    /// # Arguments
    ///
    /// * `x1`, `y1` — start point.
    /// * `x2`, `y2` — end point.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use svg_dom::SvgRoot;
    /// let svg = SvgRoot::attach("diagram")?;
    /// let wire = svg.line(50.0, 100.0, 250.0, 100.0)?;
    /// wire.set_stroke("grey")?;
    /// wire.set_stroke_width(2.0)?;
    /// # Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn line(&self, x1: f64, y1: f64, x2: f64, y2: f64) -> Result<SvgNode, Error> {
        let n = self.append_new("line")?;
        n.set_attr("x1", &x1.to_string())?;
        n.set_attr("y1", &y1.to_string())?;
        n.set_attr("x2", &x2.to_string())?;
        n.set_attr("y2", &y2.to_string())?;
        Ok(n)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<path>` element from an SVG path-data string, appends it to the root, and returns its [`SvgNode`]
    /// handle.
    ///
    /// The `d` string uses standard SVG path commands where the arguments to the uppercase command are interpretted as
    /// absolute coordinates, and the arguments to the lowercase commands as relative coordinates.
    ///
    /// |  Command  | Arguments              | Description             |
    /// |-----------|------------------------|-------------------------|
    /// │ `M` / `m` │ `x y`                  │ Move (no draw)          │
    /// │ `L` / `l` │ `x y`                  │ Line                    │
    /// │ `H` / `h` │ `x`                    │ Horizontal line         │
    /// │ `V` / `v` │ `y`                    │ Vertical line           │
    /// │ `C` / `c` │ `x1 y1 x2 y2 x y`      │ Cubic Bézier            │
    /// │ `S` / `s` │ `x2 y2 x y`            │ Smooth cubic Bézier     │
    /// │ `Q` / `q` │ `x1 y1 x y`            │ Quadratic Bézier        │
    /// │ `T` / `t` │ `x y`                  │ Smooth quadratic Bézier │
    /// │ `A` / `a` │ `rx ry rot laf sf x y` │ Elliptical arc          │
    /// │ `Z` / `z` │ None                   │ Close path              │
    ///
    /// The path can be updated later without recreating the element using [`SvgNode::set_d`].
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use svg_dom::SvgRoot;
    /// let svg = SvgRoot::attach("diagram")?;
    /// let path = svg.path("M 10 10 L 100 50 L 10 90 Z")?;
    /// path.set_fill("none")?;
    /// path.set_stroke("black")?;
    ///
    /// // Mutate the existing DOM node — the element does not need to be destroyed then recreated.
    /// path.set_d("M 20 20 Q 100 0 180 20")?;
    /// # Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn path(&self, d: &str) -> Result<SvgNode, Error> {
        let n = self.append_new("path")?;
        n.set_attr("d", d)?;
        Ok(n)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<text>` element with initial string content, appends it to the root and returns its [`SvgNode`]
    /// handle.
    ///
    /// # Arguments
    ///
    /// * `x`, `y` — position of the text anchor point.
    ///              `y` is the **baseline** of the first line of text, not the top of the bounding box.
    /// * `content` — the visible string.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use svg_dom::SvgRoot;
    /// let svg = SvgRoot::attach("diagram")?;
    /// let label = svg.text(50.0, 30.0, "SHA-256 round")?;
    /// label.set_attr("font-size", "14")?;
    /// label.set_fill("white")?;
    /// # Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn text(&self, x: f64, y: f64, content: &str) -> Result<SvgNode, Error> {
        let el = self.make_element("text")?;
        el.set_attribute("x", &x.to_string())
            .map_err(|e| Error::Dom(format!("{e:?}")))?;
        el.set_attribute("y", &y.to_string())
            .map_err(|e| Error::Dom(format!("{e:?}")))?;
        el.set_text_content(Some(content));
        self.root
            .append_child(&el)
            .map_err(|e| Error::Dom(format!("{e:?}")))?;
        Ok(SvgNode::new(el))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<g>` group element, appends it to the root, and returns its [`SvgNode`] handle.
    ///
    /// A `<g>` element has no visual appearance of its own; it is a container used to transform, clip, or style a set
    /// of child elements together.  Add children using [`SvgNode::append`].
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use svg_dom::SvgRoot;
    /// let svg = SvgRoot::attach("diagram")?;
    /// let group = svg.group()?;
    ///
    /// // Both children move with the group when its transform is updated.
    /// let box_  = svg.rect(0.0, 0.0, 80.0, 40.0)?;
    /// let label = svg.text(10.0, 26.0, "XOR")?;
    /// group.append(&box_)?;
    /// group.append(&label)?;
    ///
    /// // Translate the whole group.
    /// group.set_attr("transform", "translate(120, 60)")?;
    /// # Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn group(&self) -> Result<SvgNode, Error> {
        self.append_new("g")
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // Internal helpers
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

    fn make_element(&self, tag: &str) -> Result<SvgElement, Error> {
        self.document
            .create_element_ns(Some(SVG_NS), tag)
            .map_err(|e| Error::Dom(format!("{e:?}")))?
            .dyn_into::<SvgElement>()
            .map_err(|_| Error::CastFailed("SvgElement"))
    }

    fn append_new(&self, tag: &str) -> Result<SvgNode, Error> {
        let el = self.make_element(tag)?;
        self.root
            .append_child(&el)
            .map_err(|e| Error::Dom(format!("{e:?}")))?;
        Ok(SvgNode::new(el))
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// DOM helpers
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn document() -> Result<Document, Error> {
    web_sys::window()
        .ok_or_else(|| Error::Dom("no window".into()))?
        .document()
        .ok_or_else(|| Error::Dom("no document".into()))
}

fn set(el: &impl AsRef<web_sys::Element>, name: &str, value: &str) -> Result<(), Error> {
    el.as_ref()
        .set_attribute(name, value)
        .map_err(|e| Error::Dom(format!("{e:?}")))
}
