use crate::{
    common,
    helpers::{dispatch, dispatch_element, make_svg},
};
use std::{cell::Cell, rc::Rc};
use svg_dom::root::utils::{Point, Size};
use wasm_bindgen_test::*;
use web_sys::{KeyboardEvent, MouseEvent, PointerEvent};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Event handlers
//
// `EventTarget::dispatch_event` is synchronous: the browser fires the handler inline before `dispatch_event` returns.
// No need for any async shenanigans
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// An `on_click` handler fires when a synthetic `click` event is dispatched.
#[wasm_bindgen_test]
fn should_fire_on_click_after_synthetic_click() -> Result<(), String> {
    let rect = make_svg("node-click")
        .rect(Point::origin(), Size::new(200.0, 200.0))
        .map_err(|e| e.to_string())?;
    let fired = Rc::new(Cell::new(false));
    let fired_c = fired.clone();
    rect.on_click(move |_| {
        fired_c.set(true);
    })
    .map_err(|e| e.to_string())?;
    dispatch(&rect, "click")?;
    common::check(fired.get(), "click handler did not fire")
}

/// An `on_pointerenter` handler fires when a synthetic `pointerenter` event is dispatched.
#[wasm_bindgen_test]
fn should_fire_on_pointerenter_after_synthetic_pointerenter() -> Result<(), String> {
    let rect = make_svg("node-pointerenter")
        .rect(Point::origin(), Size::new(200.0, 200.0))
        .map_err(|e| e.to_string())?;
    let fired = Rc::new(Cell::new(false));
    let fired_c = fired.clone();
    rect.on_pointerenter(move |_| {
        fired_c.set(true);
    })
    .map_err(|e| e.to_string())?;
    dispatch(&rect, "pointerenter")?;
    common::check(fired.get(), "pointerenter handler did not fire")
}

/// An `on_pointerleave` handler fires when a synthetic `pointerleave` event is dispatched.
#[wasm_bindgen_test]
fn should_fire_on_pointerleave_after_synthetic_pointerleave() -> Result<(), String> {
    let rect = make_svg("node-pointerleave")
        .rect(Point::origin(), Size::new(200.0, 200.0))
        .map_err(|e| e.to_string())?;
    let fired = Rc::new(Cell::new(false));
    let fired_c = fired.clone();
    rect.on_pointerleave(move |_| {
        fired_c.set(true);
    })
    .map_err(|e| e.to_string())?;
    dispatch(&rect, "pointerleave")?;
    common::check(fired.get(), "pointerleave handler did not fire")
}

/// Managed mouse wrappers cover common pointer-with-mouse APIs without needing raw `Closure::forget` listeners.
#[wasm_bindgen_test]
fn should_fire_managed_mouse_event_wrappers() -> Result<(), String> {
    let rect = make_svg("node-mouse-wrappers")
        .rect(Point::origin(), Size::new(200.0, 200.0))
        .map_err(|e| e.to_string())?;
    let count = Rc::new(Cell::new(0u32));

    let c = count.clone();
    rect.on_dblclick(move |_| {
        c.set(c.get() + 1);
    })
    .map_err(|e| e.to_string())?;
    let c = count.clone();
    rect.on_contextmenu(move |e| {
        e.prevent_default();
        c.set(c.get() + 1);
    })
    .map_err(|e| e.to_string())?;
    let c = count.clone();
    rect.on_mousedown(move |_| {
        c.set(c.get() + 1);
    })
    .map_err(|e| e.to_string())?;
    let c = count.clone();
    rect.on_mouseup(move |_| {
        c.set(c.get() + 1);
    })
    .map_err(|e| e.to_string())?;
    let c = count.clone();
    rect.on_mousemove(move |_| {
        c.set(c.get() + 1);
    })
    .map_err(|e| e.to_string())?;
    let c = count.clone();
    rect.on_mouseenter(move |_| {
        c.set(c.get() + 1);
    })
    .map_err(|e| e.to_string())?;
    let c = count.clone();
    rect.on_mouseleave(move |_| {
        c.set(c.get() + 1);
    })
    .map_err(|e| e.to_string())?;

    for event_type in [
        "dblclick", "contextmenu", "mousedown", "mouseup", "mousemove", "mouseenter", "mouseleave",
    ] {
        dispatch(&rect, event_type)?;
    }

    common::check_eq(count.get(), 7)
}

