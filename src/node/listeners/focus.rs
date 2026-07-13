use crate::{SvgNode, error::Error};
use web_sys::FocusEvent;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl SvgNode {
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
    // One-shot variants
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

    /// One-shot variant of [`on_focus`](Self::on_focus): fires at most once, then is automatically removed.
    pub fn on_focus_once<F: FnOnce(FocusEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.store_listener_once("focus", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// One-shot variant of [`on_blur`](Self::on_blur).
    pub fn on_blur_once<F: FnOnce(FocusEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.store_listener_once("blur", handler)
    }
}
