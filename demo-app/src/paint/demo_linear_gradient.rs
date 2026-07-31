use crate::{
    DemoClosure, W,
    colours::*,
    dom_err,
    foreign_html::{foreign_object_document, xhtml},
    keep_demo_closure, keep_demo_node,
};
use svg_dom::{
    Error, SvgAttrs, SvgRoot,
    root::utils::{Point, Size},
};
use wasm_bindgen::{JsCast, prelude::*};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// linearGradient — horizontal, vertical, diagonal, multi-stop, and gradient stroke, laid out three rows deep
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Every row is left-justified from LEFT_MARGIN, not centred.
// Each row also sits at its own fixed y-offset.
// This canvas is taller than the shared H constant.
// `paint::demo_fe_image`'s canvas is wider than the shared W constant, for the same reason.
// This demo therefore needs its own row geometry.
// It does not use the crate-wide PAD_Y, BAND, or caption helpers.
//
// Every interactive control reads a slider's own value.
// It writes that value straight to a `<stop>`'s `offset` attribute, or to the gradient's own `gradientTransform`.
// `select_el` below finds the target element by CSS selector.
// This bypasses the typed `SvgLinearGradient` handle, which offers no way to reach these targets directly.
// `add_stop` appends a `<stop>` but does not return a handle to it.
// `SvgLinearGradient` itself is not `Clone`, so one handle cannot move into two closures.
// Selecting the live DOM element avoids both limits.
//
// The vertical-shift slider is a plain horizontal `<input type="range">`.
// A `transform: rotate(...)` rule rotates it (see `.demo-slider-vertical` in demo/style.css).
// It does not use `writing-mode: vertical-lr`.
// Chrome only gained `writing-mode` support for range inputs recently.
// Safari's support for it is still unreliable.
// A CSS `transform` rotation works in every browser.
// This control must render vertically everywhere, not only in the browser it was last tested in.
// Its label, tick marks, and endpoint values are plain SVG text and line elements, not HTML.
// A rotated HTML tick row or endpoint row would need extra CSS positioning.
// Plain SVG elements avoid that need.
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

const RECT_W: f64 = 130.0;
const RECT_H: f64 = 90.0;

const LEFT_MARGIN: f64 = 40.0; // every row starts here, rather than being centred in the canvas
const ITEM_GAP: f64 = 70.0; // horizontal gap between the two items sharing a row

const SLIDER_ROW_H: f64 = 62.0; // one HTML slider block: label + track + tick row + endpoint-value row
const SLIDER_GAP: f64 = 6.0; // gap between two stacked slider blocks, and between a slider zone and its rect
const CAPTION_OFFSET: f64 = 14.0; // caption sits this far below its rect's bottom edge
const ROW_GAP: f64 = 26.0; // gap from one row's caption down to the next row's top

const ROW1_TOP: f64 = 10.0;
const ROW1_RECT_Y: f64 = ROW1_TOP + SLIDER_ROW_H + SLIDER_GAP;
const ROW1_CAPTION_Y: f64 = ROW1_RECT_Y + RECT_H + CAPTION_OFFSET;

const ROW1_H_X: f64 = LEFT_MARGIN;
const ROW1_V_X: f64 = ROW1_H_X + RECT_W + ITEM_GAP;
const V_TRACK_GAP: f64 = 16.0; // gap between the vertical rect and its slider track
const V_TRACK_W: f64 = 16.0; // the rotated slider's own (post-rotation) thickness

const ROW2_TOP: f64 = ROW1_CAPTION_Y + ROW_GAP;
const ROW2_SLIDER_ZONE: f64 = SLIDER_ROW_H * 2.0 + SLIDER_GAP; // tall enough for the 4-stop demo's two stacked sliders
const ROW2_RECT_Y: f64 = ROW2_TOP + ROW2_SLIDER_ZONE + SLIDER_GAP;
const ROW2_CAPTION_Y: f64 = ROW2_RECT_Y + RECT_H + CAPTION_OFFSET;
// The diagonal demo has only one slider.
// It sits at the bottom of the same zone the 4-stop spectrum's two sliders reserve.
// Both rects in row 2 therefore align at ROW2_RECT_Y, however many sliders each item has above it.
const ROW2_SINGLE_SLIDER_Y: f64 = ROW2_TOP + ROW2_SLIDER_ZONE - SLIDER_ROW_H;