/// The deprecated `on_mouseover` / `on_mouseout` compatibility wrappers still route their (bubbling) event names and
/// remain managed listeners. Kept in a dedicated test so the `deprecated` allowance does not leak into the wrapper
/// coverage above.
#[wasm_bindgen_test]
#[allow(deprecated)]
fn should_fire_deprecated_mouseover_mouseout_wrappers() -> Result<(), String> {
    let rect = make_svg("node-mouse-over-out")
        .rect(Point::origin(), Size::new(200.0, 200.0))
        .map_err(|e| e.to_string())?;
    let overs = Rc::new(Cell::new(0u32));
    let outs = Rc::new(Cell::new(0u32));

    let c = overs.clone();
    rect.on_mouseover(move |_| {
        c.set(c.get() + 1);
    })
    .map_err(|e| e.to_string())?;
    let c = outs.clone();
    rect.on_mouseout(move |_| {
        c.set(c.get() + 1);
    })
    .map_err(|e| e.to_string())?;

    dispatch(&rect, "mouseover")?;
    dispatch(&rect, "mouseout")?;

    common::check_eq(overs.get(), 1)?;
    common::check_eq(outs.get(), 1)
}

/// Managed pointer wrappers cover the common SVG pointer interaction lifecycle.
#[wasm_bindgen_test]
fn should_fire_managed_pointer_event_wrappers() -> Result<(), String> {
    let rect = make_svg("node-pointer-wrappers")
        .rect(Point::origin(), Size::new(200.0, 200.0))
        .map_err(|e| e.to_string())?;
    let count = Rc::new(Cell::new(0u32));

    let c = count.clone();
    rect.on_pointerdown(move |_| {
        c.set(c.get() + 1);
    })
    .map_err(|e| e.to_string())?;
    let c = count.clone();
    rect.on_pointerup(move |_| {
        c.set(c.get() + 1);
    })
    .map_err(|e| e.to_string())?;
    let c = count.clone();
    rect.on_pointermove(move |_| {
        c.set(c.get() + 1);
    })
    .map_err(|e| e.to_string())?;
    let c = count.clone();
    rect.on_pointercancel(move |_| {
        c.set(c.get() + 1);
    })
    .map_err(|e| e.to_string())?;
    let c = count.clone();
    rect.on_pointerover(move |_| {
        c.set(c.get() + 1);
    })
    .map_err(|e| e.to_string())?;
    let c = count.clone();
    rect.on_pointerout(move |_| {
        c.set(c.get() + 1);
    })
    .map_err(|e| e.to_string())?;

    for event_type in [
        "pointerdown", "pointerup", "pointermove", "pointercancel", "pointerover", "pointerout",
    ] {
        dispatch(&rect, event_type)?;
    }

    common::check_eq(count.get(), 6)
}

