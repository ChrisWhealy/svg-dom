use std::{cell::Cell, rc::Rc};
use wasm_bindgen_test::*;
use web_sys::{MouseEvent, SvgElement};
use svg_dom::{SvgRoot, root::utils::*};

mod common;

wasm_bindgen_test_configure!(run_in_browser);

// Helper: create a container div and an SvgRoot inside it for a given test.
// Using a unique `container_id` per test prevents DOM id collisions.
fn make_svg(container_id: &str) -> SvgRoot {
    common::div(container_id);
    SvgRoot::create_in(container_id, Size { width: 200.0, height: 200.0 }).unwrap()
}

// Helper: dispatch a synthetic MouseEvent directly to a node's underlying element.
// `dispatch_event` is synchronous in browsers, so the handler fires before this returns.
fn dispatch(node: &svg_dom::SvgNode, event_type: &str) -> Result<(), String> {
    dispatch_element(node.as_element(), event_type)
}

fn dispatch_element(element: &SvgElement, event_type: &str) -> Result<(), String> {
    let event = MouseEvent::new(event_type).map_err(|e| format!("{e:?}"))?;
    element.dispatch_event(&event).map_err(|e| format!("{e:?}"))?;
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// set_attr / attr
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// An attribute written with `set_attr` is immediately readable back via `attr`.
#[wasm_bindgen_test]
fn should_read_new_value_after_calling_set_attr() -> Result<(), String> {
    let rect = make_svg("node-set-attr").rect(Point::origin(), Size::new(50.0, 50.0)).map_err(|e| e.to_string())?;
    rect.set_attr("opacity", "0.5").map_err(|e| e.to_string())?;
    common::check_eq(rect.attr("opacity"), Some("0.5".into()))
}

/// `attr` returns `None` for an attribute that has never been set.
#[wasm_bindgen_test]
fn should_return_none_for_absent_attribute() -> Result<(), String> {
    let rect = make_svg("node-attr-absent").rect(Point::origin(), Size::new(50.0, 50.0)).map_err(|e| e.to_string())?;
    common::check_eq(rect.attr("nonexistent"), None)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// remove_attr
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// After `remove_attr`, `attr` returns `None` for that attribute.
#[wasm_bindgen_test]
fn should_return_none_after_reading_removed_attribute() -> Result<(), String> {
    let rect = make_svg("node-remove-attr").rect(Point::origin(), Size::new(50.0, 50.0)).map_err(|e| e.to_string())?;
    rect.set_attr("opacity", "0.5").map_err(|e| e.to_string())?;
    rect.remove_attr("opacity").map_err(|e| e.to_string())?;
    common::check_eq(rect.attr("opacity"), None)
}

/// Calling `remove_attr` twice on the same attribute is idempotent
#[wasm_bindgen_test]
fn should_succeed_after_calling_remove_attr_twice() -> Result<(), String> {
    let rect = make_svg("node-remove-attr-twice").rect(Point::origin(), Size::new(50.0, 50.0)).map_err(|e| e.to_string())?;
    rect.set_attr("opacity", "0.5").map_err(|e| e.to_string())?;
    rect.remove_attr("opacity").map_err(|e| e.to_string())?;
    rect.remove_attr("opacity").map_err(|e| e.to_string())
}

/// Calling `remove_attr` on a non-existent attribute succeeds without error
#[wasm_bindgen_test]
fn should_successfully_remove_nonexistent_attribute() -> Result<(), String> {
    let rect = make_svg("node-remove-absent").rect(Point::origin(), Size::new(50.0, 50.0)).map_err(|e| e.to_string())?;
    rect.remove_attr("nonexistent").map_err(|e| e.to_string())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Clone semantics
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Cloning a node produces a second handle to the same underlying DOM element: a mutation made through the clone is
/// immediately visible via the original.
#[wasm_bindgen_test]
fn should_refer_to_same_dom_node_after_clone() -> Result<(), String> {
    let rect  = make_svg("node-clone").rect(Point::origin(), Size::new(50.0, 50.0)).map_err(|e| e.to_string())?;
    let clone = rect.clone();
    clone.set_fill("crimson").map_err(|e| e.to_string())?;
    common::check_eq(rect.attr("fill"), Some("crimson".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Convenience attribute setters
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `set_fill` writes the `fill` attribute.
#[wasm_bindgen_test]
fn should_update_fill_attribute_after_calling_set_fill() -> Result<(), String> {
    let rect = make_svg("node-fill").rect(Point::origin(), Size::new(50.0, 50.0)).map_err(|e| e.to_string())?;
    rect.set_fill("steelblue").map_err(|e| e.to_string())?;
    common::check_eq(rect.attr("fill"), Some("steelblue".into()))
}

/// `set_stroke` writes the `stroke` attribute.
#[wasm_bindgen_test]
fn should_update_stroke_attribute_after_calling_set_stroke() -> Result<(), String> {
    let rect = make_svg("node-stroke").rect(Point::origin(), Size::new(50.0, 50.0)).map_err(|e| e.to_string())?;
    rect.set_stroke("black").map_err(|e| e.to_string())?;
    common::check_eq(rect.attr("stroke"), Some("black".into()))
}

/// `set_stroke_width` writes the `stroke-width` attribute.
#[wasm_bindgen_test]
fn should_update_stroke_width_attribute_after_calling_set_stroke_width() -> Result<(), String> {
    let line = make_svg("node-stroke-width").line(Point::origin(), Point::new(100.0, 100.0)).map_err(|e| e.to_string())?;
    line.set_stroke_width(3.0).map_err(|e| e.to_string())?;
    common::check_eq(line.attr("stroke-width"), Some("3".into()))
}

/// `set_d` replaces the `d` attribute of a `<path>` element.
#[wasm_bindgen_test]
fn should_update_path_data_after_calling_set_d() -> Result<(), String> {
    let path = make_svg("node-set-d").path("M 0 0 L 50 50").map_err(|e| e.to_string())?;
    path.set_d("M 10 10 Q 50 0 90 10").map_err(|e| e.to_string())?;
    common::check_eq(path.attr("d"), Some("M 10 10 Q 50 0 90 10".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// append
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `append` transfers an element from one parent so it becomes the child of the new target node.
#[wasm_bindgen_test]
fn should_append_element_to_group() -> Result<(), String> {
    let svg   = make_svg("node-append");
    let group = svg.group().map_err(|e| e.to_string())?;
    let rect  = svg.rect(Point::origin(), Size::new(50.0, 50.0)).map_err(|e| e.to_string())?;
    group.append(&rect).map_err(|e| e.to_string())?;
    common::check_eq(group.as_element().child_element_count(), 1)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Event handlers
//
// `EventTarget::dispatch_event` is synchronous: the browser fires the handler inline before `dispatch_event` returns.
// No need for any async shenanigans
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// An `on_click` handler fires when a synthetic `click` event is dispatched.
#[wasm_bindgen_test]
fn should_fire_on_click_after_synthetic_click() -> Result<(), String> {
    let rect    = make_svg("node-click").rect(Point::origin(), Size::new(200.0, 200.0)).map_err(|e| e.to_string())?;
    let fired   = Rc::new(Cell::new(false));
    let fired_c = fired.clone();
    rect.on_click(move |_| { fired_c.set(true); }).map_err(|e| e.to_string())?;
    dispatch(&rect, "click")?;
    common::check(fired.get(), "click handler did not fire")
}

/// An `on_mouseover` handler fires when a synthetic `mouseover` event is dispatched.
#[wasm_bindgen_test]
fn should_fire_on_mouseover_after_synthetic_mouseover() -> Result<(), String> {
    let rect    = make_svg("node-mouseover").rect(Point::origin(), Size::new(200.0, 200.0)).map_err(|e| e.to_string())?;
    let fired   = Rc::new(Cell::new(false));
    let fired_c = fired.clone();
    rect.on_mouseover(move |_| { fired_c.set(true); }).map_err(|e| e.to_string())?;
    dispatch(&rect, "mouseover")?;
    common::check(fired.get(), "mouseover handler did not fire")
}

/// An `on_mouseout` handler fires when a synthetic `mouseout` event is dispatched.
#[wasm_bindgen_test]
fn should_fire_on_mouseout_after_synthetic_mouseout() -> Result<(), String> {
    let rect    = make_svg("node-mouseout").rect(Point::origin(), Size::new(200.0, 200.0)).map_err(|e| e.to_string())?;
    let fired   = Rc::new(Cell::new(false));
    let fired_c = fired.clone();
    rect.on_mouseout(move |_| { fired_c.set(true); }).map_err(|e| e.to_string())?;
    dispatch(&rect, "mouseout")?;
    common::check(fired.get(), "mouseout handler did not fire")
}

/// Registering multiple handlers for the same event on the same node results in all of
/// them firing when the event is dispatched.
#[wasm_bindgen_test]
fn should_fire_multiple_handlers_on_same_node() -> Result<(), String> {
    let rect    = make_svg("node-multi-handler").rect(Point::origin(), Size::new(200.0, 200.0)).map_err(|e| e.to_string())?;
    let count   = Rc::new(Cell::new(0u32));
    let count_1 = count.clone();
    let count_2 = count.clone();
    let count_3 = count.clone();
    rect.on_click(move |_| { count_1.set(count_1.get() + 1); }).map_err(|e| e.to_string())?;
    rect.on_click(move |_| { count_2.set(count_2.get() + 1); }).map_err(|e| e.to_string())?;
    rect.on_click(move |_| { count_3.set(count_3.get() + 1); }).map_err(|e| e.to_string())?;
    dispatch(&rect, "click")?;
    common::check_eq(count.get(), 3)
}

/// A handler registered on the original `SvgNode` fires when the event is dispatched via
/// a clone, confirming that both handles point to the same underlying DOM element.
#[wasm_bindgen_test]
fn should_fire_original_handler_when_dispatched_via_clone() -> Result<(), String> {
    let rect    = make_svg("node-clone-event").rect(Point::origin(), Size::new(200.0, 200.0)).map_err(|e| e.to_string())?;
    let clone   = rect.clone();
    let fired   = Rc::new(Cell::new(false));
    let fired_c = fired.clone();
    rect.on_click(move |_| { fired_c.set(true); }).map_err(|e| e.to_string())?;
    dispatch(&clone, "click")?;
    common::check(fired.get(), "click handler did not fire when dispatched via clone")
}

/// Dropping the final `SvgNode` handle removes its registered listener from the DOM before
/// dropping the stored wasm-bindgen `Closure`.  Keeping a clone of the raw DOM element lets
/// the test dispatch another event after the Rust-side listener storage has been freed.
#[wasm_bindgen_test]
fn should_remove_dom_listener_when_final_node_handle_is_dropped() -> Result<(), String> {
    let rect  = make_svg("node-drop-listener").rect(Point::origin(), Size::new(200.0, 200.0)).map_err(|e| e.to_string())?;
    let elem  = rect.as_element().clone();
    let count = Rc::new(Cell::new(0u32));
    let seen  = count.clone();

    rect.on_click(move |_| { seen.set(seen.get() + 1); }).map_err(|e| e.to_string())?;

    dispatch_element(&elem, "click")?;
    common::check_eq(count.get(), 1)?;

    drop(rect);
    dispatch_element(&elem, "click")?;

    common::check_eq(count.get(), 1)
}
