use crate::{SvgNode, error::Error};
use super::super::event::EventClosure;
use wasm_bindgen::closure::Closure;
use web_sys::KeyboardEvent;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl SvgNode {
    fn add_keyboard_listener<F: FnMut(KeyboardEvent) + 'static>(
        &self,
        event_type: &'static str,
        handler: F,
    ) -> Result<(), Error> {
        self.store_listener(event_type, EventClosure::Keyboard(Closure::new(handler)))
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
    // One-shot variants
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

    /// One-shot variant of [`on_keydown`](Self::on_keydown): fires at most once, then is automatically removed.
    pub fn on_keydown_once<F: FnOnce(KeyboardEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.store_listener_once("keydown", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// One-shot variant of [`on_keyup`](Self::on_keyup).
    pub fn on_keyup_once<F: FnOnce(KeyboardEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.store_listener_once("keyup", handler)
    }
}