/// Managed non-mouse wrappers cover keyboard, focus, wheel, drag-and-drop and generic events.
#[wasm_bindgen_test]
fn should_fire_managed_non_mouse_event_wrappers() -> Result<(), String> {
    let rect = make_svg("node-other-event-wrappers")
        .rect(Point::origin(), Size::new(200.0, 200.0))
        .map_err(|e| e.to_string())?;
    rect.set_attr("tabindex", "0").map_err(|e| e.to_string())?;
    rect.set_attr("draggable", "true").map_err(|e| e.to_string())?;
    let count = Rc::new(Cell::new(0u32));

    let c = count.clone();
    rect.on_wheel(move |_| {
        c.set(c.get() + 1);
    })
    .map_err(|e| e.to_string())?;
    let c = count.clone();
    rect.on_touchstart(move |_| {
        c.set(c.get() + 1);
    })
    .map_err(|e| e.to_string())?;
    let c = count.clone();
    rect.on_touchmove(move |_| {
        c.set(c.get() + 1);
    })
    .map_err(|e| e.to_string())?;
    let c = count.clone();
    rect.on_touchend(move |_| {
        c.set(c.get() + 1);
    })
    .map_err(|e| e.to_string())?;
    let c = count.clone();
    rect.on_touchcancel(move |_| {
        c.set(c.get() + 1);
    })
    .map_err(|e| e.to_string())?;
    let c = count.clone();
    rect.on_keydown(move |_| {
        c.set(c.get() + 1);
    })
    .map_err(|e| e.to_string())?;
    let c = count.clone();
    rect.on_keyup(move |_| {
        c.set(c.get() + 1);
    })
    .map_err(|e| e.to_string())?;
    let c = count.clone();
    rect.on_focus(move |_| {
        c.set(c.get() + 1);
    })
    .map_err(|e| e.to_string())?;
    let c = count.clone();
    rect.on_blur(move |_| {
        c.set(c.get() + 1);
    })
    .map_err(|e| e.to_string())?;
    let c = count.clone();
    rect.on_dragstart(move |_| {
        c.set(c.get() + 1);
    })
    .map_err(|e| e.to_string())?;
    let c = count.clone();
    rect.on_drag(move |_| {
        c.set(c.get() + 1);
    })
    .map_err(|e| e.to_string())?;
    let c = count.clone();
    rect.on_dragend(move |_| {
        c.set(c.get() + 1);
    })
    .map_err(|e| e.to_string())?;
    let c = count.clone();
    rect.on_dragenter(move |_| {
        c.set(c.get() + 1);
    })
    .map_err(|e| e.to_string())?;
    let c = count.clone();
    rect.on_dragleave(move |_| {
        c.set(c.get() + 1);
    })
    .map_err(|e| e.to_string())?;
    let c = count.clone();
    rect.on_dragover(move |e| {
        e.prevent_default();
        c.set(c.get() + 1);
    })
    .map_err(|e| e.to_string())?;
    let c = count.clone();
    rect.on_drop(move |_| {
        c.set(c.get() + 1);
    })
    .map_err(|e| e.to_string())?;
    let c = count.clone();
    rect.on_event("custom-svg-dom-test", move |_| {
        c.set(c.get() + 1);
    })
    .map_err(|e| e.to_string())?;

    for event_type in [
        "wheel",
        "touchstart",
        "touchmove",
        "touchend",
        "touchcancel",
        "keydown",
        "keyup",
        "focus",
        "blur",
        "dragstart",
        "drag",
        "dragend",
        "dragenter",
        "dragleave",
        "dragover",
        "drop",
        "custom-svg-dom-test",
    ] {
        dispatch(&rect, event_type)?;
    }

    common::check_eq(count.get(), 17)
}

