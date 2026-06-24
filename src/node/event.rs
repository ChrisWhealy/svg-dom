use wasm_bindgen::{JsCast, closure::Closure};
use web_sys::{
    DragEvent, Event, FocusEvent, KeyboardEvent, MouseEvent, PointerEvent, SvgElement, TouchEvent,
    WheelEvent,
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub enum EventClosure {
    Drag(Closure<dyn Fn(DragEvent)>),
    Event(Closure<dyn Fn(Event)>),
    Focus(Closure<dyn Fn(FocusEvent)>),
    Keyboard(Closure<dyn Fn(KeyboardEvent)>),
    Mouse(Closure<dyn Fn(MouseEvent)>),
    Pointer(Closure<dyn Fn(PointerEvent)>),
    Touch(Closure<dyn Fn(TouchEvent)>),
    Wheel(Closure<dyn Fn(WheelEvent)>),
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
pub struct EventListener {
    pub element: SvgElement,
    pub event_type: &'static str,
    pub closure: EventClosure,
}

impl Drop for EventListener {
    fn drop(&mut self) {
        // Remove the browser-side listener before the wasm-bindgen Closure field is
        // dropped. Otherwise the DOM can retain a callback reference to a closure
        // that no longer exists in Rust-managed memory.
        let _ = self
            .element
            .remove_event_listener_with_callback(self.event_type, self.closure.callback_ref());
    }
}
