mod event;
mod transform;

use crate::{
    error::Error,
    root::attrs::{AttrWriter, SvgAttrs},
};
use event::*;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{JsCast, prelude::*};
use web_sys::{DragEvent, Event, FocusEvent, KeyboardEvent, MouseEvent, PointerEvent, SvgElement, TouchEvent, WheelEvent};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
struct SvgNodeInner {
    element: SvgElement,

    // Listener storage is allocated only for interactive nodes.
    // Most SVG elements are passive geometry, so keeping the listener Vec behind an Option<Box<_>> avoids carrying an
    // empty Vec in every SvgNodeInner.
    //
    // Each entry stores both the event type and the Closure so the listener can be removed from the DOM before the
    // Closure is dropped. This prevents a detached DOM callback from pointing at an invalid wasm-bindgen closure.
    listeners: RefCell<Option<Box<Vec<EventListener>>>>,
}

impl Drop for SvgNodeInner {
    fn drop(&mut self) {
        // Drop the listener storage explicitly while the SVG element is still alive.
        // Each EventListener removes its own DOM callback before its Closure field is
        // freed.
        let _ = self.listeners.get_mut().take();
    }
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
/// rect.on_pointerenter(move |_| {
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
                listeners: RefCell::new(None),
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
    /// # Text measurement
    ///
    /// Returns the rendered advance width of a text element's content, in user units, by wrapping
    /// [`SVGTextContentElement.getComputedTextLength()`]. Returns `None` for non-text elements.
    ///
    /// This reflects the actual font metrics in effect (family, size, `letter-spacing`, `word-spacing`), so it is the
    /// reliable way to discover, for example, the width of a monospace digit (the CSS `ch` unit) at runtime rather than
    /// hard-coding a guess. The element must be attached to a rendered document for the measurement to be meaningful.
    ///
    /// [`SVGTextContentElement.getComputedTextLength()`]: https://developer.mozilla.org/docs/Web/API/SVGTextContentElement/getComputedTextLength
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::Point, SvgRoot};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let probe = svg.text(Point::origin(), "0")?;
    /// let ch = probe.computed_text_length().unwrap_or(0.0); // width of one monospace digit
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn computed_text_length(&self) -> Option<f64> {
        self.inner
            .element
            .dyn_ref::<web_sys::SvgTextContentElement>()
            .map(|t| t.get_computed_text_length() as f64)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// # Text content
    ///
    /// Replaces the element's text content. Use on a `<text>` element to update the string it displays without
    /// recreating it.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::Point, SvgRoot};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let label = svg.text(Point::new(10.0, 20.0), "0")?;
    /// label.set_text("42"); // live-update the displayed value
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn set_text(&self, content: &str) {
        self.inner.element.set_text_content(Some(content));
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
    /// # Write an attribute only when it changes
    ///
    /// Reads the current value with `get_attribute` but writes it only if the value changes. This avoids redundant
    /// browser-side work in high-frequency handlers where the same value is set over and over — for example a cursor
    /// style or `opacity` flag updated on every `mousemove`/`pointermove`, where the value usually repeats between
    /// frames.
    ///
    /// **WARNING**: This is not always a win: the `get_attribute` read has its own cost, so for attributes that change
    /// on every call (such as a drag `transform`) the plain [`set_attr`](Self::set_attr) is cheaper. Reach for this
    /// only on hot paths where the value frequently repeats — hover/drag cursor state, opacity flags, selected state,
    /// and the like.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg     = SvgRoot::attach("diagram")?;
    /// let surface = svg.rect(Point::origin(), Size::new(100.0, 50.0))?;
    ///
    /// // Called many times per second from a pointermove handler; the DOM is only touched when the cursor
    /// // actually needs to change.
    /// surface.set_attr_if_changed("style", "cursor:grab")?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn set_attr_if_changed(&self, name: &str, value: &str) -> Result<(), Error> {
        if self.inner.element.get_attribute(name).as_deref() == Some(value) {
            return Ok(());
        }
        self.set_attr(name, value)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Binds a reusable [`SvgAttrs`] buffer to this node and returns a chainable attribute writer.
    ///
    /// Use this when setting several numeric or formatted attributes as it avoids the need to allocate a new `String`
    /// for each attribute value.
    pub fn attrs<'a>(&'a self, attrs: &'a mut SvgAttrs) -> AttrWriter<'a> {
        attrs.writer(self)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// # Multi-attribute setter
    ///
    /// Sets several attributes on this element in sequence.
    ///
    /// This is a convenience wrapper around repeated [`set_attr`](Self::set_attr) calls. It is useful when creating or
    /// updating an element whose geometry or style is described by several attributes at once. The setter accepts both
    /// borrowed and owned strings, so it works with literal values as well as values produced by `to_string()`.
    ///
    /// If the browser rejects one of the attributes, this returns the first DOM error and stops. Attributes already set
    /// before that error are left in place, matching the behaviour you would get from issuing the same `set_attr` calls
    /// manually.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let rect = svg.rect(Point::origin(), Size::new(80.0, 40.0))?;
    ///
    /// rect.set_attrs([
    ///     ("fill", "steelblue"),
    ///     ("stroke", "white"),
    ///     ("stroke-width", "2"),
    ///     ("rx", "6"),
    /// ])?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn set_attrs<I, K, V>(&self, attrs: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        for (name, value) in attrs {
            self.set_attr(name.as_ref(), value.as_ref())?;
        }
        Ok(())
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
        let mut attrs = SvgAttrs::new();
        attrs.display(self, "stroke-width", width)
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
    fn store_listener(&self, event_type: &'static str, closure: EventClosure) -> Result<(), Error> {
        self.inner
            .element
            .add_event_listener_with_callback(event_type, closure.callback_ref())
            .map_err(|e| Error::Dom(format!("{e:?}")))?;
        self.inner
            .listeners
            .borrow_mut()
            .get_or_insert_with(|| Box::new(Vec::new()))
            .push(EventListener {
                element: self.inner.element.clone(),
                event_type,
                closure,
            });
        Ok(())
    }

    fn add_drag_listener<F: Fn(DragEvent) + 'static>(&self, event_type: &'static str, handler: F) -> Result<(), Error> {
        self.store_listener(event_type, EventClosure::Drag(Closure::new(handler)))
    }

    fn add_event_listener<F: Fn(Event) + 'static>(&self, event_type: &'static str, handler: F) -> Result<(), Error> {
        self.store_listener(event_type, EventClosure::Event(Closure::new(handler)))
    }

    fn add_focus_listener<F: Fn(FocusEvent) + 'static>(&self, event_type: &'static str, handler: F) -> Result<(), Error> {
        self.store_listener(event_type, EventClosure::Focus(Closure::new(handler)))
    }

    fn add_keyboard_listener<F: Fn(KeyboardEvent) + 'static>(&self, event_type: &'static str, handler: F) -> Result<(), Error> {
        self.store_listener(event_type, EventClosure::Keyboard(Closure::new(handler)))
    }

    fn add_mouse_listener<F: Fn(MouseEvent) + 'static>(&self, event_type: &'static str, handler: F) -> Result<(), Error> {
        self.store_listener(event_type, EventClosure::Mouse(Closure::new(handler)))
    }

    fn add_pointer_listener<F: Fn(PointerEvent) + 'static>(&self, event_type: &'static str, handler: F) -> Result<(), Error> {
        self.store_listener(event_type, EventClosure::Pointer(Closure::new(handler)))
    }

    fn add_touch_listener<F: Fn(TouchEvent) + 'static>(&self, event_type: &'static str, handler: F) -> Result<(), Error> {
        self.store_listener(event_type, EventClosure::Touch(Closure::new(handler)))
    }

    fn add_wheel_listener<F: Fn(WheelEvent) + 'static>(&self, event_type: &'static str, handler: F) -> Result<(), Error> {
        self.store_listener(event_type, EventClosure::Wheel(Closure::new(handler)))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

    /// Registers a raw typed [`Event`] handler for events not covered by a more specific helper.
    ///
    /// Prefer the typed convenience wrappers where available. Like the wrappers below, this keeps the closure owned by
    /// the node and removes the DOM listener automatically when the last `SvgNode` handle is dropped.
    pub fn on_event<F: Fn(Event) + 'static>(&self, event_type: &'static str, handler: F) -> Result<(), Error> {
        self.add_event_listener(event_type, handler)
    }

    /// Registers a `dblclick` handler.
    pub fn on_dblclick<F: Fn(MouseEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_mouse_listener("dblclick", handler)
    }

    /// Registers a `contextmenu` handler. Call `prevent_default()` on the event to suppress the browser menu.
    pub fn on_contextmenu<F: Fn(MouseEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_mouse_listener("contextmenu", handler)
    }

    /// Registers a `mousedown` handler.
    pub fn on_mousedown<F: Fn(MouseEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_mouse_listener("mousedown", handler)
    }

    /// Registers a `mouseup` handler.
    pub fn on_mouseup<F: Fn(MouseEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_mouse_listener("mouseup", handler)
    }

    /// Registers a `mousemove` handler.
    pub fn on_mousemove<F: Fn(MouseEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_mouse_listener("mousemove", handler)
    }

    /// Registers a `mousedown`/`mouseup`-independent `mouseenter` handler.
    ///
    /// For new hover behaviour, prefer [`on_pointerenter`](Self::on_pointerenter), which works across pointer devices.
    pub fn on_mouseenter<F: Fn(MouseEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_mouse_listener("mouseenter", handler)
    }

    /// Registers a `mouseleave` handler.
    ///
    /// For new hover cleanup, prefer [`on_pointerleave`](Self::on_pointerleave), which works across pointer devices.
    pub fn on_mouseleave<F: Fn(MouseEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_mouse_listener("mouseleave", handler)
    }

    /// Registers a `wheel` handler.
    pub fn on_wheel<F: Fn(WheelEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_wheel_listener("wheel", handler)
    }

    /// Registers a `touchstart` handler. Prefer pointer events when browser support allows it.
    pub fn on_touchstart<F: Fn(TouchEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_touch_listener("touchstart", handler)
    }

    /// Registers a `touchmove` handler. Prefer pointer events when browser support allows it.
    pub fn on_touchmove<F: Fn(TouchEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_touch_listener("touchmove", handler)
    }

    /// Registers a `touchend` handler. Prefer pointer events when browser support allows it.
    pub fn on_touchend<F: Fn(TouchEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_touch_listener("touchend", handler)
    }

    /// Registers a `touchcancel` handler. Prefer pointer events when browser support allows it.
    pub fn on_touchcancel<F: Fn(TouchEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_touch_listener("touchcancel", handler)
    }

    /// Registers a `keydown` handler. The SVG element usually needs to be focusable, for example with `tabindex="0"`.
    pub fn on_keydown<F: Fn(KeyboardEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_keyboard_listener("keydown", handler)
    }

    /// Registers a `keyup` handler. The SVG element usually needs to be focusable, for example with `tabindex="0"`.
    pub fn on_keyup<F: Fn(KeyboardEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_keyboard_listener("keyup", handler)
    }

    /// Registers a `focus` handler.
    pub fn on_focus<F: Fn(FocusEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_focus_listener("focus", handler)
    }

    /// Registers a `blur` handler.
    pub fn on_blur<F: Fn(FocusEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_focus_listener("blur", handler)
    }

    /// Registers a `dragstart` handler.
    pub fn on_dragstart<F: Fn(DragEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_drag_listener("dragstart", handler)
    }

    /// Registers a `drag` handler.
    pub fn on_drag<F: Fn(DragEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_drag_listener("drag", handler)
    }

    /// Registers a `dragend` handler.
    pub fn on_dragend<F: Fn(DragEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_drag_listener("dragend", handler)
    }

    /// Registers a `dragenter` handler.
    pub fn on_dragenter<F: Fn(DragEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_drag_listener("dragenter", handler)
    }

    /// Registers a `dragleave` handler.
    pub fn on_dragleave<F: Fn(DragEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_drag_listener("dragleave", handler)
    }

    /// Registers a `dragover` handler. Call `prevent_default()` to allow a drop target to receive `drop`.
    pub fn on_dragover<F: Fn(DragEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_drag_listener("dragover", handler)
    }

    /// Registers a `drop` handler.
    pub fn on_drop<F: Fn(DragEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_drag_listener("drop", handler)
    }

    /// Registers a `pointerdown` handler.
    pub fn on_pointerdown<F: Fn(PointerEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_pointer_listener("pointerdown", handler)
    }

    /// Registers a `pointerup` handler.
    pub fn on_pointerup<F: Fn(PointerEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_pointer_listener("pointerup", handler)
    }

    /// Registers a `pointermove` handler.
    pub fn on_pointermove<F: Fn(PointerEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_pointer_listener("pointermove", handler)
    }

    /// Registers a `pointercancel` handler.
    pub fn on_pointercancel<F: Fn(PointerEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_pointer_listener("pointercancel", handler)
    }

    /// Registers a `pointerover` handler. For hover behaviour on groups, prefer non-bubbling `on_pointerenter`.
    pub fn on_pointerover<F: Fn(PointerEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_pointer_listener("pointerover", handler)
    }

    /// Registers a `pointerout` handler. For hover cleanup on groups, prefer non-bubbling `on_pointerleave`.
    pub fn on_pointerout<F: Fn(PointerEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_pointer_listener("pointerout", handler)
    }

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
    /// Registers an event handler function that fires when the pointer enters this element.
    ///
    /// This wrapper uses the browser's `pointerenter` event rather than `mouseover`.
    /// `pointerenter` does **not** bubble, so it fires once when the pointer crosses the
    /// boundary of this element and does not re-fire just because the pointer moves over
    /// one of the element's children. This makes it the preferred wrapper for hover-like
    /// behaviour on both leaf elements and grouped SVG content.
    ///
    /// The handler receives a [`PointerEvent`], giving access to normal mouse-style
    /// coordinates plus pointer-specific data such as `pointer_id` and `pointer_type`.
    ///
    /// # Example — hover on a leaf element
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let rect = svg.rect(Point::new(10.0, 10.0), Size::new(120.0, 60.0))?;
    ///
    /// let r = rect.clone();
    /// rect.on_pointerenter(move |_| { let _ = r.set_fill("gold"); })?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    ///
    /// # Example — hover on a group without child bubbling
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let group = svg.group()?;
    /// let box_ = svg.rect(Point::new(0.0, 0.0), Size::new(80.0, 40.0))?;
    /// let label = svg.text(Point::new(8.0, 26.0), "XOR")?;
    /// group.append(&box_)?;
    /// group.append(&label)?;
    ///
    /// let group_enter = group.clone();
    /// group.on_pointerenter(move |_| {
    ///     let _ = group_enter.set_attr("opacity", "0.6");
    /// })?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn on_pointerenter<F: Fn(PointerEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_pointer_listener("pointerenter", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers an event handler function that fires when the pointer leaves this element.
    ///
    /// This wrapper uses `pointerleave`, the non-bubbling counterpart to `pointerenter`.
    /// It is preferred over `mouseout` for hover cleanup because child-boundary crossings
    /// inside a group do not trigger extra leave events.
    ///
    /// Commonly paired with [`on_pointerenter`](Self::on_pointerenter) to implement hover effects.
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
    /// rect.on_pointerenter(move |_| { let _ = r_over.set_fill("gold"); })?;
    ///
    /// let r_out = rect.clone();
    /// rect.on_pointerleave(move |_| { let _ = r_out.set_fill("steelblue"); })?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn on_pointerleave<F: Fn(PointerEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_pointer_listener("pointerleave", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers an event handler function for the bubbling `mouseover` event.
    ///
    /// Prefer [`on_pointerenter`](Self::on_pointerenter) for hover behaviour. `mouseover`
    /// bubbles, so handlers attached to groups can fire repeatedly as the pointer crosses
    /// child elements inside the group.
    #[deprecated(note = "prefer on_pointerenter, which uses the non-bubbling pointerenter event")]
    pub fn on_mouseover<F: Fn(MouseEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_mouse_listener("mouseover", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers an event handler function for the bubbling `mouseout` event.
    ///
    /// Prefer [`on_pointerleave`](Self::on_pointerleave) for hover cleanup. `mouseout`
    /// bubbles, so handlers attached to groups can fire repeatedly as the pointer crosses
    /// child elements inside the group.
    #[deprecated(note = "prefer on_pointerleave, which uses the non-bubbling pointerleave event")]
    pub fn on_mouseout<F: Fn(MouseEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_mouse_listener("mouseout", handler)
    }
}
