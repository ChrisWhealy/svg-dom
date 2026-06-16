use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{JsCast, prelude::*};
use web_sys::{MouseEvent, SvgElement};

use crate::error::Error;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
struct SvgNodeInner {
    element: SvgElement,

    // Closures are stored here so they live as long as the node.
    // Dropping this Vec removes the event listeners from memory (though not from the DOM — use `remove_event_listener`
    // if you need a clean teardown before the element is removed).
    closures: RefCell<Vec<Closure<dyn Fn(MouseEvent)>>>,
}

/// A cheap-to-clone handle to a live SVG DOM element.
///
/// `SvgNode` wraps an `Rc` pointing to the real browser DOM node.
///
/// Cloning is cheap — all clones refer to the **same underlying element** — so you can freely hand copies to event
/// closures, animation callbacks, or other owners without having to perform any crazy lifetime gymnastics.
///
/// **IMPORTANT** Always obtain a node through one of the factory methods on [`SvgRoot`](crate::SvgRoot) such as `rect`,
/// `circle`, or `line`, etc.  Do not attempt to construct one directly.
///
/// # Clone semantics
///
/// ```rust,no_run
/// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
/// let svg = SvgRoot::attach("diagram")?;
/// let rect = svg.rect(Point::new(10.0, 10.0), Size::new(80.0, 40.0))?;
///
/// let rect_hover = rect.clone();           // another reference to the same DOM node
/// rect.on_mouseover(move |_| {
///     let _ = rect_hover.set_fill("gold"); // mutates the same <rect>
/// })?;
/// Ok::<(), svg_dom::Error>(())
/// ```
#[derive(Clone)]
pub struct SvgNode {
    inner: Rc<SvgNodeInner>,
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl SvgNode {
    pub(crate) fn new(element: SvgElement) -> Self {
        SvgNode {
            inner: Rc::new(SvgNodeInner {
                element,
                closures: RefCell::new(Vec::new()),
            }),
        }
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// # Raw DOM access
    ///
    /// Returns a reference to the underlying `web-sys` `SvgElement`.
    ///
    /// Use this when you need to call a `web-sys` method that has not yet been wrapped by `SvgNode`, such as
    /// `get_bounding_client_rect` (requires the `DomRect` web-sys feature) or `set_inner_html`.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let rect = svg.rect(Point::new(10.0, 10.0), Size::new(80.0, 40.0))?;
    ///
    /// // Access a web-sys method not exposed directly by SvgNode.
    /// let tag = rect.as_element().tag_name(); // "rect"
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn as_element(&self) -> &SvgElement {
        &self.inner.element
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// # Attribute access
    ///
    /// Sets an arbitrary attribute on this element.
    ///
    /// This is the low-level setter used by all the convenience methods such as `set_fill` and `set_stroke`, etc.
    /// Use it when you need to set an attribute not yet wrapped by a typed helper.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let rect = svg.rect(Point::new(0.0, 0.0), Size::new(100.0, 50.0))?;
    ///
    /// rect.set_attr("rx", "8")?;           // set radius of rounded corners
    /// rect.set_attr("opacity", "0.75")?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn set_attr(&self, name: &str, value: &str) -> Result<(), Error> {
        self.inner
            .element
            .set_attribute(name, value)
            .map_err(|e| Error::Dom(format!("{e:?}")))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// # Read element attribute value
    ///
    /// Returns `None` if the attribute is not present.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let rect = svg.rect(Point::new(0.0, 0.0), Size::new(100.0, 50.0))?;
    /// rect.set_attr("class", "highlighted")?;
    ///
    /// assert_eq!(rect.attr("class").as_deref(), Some("highlighted"));
    /// assert_eq!(rect.attr("nonexistent"), None);
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn attr(&self, name: &str) -> Option<String> {
        self.inner.element.get_attribute(name)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// # Remove element attribute
    ///
    /// Has no effect if the attribute is not present.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let rect = svg.rect(Point::new(0.0, 0.0), Size::new(100.0, 50.0))?;
    /// rect.set_attr("opacity", "0.5")?;
    /// rect.remove_attr("opacity")?;         // element is fully opaque again
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn remove_attr(&self, name: &str) -> Result<(), Error> {
        self.inner
            .element
            .remove_attribute(name)
            .map_err(|e| Error::Dom(format!("{e:?}")))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `fill` attribute to a CSS colour value.
    ///
    /// Accepts any valid SVG paint value:
    ///
    /// * named colours (`"red"`)
    /// * hex codes (`"#ff0000"`)
    /// * `rgb()`/`hsl()` functions
    /// * `"none"` to make the fill transparent
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let rect = svg.rect(Point::new(0.0, 0.0), Size::new(80.0, 40.0))?;
    /// rect.set_fill("steelblue")?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn set_fill(&self, colour: &str) -> Result<(), Error> {
        self.set_attr("fill", colour)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `stroke` attribute to a CSS colour value.
    ///
    /// Use in combination with [`set_stroke_width`](Self::set_stroke_width) to control the appearance of outlines and lines.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let rect = svg.rect(Point::new(0.0, 0.0), Size::new(80.0, 40.0))?;
    /// rect.set_stroke("black")?;
    /// rect.set_stroke_width(1.5)?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn set_stroke(&self, colour: &str) -> Result<(), Error> {
        self.set_attr("stroke", colour)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `stroke-width` attribute in user units.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let line = svg.line(Point::new(0.0, 50.0), Point::new(200.0, 50.0))?;
    /// line.set_stroke("grey")?;
    /// line.set_stroke_width(3.0)?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn set_stroke_width(&self, width: f64) -> Result<(), Error> {
        self.set_attr("stroke-width", &width.to_string())
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `d` (path data) attribute on a `<path>` element.
    ///
    /// Alters an existing path created by [`SvgRoot::path`](crate::SvgRoot::path) without needing to recreate the DOM element.
    ///
    /// The `d` string uses standard SVG path commands where the arguments to the uppercase command supply absolute
    /// coordinates, and the arguments to the lowercase commands supply relative coordinates.
    ///
    /// | Command   | Arguments              | Description             |
    /// |:----------|:-----------------------|:------------------------|
    /// | `M` / `m` | `x y`                  | Move (no draw)          |
    /// | `L` / `l` | `x y`                  | Line                    |
    /// | `H` / `h` | `x`                    | Horizontal line         |
    /// | `V` / `v` | `y`                    | Vertical line           |
    /// | `C` / `c` | `x1 y1 x2 y2 x y`      | Cubic Bézier            |
    /// | `S` / `s` | `x2 y2 x y`            | Smooth cubic Bézier     |
    /// | `Q` / `q` | `x1 y1 x y`            | Quadratic Bézier        |
    /// | `T` / `t` | `x y`                  | Smooth quadratic Bézier |
    /// | `A` / `a` | `rx ry rot laf sf x y` | Elliptical arc          |
    /// | `Z` / `z` | —                      | Close path              |
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let path = svg.path("M 0 0 L 100 100")?;
    ///
    /// // Later: morph the path without touching any other attributes.
    /// path.set_d("M 0 0 Q 50 0 100 100")?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn set_d(&self, path: &str) -> Result<(), Error> {
        self.set_attr("d", path)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // Tree operations
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

    /// Appends `child` as a DOM child of this node.
    ///
    /// Use this to build up groups: create a `<g>` with [`SvgRoot::group`](crate::SvgRoot::group), then call `append` to move individual
    /// elements inside it.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let group = svg.group()?;
    /// let rect = svg.rect(Point::new(0.0, 0.0), Size::new(80.0, 40.0))?;
    /// let label = svg.text(Point::new(8.0, 26.0), "XOR")?;
    ///
    /// group.append(&rect)?;
    /// group.append(&label)?;
    ///
    /// // Moving the group moves both children.
    /// group.set_attr("transform", "translate(100, 50)")?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn append(&self, child: &SvgNode) -> Result<(), Error> {
        self.inner
            .element
            .append_child(&child.inner.element)
            .map(|_| ())
            .map_err(|e| Error::Dom(format!("{e:?}")))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // Event handlers
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    fn add_mouse_listener<F: Fn(MouseEvent) + 'static>(
        &self,
        event_type: &str,
        handler: F,
    ) -> Result<(), Error> {
        let closure = Closure::<dyn Fn(MouseEvent)>::new(handler);
        self.inner
            .element
            .add_event_listener_with_callback(event_type, closure.as_ref().unchecked_ref())
            .map_err(|e| Error::Dom(format!("{e:?}")))?;
        self.inner.closures.borrow_mut().push(closure);
        Ok(())
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers an event handler function that fires when the user clicks on the element.
    ///
    /// The closure is stored inside the `SvgNode`'s `Rc` and lives exactly as long as the last clone of this node.
    /// You can register multiple click handlers on the same node.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let rect = svg.rect(Point::new(10.0, 10.0), Size::new(120.0, 60.0))?;
    /// rect.set_fill("steelblue")?;
    ///
    /// let rect_click = rect.clone();
    /// rect.on_click(move |_evt| {
    ///     let _ = rect_click.set_fill("tomato");
    /// })?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn on_click<F: Fn(MouseEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_mouse_listener("click", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers an event handler function that fires when the pointer enters the element.
    ///
    /// Note that the `mouseover` event bubbles, so attaching a handler to this event on a DOM element with children can
    /// become problematic.  Consider this example:
    ///
    /// ```xml
    /// <g id="group">
    ///   <rect id="box" />
    ///   <text id="label">XOR</text>
    /// </g>
    /// ```
    ///
    /// If you call `group.on_mouseover(handler)`, you might expect the event handler to be fired only once when the
    /// pointer enters the `<g>` element. However, the browser natively fires the `mouseover` event whenever the pointer
    /// passes over **any** element, irrespective of whether or not an event handler has been registered for that element.
    /// Since `mouseover` bubbles up to the parent node, the event handler registered for `mouseover` on `#group` will
    /// be fired when:
    ///
    /// 1. the pointer moves over `#box` → `mouseover` fires on `#box`, then bubbles up to `#group` → your handler fires
    /// 2. the pointer slides across to `#label` → `mouseover` fires on `#label` and again, bubbles up to `#group` →
    ///    your handler fires again
    ///
    /// In order to treat the child elements belonging to `#group` as if they were a single element, you should instead
    /// use the `mouseenter` event.  It fires only when the pointer crosses the boundary of the element to which it has
    /// been attached and it does **not** bubble.
    ///
    /// So attaching `mouseenter` to `#group` gives you exactly one event trigger when the pointer enters the group,
    /// regardless of how many of the group's child nodes it passes over.
    ///
    /// Since `SvgNode` does not wrap `mouseenter` directly, register it via [`as_element`](Self::as_element) and
    /// `add_event_listener_with_callback` on the raw `web-sys` element.
    ///
    /// # Example — `mouseover` on a leaf element (no bubbling concern)
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let rect = svg.rect(Point::new(10.0, 10.0), Size::new(120.0, 60.0))?;
    ///
    /// let r = rect.clone();
    /// rect.on_mouseover(move |_| { let _ = r.set_fill("gold"); })?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    ///
    /// # Example — `mouseenter` on a group via the raw element
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// use wasm_bindgen::{JsCast, prelude::Closure};
    /// use web_sys::MouseEvent;
    /// let svg = SvgRoot::attach("diagram")?;
    /// let group = svg.group()?;
    /// let box_ = svg.rect(Point::new(0.0, 0.0), Size::new(80.0, 40.0))?;
    /// let label = svg.text(Point::new(8.0, 26.0), "XOR")?;
    /// group.append(&box_)?;
    /// group.append(&label)?;
    ///
    /// // The `mouseenter` event does not bubble, so it fires exactly once when the pointer enters the group boundary,
    /// // whilst ignoring any boundary-crossings of the group's child elements.
    /// let group_enter = group.clone();
    /// let closure = Closure::<dyn Fn(MouseEvent)>::new(move |_: MouseEvent| {
    ///     let _ = group_enter.set_attr("opacity", "0.6");
    /// });
    /// group
    ///     .as_element()
    ///     .add_event_listener_with_callback("mouseenter", closure.as_ref().unchecked_ref())
    ///     .unwrap();
    /// closure.forget(); // keep the closure alive for the lifetime of the page
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn on_mouseover<F: Fn(MouseEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_mouse_listener("mouseover", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers an event handler function that fires when the pointer leaves this element.
    ///
    /// Commonly paired with [`on_mouseover`](Self::on_mouseover) to implement hover effects.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let rect = svg.rect(Point::new(10.0, 10.0), Size::new(120.0, 60.0))?;
    /// rect.set_fill("steelblue")?;
    ///
    /// let r_over = rect.clone();
    /// rect.on_mouseover(move |_| { let _ = r_over.set_fill("gold"); })?;
    ///
    /// let r_out = rect.clone();
    /// rect.on_mouseout(move |_| { let _ = r_out.set_fill("steelblue"); })?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn on_mouseout<F: Fn(MouseEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_mouse_listener("mouseout", handler)
    }
}