/// `on_event_once` fires exactly once; a second dispatch of the same event type does not invoke the handler.
#[wasm_bindgen_test]
fn should_fire_on_event_once_exactly_once() -> Result<(), String> {
    let rect = make_svg("node-event-once")
        .rect(Point::origin(), Size::new(200.0, 200.0))
        .map_err(|e| e.to_string())?;
    let count = Rc::new(Cell::new(0u32));
    let count_cb = count.clone();

    rect.on_event_once::<MouseEvent, _>("click", move |_| {
        count_cb.set(count_cb.get() + 1);
    })
    .map_err(|e| e.to_string())?;

    dispatch(&rect, "click")?;
    common::check_eq(count.get(), 1u32)?; // fired on first dispatch

    dispatch(&rect, "click")?;
    common::check_eq(count.get(), 1u32) // did not fire on second dispatch
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Supplying the wrong event type to `on_event_once` causes the `instanceof` check to fail; the handler is never
/// called (rather than causing undefined behaviour via an unchecked cast).
#[wasm_bindgen_test]
fn should_not_fire_on_event_once_with_mismatched_type() -> Result<(), String> {
    let rect = make_svg("node-event-once-mismatch")
        .rect(Point::origin(), Size::new(200.0, 200.0))
        .map_err(|e| e.to_string())?;
    let count = Rc::new(Cell::new(0u32));
    let count_cb = count.clone();

    // Register a combination of event and event handler that can never match
    // KeyboardEvent handler for a "click" event...
    rect.on_event_once::<KeyboardEvent, _>("click", move |_| {
        count_cb.set(count_cb.get() + 1);
    })
    .map_err(|e| e.to_string())?;

    dispatch(&rect, "click")?;
    common::check_eq(count.get(), 0u32) // instanceof check fails; handler not called
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `on_click_once` fires exactly once without requiring a type-parameter annotation.
#[wasm_bindgen_test]
fn should_fire_on_click_once_exactly_once() -> Result<(), String> {
    let rect = make_svg("node-click-once")
        .rect(Point::origin(), Size::new(200.0, 200.0))
        .map_err(|e| e.to_string())?;
    let count = Rc::new(Cell::new(0u32));
    let count_cb = count.clone();

    rect.on_click_once(move |_: MouseEvent| {
        count_cb.set(count_cb.get() + 1);
    })
    .map_err(|e| e.to_string())?;

    dispatch(&rect, "click")?;
    common::check_eq(count.get(), 1u32)?; // fired on first dispatch

    dispatch(&rect, "click")?;
    common::check_eq(count.get(), 1u32) // not fired on second dispatch
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `on_pointerdown_once` fires exactly once; chosen as a representative pointer typed helper.
#[wasm_bindgen_test]
fn should_fire_on_pointerdown_once_exactly_once() -> Result<(), String> {
    let rect = make_svg("node-pointerdown-once")
        .rect(Point::origin(), Size::new(200.0, 200.0))
        .map_err(|e| e.to_string())?;
    let count = Rc::new(Cell::new(0u32));
    let count_cb = count.clone();

    rect.on_pointerdown_once(move |_: PointerEvent| {
        count_cb.set(count_cb.get() + 1);
    })
    .map_err(|e| e.to_string())?;

    dispatch(&rect, "pointerdown")?;
    common::check_eq(count.get(), 1u32)?;

    dispatch(&rect, "pointerdown")?;
    common::check_eq(count.get(), 1u32)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Registering multiple handlers for the same event on the same node results in all of
/// them firing when the event is dispatched.
#[wasm_bindgen_test]
fn should_fire_multiple_handlers_on_same_node() -> Result<(), String> {
    let rect = make_svg("node-multi-handler")
        .rect(Point::origin(), Size::new(200.0, 200.0))
        .map_err(|e| e.to_string())?;
    let count = Rc::new(Cell::new(0u32));
    let count_1 = count.clone();
    let count_2 = count.clone();
    let count_3 = count.clone();
    rect.on_click(move |_| {
        count_1.set(count_1.get() + 1);
    })
    .map_err(|e| e.to_string())?;
    rect.on_click(move |_| {
        count_2.set(count_2.get() + 1);
    })
    .map_err(|e| e.to_string())?;
    rect.on_click(move |_| {
        count_3.set(count_3.get() + 1);
    })
    .map_err(|e| e.to_string())?;
    dispatch(&rect, "click")?;
    common::check_eq(count.get(), 3)
}

/// A handler registered on the original `SvgNode` fires when the event is dispatched via
/// a clone, confirming that both handles point to the same underlying DOM element.
#[wasm_bindgen_test]
fn should_fire_original_handler_when_dispatched_via_clone() -> Result<(), String> {
    let rect = make_svg("node-clone-event")
        .rect(Point::origin(), Size::new(200.0, 200.0))
        .map_err(|e| e.to_string())?;
    let clone = rect.clone();
    let fired = Rc::new(Cell::new(false));
    let fired_c = fired.clone();
    rect.on_click(move |_| {
        fired_c.set(true);
    })
    .map_err(|e| e.to_string())?;
    dispatch(&clone, "click")?;
    common::check(fired.get(), "click handler did not fire when dispatched via clone")
}

/// Dropping the final `SvgNode` handle removes its registered listener from the DOM before
/// dropping the stored wasm-bindgen `Closure`.  Keeping a clone of the raw DOM element lets
/// the test dispatch another event after the Rust-side listener storage has been freed.
#[wasm_bindgen_test]
fn should_remove_dom_listener_when_final_node_handle_is_dropped() -> Result<(), String> {
    let rect = make_svg("node-drop-listener")
        .rect(Point::origin(), Size::new(200.0, 200.0))
        .map_err(|e| e.to_string())?;
    let elem = rect.as_element().clone();
    let count = Rc::new(Cell::new(0u32));
    let seen = count.clone();

    rect.on_click(move |_| {
        seen.set(seen.get() + 1);
    })
    .map_err(|e| e.to_string())?;

    dispatch_element(&elem, "click")?;
    common::check_eq(count.get(), 1)?;

    drop(rect);
    dispatch_element(&elem, "click")?;

    common::check_eq(count.get(), 1)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Explicit listener removal — clear_listeners / remove_listeners
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `clear_listeners` detaches every managed listener while leaving the node in the document, so subsequently
/// dispatched events of any type find nothing to fire.
#[wasm_bindgen_test]
fn should_detach_all_listeners_via_clear_listeners() -> Result<(), String> {
    let rect = make_svg("node-clear-listeners")
        .rect(Point::origin(), Size::new(200.0, 200.0))
        .map_err(|e| e.to_string())?;
    let count = Rc::new(Cell::new(0u32));
    let click = count.clone();
    let mv = count.clone();
    rect.on_click(move |_| click.set(click.get() + 1)).map_err(|e| e.to_string())?;
    rect.on_pointermove(move |_| mv.set(mv.get() + 1)).map_err(|e| e.to_string())?;

    dispatch(&rect, "click")?;
    dispatch(&rect, "pointermove")?;
    common::check_eq(count.get(), 2)?;

    rect.clear_listeners();
    dispatch(&rect, "click")?;
    dispatch(&rect, "pointermove")?;
    common::check_eq(count.get(), 2)
}

/// `remove_listeners(event_type)` detaches only the listeners for that event, leaving listeners for other events
/// firing as before.
#[wasm_bindgen_test]
fn should_detach_only_matching_event_via_remove_listeners() -> Result<(), String> {
    let rect = make_svg("node-remove-listeners")
        .rect(Point::origin(), Size::new(200.0, 200.0))
        .map_err(|e| e.to_string())?;
    let clicks = Rc::new(Cell::new(0u32));
    let moves = Rc::new(Cell::new(0u32));
    let click = clicks.clone();
    let mv = moves.clone();
    rect.on_click(move |_| click.set(click.get() + 1)).map_err(|e| e.to_string())?;
    rect.on_pointermove(move |_| mv.set(mv.get() + 1)).map_err(|e| e.to_string())?;

    rect.remove_listeners("click");
    dispatch(&rect, "click")?;
    dispatch(&rect, "pointermove")?;

    common::check_eq(clicks.get(), 0)?;
    common::check_eq(moves.get(), 1)
}

/// Removing listeners from a `Many` store down to a single survivor keeps that survivor working (the store is
/// downgraded back to `One` internally, but that is transparent to the caller).
#[wasm_bindgen_test]
fn should_keep_surviving_listener_after_remove_listeners() -> Result<(), String> {
    let rect = make_svg("node-remove-survivor")
        .rect(Point::origin(), Size::new(200.0, 200.0))
        .map_err(|e| e.to_string())?;
    let clicks = Rc::new(Cell::new(0u32));
    let click = clicks.clone();
    rect.on_click(move |_| click.set(click.get() + 1)).map_err(|e| e.to_string())?;
    rect.on_pointermove(|_| {}).map_err(|e| e.to_string())?;
    rect.on_pointerdown(|_| {}).map_err(|e| e.to_string())?;

    // Remove the two pointer types, leaving only the click listener.
    rect.remove_listeners("pointermove");
    rect.remove_listeners("pointerdown");

    dispatch(&rect, "click")?;
    common::check_eq(clicks.get(), 1)
}

/// `clear_listeners` and `remove_listeners` are harmless no-ops on a node that has no managed listeners (including a
/// passive node that never allocated listener storage).
#[wasm_bindgen_test]
fn should_be_noop_to_remove_listeners_when_none_registered() -> Result<(), String> {
    let rect = make_svg("node-remove-none")
        .rect(Point::origin(), Size::new(200.0, 200.0))
        .map_err(|e| e.to_string())?;
    rect.remove_listeners("click"); // no storage allocated yet
    rect.clear_listeners();
    common::check(true, "removing absent listeners must not panic")
}

/// A node can have fresh listeners registered after `clear_listeners`, confirming the store returns to a usable empty
/// state rather than being permanently disabled.
#[wasm_bindgen_test]
fn should_allow_reregistration_after_clear_listeners() -> Result<(), String> {
    let rect = make_svg("node-reregister")
        .rect(Point::origin(), Size::new(200.0, 200.0))
        .map_err(|e| e.to_string())?;
    let count = Rc::new(Cell::new(0u32));
    let first = count.clone();
    rect.on_click(move |_| first.set(first.get() + 1)).map_err(|e| e.to_string())?;

    rect.clear_listeners();

    let second = count.clone();
    rect.on_click(move |_| second.set(second.get() + 1))
        .map_err(|e| e.to_string())?;
    dispatch(&rect, "click")?;

    common::check_eq(count.get(), 1)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Self-referential listeners — strong cycle vs WeakSvgNode
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// A listener that captures a *weak* self-handle does not keep its node alive: dropping the last strong handle frees
/// the node and removes the listener, which is exactly what `downgrade` is for.
#[wasm_bindgen_test]
fn should_drop_node_when_listener_captures_weak_self() -> Result<(), String> {
    let rect = make_svg("node-weak-self")
        .rect(Point::origin(), Size::new(200.0, 200.0))
        .map_err(|e| e.to_string())?;
    let elem = rect.as_element().clone();
    let count = Rc::new(Cell::new(0u32));

    let seen = count.clone();
    let rect_weak = rect.downgrade();
    rect.on_click(move |_| {
        seen.set(seen.get() + 1);
        if let Some(rect) = rect_weak.upgrade() {
            let _ = rect.set_fill("gold");
        }
    })
    .map_err(|e| e.to_string())?;

    dispatch_element(&elem, "click")?;
    common::check_eq(count.get(), 1)?;

    // The closure holds only a Weak, so `rect` is the last strong handle. Dropping it frees the node and removes the
    // listener, so a further event must not fire.
    drop(rect);
    dispatch_element(&elem, "click")?;
    common::check_eq(count.get(), 1)
}

/// Documents the footgun that [`svg_dom::WeakSvgNode`] exists to avoid: a listener capturing a *strong* clone of its
/// own node forms a reference cycle, so dropping the "last" external handle does **not** free the node — the listener
/// stays live and keeps firing. Self-mutating listeners should capture a weak handle instead (see the test above).
#[wasm_bindgen_test]
fn should_leak_when_listener_captures_strong_self() -> Result<(), String> {
    let rect = make_svg("node-strong-self")
        .rect(Point::origin(), Size::new(200.0, 200.0))
        .map_err(|e| e.to_string())?;
    let elem = rect.as_element().clone();
    let count = Rc::new(Cell::new(0u32));

    let seen = count.clone();
    let rect_strong = rect.clone(); // strong self-capture — this is the cycle
    rect.on_click(move |_| {
        seen.set(seen.get() + 1);
        let _ = rect_strong.set_fill("gold");
    })
    .map_err(|e| e.to_string())?;

    dispatch_element(&elem, "click")?;
    common::check_eq(count.get(), 1)?;

    // Dropping the external handle does NOT free the node: the closure still owns a strong clone of it, so the listener
    // remains attached and fires again. This is the leak that capturing a WeakSvgNode would prevent.
    drop(rect);
    dispatch_element(&elem, "click")?;
    common::check_eq(count.get(), 2)
}
