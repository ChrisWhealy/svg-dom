use wasm_bindgen::{JsCast, closure::Closure};
use web_sys::{
    DragEvent, Event, FocusEvent, KeyboardEvent, MouseEvent, PointerEvent, SvgElement, TouchEvent, WheelEvent,
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub enum EventClosure {
    Drag(Closure<dyn FnMut(DragEvent)>),
    Event(Closure<dyn FnMut(Event)>),
    Focus(Closure<dyn FnMut(FocusEvent)>),
    Keyboard(Closure<dyn FnMut(KeyboardEvent)>),
    Mouse(Closure<dyn FnMut(MouseEvent)>),
    Pointer(Closure<dyn FnMut(PointerEvent)>),
    Touch(Closure<dyn FnMut(TouchEvent)>),
    Wheel(Closure<dyn FnMut(WheelEvent)>),
}

impl EventClosure {
    pub fn callback_ref(&self) -> &js_sys::Function {
        match self {
            EventClosure::Drag(closure) => closure.as_ref().unchecked_ref(),
            EventClosure::Event(closure) => closure.as_ref().unchecked_ref(),
            EventClosure::Focus(closure) => closure.as_ref().unchecked_ref(),
            EventClosure::Keyboard(closure) => closure.as_ref().unchecked_ref(),
            EventClosure::Mouse(closure) => closure.as_ref().unchecked_ref(),
            EventClosure::Pointer(closure) => closure.as_ref().unchecked_ref(),
            EventClosure::Touch(closure) => closure.as_ref().unchecked_ref(),
            EventClosure::Wheel(closure) => closure.as_ref().unchecked_ref(),
        }
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// A single managed listener: the browser event name plus the wasm-bindgen closure registered for it.
///
/// It deliberately does **not** store its own element handle. Detaching the browser-side callback is the owning
/// node's responsibility — `SvgNodeInner::drop` calls [`ListenerStore::detach_all`] with the node's element before
/// the closures are dropped. Because dropping the node is the only path that drops listeners, that single call is
/// sufficient, and it avoids cloning an `SvgElement` (a wasm/JS ref-clone, plus a held JS-table slot) per listener.
pub struct EventListener {
    pub event_type: &'static str,
    pub closure: EventClosure,
}

impl EventListener {
    /// Removes this listener's browser-side callback from `element` (supplied by the owning node).
    fn detach(&self, element: &SvgElement) {
        let _ = element.remove_event_listener_with_callback(self.event_type, self.closure.callback_ref());
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// A node's managed-listener storage, sized for the common case.
///
/// The first listener is held inline as `One`, so registering it costs a single heap allocation (the surrounding
/// `Box<ListenerStore>`) instead of the two an empty `Vec` would need — one for the `Box<Vec>` and another for the
/// element buffer on first `push`. A second listener upgrades the store to `Many`. Most interactive nodes have only
/// one or two listeners, so this keeps the common case lean while still supporting any number.
pub enum ListenerStore {
    One(EventListener),
    Many(Vec<EventListener>),
}

impl ListenerStore {
    /// Adds a listener, upgrading a single-listener `One` store into a `Many` on the second insert.
    pub fn push(&mut self, listener: EventListener) {
        // Swap in a non-allocating placeholder so the existing contents can be moved out by value (no panic path).
        *self = match std::mem::replace(self, ListenerStore::Many(Vec::new())) {
            ListenerStore::One(first) => ListenerStore::Many(vec![first, listener]),
            ListenerStore::Many(mut many) => {
                many.push(listener);
                ListenerStore::Many(many)
            },
        };
    }

    /// Detaches every listener's browser-side callback from `element`.
    ///
    /// Must run before the store (and its closures) is dropped, so the DOM never retains a callback that points at a
    /// freed wasm-bindgen closure. `SvgNodeInner::drop` is the sole caller — and the only place listeners are dropped.
    pub fn detach_all(&self, element: &SvgElement) {
        match self {
            ListenerStore::One(listener) => listener.detach(element),
            ListenerStore::Many(listeners) => {
                for listener in listeners {
                    listener.detach(element);
                }
            },
        }
    }
}
