use crate::{
    DemoClosure,
    colours::CAPTION,
    dom_err,
    foreign_html::{foreign_object_document, xhtml},
    keep_demo_closure, keep_demo_node,
};
use svg_dom::{
    Error, SvgAttrs, SvgFilter, SvgRoot,
    root::utils::{Point, Size},
};
use wasm_bindgen::{JsCast, prelude::*};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// One file per demo, each exporting its own `fn demo()` — the same split `texts/mod.rs` uses, and for the same
// reason: `demo_gallery!`'s `$path:path` grammar accepts any nesting depth, so `paint::demo_filter::demo` works
// exactly like `texts::demo_text::demo`, and DEMO_SOURCE_FILES (lib.rs) keys each file by its own module path so
// two files can each define `fn demo()` without colliding on lookup.
//
// Unlike `structure`, this module does have shared helpers.
// `widen_filter_region`/`exact_filter_region` below are used by more than one demo file (`demo_turbulence`,
// `demo_morphology`, `demo_fe_image`, `demo_fe_tile`). `demo_filter` computes its own filter regions instead:
// its own slider-driven MAX_ constants reach values `widen_filter_region`'s fixed -50%/-50%/200%/200% region is
// too tight for (see its own doc comment for the margin it uses instead).
// `row_caption`/`side_label`/`select_el`/`fill_ticks`/`HSlider`/`build_h_slider`/`build_v_slider` below are used
// by the gradient demos (`demo_linear_gradient`, `demo_radial_gradient`) and by `demo_filter`'s blur circle and
// drop-shadow sliders (the latter including its own vertical dy slider), which each lay their own controls out
// in custom rows rather than following the shared W/H/PAD_Y/BAND/caption convention every other demo uses.
// `demo_filter` does not need `select_el`, though: unlike the gradient setters it calls, `gaussian_blur` and
// `drop_shadow` both return a live `SvgNode`, which it retains directly instead of reaching back into the DOM
// through a CSS selector.
// All of these live here rather than being duplicated per file. Plain private `fn`s, not `pub(crate)`: nothing
// outside `paint` needs them, and a private item defined here is already visible to every descendant module
// (i.e. every file below), so no `pub` qualifier is needed for `super::widen_filter_region(f)` (or
// `super::row_caption(...)`) to resolve.
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) mod demo_blend;
pub(crate) mod demo_clip_path;
pub(crate) mod demo_color_matrix;
pub(crate) mod demo_component_transfer;
pub(crate) mod demo_convolve_matrix;
pub(crate) mod demo_fe_image;
pub(crate) mod demo_fe_tile;
pub(crate) mod demo_filter;
pub(crate) mod demo_light_sources;
pub(crate) mod demo_lighting;
pub(crate) mod demo_linear_gradient;
pub(crate) mod demo_mask;
pub(crate) mod demo_morphology;
pub(crate) mod demo_pattern;
pub(crate) mod demo_radial_gradient;
pub(crate) mod demo_turbulence;

// Widens a filter's region from the SVG default (-10%/-10%/120%/120% of the referencing element's bounding box)
// to -50%/-50%/200%/200%, via the typed set_x/set_y/set_width/set_height setters rather than the generic
// set_attrs escape hatch. Shared by every build_filter closure that needs the same wider region to avoid visibly
// clipping their blur or offset shadow.
fn widen_filter_region(f: &SvgFilter) -> Result<(), Error> {
    f.set_x(-0.5)?;
    f.set_y(-0.5)?;
    f.set_width(2.0)?;
    f.set_height(2.0)?;
    Ok(())
}

