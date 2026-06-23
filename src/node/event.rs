use wasm_bindgen::{JsCast, closure::Closure};
use web_sys::{MouseEvent, PointerEvent, SvgElement};

pub enum EventClosure {
    Mouse(Closure<dyn Fn(MouseEvent)>),
    Pointer(Closure<dyn Fn(PointerEvent)>),
}

impl EventClosure {
    pub fn callback_ref(&self) -> &js_sys::Function {
        match self {
            EventClosure::Mouse(closure) => closure.as_ref().unchecked_ref(),
            EventClosure::Pointer(closure) => closure.as_ref().unchecked_ref(),
        }
    }
}

pub struct EventListener {
    pub element: SvgElement,
    pub event_type: String,
    pub closure: EventClosure,
}

impl Drop for EventListener {
    fn drop(&mut self) {
        // Remove the browser-side listener before the wasm-bindgen Closure field is
        // dropped.  Otherwise the DOM can retain a callback reference to a closure
        // that no longer exists in Rust-managed memory.
        let _ = self
            .element
            .remove_event_listener_with_callback(&self.event_type, self.closure.callback_ref());
    }
}
