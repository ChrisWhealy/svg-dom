use svg_dom::{SvgNode, SvgRoot, root::utils::Size};
use web_sys::{DragEvent, Event, FocusEvent, KeyboardEvent, MouseEvent, PointerEvent, SvgElement, WheelEvent};

/// Creates a container div and an `SvgRoot` inside it for a given test.
/// Using a unique `container_id` per test prevents DOM id collisions.
pub fn make_svg(container_id: &str) -> SvgRoot {
    crate::common::div(container_id);
    SvgRoot::create_in(container_id, Size { width: 200.0, height: 200.0 }).unwrap()
}

/// Dispatches a synthetic mouse/pointer event directly to a node's underlying element.
/// `dispatch_event` is synchronous in browsers, so the handler fires before this returns.
pub fn dispatch(node: &SvgNode, event_type: &str) -> Result<(), String> {
    dispatch_element(node.as_element(), event_type)
}

pub fn dispatch_element(element: &SvgElement, event_type: &str) -> Result<(), String> {
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
        // Desktop Firefox does not expose the `TouchEvent` constructor, so dispatch a generically-typed event carrying
        // the touch type name. A listener fires on the event-type string regardless of the concrete event interface, so
        // this still exercises the managed `on_touch*` wrappers in every browser.
        let event = Event::new(event_type).map_err(|e| format!("{e:?}"))?;
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
