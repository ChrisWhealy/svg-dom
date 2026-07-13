use super::super::event::EventClosure;
use crate::{SvgNode, error::Error};
use wasm_bindgen::closure::Closure;
use web_sys::DragEvent;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl SvgNode {
    fn add_drag_listener<F: FnMut(DragEvent) + 'static>(
        &self,
        event_type: &'static str,
        handler: F,
    ) -> Result<(), Error> {
        self.store_listener(event_type, EventClosure::Drag(Closure::new(handler)))
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
    // One-shot variants
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

    /// One-shot variant of [`on_dragstart`](Self::on_dragstart): fires at most once, then is automatically removed.
    pub fn on_dragstart_once<F: FnOnce(DragEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.store_listener_once("dragstart", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// One-shot variant of [`on_drag`](Self::on_drag).
    pub fn on_drag_once<F: FnOnce(DragEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.store_listener_once("drag", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// One-shot variant of [`on_dragend`](Self::on_dragend).
    pub fn on_dragend_once<F: FnOnce(DragEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.store_listener_once("dragend", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// One-shot variant of [`on_dragenter`](Self::on_dragenter).
    pub fn on_dragenter_once<F: FnOnce(DragEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.store_listener_once("dragenter", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// One-shot variant of [`on_dragleave`](Self::on_dragleave).
    pub fn on_dragleave_once<F: FnOnce(DragEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.store_listener_once("dragleave", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// One-shot variant of [`on_dragover`](Self::on_dragover).
    pub fn on_dragover_once<F: FnOnce(DragEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.store_listener_once("dragover", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// One-shot variant of [`on_drop`](Self::on_drop).
    pub fn on_drop_once<F: FnOnce(DragEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.store_listener_once("drop", handler)
    }
}
