use super::super::event::EventClosure;
use crate::{SvgNode, error::Error};
use wasm_bindgen::closure::Closure;
use web_sys::WheelEvent;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl SvgNode {
    fn add_wheel_listener<F: FnMut(WheelEvent) + 'static>(
        &self,
        event_type: &'static str,
        handler: F,
    ) -> Result<(), Error> {
        self.store_listener(event_type, EventClosure::Wheel(Closure::new(handler)))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    fn add_wheel_listener_passive<F: FnMut(WheelEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.store_listener_passive("wheel", EventClosure::Wheel(Closure::new(handler)))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a `wheel` handler.
    ///
    /// Call [`prevent_default`](web_sys::Event::prevent_default) inside to suppress the browser's default scroll
    /// or zoom behaviour.
    /// When scroll prevention is not needed, prefer [`on_wheel_passive`](Self::on_wheel_passive), which registers
    /// the listener with `{ passive: true }` so the compositor thread is never blocked.
    pub fn on_wheel<F: FnMut(WheelEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_wheel_listener("wheel", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a passive `wheel` handler.
    ///
    /// Unlike [`on_wheel`](Self::on_wheel), the listener is registered with `{ passive: true }`.
    /// The browser does not wait for the handler to return before beginning its scroll or zoom update, so the
    /// compositor thread is never blocked.
    ///
    /// Any [`prevent_default`](web_sys::Event::prevent_default) call made inside the handler is silently ignored.
    ///
    /// Prefer this over [`on_wheel`](Self::on_wheel) when the handler only *reads* wheel data (e.g. accumulates a zoom
    /// level) without needing to suppress the browser's default scroll behaviour.
    pub fn on_wheel_passive<F: FnMut(WheelEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_wheel_listener_passive(handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // One-shot variant
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

    /// One-shot variant of [`on_wheel`](Self::on_wheel): fires at most once, then is automatically removed.
    pub fn on_wheel_once<F: FnOnce(WheelEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.store_listener_once("wheel", handler)
    }
}
