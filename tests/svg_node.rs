use std::{cell::Cell, rc::Rc};
use wasm_bindgen_test::*;
use web_sys::{DragEvent, Event, FocusEvent, KeyboardEvent, MouseEvent, PointerEvent, SvgElement, TouchEvent, WheelEvent};
use svg_dom::{SvgRoot, root::utils::*};

mod common;

wasm_bindgen_test_configure!(run_in_browser);

// Helper: create a container div and an SvgRoot inside it for a given test.
// Using a unique `container_id` per test prevents DOM id collisions.
fn make_svg(container_id: &str) -> SvgRoot {
    common::div(container_id);
    SvgRoot::create_in(container_id, Size { width: 200.0, height: 200.0 }).unwrap()
}

// Helper: dispatch a synthetic mouse/pointer event directly to a node's underlying element.
// `dispatch_event` is synchronous in browsers, so the handler fires before this returns.
fn dispatch(node: &svg_dom::SvgNode, event_type: &str) -> Result<(), String> {
    dispatch_element(node.as_element(), event_type)
}

fn dispatch_element(element: &SvgElement, event_type: &str) -> Result<(), String> {
    if event_type.starts_with("pointer") {
        let event = PointerEvent::new(event_type).map_err(|e| format!("{e:?}"))?;
        element.dispatch_event(&event).map_err(|e| format!("{e:?}"))?;
    } else if event_type.starts_with("key") {
        let event = KeyboardEvent::new(event_type).map_err(|e| format!("{e:?}"))?;
        element.dispatch_event(&event).map_err(|e| format!("{e:?}"))?;
    } else if matches!(event_type, "focus" | "blur") {
        let event = FocusEvent::new(event_type).map_err(|e| format!("{e:?}"))?;
        element.dispatch_event(&event).map_err(|e| format!("{e:?}"))?;
    } else if event_type == "wheel" {
        let event = WheelEvent::new(event_type).map_err(|e| format!("{e:?}"))?;
        element.dispatch_event(&event).map_err(|e| format!("{e:?}"))?;
    } else if event_type.starts_with("touch") {
        let event = TouchEvent::new(event_type).map_err(|e| format!("{e:?}"))?;
        element.dispatch_event(&event).map_err(|e| format!("{e:?}"))?;
    } else if event_type.starts_with("drag") || event_type == "drop" {
        let event = DragEvent::new(event_type).map_err(|e| format!("{e:?}"))?;
        element.dispatch_event(&event).map_err(|e| format!("{e:?}"))?;
    } else if event_type == "custom-svg-dom-test" {
        let event = Event::new(event_type).map_err(|e| format!("{e:?}"))?;
        element.dispatch_event(&event).map_err(|e| format!("{e:?}"))?;
    } else {
        let event = MouseEvent::new(event_type).map_err(|e| format!("{e:?}"))?;
        element.dispatch_event(&event).map_err(|e| format!("{e:?}"))?;
    }
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

/// `set_attrs` writes each supplied attribute and all values are immediately readable back.
#[wasm_bindgen_test]
fn should_set_multiple_attributes_in_one_call() -> Result<(), String> {
    let rect = make_svg("node-set-attrs")
        .rect(Point::origin(), Size::new(50.0, 50.0))
        .map_err(|e| e.to_string())?;

    rect.set_attrs([
        ("fill", "steelblue"),
        ("stroke", "white"),
        ("stroke-width", "2"),
    ])
    .map_err(|e| e.to_string())?;

    common::check_eq(rect.attr("fill"), Some("steelblue".into()))?;
    common::check_eq(rect.attr("stroke"), Some("white".into()))?;
    common::check_eq(rect.attr("stroke-width"), Some("2".into()))
}

/// `set_attrs` accepts owned String values as well as string literals.
#[wasm_bindgen_test]
fn should_set_multiple_owned_string_attributes_in_one_call() -> Result<(), String> {
    let line = make_svg("node-set-attrs-owned")
        .line(Point::origin(), Point::new(10.0, 20.0))
        .map_err(|e| e.to_string())?;

    line.set_attrs([
        ("x1", 1.0_f64.to_string()),
        ("y1", 2.0_f64.to_string()),
        ("x2", 3.0_f64.to_string()),
        ("y2", 4.0_f64.to_string()),
    ])
    .map_err(|e| e.to_string())?;

    common::check_eq(line.attr("x1"), Some("1".into()))?;
    common::check_eq(line.attr("y1"), Some("2".into()))?;
    common::check_eq(line.attr("x2"), Some("3".into()))?;
    common::check_eq(line.attr("y2"), Some("4".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// set_attr_if_changed
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `set_attr_if_changed` writes an attribute that was previously absent and the new value is readable back.
#[wasm_bindgen_test]
fn should_write_absent_attribute_with_set_attr_if_changed() -> Result<(), String> {
    let rect = make_svg("node-set-attr-if-changed-absent")
        .rect(Point::origin(), Size::new(50.0, 50.0))
        .map_err(|e| e.to_string())?;
    rect.set_attr_if_changed("style", "cursor:grab").map_err(|e| e.to_string())?;
    common::check_eq(rect.attr("style"), Some("cursor:grab".into()))
}

/// `set_attr_if_changed` updates the attribute when the new value differs from the current one.
#[wasm_bindgen_test]
fn should_update_attribute_when_value_differs() -> Result<(), String> {
    let rect = make_svg("node-set-attr-if-changed-update")
        .rect(Point::origin(), Size::new(50.0, 50.0))
        .map_err(|e| e.to_string())?;
    rect.set_attr_if_changed("style", "cursor:grab").map_err(|e| e.to_string())?;
    rect.set_attr_if_changed("style", "cursor:grabbing").map_err(|e| e.to_string())?;
    common::check_eq(rect.attr("style"), Some("cursor:grabbing".into()))
}

/// Setting the same value again via `set_attr_if_changed` leaves the attribute unchanged and reports success.
#[wasm_bindgen_test]
fn should_leave_attribute_unchanged_when_value_matches() -> Result<(), String> {
    let rect = make_svg("node-set-attr-if-changed-noop")
        .rect(Point::origin(), Size::new(50.0, 50.0))
        .map_err(|e| e.to_string())?;
    rect.set_attr_if_changed("opacity", "0.5").map_err(|e| e.to_string())?;
    rect.set_attr_if_changed("opacity", "0.5").map_err(|e| e.to_string())?;
    common::check_eq(rect.attr("opacity"), Some("0.5".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// CachedAttr
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// The first `CachedAttr::set` writes, since there is no remembered value to compare against.
#[wasm_bindgen_test]
fn should_write_first_cached_value() -> Result<(), String> {
    let rect = make_svg("node-cached-attr-first")
        .rect(Point::origin(), Size::new(50.0, 50.0))
        .map_err(|e| e.to_string())?;
    let mut cache = svg_dom::CachedAttr::new();
    cache.set(&rect, "style", "cursor:grab").map_err(|e| e.to_string())?;
    common::check_eq(rect.attr("style"), Some("cursor:grab".into()))
}

/// When the cached value is unchanged, `set` does not touch the DOM. We prove this by mutating the attribute behind the
/// cache's back: a cached no-op must leave that external value in place rather than rewriting the cached one.
#[wasm_bindgen_test]
fn should_skip_dom_write_when_cached_value_unchanged() -> Result<(), String> {
    let rect = make_svg("node-cached-attr-noop")
        .rect(Point::origin(), Size::new(50.0, 50.0))
        .map_err(|e| e.to_string())?;
    let mut cache = svg_dom::CachedAttr::new();
    cache.set(&rect, "style", "cursor:grab").map_err(|e| e.to_string())?;

    // Change the attribute through a different path; the cache still believes "cursor:grab" is current.
    rect.set_attr("style", "cursor:wait").map_err(|e| e.to_string())?;

    // Same value as cached → no write, so the external "cursor:wait" survives.
    cache.set(&rect, "style", "cursor:grab").map_err(|e| e.to_string())?;
    common::check_eq(rect.attr("style"), Some("cursor:wait".into()))
}

/// A changed value writes through and updates the cache.
#[wasm_bindgen_test]
fn should_write_changed_cached_value() -> Result<(), String> {
    let rect = make_svg("node-cached-attr-change")
        .rect(Point::origin(), Size::new(50.0, 50.0))
        .map_err(|e| e.to_string())?;
    let mut cache = svg_dom::CachedAttr::new();
    cache.set(&rect, "style", "cursor:grab").map_err(|e| e.to_string())?;
    cache.set(&rect, "style", "cursor:grabbing").map_err(|e| e.to_string())?;
    common::check_eq(rect.attr("style"), Some("cursor:grabbing".into()))
}

/// After `invalidate`, the next `set` writes even if the value matches what was last written.
#[wasm_bindgen_test]
fn should_write_after_invalidate() -> Result<(), String> {
    let rect = make_svg("node-cached-attr-invalidate")
        .rect(Point::origin(), Size::new(50.0, 50.0))
        .map_err(|e| e.to_string())?;
    let mut cache = svg_dom::CachedAttr::new();
    cache.set(&rect, "style", "cursor:grab").map_err(|e| e.to_string())?;

    rect.set_attr("style", "cursor:wait").map_err(|e| e.to_string())?;
    cache.invalidate();

    // Cache was invalidated, so this writes through and restores "cursor:grab".
    cache.set(&rect, "style", "cursor:grab").map_err(|e| e.to_string())?;
    common::check_eq(rect.attr("style"), Some("cursor:grab".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// transform helpers (set_translate / set_rotate / set_scale / ...)
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `set_translate` writes a `translate(x, y)` transform formatted to one decimal place.
#[wasm_bindgen_test]
fn should_write_translate_transform() -> Result<(), String> {
    let node = make_svg("node-set-translate").group().map_err(|e| e.to_string())?;
    let mut buf = String::new();
    node.set_translate(&mut buf, 100.0, 50.0).map_err(|e| e.to_string())?;
    common::check_eq(node.attr("transform"), Some("translate(100.0, 50.0)".into()))
}

/// The same scratch buffer can be reused across calls and the latest value wins.
#[wasm_bindgen_test]
fn should_reuse_scratch_buffer_across_translate_calls() -> Result<(), String> {
    let node = make_svg("node-translate-reuse").group().map_err(|e| e.to_string())?;
    let mut buf = String::new();
    node.set_translate(&mut buf, 1.0, 2.0).map_err(|e| e.to_string())?;
    node.set_translate(&mut buf, 33.0, 44.0).map_err(|e| e.to_string())?;
    common::check_eq(node.attr("transform"), Some("translate(33.0, 44.0)".into()))
}

/// `set_rotate` writes a single-argument `rotate(angle)` transform.
#[wasm_bindgen_test]
fn should_write_rotate_transform() -> Result<(), String> {
    let node = make_svg("node-set-rotate").group().map_err(|e| e.to_string())?;
    let mut buf = String::new();
    node.set_rotate(&mut buf, 45.0).map_err(|e| e.to_string())?;
    common::check_eq(node.attr("transform"), Some("rotate(45.0)".into()))
}

/// `set_rotate_about` writes a `rotate(angle, cx, cy)` transform.
#[wasm_bindgen_test]
fn should_write_rotate_about_transform() -> Result<(), String> {
    let node = make_svg("node-set-rotate-about").group().map_err(|e| e.to_string())?;
    let mut buf = String::new();
    node.set_rotate_about(&mut buf, 90.0, 10.0, 20.0).map_err(|e| e.to_string())?;
    common::check_eq(node.attr("transform"), Some("rotate(90.0, 10.0, 20.0)".into()))
}

/// `set_scale` writes a uniform `scale(s)` transform formatted to three decimal places.
#[wasm_bindgen_test]
fn should_write_uniform_scale_transform() -> Result<(), String> {
    let node = make_svg("node-set-scale").group().map_err(|e| e.to_string())?;
    let mut buf = String::new();
    node.set_scale(&mut buf, 1.5).map_err(|e| e.to_string())?;
    common::check_eq(node.attr("transform"), Some("scale(1.500)".into()))
}

/// `set_scale_xy` writes a non-uniform `scale(x, y)` transform.
#[wasm_bindgen_test]
fn should_write_non_uniform_scale_transform() -> Result<(), String> {
    let node = make_svg("node-set-scale-xy").group().map_err(|e| e.to_string())?;
    let mut buf = String::new();
    node.set_scale_xy(&mut buf, 2.0, 0.5).map_err(|e| e.to_string())?;
    common::check_eq(node.attr("transform"), Some("scale(2.000, 0.500)".into()))
}

/// `set_translate_scale` writes the combined `translate(...) scale(...)` shape used by pan/zoom code.
#[wasm_bindgen_test]
fn should_write_translate_scale_transform() -> Result<(), String> {
    let node = make_svg("node-set-translate-scale").group().map_err(|e| e.to_string())?;
    let mut buf = String::new();
    node.set_translate_scale(&mut buf, 12.0, 34.0, 2.0).map_err(|e| e.to_string())?;
    common::check_eq(node.attr("transform"), Some("translate(12.0, 34.0) scale(2.000)".into()))
}

/// `set_transform_fmt` writes an arbitrary transform built from `format_args!`.
#[wasm_bindgen_test]
fn should_write_arbitrary_transform_via_fmt() -> Result<(), String> {
    let node = make_svg("node-set-transform-fmt").group().map_err(|e| e.to_string())?;
    let mut buf = String::new();
    let (x, y, angle) = (10.0, 20.0, 45.0);
    node.set_transform_fmt(&mut buf, format_args!("translate({x:.1}, {y:.1}) rotate({angle:.1})"))
        .map_err(|e| e.to_string())?;
    common::check_eq(node.attr("transform"), Some("translate(10.0, 20.0) rotate(45.0)".into()))
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

/// An `on_pointerenter` handler fires when a synthetic `pointerenter` event is dispatched.
#[wasm_bindgen_test]
fn should_fire_on_pointerenter_after_synthetic_pointerenter() -> Result<(), String> {
    let rect    = make_svg("node-pointerenter").rect(Point::origin(), Size::new(200.0, 200.0)).map_err(|e| e.to_string())?;
    let fired   = Rc::new(Cell::new(false));
    let fired_c = fired.clone();
    rect.on_pointerenter(move |_| { fired_c.set(true); }).map_err(|e| e.to_string())?;
    dispatch(&rect, "pointerenter")?;
    common::check(fired.get(), "pointerenter handler did not fire")
}

/// An `on_pointerleave` handler fires when a synthetic `pointerleave` event is dispatched.
#[wasm_bindgen_test]
fn should_fire_on_pointerleave_after_synthetic_pointerleave() -> Result<(), String> {
    let rect    = make_svg("node-pointerleave").rect(Point::origin(), Size::new(200.0, 200.0)).map_err(|e| e.to_string())?;
    let fired   = Rc::new(Cell::new(false));
    let fired_c = fired.clone();
    rect.on_pointerleave(move |_| { fired_c.set(true); }).map_err(|e| e.to_string())?;
    dispatch(&rect, "pointerleave")?;
    common::check(fired.get(), "pointerleave handler did not fire")
}

/// Managed mouse wrappers cover common pointer-with-mouse APIs without needing raw `Closure::forget` listeners.
#[wasm_bindgen_test]
fn should_fire_managed_mouse_event_wrappers() -> Result<(), String> {
    let rect  = make_svg("node-mouse-wrappers").rect(Point::origin(), Size::new(200.0, 200.0)).map_err(|e| e.to_string())?;
    let count = Rc::new(Cell::new(0u32));

    let c = count.clone(); rect.on_dblclick(move |_| { c.set(c.get() + 1); }).map_err(|e| e.to_string())?;
    let c = count.clone(); rect.on_contextmenu(move |e| { e.prevent_default(); c.set(c.get() + 1); }).map_err(|e| e.to_string())?;
    let c = count.clone(); rect.on_mousedown(move |_| { c.set(c.get() + 1); }).map_err(|e| e.to_string())?;
    let c = count.clone(); rect.on_mouseup(move |_| { c.set(c.get() + 1); }).map_err(|e| e.to_string())?;
    let c = count.clone(); rect.on_mousemove(move |_| { c.set(c.get() + 1); }).map_err(|e| e.to_string())?;
    let c = count.clone(); rect.on_mouseenter(move |_| { c.set(c.get() + 1); }).map_err(|e| e.to_string())?;
    let c = count.clone(); rect.on_mouseleave(move |_| { c.set(c.get() + 1); }).map_err(|e| e.to_string())?;

    for event_type in ["dblclick", "contextmenu", "mousedown", "mouseup", "mousemove", "mouseenter", "mouseleave"] {
        dispatch(&rect, event_type)?;
    }

    common::check_eq(count.get(), 7)
}

/// Managed pointer wrappers cover the common SVG pointer interaction lifecycle.
#[wasm_bindgen_test]
fn should_fire_managed_pointer_event_wrappers() -> Result<(), String> {
    let rect  = make_svg("node-pointer-wrappers").rect(Point::origin(), Size::new(200.0, 200.0)).map_err(|e| e.to_string())?;
    let count = Rc::new(Cell::new(0u32));

    let c = count.clone(); rect.on_pointerdown(move |_| { c.set(c.get() + 1); }).map_err(|e| e.to_string())?;
    let c = count.clone(); rect.on_pointerup(move |_| { c.set(c.get() + 1); }).map_err(|e| e.to_string())?;
    let c = count.clone(); rect.on_pointermove(move |_| { c.set(c.get() + 1); }).map_err(|e| e.to_string())?;
    let c = count.clone(); rect.on_pointercancel(move |_| { c.set(c.get() + 1); }).map_err(|e| e.to_string())?;
    let c = count.clone(); rect.on_pointerover(move |_| { c.set(c.get() + 1); }).map_err(|e| e.to_string())?;
    let c = count.clone(); rect.on_pointerout(move |_| { c.set(c.get() + 1); }).map_err(|e| e.to_string())?;

    for event_type in ["pointerdown", "pointerup", "pointermove", "pointercancel", "pointerover", "pointerout"] {
        dispatch(&rect, event_type)?;
    }

    common::check_eq(count.get(), 6)
}

/// Managed non-mouse wrappers cover keyboard, focus, wheel, drag-and-drop and generic events.
#[wasm_bindgen_test]
fn should_fire_managed_non_mouse_event_wrappers() -> Result<(), String> {
    let rect  = make_svg("node-other-event-wrappers").rect(Point::origin(), Size::new(200.0, 200.0)).map_err(|e| e.to_string())?;
    rect.set_attr("tabindex", "0").map_err(|e| e.to_string())?;
    rect.set_attr("draggable", "true").map_err(|e| e.to_string())?;
    let count = Rc::new(Cell::new(0u32));

    let c = count.clone(); rect.on_wheel(move |_| { c.set(c.get() + 1); }).map_err(|e| e.to_string())?;
    let c = count.clone(); rect.on_touchstart(move |_| { c.set(c.get() + 1); }).map_err(|e| e.to_string())?;
    let c = count.clone(); rect.on_touchmove(move |_| { c.set(c.get() + 1); }).map_err(|e| e.to_string())?;
    let c = count.clone(); rect.on_touchend(move |_| { c.set(c.get() + 1); }).map_err(|e| e.to_string())?;
    let c = count.clone(); rect.on_touchcancel(move |_| { c.set(c.get() + 1); }).map_err(|e| e.to_string())?;
    let c = count.clone(); rect.on_keydown(move |_| { c.set(c.get() + 1); }).map_err(|e| e.to_string())?;
    let c = count.clone(); rect.on_keyup(move |_| { c.set(c.get() + 1); }).map_err(|e| e.to_string())?;
    let c = count.clone(); rect.on_focus(move |_| { c.set(c.get() + 1); }).map_err(|e| e.to_string())?;
    let c = count.clone(); rect.on_blur(move |_| { c.set(c.get() + 1); }).map_err(|e| e.to_string())?;
    let c = count.clone(); rect.on_dragstart(move |_| { c.set(c.get() + 1); }).map_err(|e| e.to_string())?;
    let c = count.clone(); rect.on_drag(move |_| { c.set(c.get() + 1); }).map_err(|e| e.to_string())?;
    let c = count.clone(); rect.on_dragend(move |_| { c.set(c.get() + 1); }).map_err(|e| e.to_string())?;
    let c = count.clone(); rect.on_dragenter(move |_| { c.set(c.get() + 1); }).map_err(|e| e.to_string())?;
    let c = count.clone(); rect.on_dragleave(move |_| { c.set(c.get() + 1); }).map_err(|e| e.to_string())?;
    let c = count.clone(); rect.on_dragover(move |e| { e.prevent_default(); c.set(c.get() + 1); }).map_err(|e| e.to_string())?;
    let c = count.clone(); rect.on_drop(move |_| { c.set(c.get() + 1); }).map_err(|e| e.to_string())?;
    let c = count.clone(); rect.on_event("custom-svg-dom-test", move |_| { c.set(c.get() + 1); }).map_err(|e| e.to_string())?;

    for event_type in [
        "wheel", "touchstart", "touchmove", "touchend", "touchcancel", "keydown", "keyup", "focus", "blur",
        "dragstart", "drag", "dragend", "dragenter", "dragleave", "dragover", "drop", "custom-svg-dom-test",
    ] {
        dispatch(&rect, event_type)?;
    }

    common::check_eq(count.get(), 17)
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

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// SvgAttrs / AttrWriter
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `SvgAttrs` reuses its scratch buffer while setting string, numeric and formatted attributes.
#[wasm_bindgen_test]
fn should_set_attributes_with_reusable_attr_writer() -> Result<(), String> {
    let rect = make_svg("node-svg-attrs").rect(Point::origin(), Size::new(50.0, 50.0)).map_err(|e| e.to_string())?;
    let mut attrs = svg_dom::SvgAttrs::with_capacity(64);

    rect.attrs(&mut attrs)
        .fill("steelblue").map_err(|e| e.to_string())?
        .stroke("white").map_err(|e| e.to_string())?
        .stroke_width(2.5).map_err(|e| e.to_string())?
        .fmt("transform", format_args!("translate({}, {})", 10, 20)).map_err(|e| e.to_string())?;

    common::check_eq(rect.attr("fill"), Some("steelblue".into()))?;
    common::check_eq(rect.attr("stroke"), Some("white".into()))?;
    common::check_eq(rect.attr("stroke-width"), Some("2.5".into()))?;
    common::check_eq(rect.attr("transform"), Some("translate(10, 20)".into()))?;
    common::check(attrs.capacity() >= 64, "SvgAttrs should retain its scratch allocation")
}