// Narrows a filter's region from the SVG default (-10%/-10%/120%/120% of the referencing element's bounding box)
// down to exactly 0%/0%/100%/100% — i.e. the referencing element's own bounding box, with no margin. feImage's
// content fills its primitive subregion, which defaults to the *filter region* rather than the referencing
// element's box, so the default 120% padding makes an imported image render visibly larger than the same image
// placed directly via a plain <image> element. Unlike widen_filter_region's blur/offset use case, feImage has
// nothing that spills past its own source image's edges, so it needs no such margin.
fn exact_filter_region(f: &SvgFilter) -> Result<(), Error> {
    f.set_x(0.0)?;
    f.set_y(0.0)?;
    f.set_width(1.0)?;
    f.set_height(1.0)?;
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Draws one row's caption text at an explicit `y`.
/// It matches `crate::caption`'s own visual style: `CAPTION` fill, 11px, centred.
/// A demo with its own custom row layout cannot use `crate::caption` directly.
/// That helper hard-codes `y` to the shared PAD_Y/BAND band, which a taller or differently-shaped canvas does
/// not follow.
fn row_caption(svg: &SvgRoot, cx: f64, y: f64, text: &str) -> Result<(), Error> {
    let t = svg.text(Point::new(cx, y), text)?;
    let mut attrs = SvgAttrs::new();
    t.attrs(&mut attrs)
        .fill(CAPTION)?
        .apply([("font-size", "11"), ("text-anchor", "middle")])?;
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Draws a vertical slider's own endpoint value, or similar side note, as plain SVG text starting at `x`.
/// The text is left-aligned, not centred like `row_caption`'s own text.
/// These values sit beside a vertical track, where left-aligned text reads better.
fn side_label(svg: &SvgRoot, x: f64, y: f64, text: &str) -> Result<(), Error> {
    let t = svg.text(Point::new(x, y), text)?;
    let mut attrs = SvgAttrs::new();
    t.attrs(&mut attrs).fill(CAPTION)?.apply([("font-size", "11")])?;
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Selects one live element inside `svg`, by CSS selector.
/// An escape hatch for a demo whose own setter writes an attribute once, at construction, and returns no handle
/// a later interactive control could hold onto to change it again — currently only the gradient demos
/// (`add_stop`/`set_gradient_transform`/`set_fx`/`set_fy` and friends). Selecting the live DOM element directly
/// reaches it anyway. A demo whose own primitive constructor *does* return a retained handle (for example
/// `demo_filter`'s `gaussian_blur`/`drop_shadow`) should hold onto that instead of calling this.
fn select_el(svg: &SvgRoot, selector: &str) -> Result<web_sys::Element, Error> {
    svg.root
        .query_selector(selector)
        .map_err(dom_err)?
        .ok_or_else(|| Error::Dom(format!("no element matching {selector:?}")))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Fills `ticks_row` with tick marks proportioned across `[min, max]`, every `tick_step` units from `min`, plus
/// one final tick at `max`.
/// That final tick may sit closer than `tick_step` to its neighbour.
/// `texts::demo_text_path`'s own hand-drawn ticks use this same trailing-tick shape.
///
/// Clears any tick marks `ticks_row` already holds first, so this doubles as a refresh: a demo whose sliders
/// constrain each other's live range can call this again whenever that range changes, keeping the tick marks
/// proportioned to the *current* range, not the range they were first built with.
fn fill_ticks(ticks_row: &web_sys::Element, min: i32, max: i32, tick_step: i32) -> Result<(), Error> {
    let document = ticks_row
        .owner_document()
        .ok_or_else(|| Error::Dom("tick row has no owner document".into()))?;
    ticks_row.set_inner_html("");

    let span = f64::from(max - min);
    let mut tick = min;
    while tick < max {
        let percent = f64::from(tick - min) / span * 100.0;
        let mark = xhtml(&document, "span")?;
        mark.set_attribute("class", "demo-tick-mark").map_err(dom_err)?;
        mark.set_attribute("style", &format!("left:{percent:.2}%;")).map_err(dom_err)?;
        ticks_row.append_child(&mark).map_err(dom_err)?;
        tick += tick_step;
    }
    let final_mark = xhtml(&document, "span")?;
    final_mark.set_attribute("class", "demo-tick-mark").map_err(dom_err)?;
    final_mark.set_attribute("style", "left:100%;").map_err(dom_err)?;
    ticks_row.append_child(&final_mark).map_err(dom_err)?;
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// The pieces of a slider built by [`build_h_slider`] that a caller with dynamically-constrained sliders (for
/// example `demo_linear_gradient`'s two spectrum stops) needs to keep updated after construction: their own live
/// endpoint text and tick marks, alongside the `<input>` itself.
/// Callers that never need to update those after construction simply take `.input` and ignore the rest.
#[derive(Clone)]
pub(crate) struct HSlider {
    pub(crate) input: web_sys::HtmlInputElement,
    pub(crate) lo_label: web_sys::Element,
    pub(crate) hi_label: web_sys::Element,
    pub(crate) ticks_row: web_sys::Element,
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Builds one labelled horizontal `<input type="range">`.
/// It includes tick marks and endpoint value text.
/// The control sits inside its own `<foreignObject>` at `pos`, sized `size`.
///
/// `range` is `(min, max, default)`.
/// `tick_step` is passed straight through to [`fill_ticks`].
/// `endpoints` is the `(min, max)` text shown on either side of the track, for example `"10%"`/`"100%"` or
/// `"-90°"`/`"+90°"`.
/// This text is free-form, since not every slider's endpoints share its raw `i32` value's unit.
/// `labels` is `(visible, accessible)`.
/// `visible` is the short caption drawn above the track, read alongside the row's own caption below the shape.
/// `accessible` is the control's `aria-label`, read alone by assistive technology, with no such visual context.
/// A screen reader user needs `accessible` to name the shape or gradient too, for example
/// "horizontal gradient stop 2", not just "stop 2".
fn build_h_slider(
    svg: &SvgRoot,
    pos: Point,
    size: Size,
    labels: (&str, &str),
    range: (i32, i32, i32),
    tick_step: i32,
    endpoints: (&str, &str),
) -> Result<HSlider, Error> {
    let (visible_label, accessible_label) = labels;
    let (min, max, default) = range;
    let (min_label, max_label) = endpoints;

    let fo = svg.foreign_object(pos, size)?;
    let document = foreign_object_document(&fo)?;

    let container = xhtml(&document, "div")?;
    container.set_attribute("class", "demo-slider-container").map_err(dom_err)?;

    let label_el = xhtml(&document, "div")?;
    label_el.set_attribute("class", "demo-slider-label").map_err(dom_err)?;
    label_el.set_text_content(Some(visible_label));
    container.append_child(&label_el).map_err(dom_err)?;

    let slider = xhtml(&document, "input")?
        .dyn_into::<web_sys::HtmlInputElement>()
        .map_err(|_| Error::Dom("createElement(\"input\") did not return an HtmlInputElement".into()))?;
    slider.set_type("range");
    slider.set_min(&min.to_string());
    slider.set_max(&max.to_string());
    slider.set_step("1");
    slider.set_value(&default.to_string());
    slider.set_attribute("class", "demo-slider").map_err(dom_err)?;
    slider.set_attribute("aria-label", accessible_label).map_err(dom_err)?;
    container.append_child(&slider).map_err(dom_err)?;

    let ticks_row = xhtml(&document, "div")?;
    ticks_row.set_attribute("class", "demo-tick-row").map_err(dom_err)?;
    fill_ticks(&ticks_row, min, max, tick_step)?;
    container.append_child(&ticks_row).map_err(dom_err)?;

    let endpoint_labels = xhtml(&document, "div")?;
    endpoint_labels
        .set_attribute("class", "demo-endpoint-labels")
        .map_err(dom_err)?;
    let lo_label = xhtml(&document, "span")?;
    lo_label.set_text_content(Some(min_label));
    let hi_label = xhtml(&document, "span")?;
    hi_label.set_text_content(Some(max_label));
    endpoint_labels.append_child(&lo_label).map_err(dom_err)?;
    endpoint_labels.append_child(&hi_label).map_err(dom_err)?;
    container.append_child(&endpoint_labels).map_err(dom_err)?;

    fo.as_element().append_child(&container).map_err(dom_err)?;
    keep_demo_node(fo);
    Ok(HSlider {
        input: slider,
        lo_label,
        hi_label,
        ticks_row,
    })
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Builds one vertical-shift `<input type="range">`: a plain horizontal range input, rotated 90° with a CSS
/// `transform` (see `.demo-slider-vertical` in demo/style.css), not `writing-mode: vertical-lr`. Chrome only
/// gained `writing-mode` support for range inputs recently and Safari's support is still unreliable, whereas a
/// CSS `transform` rotation works in every browser, and this control must render vertically everywhere, not just
/// the browser it was last tested in.
///
/// `aria-orientation="vertical"` is set here too: a rotated `<input>` stays a horizontal slider as far as the
/// DOM and assistive technology know, since nothing about the CSS transform that turns it visually vertical
/// reaches ARIA on its own.
///
/// Its keyboard handling is remapped here as well. A plain rotated `<input>` still treats ArrowUp as "increment"
/// natively, which — once top is min, as every caller of this function arranges — moves the thumb visually
/// *down*, the opposite of what a sighted user pressing the up arrow expects. ArrowUp here decrements instead,
/// ArrowDown increments; ArrowLeft/ArrowRight keep their ordinary native behaviour unchanged.
///
/// The control sits inside its own `<foreignObject>` at `pos`, sized `(track_w, track_length)`. Its own
/// pre-rotation width is set inline to `track_length`, rather than through a fixed CSS value, so this one
/// function serves any caller's own track length, not just one hard-coded pixel size.
///
/// Unlike [`build_h_slider`], this returns just the `<input>`: none of this file's callers draws HTML tick marks
/// or endpoint labels for a vertical track. Each draws plain SVG text/line elements alongside it instead, which
/// need no CSS positioning of their own once the control itself is already rotated.
fn build_v_slider(
    svg: &SvgRoot,
    pos: Point,
    track_w: f64,
    track_length: f64,
    accessible_label: &str,
    range: (i32, i32, i32),
) -> Result<web_sys::HtmlInputElement, Error> {
    let (min, max, default) = range;

    let fo = svg.foreign_object(pos, Size::new(track_w, track_length))?;
    let document = foreign_object_document(&fo)?;
    let wrap = xhtml(&document, "div")?;
    wrap.set_attribute("class", "demo-slider-vertical-wrap").map_err(dom_err)?;

    let slider = xhtml(&document, "input")?
        .dyn_into::<web_sys::HtmlInputElement>()
        .map_err(|_| Error::Dom("createElement(\"input\") did not return an HtmlInputElement".into()))?;
    slider.set_type("range");
    slider.set_min(&min.to_string());
    slider.set_max(&max.to_string());
    slider.set_step("1");
    slider.set_value(&default.to_string());
    slider.set_attribute("class", "demo-slider-vertical").map_err(dom_err)?;
    slider.set_attribute("aria-label", accessible_label).map_err(dom_err)?;
    slider.set_attribute("aria-orientation", "vertical").map_err(dom_err)?;
    slider
        .set_attribute("style", &format!("width:{track_length}px"))
        .map_err(dom_err)?;

    wrap.append_child(&slider).map_err(dom_err)?;
    fo.as_element().append_child(&wrap).map_err(dom_err)?;
    keep_demo_node(fo);

    let key_slider = slider.clone();
    let on_keydown: DemoClosure = Closure::new(move |e: web_sys::Event| {
        let e: web_sys::KeyboardEvent = e.unchecked_into();
        let delta = match e.key().as_str() {
            "ArrowUp" => -1.0,
            "ArrowDown" => 1.0,
            _ => return,
        };
        e.prevent_default();
        let new_value = key_slider.value_as_number() + delta;
        key_slider.set_value(&new_value.to_string());
        let input_event = web_sys::Event::new("input").expect("create input event");
        let _ = key_slider.dispatch_event(&input_event);
    });
    slider
        .add_event_listener_with_callback("keydown", on_keydown.as_ref().unchecked_ref())
        .map_err(dom_err)?;
    keep_demo_closure(on_keydown);

    Ok(slider)
}
