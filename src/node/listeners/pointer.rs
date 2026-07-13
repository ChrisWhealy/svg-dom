use crate::{SvgNode, error::Error};
use super::super::event::EventClosure;
use wasm_bindgen::closure::Closure;
use web_sys::PointerEvent;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl SvgNode {
    fn add_pointer_listener<F: FnMut(PointerEvent) + 'static>(
        &self,
        event_type: &'static str,
        handler: F,
    ) -> Result<(), Error> {
        self.store_listener(event_type, EventClosure::Pointer(Closure::new(handler)))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a `pointerdown` handler.
    pub fn on_pointerdown<F: FnMut(PointerEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_pointer_listener("pointerdown", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a `pointerup` handler.
    pub fn on_pointerup<F: FnMut(PointerEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_pointer_listener("pointerup", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a `pointermove` handler.
    pub fn on_pointermove<F: FnMut(PointerEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_pointer_listener("pointermove", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a `pointercancel` handler.
    pub fn on_pointercancel<F: FnMut(PointerEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_pointer_listener("pointercancel", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a `pointerover` handler. For hover behaviour on groups, prefer non-bubbling `on_pointerenter`.
    pub fn on_pointerover<F: FnMut(PointerEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_pointer_listener("pointerover", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a `pointerout` handler. For hover cleanup on groups, prefer non-bubbling `on_pointerleave`.
    pub fn on_pointerout<F: FnMut(PointerEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_pointer_listener("pointerout", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers an event handler function that fires when the pointer enters this element.
    ///
    /// This wrapper uses the browser's `pointerenter` event rather than `mouseover`.
    /// `pointerenter` does **not** bubble, so it fires once when the pointer crosses the
    /// boundary of this element and does not re-fire just because the pointer moves over
    /// one of the element's children. This makes it the preferred wrapper for hover-like
    /// behaviour on both leaf elements and grouped SVG content.
    ///
    /// The handler receives a [`PointerEvent`], giving access to normal mouse-style
    /// coordinates plus pointer-specific data such as `pointer_id` and `pointer_type`.
    ///
    /// # Example — hover on a leaf element
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let rect = svg.rect(Point::new(10.0, 10.0), Size::new(120.0, 60.0))?;
    ///
    /// let r = rect.clone();
    /// rect.on_pointerenter(move |_| { let _ = r.set_fill("gold"); })?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    ///
    /// # Example — hover on a group without child bubbling
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let group = svg.group()?;
    /// let box_ = svg.rect(Point::new(0.0, 0.0), Size::new(80.0, 40.0))?;
    /// let label = svg.text(Point::new(8.0, 26.0), "XOR")?;
    /// group.append(&box_)?;
    /// group.append(&label)?;
    ///
    /// let group_enter = group.clone();
    /// group.on_pointerenter(move |_| {
    ///     let _ = group_enter.set_attr("opacity", "0.6");
    /// })?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn on_pointerenter<F: FnMut(PointerEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_pointer_listener("pointerenter", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers an event handler function that fires when the pointer leaves this element.
    ///
    /// This wrapper uses `pointerleave`, the non-bubbling counterpart to `pointerenter`.
    /// It is preferred over `mouseout` for hover cleanup because child-boundary crossings
    /// inside a group do not trigger extra leave events.
    ///
    /// Commonly paired with [`on_pointerenter`](Self::on_pointerenter) to implement hover effects.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let rect = svg.rect(Point::new(10.0, 10.0), Size::new(120.0, 60.0))?;
    /// rect.set_fill("steelblue")?;
    ///
    /// let r_over = rect.clone();
    /// rect.on_pointerenter(move |_| { let _ = r_over.set_fill("gold"); })?;
    ///
    /// let r_out = rect.clone();
    /// rect.on_pointerleave(move |_| { let _ = r_out.set_fill("steelblue"); })?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn on_pointerleave<F: FnMut(PointerEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.add_pointer_listener("pointerleave", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // One-shot variants
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

    /// One-shot variant of [`on_pointerdown`](Self::on_pointerdown): fires at most once, then is automatically removed.
    pub fn on_pointerdown_once<F: FnOnce(PointerEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.store_listener_once("pointerdown", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// One-shot variant of [`on_pointerup`](Self::on_pointerup).
    pub fn on_pointerup_once<F: FnOnce(PointerEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.store_listener_once("pointerup", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// One-shot variant of [`on_pointermove`](Self::on_pointermove).
    pub fn on_pointermove_once<F: FnOnce(PointerEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.store_listener_once("pointermove", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// One-shot variant of [`on_pointercancel`](Self::on_pointercancel).
    pub fn on_pointercancel_once<F: FnOnce(PointerEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.store_listener_once("pointercancel", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// One-shot variant of [`on_pointerover`](Self::on_pointerover).
    pub fn on_pointerover_once<F: FnOnce(PointerEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.store_listener_once("pointerover", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// One-shot variant of [`on_pointerout`](Self::on_pointerout).
    pub fn on_pointerout_once<F: FnOnce(PointerEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.store_listener_once("pointerout", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// One-shot variant of [`on_pointerenter`](Self::on_pointerenter).
    pub fn on_pointerenter_once<F: FnOnce(PointerEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.store_listener_once("pointerenter", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// One-shot variant of [`on_pointerleave`](Self::on_pointerleave).
    pub fn on_pointerleave_once<F: FnOnce(PointerEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.store_listener_once("pointerleave", handler)
    }
}
