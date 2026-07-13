use super::super::event::EventClosure;
use crate::{SvgNode, error::Error};
use wasm_bindgen::closure::Closure;
use web_sys::MouseEvent;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl SvgNode {
    fn add_mouse_listener<F: FnMut(MouseEvent) + 'static>(
        &self,
        event_type: &'static str,
        handler: F,
    ) -> Result<(), Error> {
        self.store_listener(event_type, EventClosure::Mouse(Closure::new(handler)))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers an event handler function that fires when the user clicks on the element.
    ///
    /// The closure is stored inside the `SvgNode`'s `Rc` and lives exactly as long as the last clone of this node.
    /// You can register multiple click handlers on the same node.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let rect = svg.rect(Point::new(10.0, 10.0), Size::new(120.0, 60.0))?;
    /// rect.set_fill("steelblue")?;
    ///
    /// let rect_click = rect.clone();
    /// rect.on_click(move |_evt| {
    ///     let _ = rect_click.set_fill("tomato");
    /// })?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn on_click<F: FnMut(MouseEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_mouse_listener("click", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a `dblclick` handler.
    pub fn on_dblclick<F: FnMut(MouseEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_mouse_listener("dblclick", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a `contextmenu` handler. Call `prevent_default()` on the event to suppress the browser menu.
    pub fn on_contextmenu<F: FnMut(MouseEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_mouse_listener("contextmenu", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a `mousedown` handler.
    pub fn on_mousedown<F: FnMut(MouseEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_mouse_listener("mousedown", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a `mouseup` handler.
    pub fn on_mouseup<F: FnMut(MouseEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_mouse_listener("mouseup", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a `mousemove` handler.
    pub fn on_mousemove<F: FnMut(MouseEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_mouse_listener("mousemove", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a `mousedown`/`mouseup`-independent `mouseenter` handler.
    ///
    /// For new hover behaviour, prefer [`on_pointerenter`](Self::on_pointerenter), which works across pointer devices.
    pub fn on_mouseenter<F: FnMut(MouseEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_mouse_listener("mouseenter", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a `mouseleave` handler.
    ///
    /// For new hover cleanup, prefer [`on_pointerleave`](Self::on_pointerleave), which works across pointer devices.
    pub fn on_mouseleave<F: FnMut(MouseEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_mouse_listener("mouseleave", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers an event handler function for the bubbling `mouseover` event.
    ///
    /// Prefer [`on_pointerenter`](Self::on_pointerenter) for hover behaviour. `mouseover`
    /// bubbles, so handlers attached to groups can fire repeatedly as the pointer crosses
    /// child elements inside the group.
    #[deprecated(note = "prefer on_pointerenter, which uses the non-bubbling pointerenter event")]
    pub fn on_mouseover<F: FnMut(MouseEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_mouse_listener("mouseover", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers an event handler function for the bubbling `mouseout` event.
    ///
    /// Prefer [`on_pointerleave`](Self::on_pointerleave) for hover cleanup. `mouseout`
    /// bubbles, so handlers attached to groups can fire repeatedly as the pointer crosses
    /// child elements inside the group.
    #[deprecated(note = "prefer on_pointerleave, which uses the non-bubbling pointerleave event")]
    pub fn on_mouseout<F: FnMut(MouseEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_mouse_listener("mouseout", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // One-shot variants
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

    /// One-shot variant of [`on_click`](Self::on_click): fires at most once, then is automatically removed.
    pub fn on_click_once<F: FnOnce(MouseEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.store_listener_once("click", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// One-shot variant of [`on_dblclick`](Self::on_dblclick).
    pub fn on_dblclick_once<F: FnOnce(MouseEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.store_listener_once("dblclick", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// One-shot variant of [`on_contextmenu`](Self::on_contextmenu).
    pub fn on_contextmenu_once<F: FnOnce(MouseEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.store_listener_once("contextmenu", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// One-shot variant of [`on_mousedown`](Self::on_mousedown).
    pub fn on_mousedown_once<F: FnOnce(MouseEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.store_listener_once("mousedown", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// One-shot variant of [`on_mouseup`](Self::on_mouseup).
    pub fn on_mouseup_once<F: FnOnce(MouseEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.store_listener_once("mouseup", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// One-shot variant of [`on_mousemove`](Self::on_mousemove).
    pub fn on_mousemove_once<F: FnOnce(MouseEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.store_listener_once("mousemove", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// One-shot variant of [`on_mouseenter`](Self::on_mouseenter).
    pub fn on_mouseenter_once<F: FnOnce(MouseEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.store_listener_once("mouseenter", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// One-shot variant of [`on_mouseleave`](Self::on_mouseleave).
    pub fn on_mouseleave_once<F: FnOnce(MouseEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.store_listener_once("mouseleave", handler)
    }
}
