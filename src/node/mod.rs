mod cached;
mod event;
mod transform;

pub use cached::CachedAttr;

use crate::{
    dom_err,
    error::Error,
    root::attrs::{AttrWriter, SvgAttrs},
};
use event::*;
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};
use wasm_bindgen::{JsCast, prelude::*};
use web_sys::{
    AddEventListenerOptions, DragEvent, Event, FocusEvent, KeyboardEvent, MouseEvent, PointerEvent, SvgElement,
    TouchEvent, WheelEvent,
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
struct SvgNodeInner {
    element: SvgElement,

    // Listener storage is allocated only for interactive nodes. Most SVG elements are passive geometry, so keeping the
    // store behind `Option<Box<_>>` means a passive node carries no inline collection and allocates nothing until its
    // first listener — the `Option<Box<…>>` is a single null pointer when empty.
    //
    // `ListenerStore` then holds the first listener inline (`One`), so registering one listener is a single heap
    // allocation (the `Box`) rather than the two an empty `Vec` would need (its own box plus an element buffer on the
    // first push). A second listener upgrades `One` into `Many(Vec)`. See docs/design_notes.md.
    //
    // Each entry stores both the event type and the Closure so the listener can be removed from the DOM before the
    // Closure is dropped. This prevents a detached DOM callback from pointing at an invalid wasm-bindgen closure.
    listeners: RefCell<Option<Box<ListenerStore>>>,
}

impl Drop for SvgNodeInner {
    fn drop(&mut self) {
        // Listeners no longer hold their own element handle, so detach every browser-side callback here — using the
        // node's still-live element — *before* the store (and its closures) is dropped. Detaching first stops the DOM
        // from briefly retaining a callback that points at a freed wasm-bindgen closure. Dropping the node is the only
        // path that drops listeners, so this single call is the complete cleanup.
        if let Some(store) = self.listeners.get_mut().take() {
            store.detach_all(&self.element);
        }
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// A non-owning handle to the same DOM element as an [`SvgNode`].
///
/// `WeakSvgNode` holds a [`Weak`] reference, so it does **not** keep the node alive.
///
/// To obtain a `WeakSvgNode`, use [`SvgNode::downgrade`].  It can then be turned back into a live [`SvgNode`] with
/// [`upgrade`](Self::upgrade), which yields `None` once every strong handle to the node has been dropped.
///
/// # Why this exists: To avoid a reference cycle
///
/// A managed event listener is *owned by the node it is registered on*. If that listener's closure also captures a
/// **strong** [`SvgNode`] clone of the same node, the node ends up owning a reference to itself:
///
/// ```text
/// SvgNodeInner ─▶ EventListener ─▶ Closure ─▶ SvgNode (Rc) ─▶ the same SvgNodeInner
/// ```
///
/// That cycle keeps the node's strong count above zero even after every external handle is dropped, so the node is
/// never freed and its managed listener is never removed from the DOM. For a node that lives for the whole page this
/// is harmless (it was never going to be dropped anyway), but for a node you create, knowing that it will be discarded
/// later, this is a genuine leak that defeats the crate's automatic listener cleanup.
///
/// Capturing a `WeakSvgNode` instead breaks the cycle: the closure no longer keeps the node alive, so dropping the last
/// strong handle frees the node and removes its listener as expected. Inside the closure call [`upgrade`](Self::upgrade)
/// to obtain a temporary live handle for the duration of the event.
///
/// # Example
///
/// ```rust,no_run
/// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
/// let svg = SvgRoot::attach("diagram")?;
/// let rect = svg.rect(Point::new(10.0, 10.0), Size::new(80.0, 40.0))?;
///
/// // Capture a *weak* handle so the listener does not keep `rect` alive.
/// let rect_weak = rect.downgrade();
/// rect.on_pointerenter(move |_| {
///     if let Some(rect) = rect_weak.upgrade() {
///         let _ = rect.set_fill("gold");
///     }
/// })?;
/// Ok::<(), svg_dom::Error>(())
/// ```
#[derive(Clone)]
pub struct WeakSvgNode {
    inner: Weak<SvgNodeInner>,
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl WeakSvgNode {
    /// Attempts to obtain a live [`SvgNode`] from this weak handle.
    ///
    /// Returns `None` once the node has been dropped — that is, after the last strong [`SvgNode`] handle is gone.
    pub fn upgrade(&self) -> Option<SvgNode> {
        self.inner.upgrade().map(|inner| SvgNode { inner })
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
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
    /// # Weak handle
    ///
    /// Returns a non-owning [`WeakSvgNode`] for this element.
    ///
    /// Use this when a managed event listener on a node must refer back to that **same** node: capturing a strong
    /// [`clone`](Self::clone) in such a closure creates a reference cycle that keeps the node (and its DOM listener)
    /// alive forever. See [`WeakSvgNode`] for the full explanation and an example.
    pub fn downgrade(&self) -> WeakSvgNode {
        WeakSvgNode {
            inner: Rc::downgrade(&self.inner),
        }
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
    /// # Text content from `format_args!`, through a caller-owned buffer
    ///
    /// Formats `args` into the supplied scratch buffer and sets the result as this element's text content, reusing the
    /// buffer's allocation across calls. This is the text-content counterpart to
    /// [`set_attr_display`](Self::set_attr_display): use it for a label whose value changes on every event — a
    /// coordinate or status readout updated on each `pointermove`, say — where `set_text(&format!(...))` would allocate
    /// and drop a fresh `String` per event.
    ///
    /// Keep one buffer in the handler's state and pass it on every call. If instead the text usually *repeats* between
    /// events, prefer [`CachedAttr::set_text`](crate::CachedAttr::set_text), which skips the DOM write entirely when the
    /// value is unchanged.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::Point, SvgRoot};
    /// let svg     = SvgRoot::attach("diagram")?;
    /// let readout = svg.text(Point::new(10.0, 20.0), "")?;
    ///
    /// let mut buf = String::new();
    /// let (x, y) = (12.0, 34.0);
    /// readout.set_text_fmt(&mut buf, format_args!("box: {x:.0}, {y:.0}"))?; // no per-call String allocation
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn set_text_fmt(&self, scratch: &mut String, args: std::fmt::Arguments<'_>) -> Result<(), Error> {
        use std::fmt::Write;
        scratch.clear();
        scratch.write_fmt(args)?;
        self.set_text(scratch);
        Ok(())
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// # Text content from a [`Display`](std::fmt::Display) value, through a caller-owned buffer
    ///
    /// Convenience wrapper over [`set_text_fmt`](Self::set_text_fmt) for the common case of a single displayable value
    /// (a counter, a measurement) rather than a formatted string.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::Point, SvgRoot};
    /// let svg   = SvgRoot::attach("diagram")?;
    /// let label = svg.text(Point::new(10.0, 20.0), "")?;
    ///
    /// let mut buf = String::new();
    /// label.set_text_display(&mut buf, 42)?; // live counter, no per-call allocation
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn set_text_display<T: std::fmt::Display>(&self, scratch: &mut String, value: T) -> Result<(), Error> {
        self.set_text_fmt(scratch, format_args!("{value}"))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// # Attribute access
    ///
    /// Sets an arbitrary attribute on this element.
    ///
    /// This is the low-level setter used by all the convenience methods such as `set_fill` and `set_stroke`, etc.
    /// Use it when you need to set an attribute not yet wrapped by a typed helper.
    ///
    /// # Security
    ///
    /// `name` and `value` are written **verbatim** via `setAttribute`. Setting an event-handler attribute (`onclick`,
    /// `onload`, …) or an `href` of the form `javascript:…` from attacker-controlled input can execute script. Do not
    /// pass untrusted data as an attribute name or value without validating it first.
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
        self.inner.element.set_attribute(name, value).map_err(dom_err)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// # Write an attribute only when it changes
    ///
    /// Reads the current value with `get_attribute` but writes it only if the value changes. This avoids a redundant DOM
    /// write in handlers where a value that doesn't change very often might be arbitrarily rewritten by the event
    /// handler. For example, a cursor style or `opacity` flag is updated on every `mousemove`/`pointermove`.
    ///
    /// **WARNING** This is not free and does not always represent a win.
    ///
    /// The read performed by `get_attribute` **allocates a fresh `String` for the current value which then crosses the
    /// WASM/JS boundary on every call**, even if nothing is written.
    ///
    /// So:
    ///
    /// * For attributes that change on *every* call (such as a drag `transform`), the plain [`set_attr`](Self::set_attr)
    ///   is cheaper — skip this entirely.
    /// * For *occasional* de-duplication it is fine as-is.
    /// * For a *genuinely high-frequency* path where the value usually repeats, prefer [`CachedAttr`], which remembers
    ///   the last value on the Rust side: the unchanged case is then a plain string comparison with no allocation and no
    ///   call into JS at all.
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
    /// # Write a numeric (or otherwise [`Display`](std::fmt::Display)) attribute through a caller-owned buffer
    ///
    /// Formats `value` into the supplied scratch buffer and writes it as `name`, reusing the buffer's allocation across
    /// calls. This is the allocation-light counterpart to the convenience numeric setters such as
    /// [`set_stroke_width`](Self::set_stroke_width), which each allocate a short-lived `String` per call.
    ///
    /// Reach for this on hot paths that update a numeric attribute every event or frame — an animated `stroke-width`, a
    /// live `rx`, `font-size`, `r`, and the like. Keep one buffer in the handler's state and pass it on every call, the
    /// same pattern the [transform setters](Self::set_translate) use.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let ring = svg.circle(Point::new(50.0, 50.0), 20.0)?;
    ///
    /// let mut buf = String::new();
    /// ring.set_attr_display(&mut buf, "stroke-width", 2.5)?; // no per-call String allocation
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn set_attr_display<T: std::fmt::Display>(
        &self,
        scratch: &mut String,
        name: &str,
        value: T,
    ) -> Result<(), Error> {
        use std::fmt::Write;
        scratch.clear();
        write!(scratch, "{value}")?;
        self.set_attr(name, scratch)
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
    /// The same security caveat as [`set_attr`](Self::set_attr) applies: names and values are written verbatim, so do
    /// not pass untrusted input.
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
        self.inner.element.remove_attribute(name).map_err(dom_err)
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
    /// This convenience setter formats `width` into a short-lived `String` that is allocated and dropped on each call —
    /// fine for one-off styling. If you animate the stroke width on a hot path (a pulsing highlight, a hover/drag
    /// emphasis), prefer [`set_attr_display`](Self::set_attr_display) with a reused buffer, or an
    /// [`AttrWriter`]/[`SvgAttrs`], to avoid that per-call allocation.
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
            .map_err(dom_err)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Detaches this node from its parent in the DOM.
    ///
    /// The `SvgNode` handle remains valid after removal — it simply points at an element that is no longer part of the
    /// document tree, so it can be re-inserted later with [`append`](Self::append) or [`insert_before`](Self::insert_before).
    ///
    /// Any managed event listeners stay registered on the (now detached) element and are still removed when the last
    /// handle is dropped.
    ///
    /// Removing a node is idempotent. That is, removing an already-detached node or a node that was never attached is a
    /// harmless no-op.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let rect = svg.rect(Point::new(0.0, 0.0), Size::new(40.0, 40.0))?;
    /// rect.remove(); // the <rect> is taken out of the document
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn remove(&self) {
        self.inner.element.remove();
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Inserts the `SvgNode` called `new_child` immediately before the existing `SvgNode` called `reference`.
    ///
    /// This is the tree operation for **z-order control**: SVG paints children in document order, so inserting a node
    /// before an existing sibling places it *behind* that sibling without rebuilding the rest of the tree.
    ///
    /// To have the new child appear at the top of the visibility stack, use [`append`](Self::append) instead.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if `reference` is not a child of this node, mirroring the underlying `Node.insertBefore`
    /// DOM call.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let group = svg.group()?;
    /// let front = svg.rect(Point::new(0.0, 0.0), Size::new(40.0, 40.0))?;
    /// group.append(&front)?;
    ///
    /// // Slip a new rect behind `front` in the group's paint order.
    /// let behind = svg.rect(Point::new(10.0, 10.0), Size::new(40.0, 40.0))?;
    /// group.insert_before(&behind, &front)?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn insert_before(&self, new_child: &SvgNode, reference: &SvgNode) -> Result<(), Error> {
        let reference_node: &web_sys::Node = &reference.inner.element;
        self.inner
            .element
            .insert_before(&new_child.inner.element, Some(reference_node))
            .map(|_| ())
            .map_err(dom_err)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Removes all child nodes of this node, leaving it empty. On a `<text>` node this clears the text.
    ///
    /// This is the bulk counterpart to [`remove`](Self::remove): the idiomatic way to wipe a container such as a `<g>`
    /// before rebuilding its contents. Any `SvgNode` handles the caller still holds for the removed children remain
    /// valid but detached.
    ///
    /// Calling [`clear`](Self::clear) is idempotent. That is, calling it on a node that has no children is a harmless
    /// no-op.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let group = svg.group()?;
    /// group.append(&svg.rect(Point::origin(), Size::new(10.0, 10.0))?)?;
    /// group.append(&svg.circle(Point::new(20.0, 20.0), 5.0)?)?;
    ///
    /// group.clear(); // the <g> is now empty, ready to be rebuilt
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn clear(&self) {
        // Setting text content to nothing detaches every existing child node.
        self.inner.element.set_text_content(None);
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Remove the current node in the DOM and replace it with `replacement`, which then occupies the same sibling
    /// position as the node it replaced.
    ///
    /// Use this to swap one element for another without disturbing the surrounding paint order. After the call this
    /// node is detached (its handle remains valid) and `replacement` occupies its former place.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if this node has no parent, since a detached or root node cannot be replaced in place.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let placeholder = svg.rect(Point::origin(), Size::new(40.0, 40.0))?;
    ///
    /// // Swap the placeholder rect for a circle in the same spot.
    /// let circle = svg.circle(Point::new(20.0, 20.0), 20.0)?;
    /// placeholder.replace_with(&circle)?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn replace_with(&self, replacement: &SvgNode) -> Result<(), Error> {
        let parent = self
            .inner
            .element
            .parent_node()
            .ok_or_else(|| Error::Dom("cannot replace a node that has no parent".into()))?;
        parent
            .replace_child(&replacement.inner.element, &self.inner.element)
            .map(|_| ())
            .map_err(dom_err)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Returns a handle to this node's parent element, or `None` if it either has no parent or the parent is not an SVG
    /// element.
    ///
    /// `None` is returned when either:
    /// - the node is detached (it currently has no parent), or
    /// - the parent exists but is not an SVG element - for example the root `<svg>`, whose parent is the surrounding
    ///   HTML container, not another SVG element.
    ///
    /// # ⚠️ Caution ⚠️
    ///
    /// The returned handle is **not** a factory handle!
    ///
    /// Every other [`SvgNode`] you hold was produced by a factory method ([`SvgRoot::rect`](crate::SvgRoot::rect) and
    /// friends) or is a [`clone`](Self::clone) of one. The handle returned here is different in kind: it wraps an
    /// element that `svg-dom` almost certainly did **not** create, so it is a brand-new, *independent owner* of that
    /// element rather than another reference to an existing owner.
    ///
    /// This fact has practical and potentially significant consequences:
    ///
    /// - **Its managed-listener storage is empty.**
    ///
    ///   Managed event listeners (the `on_*` helpers) are tracked per *handle lineage* — a handle together with its
    ///   clones — and **not** per DOM element. This handle therefore does not share listener storage with whatever
    ///   handle originally created or manages the parent, and it cannot see, remove, or otherwise interact with any
    ///   listeners that were registered through those other handles.
    ///
    /// - **If you register a listener through this handle, this handle owns it**, with the usual lifetime: the listener
    ///   is detached when the last clone of *this* handle is dropped. So, just as for a factory handle, you must keep
    ///   this handle alive (store it somewhere lasting) if you want a listener registered on it to persist.
    ///
    /// - It is otherwise an ordinary handle: it points at the same live DOM element, so reading or mutating its
    ///   attributes and text takes effect immediately and is visible through any other handle to that element.
    ///
    /// Consequently, treat `parent()` as **read-only navigation** - for example, walking up to a containing `<g>`
    /// from inside an event callback, rather than as a way to re-acquire listener ownership of an element that is
    /// already managed elsewhere. Where you can, keep and reuse the factory handles you already hold instead.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let group = svg.group()?;
    /// let rect = svg.rect(Point::origin(), Size::new(40.0, 40.0))?;
    /// group.append(&rect)?;
    ///
    /// // Walk up to the containing <g>. Note this is a fresh, independent handle to that element.
    /// if let Some(parent) = rect.parent() {
    ///     parent.set_attr("transform", "translate(10, 10)")?;
    /// }
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn parent(&self) -> Option<SvgNode> {
        self.inner
            .element
            .parent_node()
            .and_then(|n| n.dyn_into::<SvgElement>().ok())
            .map(SvgNode::new)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Detaches and drops **all** managed event listeners registered through this handle lineage, leaving the node
    /// itself — its DOM element, children, and attributes — intact.
    ///
    /// This is the listener counterpart to [`clear`](Self::clear): use it on a long-lived node whose behaviour changes
    /// over time — one that swaps in mode-specific handlers, say — to discard the previous set before registering new
    /// ones, without having to drop and recreate the node.
    ///
    /// Listeners are tracked per *handle lineage* (a handle together with its clones), exactly as described for
    /// [`parent`](Self::parent), so this removes only the listeners registered through this handle or its clones — not
    /// any registered through an independent handle to the same element. Calling it is idempotent: a node with no
    /// managed listeners is a harmless no-op.
    ///
    /// # ⚠️ Caution ⚠️
    ///
    /// Do not remove any listener that is currently running!
    ///
    /// Removing a listener drops its underlying wasm-bindgen closure. Calling this from **inside one of that same
    /// node's handlers** would free the closure that is currently executing, which is undefined behaviour. Remove
    /// listeners from a different context — another event, an animation-frame tick, or any code that is not itself one
    /// of the node's managed handlers. (Removing a *different* node's listeners from within a handler is always fine.)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let rect = svg.rect(Point::origin(), Size::new(40.0, 40.0))?;
    /// rect.on_click(|_| { /* … */ })?;
    ///
    /// rect.clear_listeners(); // the click handler is detached; the <rect> stays in the document
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn clear_listeners(&self) {
        // Mirror `SvgNodeInner::drop`: detach every browser-side callback from the live element *before* the store
        // (and its closures) is dropped, so the DOM never briefly retains a callback pointing at a freed closure.
        if let Some(store) = self.inner.listeners.borrow_mut().take() {
            store.detach_all(&self.inner.element);
        }
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Detaches and drops every managed listener registered for a single `event_type`, leaving listeners for other
    /// events — and the node itself — intact.
    ///
    /// `event_type` is the browser event name used by the corresponding `on_*` helper: `on_click` registers
    /// `"click"`, `on_pointermove` registers `"pointermove"`, and so on.
    ///
    /// Event type removal is idempotent.  That is, removing an event type that either does not exist or has no
    /// registered listeners is a harmless no-op.
    ///
    /// # ⚠️ Caution ⚠️
    ///
    /// As with [`clear_listeners`](Self::clear_listeners), this affects only listeners registered through this handle
    /// lineage, and the same caveat applies: do not call it for any event whose handler is currently running, as
    /// that would free the executing closure.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let rect = svg.rect(Point::origin(), Size::new(40.0, 40.0))?;
    /// rect.on_click(|_| { /* … */ })?;
    /// rect.on_pointermove(|_| { /* … */ })?;
    ///
    /// rect.remove_listeners("click"); // only the click handler goes; pointermove stays
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn remove_listeners(&self, event_type: &'static str) {
        let mut guard = self.inner.listeners.borrow_mut();
        if let Some(store) = guard.as_deref_mut() {
            // `remove_by_type` reports whether the store is now empty so the `Box` can be dropped.
            if store.remove_by_type(&self.inner.element, event_type) {
                *guard = None;
            }
        }
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // Event handlers
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    fn store_listener(&self, event_type: &'static str, closure: EventClosure) -> Result<(), Error> {
        self.inner
            .element
            .add_event_listener_with_callback(event_type, closure.callback_ref())
            .map_err(dom_err)?;
        self.push_listener(event_type, closure);
        Ok(())
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    fn push_listener(&self, event_type: &'static str, closure: EventClosure) {
        let listener = EventListener { event_type, closure };
        let mut guard = self.inner.listeners.borrow_mut();
        match guard.as_deref_mut() {
            // First listener: store it inline (one allocation), no Vec yet.
            None => *guard = Some(Box::new(ListenerStore::One(listener))),
            // Subsequent listeners: push, upgrading `One` to `Many` on the second.
            Some(store) => store.push(listener),
        }
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a one-shot listener using `{ once: true }`.
    ///
    /// The `FnOnce` is wrapped in a `FnMut` via `Option::take` so it can be stored in the existing `EventClosure::Event`
    /// slot.  The browser removes the listener after the first call; the empty closure shell remains in the store until
    /// node drop or `clear_listeners`, but the captured values are released as soon as the handler fires.
    fn store_listener_once<E, F>(&self, event_type: &'static str, handler: F) -> Result<(), Error>
    where
        E: JsCast + 'static,
        F: FnOnce(E) + 'static,
    {
        let mut handler_opt = Some(handler);
        // Wrap the FnOnce in a FnMut so it fits the existing Closure<dyn FnMut(Event)> type.
        // `unchecked_into` skips the instanceof check — the browser always sends the correct concrete
        // event type for the registered event name, so the cast is always valid.
        let closure: Closure<dyn FnMut(Event)> = Closure::new(move |e: Event| {
            if let Some(h) = handler_opt.take() {
                h(e.unchecked_into::<E>());
            }
        });
        let options = AddEventListenerOptions::new();
        options.set_once(true);
        self.inner
            .element
            .add_event_listener_with_callback_and_add_event_listener_options(
                event_type,
                closure.as_ref().unchecked_ref(),
                &options,
            )
            .map_err(dom_err)?;
        self.push_listener(event_type, EventClosure::Event(closure));
        Ok(())
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    fn add_drag_listener<F: FnMut(DragEvent) + 'static>(
        &self,
        event_type: &'static str,
        handler: F,
    ) -> Result<(), Error> {
        self.store_listener(event_type, EventClosure::Drag(Closure::new(handler)))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    fn add_event_listener<F: FnMut(Event) + 'static>(&self, event_type: &'static str, handler: F) -> Result<(), Error> {
        self.store_listener(event_type, EventClosure::Event(Closure::new(handler)))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    fn add_focus_listener<F: FnMut(FocusEvent) + 'static>(
        &self,
        event_type: &'static str,
        handler: F,
    ) -> Result<(), Error> {
        self.store_listener(event_type, EventClosure::Focus(Closure::new(handler)))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    fn add_keyboard_listener<F: FnMut(KeyboardEvent) + 'static>(
        &self,
        event_type: &'static str,
        handler: F,
    ) -> Result<(), Error> {
        self.store_listener(event_type, EventClosure::Keyboard(Closure::new(handler)))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    fn add_mouse_listener<F: FnMut(MouseEvent) + 'static>(
        &self,
        event_type: &'static str,
        handler: F,
    ) -> Result<(), Error> {
        self.store_listener(event_type, EventClosure::Mouse(Closure::new(handler)))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    fn add_pointer_listener<F: FnMut(PointerEvent) + 'static>(
        &self,
        event_type: &'static str,
        handler: F,
    ) -> Result<(), Error> {
        self.store_listener(event_type, EventClosure::Pointer(Closure::new(handler)))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    fn add_touch_listener<F: FnMut(TouchEvent) + 'static>(
        &self,
        event_type: &'static str,
        handler: F,
    ) -> Result<(), Error> {
        self.store_listener(event_type, EventClosure::Touch(Closure::new(handler)))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    fn add_wheel_listener<F: FnMut(WheelEvent) + 'static>(
        &self,
        event_type: &'static str,
        handler: F,
    ) -> Result<(), Error> {
        self.store_listener(event_type, EventClosure::Wheel(Closure::new(handler)))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a raw typed [`Event`] handler for events not covered by a more specific helper.
    ///
    /// Prefer the typed convenience wrappers where available. Like the wrappers below, this keeps the closure owned by
    /// the node and removes the DOM listener automatically when the last `SvgNode` handle is dropped.
    ///
    /// Handlers are [`FnMut`], so a hot handler can *own and mutate* its own scratch state directly — for example a
    /// reusable [`SvgAttrs`] or a `String` buffer — without wrapping it in `Rc<RefCell<…>>`. (The
    /// one constraint is that a handler must not be dispatched *re-entrantly* — i.e. synchronously triggering the same
    /// event on the same node from within the handler — which would panic, just as a re-entrant `RefCell` borrow
    /// would.)
    ///
    /// **Cycle caveat:**
    ///
    /// If the handler needs to mutate the *same* node it is attached to, capture a [`downgrade`](Self::downgrade)d
    /// [`WeakSvgNode`] and [`upgrade`](WeakSvgNode::upgrade) it inside the closure, rather than a strong [`clone`](Self::clone).
    ///
    /// A strong self-capture forms a reference cycle that keeps the node alive forever and prevents this automatic
    /// listener cleanup. See [`WeakSvgNode`] for details.
    pub fn on_event<F: FnMut(Event) + 'static>(&self, event_type: &'static str, handler: F) -> Result<(), Error> {
        self.add_event_listener(event_type, handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a one-shot event handler: the closure is called at most once, and the browser automatically removes
    /// the listener after the first invocation (using the native `{ once: true }` `addEventListener` option).
    ///
    /// The key advantage over an `FnMut` handler that calls [`remove_listeners`](Self::remove_listeners) on itself is
    /// that no manual removal is needed and the "remove the listener currently running" footgun is entirely avoided.
    ///
    /// The handler receives a typed event `E`; `E` must be the concrete web-sys event type appropriate for
    /// `event_type` (e.g. `MouseEvent` for `"click"`, `PointerEvent` for `"pointerdown"`).  Using the wrong type is
    /// undefined behaviour (the cast from the raw `Event` is unchecked for performance).
    ///
    /// The captured values inside `handler` are freed as soon as the first event fires, even if the node (and its
    /// listener store) lives on.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// use web_sys::PointerEvent;
    ///
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let rect = svg.rect(Point::origin(), Size::new(100.0, 50.0))?;
    ///
    /// // Record the coordinates of the first pointer interaction.
    /// rect.on_event_once::<PointerEvent, _>("pointerdown", |e| {
    ///     let (_x, _y) = (e.client_x(), e.client_y());
    /// })?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn on_event_once<E, F>(&self, event_type: &'static str, handler: F) -> Result<(), Error>
    where
        E: JsCast + 'static,
        F: FnOnce(E) + 'static,
    {
        self.store_listener_once(event_type, handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a `dblclick` handler.
    pub fn on_dblclick<F: FnMut(MouseEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_mouse_listener("dblclick", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a `contextmenu` handler. Call `prevent_default()` on the event to suppress the browser menu.
    pub fn on_contextmenu<F: FnMut(MouseEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_mouse_listener("contextmenu", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a `mousedown` handler.
    pub fn on_mousedown<F: FnMut(MouseEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_mouse_listener("mousedown", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a `mouseup` handler.
    pub fn on_mouseup<F: FnMut(MouseEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_mouse_listener("mouseup", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a `mousemove` handler.
    pub fn on_mousemove<F: FnMut(MouseEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_mouse_listener("mousemove", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a `mousedown`/`mouseup`-independent `mouseenter` handler.
    ///
    /// For new hover behaviour, prefer [`on_pointerenter`](Self::on_pointerenter), which works across pointer devices.
    pub fn on_mouseenter<F: FnMut(MouseEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_mouse_listener("mouseenter", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a `mouseleave` handler.
    ///
    /// For new hover cleanup, prefer [`on_pointerleave`](Self::on_pointerleave), which works across pointer devices.
    pub fn on_mouseleave<F: FnMut(MouseEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_mouse_listener("mouseleave", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a `wheel` handler.
    pub fn on_wheel<F: FnMut(WheelEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_wheel_listener("wheel", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a `touchstart` handler. Prefer pointer events when browser support allows it.
    pub fn on_touchstart<F: FnMut(TouchEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_touch_listener("touchstart", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a `touchmove` handler. Prefer pointer events when browser support allows it.
    pub fn on_touchmove<F: FnMut(TouchEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_touch_listener("touchmove", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a `touchend` handler. Prefer pointer events when browser support allows it.
    pub fn on_touchend<F: FnMut(TouchEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_touch_listener("touchend", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a `touchcancel` handler. Prefer pointer events when browser support allows it.
    pub fn on_touchcancel<F: FnMut(TouchEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_touch_listener("touchcancel", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a `keydown` handler. The SVG element usually needs to be focusable, for example with `tabindex="0"`.
    pub fn on_keydown<F: FnMut(KeyboardEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_keyboard_listener("keydown", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a `keyup` handler. The SVG element usually needs to be focusable, for example with `tabindex="0"`.
    pub fn on_keyup<F: FnMut(KeyboardEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_keyboard_listener("keyup", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a `focus` handler.
    pub fn on_focus<F: FnMut(FocusEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_focus_listener("focus", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a `blur` handler.
    pub fn on_blur<F: FnMut(FocusEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_focus_listener("blur", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a `dragstart` handler.
    pub fn on_dragstart<F: FnMut(DragEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_drag_listener("dragstart", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a `drag` handler.
    pub fn on_drag<F: FnMut(DragEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_drag_listener("drag", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a `dragend` handler.
    pub fn on_dragend<F: FnMut(DragEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_drag_listener("dragend", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a `dragenter` handler.
    pub fn on_dragenter<F: FnMut(DragEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_drag_listener("dragenter", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a `dragleave` handler.
    pub fn on_dragleave<F: FnMut(DragEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_drag_listener("dragleave", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a `dragover` handler. Call `prevent_default()` to allow a drop target to receive `drop`.
    pub fn on_dragover<F: FnMut(DragEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_drag_listener("dragover", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a `drop` handler.
    pub fn on_drop<F: FnMut(DragEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_drag_listener("drop", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a `pointerdown` handler.
    pub fn on_pointerdown<F: FnMut(PointerEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_pointer_listener("pointerdown", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a `pointerup` handler.
    pub fn on_pointerup<F: FnMut(PointerEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_pointer_listener("pointerup", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a `pointermove` handler.
    pub fn on_pointermove<F: FnMut(PointerEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_pointer_listener("pointermove", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a `pointercancel` handler.
    pub fn on_pointercancel<F: FnMut(PointerEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_pointer_listener("pointercancel", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a `pointerover` handler. For hover behaviour on groups, prefer non-bubbling `on_pointerenter`.
    pub fn on_pointerover<F: FnMut(PointerEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_pointer_listener("pointerover", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a `pointerout` handler. For hover cleanup on groups, prefer non-bubbling `on_pointerleave`.
    pub fn on_pointerout<F: FnMut(PointerEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_pointer_listener("pointerout", handler)
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
    pub fn on_click<F: FnMut(MouseEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
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
    pub fn on_pointerenter<F: FnMut(PointerEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
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
    pub fn on_pointerleave<F: FnMut(PointerEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_pointer_listener("pointerleave", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers an event handler function for the bubbling `mouseover` event.
    ///
    /// Prefer [`on_pointerenter`](Self::on_pointerenter) for hover behaviour. `mouseover`
    /// bubbles, so handlers attached to groups can fire repeatedly as the pointer crosses
    /// child elements inside the group.
    #[deprecated(note = "prefer on_pointerenter, which uses the non-bubbling pointerenter event")]
    pub fn on_mouseover<F: FnMut(MouseEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_mouse_listener("mouseover", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers an event handler function for the bubbling `mouseout` event.
    ///
    /// Prefer [`on_pointerleave`](Self::on_pointerleave) for hover cleanup. `mouseout`
    /// bubbles, so handlers attached to groups can fire repeatedly as the pointer crosses
    /// child elements inside the group.
    #[deprecated(note = "prefer on_pointerleave, which uses the non-bubbling pointerleave event")]
    pub fn on_mouseout<F: FnMut(MouseEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_mouse_listener("mouseout", handler)
    }
}
