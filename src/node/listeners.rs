use crate::{SvgNode, error::Error};
use wasm_bindgen::JsCast;
use web_sys::{DragEvent, Event, FocusEvent, KeyboardEvent, MouseEvent, PointerEvent, TouchEvent, WheelEvent};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl SvgNode {
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a raw typed [`Event`] handler for events not covered by a more specific helper.
    ///
    /// Prefer the typed convenience wrappers where available. Like the wrappers below, this keeps the closure owned by
    /// the node and removes the DOM listener automatically when the last `SvgNode` handle is dropped.
    ///
    /// Handlers are [`FnMut`], so a hot handler can *own and mutate* its own scratch state directly — for example a
    /// reusable [`SvgAttrs`](crate::SvgAttrs) or a `String` buffer — without wrapping it in `Rc<RefCell<...>>`. (The
    /// one constraint is that a handler must not be dispatched *re-entrantly* — i.e. synchronously triggering the same
    /// event on the same node from within the handler — which would panic, just as a re-entrant `RefCell` borrow
    /// would.)
    ///
    /// **Cycle caveat:**
    ///
    /// If the handler needs to mutate the *same* node it is attached to, capture a [`downgrade`](Self::downgrade)d
    /// [`WeakSvgNode`](crate::WeakSvgNode) and [`upgrade`](crate::WeakSvgNode::upgrade) it inside the closure, rather than a strong [`clone`](Self::clone).
    ///
    /// A strong self-capture forms a reference cycle that keeps the node alive forever and prevents this automatic
    /// listener cleanup. See [`WeakSvgNode`](crate::WeakSvgNode) for details.
    pub fn on_event<F: FnMut(Event) + 'static>(&self, event_type: &'static str, handler: F) -> Result<(), Error> {
        self.add_event_listener(event_type, handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Registers a one-shot event handler: the closure is called at most once, and the browser automatically removes
    /// the listener after the first invocation (using the native `{ once: true }` `addEventListener` option).
    ///
    /// The key advantage over an `FnMut` handler that calls [`remove_listeners`](Self::remove_listeners) on itself is
    /// that no manual removal is needed and the "remove the listener currently running" footgun is entirely avoided.
    ///
    /// The handler receives a typed event `E`; `E` should be the concrete web-sys event type appropriate for
    /// `event_type` (e.g. `MouseEvent` for `"click"`, `PointerEvent` for `"pointerdown"`).
    /// When the types match, the captured values inside `handler` are freed on the first dispatch, even if the node
    /// (and its listener store) lives on.
    /// A small listener shell (the `FnMut` wrapper closure) remains in the store until `clear_listeners`,
    /// `remove_listeners`, or node drop — the same lifetime as every other managed listener.
    ///
    /// If `E` does not match the event the browser actually dispatches, the `instanceof` check fails, the handler is
    /// silently not called, and the captured values are held until the node is dropped or its listeners are cleared.
    ///
    /// Prefer a typed helper such as [`on_click_once`](Self::on_click_once) or
    /// [`on_pointerdown_once`](Self::on_pointerdown_once) where one exists — they bake in the correct event type so
    /// the mismatch footgun cannot occur.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// use web_sys::PointerEvent;
    ///
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let rect = svg.rect(Point::origin(), Size::new(100.0, 50.0))?;
    ///
    /// // Record the coordinates of the first pointer interaction.
    /// rect.on_event_once::<PointerEvent, _>("pointerdown", |e| {
    ///     let (_x, _y) = (e.client_x(), e.client_y());
    /// })?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn on_event_once<E, F>(&self, event_type: &'static str, handler: F) -> Result<(), Error>
    where
        E: JsCast + 'static,
        F: FnOnce(E) + 'static,
    {
        self.store_listener_once(event_type, handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // Typed one-shot helpers — the event type is baked in, so the instanceof mismatch footgun cannot occur.
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

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// One-shot variant of [`on_pointerdown`](Self::on_pointerdown).
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

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// One-shot variant of [`on_wheel`](Self::on_wheel).
    pub fn on_wheel_once<F: FnOnce(WheelEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.store_listener_once("wheel", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// One-shot variant of [`on_touchstart`](Self::on_touchstart).
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

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// One-shot variant of [`on_keydown`](Self::on_keydown).
    pub fn on_keydown_once<F: FnOnce(KeyboardEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.store_listener_once("keydown", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// One-shot variant of [`on_keyup`](Self::on_keyup).
    pub fn on_keyup_once<F: FnOnce(KeyboardEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.store_listener_once("keyup", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// One-shot variant of [`on_focus`](Self::on_focus).
    pub fn on_focus_once<F: FnOnce(FocusEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.store_listener_once("focus", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// One-shot variant of [`on_blur`](Self::on_blur).
    pub fn on_blur_once<F: FnOnce(FocusEvent) + 'static>(&self, handler: F) -> Result<(), Error> {
        self.store_listener_once("blur", handler)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// One-shot variant of [`on_dragstart`](Self::on_dragstart).
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
}
