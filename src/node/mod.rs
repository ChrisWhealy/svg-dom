mod attrs;
mod cached;
mod event;
mod listeners;
mod text;
mod transform;
mod tree;

pub use cached::CachedAttr;
pub use text::{DominantBaseline, TextAnchor};

use crate::{dom_err, error::Error};
use event::*;
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};
use wasm_bindgen::{JsCast, prelude::*};
use web_sys::{AddEventListenerOptions, Event, SvgElement};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
struct SvgNodeInner {
    element: SvgElement,

    // Listener storage is allocated only for interactive nodes. Most SVG elements are passive geometry, so keeping the
    // store behind `Option<Box<_>>` means a passive node carries no inline collection and allocates nothing until its
    // first listener — the `Option<Box<...>>` is a single null pointer when empty.
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
/// let rect2 = rect.clone(); // same underlying DOM node
/// rect2.set_fill("gold")?;  // visible through both `rect` and `rect2`
///
/// // When a listener needs to mutate the *same* node it is registered on, capture a *weak*
/// // handle.  A strong clone creates a cycle (node → listener store → closure → node) that
/// // keeps the node alive indefinitely and defeats the automatic listener cleanup.
/// let rect_weak = rect.downgrade();
/// rect.on_pointerenter(move |_| {
///     if let Some(r) = rect_weak.upgrade() {
///         let _ = r.set_fill("gold");
///     }
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
    // Listener management — public API
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
    /// rect.on_click(|_| { /* ... */ })?;
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
    /// rect.on_click(|_| { /* ... */ })?;
    /// rect.on_pointermove(|_| { /* ... */ })?;
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
    // Listener infrastructure — private; called by the typed helpers in `node/listeners/`.
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
    /// slot.  The browser removes the listener after the first call; the closure shell remains in the store until
    /// node drop or `clear_listeners`.  When the cast succeeds the captured values are released immediately; if the
    /// cast fails they are held until the shell is freed.
    fn store_listener_once<E, F>(&self, event_type: &'static str, handler: F) -> Result<(), Error>
    where
        E: JsCast + 'static,
        F: FnOnce(E) + 'static,
    {
        let mut handler_opt = Some(handler);
        // Wrap the FnOnce in a FnMut so it fits the existing Closure<dyn FnMut(Event)> type.
        // `dyn_into` performs an instanceof check: if the caller supplied a mismatched `E` the cast returns Err and the
        // handler is silently not called (which is safer than hoping nothing bad will happen...)
        let closure: Closure<dyn FnMut(Event)> = Closure::new(move |e: Event| {
            if let Ok(typed) = e.dyn_into::<E>() {
                if let Some(h) = handler_opt.take() {
                    h(typed);
                }
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
    /// Registers a listener using `{ passive: true }`.
    ///
    /// The browser never waits for the handler to return before proceeding with its scroll or touch update, so the
    /// compositor thread is not blocked. Any `prevent_default()` call made inside the handler is silently ignored by
    /// the browser (it does not panic or error on the Rust side).
    fn store_listener_passive(&self, event_type: &'static str, closure: EventClosure) -> Result<(), Error> {
        let options = AddEventListenerOptions::new();
        options.set_passive(true);
        self.inner
            .element
            .add_event_listener_with_callback_and_add_event_listener_options(
                event_type,
                closure.callback_ref(),
                &options,
            )
            .map_err(dom_err)?;
        self.push_listener(event_type, closure);
        Ok(())
    }
}
