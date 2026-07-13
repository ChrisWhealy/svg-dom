use super::super::event::EventClosure;
use crate::{SvgNode, error::Error};
use wasm_bindgen::closure::Closure;
use web_sys::TouchEvent;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl SvgNode {
    fn add_touch_listener<F: FnMut(TouchEvent) + 'static>(
        &self,
        event_type: &'static str,
        handler: F,
    ) -> Result<(), Error> {
        self.store_listener(event_type, EventClosure::Touch(Closure::new(handler)))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    fn add_touch_listener_passive<F: FnMut(TouchEvent) + 'static>(
        &self,
        event_type: &'static str,
        handler: F,
    ) -> Result<(), Error> {
        self.store_listener_passive(event_type, EventClosure::Touch(Closure::new(handler)))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a `touchstart` handler. Prefer pointer events when browser support allows it.
    ///
    /// When suppressing the default touch behaviour is not needed, prefer
    /// [`on_touchstart_passive`](Self::on_touchstart_passive).
    pub fn on_touchstart<F: FnMut(TouchEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_touch_listener("touchstart", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a passive `touchstart` handler.
    ///
    /// Registered with `{ passive: true }`: the browser may begin handling the initial touch contact immediately
    /// without waiting for the handler.
    /// Any [`prevent_default`](web_sys::Event::prevent_default) call made inside the handler is silently ignored.
    ///
    /// Use this when the handler only records the start position without needing to suppress the browser's default
    /// touch behaviour (e.g. pull-to-refresh or page scroll from the first contact).
    ///
    /// When suppression from the first contact is required, use [`on_touchstart`](Self::on_touchstart) instead.
    pub fn on_touchstart_passive<F: FnMut(TouchEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_touch_listener_passive("touchstart", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a `touchmove` handler. Prefer pointer events when browser support allows it.
    ///
    /// When suppressing touch-scroll is not needed, prefer [`on_touchmove_passive`](Self::on_touchmove_passive).
    pub fn on_touchmove<F: FnMut(TouchEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_touch_listener("touchmove", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a passive `touchmove` handler.
    ///
    /// Registered with `{ passive: true }`: the browser may begin its touch-scroll immediately without waiting for
    /// the handler.
    /// Any [`prevent_default`](web_sys::Event::prevent_default) call made inside the handler is silently ignored.
    ///
    /// Use this when the handler only *reads* touch coordinates (e.g. for an on-screen readout) without needing to
    /// suppress the browser's touch-scroll behaviour.
    ///
    /// When scroll suppression is required (e.g. a custom drag gesture), use [`on_touchmove`](Self::on_touchmove)
    /// instead.
    pub fn on_touchmove_passive<F: FnMut(TouchEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_touch_listener_passive("touchmove", handler)
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
    // One-shot variants
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

    /// One-shot variant of [`on_touchstart`](Self::on_touchstart): fires at most once, then is automatically removed.
    pub fn on_touchstart_once<F: FnOnce(TouchEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.store_listener_once("touchstart", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// One-shot variant of [`on_touchmove`](Self::on_touchmove).
    pub fn on_touchmove_once<F: FnOnce(TouchEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.store_listener_once("touchmove", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// One-shot variant of [`on_touchend`](Self::on_touchend).
    pub fn on_touchend_once<F: FnOnce(TouchEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.store_listener_once("touchend", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// One-shot variant of [`on_touchcancel`](Self::on_touchcancel).
    pub fn on_touchcancel_once<F: FnOnce(TouchEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.store_listener_once("touchcancel", handler)
    }
}
