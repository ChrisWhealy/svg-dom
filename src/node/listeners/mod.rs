mod drag;
mod focus;
mod keyboard;
mod mouse;
mod pointer;
mod touch;
mod wheel;

use crate::{SvgNode, error::Error};
use super::event::EventClosure;
use wasm_bindgen::{JsCast, closure::Closure};
use web_sys::Event;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl SvgNode {
    fn add_event_listener<F: FnMut(Event) + 'static>(&self, event_type: &'static str, handler: F) -> Result<(), Error> {
        self.store_listener(event_type, EventClosure::Event(Closure::new(handler)))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a raw typed [`Event`] handler for events not covered by a more specific helper.
    ///
    /// Prefer the typed convenience wrappers where available. Like the wrappers below, this keeps the closure owned by
    /// the node and removes the DOM listener automatically when the last `SvgNode` handle is dropped.
    ///
    /// Handlers are [`FnMut`], so a hot handler can *own and mutate* its own scratch state directly — for example a
    /// reusable [`SvgAttrs`](crate::SvgAttrs) or a `String` buffer — without wrapping it in `Rc<RefCell<...>>`. (The
    /// one constraint is that a handler must not be dispatched *re-entrantly* — i.e. synchronously triggering the same
    /// event on the same node from within the handler — which would panic, just as a re-entrant `RefCell` borrow
    /// would.)
    ///
    /// **Cycle caveat:**
    ///
    /// If the handler needs to mutate the *same* node it is attached to, capture a [`downgrade`](Self::downgrade)d
    /// [`WeakSvgNode`](crate::WeakSvgNode) and [`upgrade`](crate::WeakSvgNode::upgrade) it inside the closure, rather
    /// than a strong [`clone`](Self::clone).
    ///
    /// A strong self-capture forms a reference cycle that keeps the node alive forever and prevents this automatic
    /// listener cleanup. See [`WeakSvgNode`](crate::WeakSvgNode) for details.
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
    /// The handler receives a typed event `E`; `E` should be the concrete web-sys event type appropriate for
    /// `event_type` (e.g. `MouseEvent` for `"click"`, `PointerEvent` for `"pointerdown"`).
    /// When the types match, the captured values inside `handler` are freed on the first dispatch, even if the node
    /// (and its listener store) lives on.
    /// A small listener shell (the `FnMut` wrapper closure) remains in the store until `clear_listeners`,
    /// `remove_listeners`, or node drop — the same lifetime as every other managed listener.
    ///
    /// If `E` does not match the event the browser actually dispatches, the `instanceof` check fails, the handler is
    /// silently not called, and the captured values are held until the node is dropped or its listeners are cleared.
    ///
    /// Prefer a typed helper such as [`on_click_once`](Self::on_click_once) or
    /// [`on_pointerdown_once`](Self::on_pointerdown_once) where one exists — they bake in the correct event type so
    /// the mismatch footgun cannot occur.
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
}