const ROW2_D_X: f64 = LEFT_MARGIN;
const ROW2_S_X: f64 = ROW2_D_X + RECT_W + ITEM_GAP;

const ROW3_TOP: f64 = ROW2_CAPTION_Y + ROW_GAP;
const ROW3_CAPTION_Y: f64 = ROW3_TOP + RECT_H + CAPTION_OFFSET;
const ROW3_X: f64 = LEFT_MARGIN;

const LG_H: f64 = ROW3_CAPTION_Y + 12.0;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Draws one row's caption text at an explicit `y`.
/// It matches `crate::caption`'s own visual style: `CAPTION` fill, 11px, centred.
/// This demo cannot use `crate::caption` directly.
/// That helper hard-codes `y` to the shared PAD_Y/BAND band.
/// This taller canvas does not follow that band.
fn row_caption(svg: &SvgRoot, cx: f64, y: f64, text: &str) -> Result<(), Error> {
    let t = svg.text(Point::new(cx, y), text)?;
    let mut attrs = SvgAttrs::new();
    t.attrs(&mut attrs)
        .fill(CAPTION)?
        .apply([("font-size", "11"), ("text-anchor", "middle")])?;
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Draws the vertical-shift slider's endpoint values as plain SVG text, starting at `x`.
/// The text is left-aligned, not centred like `row_caption`'s own text.
/// These values sit beside the vertical track, where left-aligned text reads better.
fn side_label(svg: &SvgRoot, x: f64, y: f64, text: &str) -> Result<(), Error> {
    let t = svg.text(Point::new(x, y), text)?;
    let mut attrs = SvgAttrs::new();
    t.attrs(&mut attrs).fill(CAPTION)?.apply([("font-size", "11")])?;
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Selects one live element inside this demo's own `<svg>`, by CSS selector.
/// This file's own doc comment above explains why this escape hatch is needed.
fn select_el(svg: &SvgRoot, selector: &str) -> Result<web_sys::Element, Error> {
    svg.root
        .query_selector(selector)
        .map_err(dom_err)?
        .ok_or_else(|| Error::Dom(format!("linearGradient demo: no element matching {selector:?}")))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Builds one labelled horizontal `<input type="range">`.
/// It includes tick marks and endpoint value text.
/// The control sits inside its own `<foreignObject>` at `pos`, sized `(w, SLIDER_ROW_H)`.
///
/// `range` is `(min, max, default)`.
/// `tick_step` places a tick mark every `tick_step` units from `min`.
/// It also places one final tick at `max`.
/// That final tick may sit closer than `tick_step` to its neighbour.
/// `texts::demo_text_path`'s own hand-drawn ticks use this same trailing-tick shape.
/// `endpoints` is the `(min, max)` text shown on either side of the track, for example `"10%"`/`"100%"` or
/// `"-90°"`/`"+90°"`.
/// This text is free-form, since not every slider's endpoints share its raw `i32` value's unit.
/// `label` serves two roles: it is both the visible caption above the track and the control's `aria-label`.
///
/// Four of this demo's five interactive controls are a plain horizontal slider like this one.
/// Only the vertical-shift slider needs a different shape.
/// Its own call site builds it by hand instead.
fn build_h_slider(
    svg: &SvgRoot,
    pos: Point,
    w: f64,
    label: &str,
    range: (i32, i32, i32),
    tick_step: i32,
    endpoints: (&str, &str),
) -> Result<web_sys::HtmlInputElement, Error> {
    let (min, max, default) = range;
    let (min_label, max_label) = endpoints;

    let fo = svg.foreign_object(pos, Size::new(w, SLIDER_ROW_H))?;
    let document = foreign_object_document(&fo)?;

    let container = xhtml(&document, "div")?;
    container.set_attribute("class", "demo-slider-container").map_err(dom_err)?;

    let label_el = xhtml(&document, "div")?;
    label_el.set_attribute("class", "demo-slider-label").map_err(dom_err)?;
    label_el.set_text_content(Some(label));
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
    slider.set_attribute("aria-label", label).map_err(dom_err)?;
    container.append_child(&slider).map_err(dom_err)?;

    let ticks_row = xhtml(&document, "div")?;
    ticks_row.set_attribute("class", "demo-tick-row").map_err(dom_err)?;
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
    container.append_child(&ticks_row).map_err(dom_err)?;

    let endpoint_labels = xhtml(&document, "div")?;
    endpoint_labels
        .set_attribute("class", "demo-endpoint-labels")
        .map_err(dom_err)?;
    let lo = xhtml(&document, "span")?;
    lo.set_text_content(Some(min_label));
    let hi = xhtml(&document, "span")?;
    hi.set_text_content(Some(max_label));
    endpoint_labels.append_child(&lo).map_err(dom_err)?;
    endpoint_labels.append_child(&hi).map_err(dom_err)?;
    container.append_child(&endpoint_labels).map_err(dom_err)?;

    fo.as_element().append_child(&container).map_err(dom_err)?;
    keep_demo_node(fo);
    Ok(slider)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-linear-gradient", Size::new(W, LG_H))?;

    // All gradient ids must be globally unique in the document, so use a per-demo prefix.
    let defs = svg.build_defs(|d| {
        // 1. Horizontal (default x1/y1/x2/y2): steelblue left → coral right.
        d.build_linear_gradient("demo-lg-h", |g| {
            g.add_stop(0.0, STEELBLUE)?;
            g.add_stop(1.0, CORAL)?;
            Ok(())
        })?;

        // 2. Vertical gradient: set x2=0, y2=1 to rotate the axis 90°.
        d.build_linear_gradient("demo-lg-v", |g| {
            g.add_stop(0.0, GOLDENROD)?;
            g.add_stop(1.0, "midnightblue")?;
            g.set_x2(0.0)?;
            g.set_y2(1.0)?;
            Ok(())
        })?;

        // 3. Diagonal: gradientTransform rotates the gradient vector 45°.
        //    Keeping the default horizontal endpoints and rotating is simpler than computing
        //    trigonometric endpoint coordinates by hand.
        d.build_linear_gradient("demo-lg-d", |g| {
            g.add_stop(0.0, TEAL)?;
            g.add_stop(1.0, MEDIUM_ORCHID)?;
            g.set_gradient_transform("rotate(45, 0.5, 0.5)")?;
            Ok(())
        })?;

        // 4. Multi-stop sunrise spectrum (4 stops).
        d.build_linear_gradient("demo-lg-s", |g| {
            g.add_stop(0.0, "#1a1a2e")?;
            g.add_stop(0.35, DARK_ORANGE)?;
            g.add_stop(0.65, GOLDENROD)?;
            g.add_stop(1.0, "#fffde7")?;
            Ok(())
        })?;

        // 5. Gradient stroke: a thin-to-thick colour sweep applied to stroke, not fill.
        d.build_linear_gradient("demo-lg-stroke", |g| {
            g.add_stop(0.0, MEDIUM_SEA_GREEN)?;
            g.add_stop(1.0, CORAL)?;
            Ok(())
        })?;

        Ok(())
    })?;

    // `defs` is used only to hold the gradients here.
    // Binding it to `_` below tells the compiler this is intentional.
    let _ = defs;

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // Row 1 — horizontal (slider above shifts the second stop left/right) and vertical (slider beside shifts it
    // up/down)
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    let r1 = svg.rect(Point::new(ROW1_H_X, ROW1_RECT_Y), Size::new(RECT_W, RECT_H))?;
    r1.set_fill_gradient("demo-lg-h")?;
    row_caption(&svg, ROW1_H_X + RECT_W / 2.0, ROW1_CAPTION_Y, "horizontal")?;

    let h_stop = select_el(&svg, "#demo-lg-h stop:nth-child(2)")?;
    let h_slider = build_h_slider(
        &svg,
        Point::new(ROW1_H_X, ROW1_TOP),
        RECT_W,
        "shift stop 2",
        (10, 100, 100),
        30,
        ("10%", "100%"),
    )?;
    // Matches the slider's own default value (100) above, set once here so a screen reader announces the real
    // starting value, not "no value", before any interaction happens.
    h_slider.set_attribute("aria-valuetext", "100%").map_err(dom_err)?;
    {
        let slider = h_slider.clone();
        let stop = h_stop.clone();
        let on_input: DemoClosure = Closure::new(move |_: web_sys::Event| {
            let percent = slider.value_as_number();
            let _ = stop.set_attribute("offset", &format!("{:.2}", percent / 100.0));
            let _ = slider.set_attribute("aria-valuetext", &format!("{percent:.0}%"));
        });
        h_slider
            .add_event_listener_with_callback("input", on_input.as_ref().unchecked_ref())
            .map_err(dom_err)?;
        keep_demo_closure(on_input);
    }

    let r2 = svg.rect(Point::new(ROW1_V_X, ROW1_RECT_Y), Size::new(RECT_W, RECT_H))?;
    r2.set_fill_gradient("demo-lg-v")?;
    row_caption(&svg, ROW1_V_X + RECT_W / 2.0, ROW1_CAPTION_Y, "vertical")?;

    // Vertical-shift control: a plain <input type="range">, rotated with `transform`.
    // See this file's own doc comment above for why not `writing-mode`.
    // Plain SVG text and line elements decorate it, not HTML.
    // Once the control is rotated, these SVG elements need no CSS positioning of their own.
    let v_stop = select_el(&svg, "#demo-lg-v stop:nth-child(2)")?;
    let v_track_x = ROW1_V_X + RECT_W + V_TRACK_GAP;

    row_caption(&svg, v_track_x + V_TRACK_W / 2.0, ROW1_TOP + 10.0, "shift stop 2")?;

    // Endpoint values: min (10%) sits at the top of the track, max (100%) at the bottom.
    // This matches the track's own top-is-min orientation, established below.
    // Up therefore moves the stop up, the same spatial logic the horizontal slider's left-is-smaller framing uses.
    side_label(&svg, v_track_x + V_TRACK_W + 6.0, ROW1_RECT_Y + 4.0, "10%")?;
    side_label(&svg, v_track_x + V_TRACK_W + 6.0, ROW1_RECT_Y + RECT_H - 2.0, "100%")?;

    // Three tick marks mark the min, middle, and max values.
    // Each is a short line crossing the track.
    // They sit proportionally along [min, max], the same way build_h_slider's own tick marks do.
    // They are not evenly spaced in pixels.
    for tick_percent in [10.0, 55.0, 100.0_f64] {
        let fraction = (tick_percent - 10.0) / (100.0 - 10.0);
        let y = ROW1_RECT_Y + fraction * RECT_H;
        let mark = svg.line(Point::new(v_track_x - 3.0, y), Point::new(v_track_x + V_TRACK_W + 3.0, y))?;
        mark.set_stroke(GUIDE)?;
        mark.set_stroke_width(1.0)?;
    }

    let v_slider_fo = svg.foreign_object(Point::new(v_track_x, ROW1_RECT_Y), Size::new(V_TRACK_W, RECT_H))?;
    let v_document = foreign_object_document(&v_slider_fo)?;
    let v_wrap = xhtml(&v_document, "div")?;
    v_wrap.set_attribute("class", "demo-slider-vertical-wrap").map_err(dom_err)?;
    let v_slider = xhtml(&v_document, "input")?
        .dyn_into::<web_sys::HtmlInputElement>()
        .map_err(|_| Error::Dom("createElement(\"input\") did not return an HtmlInputElement".into()))?;
    v_slider.set_type("range");
    v_slider.set_min("10");
    v_slider.set_max("100");
    v_slider.set_step("1");
    v_slider.set_value("100");
    v_slider.set_attribute("class", "demo-slider-vertical").map_err(dom_err)?;
    v_slider
        .set_attribute("aria-label", "shift the vertical gradient's second stop")
        .map_err(dom_err)?;
    // Matches the slider's own default value (100) above, set once here for the same reason the horizontal
    // slider's own aria-valuetext is set at construction, not only after the first input event.
    v_slider.set_attribute("aria-valuetext", "100%").map_err(dom_err)?;
    v_wrap.append_child(&v_slider).map_err(dom_err)?;
    v_slider_fo.as_element().append_child(&v_wrap).map_err(dom_err)?;
    keep_demo_node(v_slider_fo);
    {
        let slider = v_slider.clone();
        let stop = v_stop.clone();
        let on_input: DemoClosure = Closure::new(move |_: web_sys::Event| {
            let percent = slider.value_as_number();
            let _ = stop.set_attribute("offset", &format!("{:.2}", percent / 100.0));
            let _ = slider.set_attribute("aria-valuetext", &format!("{percent:.0}%"));
        });
        v_slider
            .add_event_listener_with_callback("input", on_input.as_ref().unchecked_ref())
            .map_err(dom_err)?;
        keep_demo_closure(on_input);
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // Row 2 — diagonal (slider rotates ±90° about the initial 45°) and 4-stop spectrum (two sliders keep the middle
    // stops ordered)
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    let r3 = svg.rect(Point::new(ROW2_D_X, ROW2_RECT_Y), Size::new(RECT_W, RECT_H))?;
    r3.set_fill_gradient("demo-lg-d")?;
    let rotate_label = svg.text(Point::new(ROW2_D_X + RECT_W / 2.0, ROW2_CAPTION_Y), "rotate 45°")?;
    let mut rotate_label_attrs = SvgAttrs::new();
    rotate_label
        .attrs(&mut rotate_label_attrs)
        .fill(CAPTION)?
        .apply([("font-size", "11"), ("text-anchor", "middle")])?;

    let d_gradient = select_el(&svg, "#demo-lg-d")?;
    const ROTATE_BASE: f64 = 45.0;
    let rotate_slider = build_h_slider(
        &svg,
        Point::new(ROW2_D_X, ROW2_SINGLE_SLIDER_Y),
        RECT_W,
        "rotate",
        (-90, 90, 0),
        45,
        ("-90°", "+90°"),
    )?;
    // Matches the slider's own default value (0, i.e. ROTATE_BASE + 0) above, set once here so the accessible
    // value starts as the absolute angle "rotate 45°", the same text `rotate_label` shows, not the raw slider
    // value "0". Without this, a screen reader would announce "0" until the first interaction, then jump to an
    // unrelated absolute angle on the very first move.
    rotate_slider
        .set_attribute("aria-valuetext", &format!("rotate {ROTATE_BASE:.0}°"))
        .map_err(dom_err)?;
    {
        let slider = rotate_slider.clone();
        let gradient = d_gradient.clone();
        let label = rotate_label.clone();
        let mut buf = String::new();
        let on_input: DemoClosure = Closure::new(move |_: web_sys::Event| {
            let total = ROTATE_BASE + slider.value_as_number();
            let _ = gradient.set_attribute("gradientTransform", &format!("rotate({total}, 0.5, 0.5)"));
            let _ = label.set_text_fmt(&mut buf, format_args!("rotate {total:.0}°"));
            let _ = slider.set_attribute("aria-valuetext", &buf);
        });
        rotate_slider
            .add_event_listener_with_callback("input", on_input.as_ref().unchecked_ref())
            .map_err(dom_err)?;
        keep_demo_closure(on_input);
    }

    let r4 = svg.rect(Point::new(ROW2_S_X, ROW2_RECT_Y), Size::new(RECT_W, RECT_H))?;
    r4.set_fill_gradient("demo-lg-s")?;
    row_caption(&svg, ROW2_S_X + RECT_W / 2.0, ROW2_CAPTION_Y, "4-stop spectrum")?;

    // The middle two stops start at 0.35 (index 1) and 0.65 (index 2).
    // These two stops stay ordered by construction, both against each other and against the fixed outer stops
    // (0.0 and 1.0).
    //
    // stop2/stop3 keep a live one-point gap between each other. `s2_slider`'s own `max` attribute always tracks
    // `s3_slider.value - 1`. `s3_slider`'s own `min` attribute always tracks `s2_slider.value + 1`. The browser
    // itself then refuses to move either thumb past the other, keyboard included, before this demo's own
    // `on_input` handler ever runs. Neither handler clamps or writes back a value any more: each slider's native
    // sanitisation already guarantees the value it reports is in range.
    //
    // `(1, 98)` and `(2, 99)` below, and the "1%"/"98%"/"2%"/"99%" endpoint labels and tick marks
    // `build_h_slider` draws from them, describe each slider's absolute possible range against the fixed outer
    // stops, not its live range against the other slider. Those stay fixed on purpose, as a description of the
    // total range each stop could ever reach, distinct from the live `max`/`min` this section maintains.
    let s2_stop = select_el(&svg, "#demo-lg-s stop:nth-child(2)")?;
    let s3_stop = select_el(&svg, "#demo-lg-s stop:nth-child(3)")?;
    let s2_slider = build_h_slider(
        &svg,
        Point::new(ROW2_S_X, ROW2_TOP),
        RECT_W,
        "stop 2",
        (1, 98, 35),
        25,
        ("1%", "98%"),
    )?;
    // Matches this slider's own default value (35) above, set once here rather than only after the first input
    // event, the same reason every other slider in this file sets its own initial aria-valuetext.
    s2_slider.set_attribute("aria-valuetext", "35%").map_err(dom_err)?;
    let s3_slider = build_h_slider(
        &svg,
        Point::new(ROW2_S_X, ROW2_TOP + SLIDER_ROW_H + SLIDER_GAP),
        RECT_W,
        "stop 3",
        (2, 99, 65),
        25,
        ("2%", "99%"),
    )?;
    // Matches this slider's own default value (65) above, for the same reason.
    s3_slider.set_attribute("aria-valuetext", "65%").map_err(dom_err)?;

    // Establishes the live one-point gap from each slider's own default value, before either has fired an
    // `on_input` event of its own.
    s2_slider.set_max(&(s3_slider.value_as_number() - 1.0).to_string());
    s3_slider.set_min(&(s2_slider.value_as_number() + 1.0).to_string());
    {
        let this_slider = s2_slider.clone();
        let other_slider = s3_slider.clone();
        let stop = s2_stop.clone();
        let on_input: DemoClosure = Closure::new(move |_: web_sys::Event| {
            let value = this_slider.value_as_number();
            let _ = stop.set_attribute("offset", &format!("{:.3}", value / 100.0));
            let _ = this_slider.set_attribute("aria-valuetext", &format!("{value:.0}%"));
            other_slider.set_min(&(value + 1.0).to_string());
        });
        s2_slider
            .add_event_listener_with_callback("input", on_input.as_ref().unchecked_ref())
            .map_err(dom_err)?;
        keep_demo_closure(on_input);
    }
    {
        let this_slider = s3_slider.clone();
        let other_slider = s2_slider.clone();
        let stop = s3_stop.clone();
        let on_input: DemoClosure = Closure::new(move |_: web_sys::Event| {
            let value = this_slider.value_as_number();
            let _ = stop.set_attribute("offset", &format!("{:.3}", value / 100.0));
            let _ = this_slider.set_attribute("aria-valuetext", &format!("{value:.0}%"));
            other_slider.set_max(&(value - 1.0).to_string());
        });
        s3_slider
            .add_event_listener_with_callback("input", on_input.as_ref().unchecked_ref())
            .map_err(dom_err)?;
        keep_demo_closure(on_input);
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // Row 3 — gradient stroke, unchanged from before, just relocated into its own row
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    let stroke_mid_y = ROW3_TOP + RECT_H / 2.0;
    let path_str = format!(
        "M {:.1} {:.1} C {:.1} {:.1} {:.1} {:.1} {:.1} {:.1}",
        ROW3_X,
        stroke_mid_y + 35.0,
        ROW3_X + 40.0,
        stroke_mid_y - 45.0,
        ROW3_X + 90.0,
        stroke_mid_y + 45.0,
        ROW3_X + RECT_W,
        stroke_mid_y - 35.0,
    );
    let stroke_path = svg.path(&path_str)?;
    stroke_path.set_fill("none")?;
    stroke_path.set_stroke_gradient("demo-lg-stroke")?;
    stroke_path.set_stroke_width(14.0)?;
    stroke_path.set_attr("stroke-linecap", "round")?;
    row_caption(&svg, ROW3_X + RECT_W / 2.0, ROW3_CAPTION_Y, "gradient stroke")?;

    Ok(())
}
