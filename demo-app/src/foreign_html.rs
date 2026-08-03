use std::rc::Rc;

use crate::{DemoClosure, NS_XHTML, dom_err, keep_demo_closure};
use svg_dom::{Error, SvgNode};
use wasm_bindgen::{JsCast, prelude::*};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Shared plumbing for building plain HTML inside an SVG `<foreignObject>`.
// Every demo module that builds a `<foreignObject>` control uses this file.
// Examples include `texts::demo_text`'s radio groups, `structure::demo_marker_view_box`'s slider, and
// `paint::demo_blend`'s BlendMode dropdown.
// This module is crate-local (`pub(crate)`), not part of `demo-app`'s public surface.
// No code outside this crate builds these controls.
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Creates an XHTML-namespaced element.
/// This shortens `create_element_ns(Some(NS_XHTML), tag).map_err(dom_err)` to one call.
/// Every plain HTML element built inside a `<foreignObject>` needs this call.
pub(crate) fn xhtml(document: &web_sys::Document, tag: &str) -> Result<web_sys::Element, Error> {
    document.create_element_ns(Some(NS_XHTML), tag).map_err(dom_err)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Returns the owner `Document` of an SVG node built via `SvgRoot::foreign_object`.
/// Every interactive control needs this document to build the plain HTML it places inside that `<foreignObject>`.
pub(crate) fn foreign_object_document(fo: &SvgNode) -> Result<web_sys::Document, Error> {
    fo.as_element()
        .owner_document()
        .ok_or_else(|| Error::Dom("cannot identify owner document for SVG foreignObject".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Builds a `<fieldset class="demo-control-group">` with a `<legend>` and one radio row per option.
///
/// Each row is a `<label class="demo-control-row">` that wraps an `<input type="radio">` where the `default` value
/// starts checked.
/// See `demo/style.css`'s `.demo-control-group` rule for the styling.
/// That styling lives in CSS, not as a repeated inline `style` string.
///
/// Selecting an option dispatches a real `change` event. That event calls `on_select` with the selected option's value.
/// Each caller writes its own attribute-setting call inside `on_select`, for example `set_text_anchor`.
/// This function only builds the shared radio group; it never sets the SVG attribute the callback actually changes.
/// A caller that needs to teach its own setter call, not this shared shape, is why the callback stays open.
///
/// Every closure this function creates is parked via [`keep_demo_closure`].
/// Parking keeps each closure alive after this function returns.
/// This function returns the finished `<fieldset>`, that has not yet been attached to the document.
/// The caller decides where the `<fieldset>` goes, usually inside its own `<foreignObject>`.
/// The caller must also call `keep_demo_node` on that `<foreignObject>`'s own `SvgNode`.
pub(crate) fn radio_group<T: Copy + PartialEq + 'static>(
    document: &web_sys::Document,
    legend_text: &str,
    input_name: &str,
    options: &[(T, &str)],
    default: T,
    on_select: impl Fn(T) + 'static,
) -> Result<web_sys::Element, Error> {
    let fieldset = xhtml(document, "fieldset")?;
    fieldset.set_attribute("class", "demo-control-group").map_err(dom_err)?;

    let legend = xhtml(document, "legend")?;
    legend.set_text_content(Some(legend_text));
    fieldset.append_child(&legend).map_err(dom_err)?;

    // Shared across every radio's own listener below. Each listener calls the same caller-supplied logic.
    let on_select = Rc::new(on_select);

    for &(value, label) in options {
        let row = xhtml(document, "label")?;
        row.set_attribute("class", "demo-control-row").map_err(dom_err)?;

        let input = xhtml(document, "input")?
            .dyn_into::<web_sys::HtmlInputElement>()
            .map_err(|_| Error::Dom("createElement(\"input\") did not return an HtmlInputElement".into()))?;
        input.set_type("radio");
        input.set_name(input_name);
        input.set_checked(value == default);

        // Each radio's own closure already knows its own option value. So on_select needs no lookup step.
        // A hand-rolled version with SVG elements would need to clear the other two selections itself.
        // The browser already does this through the shared `name` attribute above.
        let on_select = on_select.clone();
        let on_change: DemoClosure = Closure::new(move |_: web_sys::Event| on_select(value));
        input
            .add_event_listener_with_callback("change", on_change.as_ref().unchecked_ref())
            .map_err(dom_err)?;
        keep_demo_closure(on_change);

        row.append_child(&input).map_err(dom_err)?;
        row.append_child(&document.create_text_node(label)).map_err(dom_err)?;
        fieldset.append_child(&row).map_err(dom_err)?;
    }

    Ok(fieldset)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Builds a `<div class="demo-slider-container">` holding a visible label and a native
/// `<select class="demo-select">`, one `<option>` per entry in `options`.
///
/// [`radio_group`] lays out one visible control per option and is suitable in cases where there are small number of
/// choices.  However, for a long option list such as `BlendMode`'s sixteen keywords, that layout contributes to a poor
/// user experience.  One `<select>` collapses every choice into a single control instead.
///
/// Selecting an option dispatches a real `change` event. `on_select` then receives the selected option's own value, the
/// same shape [`radio_group`]'s own callback uses.
/// `labels` is `(visible, accessible)` and matches `build_h_slider`'s own convention.
/// `visible` sits above the control as plain text.
/// `accessible` becomes the `<select>`'s own `aria-label`.
///
/// Each `<option>`'s own `value` attribute is its index into `options`, not a caller-facing string.
/// `on_select` hands back the option's own `T` directly.
/// No caller needs to parse a string value back into its own `T`.
///
/// The closure this function creates is parked via [`keep_demo_closure`].
/// The returned `<div>` is not yet attached to the document.
/// The caller decides where it goes, usually inside its own `<foreignObject>`.
/// The caller must also call `keep_demo_node` on that `<foreignObject>`'s own `SvgNode`.
pub(crate) fn select_dropdown<T: Copy + PartialEq + 'static>(
    document: &web_sys::Document,
    labels: (&str, &str),
    options: &[(T, &str)],
    default: T,
    on_select: impl Fn(T) + 'static,
) -> Result<web_sys::Element, Error> {
    let (visible_label, accessible_label) = labels;
    let container = xhtml(document, "div")?;
    container.set_attribute("class", "demo-slider-container").map_err(dom_err)?;

    let label_el = xhtml(document, "div")?;
    label_el.set_attribute("class", "demo-slider-label").map_err(dom_err)?;
    label_el.set_text_content(Some(visible_label));
    container.append_child(&label_el).map_err(dom_err)?;

    let select = xhtml(document, "select")?
        .dyn_into::<web_sys::HtmlSelectElement>()
        .map_err(|_| Error::Dom("createElement(\"select\") did not return an HtmlSelectElement".into()))?;
    select.set_attribute("class", "demo-select").map_err(dom_err)?;
    select.set_attribute("aria-label", accessible_label).map_err(dom_err)?;

    let values: Vec<T> = options.iter().map(|&(value, _)| value).collect();

    for (index, &(value, label)) in options.iter().enumerate() {
        let option = xhtml(document, "option")?;
        option.set_attribute("value", &index.to_string()).map_err(dom_err)?;
        option.set_text_content(Some(label));
        if value == default {
            option.set_attribute("selected", "").map_err(dom_err)?;
        }
        select.append_child(&option).map_err(dom_err)?;
    }

    let change_select = select.clone();
    let on_change: DemoClosure = Closure::new(move |_: web_sys::Event| {
        if let Ok(index) = change_select.value().parse::<usize>() {
            if let Some(&value) = values.get(index) {
                on_select(value);
            }
        }
    });
    select
        .add_event_listener_with_callback("change", on_change.as_ref().unchecked_ref())
        .map_err(dom_err)?;
    keep_demo_closure(on_change);

    container.append_child(&select).map_err(dom_err)?;
    Ok(container)
}
