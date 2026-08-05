use crate::{DemoClosure, W, colours::*, dom_err, keep_demo_closure};
use svg_dom::{
    Error, SvgRoot, TextAnchor,
    root::{
        filter::{Channel, TransferFunction},
        utils::{Point, Size},
    },
};
use wasm_bindgen::{JsCast, prelude::*};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// feComponentTransfer
// One gradient source feeds three filters. Each filter applies a different TransferFunction. Each one has its
// own live slider.
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// component_transfer returns only the feComponentTransfer container's own SvgNode. It does not return a handle
// to any feFuncX child. gaussian_blur, drop_shadow, and blend each return their own primitive node directly. So
// none of the three sliders below can retain a primitive node the way demo_filter's or demo_blend's own sliders
// do. select_rgb_funcs and select_el reach each feFuncX child by CSS selector instead. select_el lives in
// paint/mod.rs. The gradient demos already share it, for that same reason: no retained handle.
//
// build_h_slider's own range is an integer (min, max, default). Its own step is fixed at 1. Gamma's own
// exponent slider and alpha's own slope slider each drive a fractional value. Each one instead drives an
// integer slider. Its own on_input handler then computes the real value from that integer. demo_radial_gradient's
// own percent sliders already use this same technique, for offset, fx, and fy. Each of the two sliders also
// sets its own aria-valuetext to the real value. A screen reader then announces that real value. It does not
// announce the raw unscaled slider position. The discrete slider needs no such scaling. Its raw integer value
// already is the real step count. discrete_table builds tableValues from that same step count.
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

const RECT_W: f64 = 160.0;
const RECT_H: f64 = 100.0;
const LEFT_MARGIN: f64 = 20.0;
const ITEM_GAP: f64 = 30.0; // gap between rectangles

const RECT2_X: f64 = LEFT_MARGIN + RECT_W + ITEM_GAP;
const RECT3_X: f64 = RECT2_X + RECT_W + ITEM_GAP;
const RECT4_X: f64 = RECT3_X + RECT_W + ITEM_GAP;

const SLIDER_ROW_H: f64 = 62.0; // one HTML slider block: label + track + tick row + endpoint-value row
const CONTROL_GAP: f64 = 6.0; // gap between a slider and the rectangle it drives
const CAPTION_OFFSET: f64 = 14.0; // caption sits this far below its rectangle's own bottom edge

const TOP: f64 = 10.0;

// All four rectangles share the same row. Only three of them have a slider above them. demo_blend's own
// original and blended pair already uses this same layout: one shared row, one unused control zone. Every
// caption below still lines up.
const RECT_Y: f64 = TOP + SLIDER_ROW_H + CONTROL_GAP;
const CAPTION_Y: f64 = RECT_Y + RECT_H + CAPTION_OFFSET;

const COMPONENT_TRANSFER_H: f64 = CAPTION_Y + 12.0;

const MIN_GAMMA_TENTHS: i32 = 2; // 0.2
const MAX_GAMMA_TENTHS: i32 = 50; // 5.0
const DEFAULT_GAMMA_TENTHS: i32 = 22; // 2.2, a recognisable display-gamma value
const DEFAULT_GAMMA_EXPONENT: f64 = DEFAULT_GAMMA_TENTHS as f64 / 10.0;

const MIN_STEPS: i32 = 2;
const MAX_STEPS: i32 = 8;
const DEFAULT_STEPS: i32 = 4;

const MIN_SLOPE_PERCENT: i32 = 0;
const MAX_SLOPE_PERCENT: i32 = 100;
const DEFAULT_SLOPE_PERCENT: i32 = 40;
const DEFAULT_SLOPE: f64 = DEFAULT_SLOPE_PERCENT as f64 / 100.0;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Builds a discrete table with `steps` evenly spaced entries, from `0.0` to `1.0`.
/// Each value is rounded to 3 decimal places.
/// `steps` must be at least `2`.
/// This file's own slider never goes below `MIN_STEPS`, so the `steps - 1` divisor below never reaches zero.
///
/// Rounding matters here, not only when formatting the live `tableValues` string below.
/// `TransferFunction::Discrete`'s own construction-time path writes each `f64` via a plain `Display` write.
/// That path lives in `SpaceSeparated`, inside `primitives/mod.rs`.
/// It writes at full precision, with no rounding of its own.
/// An unrounded fraction like `1.0 / 3.0` would construct as the long `0.3333333333333333`.
/// A slider move would then produce the short `0.333` instead, once `discrete_table_values_attr` below sets the
/// attribute again.
/// Rounding the table's own numbers once, here, avoids that mismatch.
/// This keeps the initial attribute text and every later update identical, for the same step count.
fn discrete_table(steps: i32) -> Vec<f64> {
    (0..steps)
        .map(|i| (f64::from(i) / f64::from(steps - 1) * 1000.0).round() / 1000.0)
        .collect()
}

/// Formats `discrete_table`'s own values as the space-separated string `tableValues` needs.
/// This uses the same plain `Display` formatting `TransferFunction::Discrete`'s own construction-time path uses.
/// See `discrete_table`'s own doc comment above for why that matters.
fn discrete_table_values_attr(steps: i32) -> String {
    discrete_table(steps).iter().map(f64::to_string).collect::<Vec<_>>().join(" ")
}

/// Selects `<feFuncR>`, `<feFuncG>`, and `<feFuncB>` inside the filter `id`, in that order.
/// This demo's own gamma filter and posterize filter each drive these three channels identically.
/// Neither filter's own `component_transfer` call returns a handle to these children.
/// See this file's own top comment for why.
/// Each channel is instead reached directly, by CSS selector, via `select_el`.
fn select_rgb_funcs(svg: &SvgRoot, id: &str) -> Result<[web_sys::Element; 3], Error> {
    Ok([
        super::select_el(svg, &format!("#{id} feFuncR"))?,
        super::select_el(svg, &format!("#{id} feFuncG"))?,
        super::select_el(svg, &format!("#{id} feFuncB"))?,
    ])
}

fn gamma_caption_text(exponent: f64) -> String {
    format!("Gamma({exponent:.1})")
}

fn discrete_caption_text(steps: i32) -> String {
    format!("Discrete({steps}-step)")
}

fn alpha_caption_text(slope: f64) -> String {
    format!("Alpha Linear({slope:.1})")
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-component-transfer", Size::new(W, COMPONENT_TRANSFER_H))?;

    svg.build_defs(|d| {
        // Same multi-hue gradient technique as the feColorMatrix and feBlend demos.
        // A flat source colour would make the gamma and posterize effects below invisible.
        // Both only become visible where the input colour actually varies across the shape.
        d.build_linear_gradient("component-transfer-source", |g| {
            g.add_stop(0.0, STEELBLUE)?;
            g.add_stop(0.5, GOLD)?;
            g.add_stop(1.0, CRIMSON)?;
            Ok(())
        })?;

        // Gamma-darken all three colour channels identically.
        // An exponent above 1.0 darkens midtones.
        // DEFAULT_GAMMA_TENTHS is only this slider's own starting point now, not a fixed effect.
        d.build_filter("component-transfer-gamma", |f| {
            let gamma = TransferFunction::Gamma {
                amplitude: 1.0,
                exponent: DEFAULT_GAMMA_EXPONENT,
                offset: 0.0,
            };
            f.component_transfer(&[
                (Channel::Red, gamma.clone()),
                (Channel::Green, gamma.clone()),
                (Channel::Blue, gamma),
            ])?;
            Ok(())
        })?;

        // Posterise all three colour channels to DEFAULT_STEPS discrete steps.
        // The stepping shows as hard colour bands.
        // The smooth gradient would otherwise blend continuously.
        d.build_filter("component-transfer-discrete", |f| {
            let posterize = TransferFunction::Discrete(discrete_table(DEFAULT_STEPS));
            f.component_transfer(&[
                (Channel::Red, posterize.clone()),
                (Channel::Green, posterize.clone()),
                (Channel::Blue, posterize),
            ])?;
            Ok(())
        })?;

        // Fade alpha via a linear remap.
        // This filter touches only the Alpha channel.
        // Unlike every other filter in this demo, it leaves colour untouched.
        // This gallery has a dark canvas background.
        // The faded rectangle visibly blends toward it.
        d.build_filter("component-transfer-alpha", |f| {
            f.component_transfer(&[(
                Channel::Alpha,
                TransferFunction::Linear {
                    slope: DEFAULT_SLOPE,
                    intercept: 0.0,
                },
            )])?;
            Ok(())
        })?;

        Ok(())
    })?;

    // This demo needs no filter region widening.
    // That is a property of the specific functions used below, not a general property of feComponentTransfer.
    // The gamma slider and the posterize slider only ever touch RGB.
    // Alpha stays untouched at every slider position.
    // The transparent-versus-opaque silhouette stays untouched too.
    // The alpha slider only ever changes slope.
    // intercept stays fixed at 0.0.
    // So f(0) = slope * 0.0 + 0.0 = 0.0.
    // This holds for every reachable slope, not just this demo's own default.
    //
    // A transfer function can have f(0) greater than 0 on Channel::Alpha.
    // A non-zero intercept or offset would do this.
    // A Table or Discrete list starting above 0.0 would also do this.
    // No slider here can produce either case.
    // Such a function would paint every fully-transparent pixel in the primitive subregion.
    // Here that subregion is the whole filter region, because `in` is SourceGraphic.
    // The rectangle would then show a rectangular halo.
    let r1 = svg.rect(Point::new(LEFT_MARGIN, RECT_Y), Size::new(RECT_W, RECT_H))?;
    r1.set_fill_gradient("component-transfer-source")?;
    super::row_caption(&svg, LEFT_MARGIN + RECT_W / 2.0, CAPTION_Y, "original")?;

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // Gamma
    // A slider above the rectangle drives all three colour channels' own exponent live.
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    let r2 = svg.rect(Point::new(RECT2_X, RECT_Y), Size::new(RECT_W, RECT_H))?;
    r2.set_fill_gradient("component-transfer-source")?;
    r2.set_filter("component-transfer-gamma")?;

    let gamma_caption = svg.text(
        Point::new(RECT2_X + RECT_W / 2.0, CAPTION_Y),
        &gamma_caption_text(DEFAULT_GAMMA_EXPONENT),
    )?;
    gamma_caption.set_fill(CAPTION)?;
    gamma_caption.set_attr("font-size", "11")?;
    gamma_caption.set_text_anchor(TextAnchor::Middle)?;

    let gamma_funcs = select_rgb_funcs(&svg, "component-transfer-gamma")?;
    let gamma_slider = super::build_h_slider(
        &svg,
        Point::new(RECT2_X, TOP),
        Size::new(RECT_W, SLIDER_ROW_H),
        ("gamma exponent", "component transfer gamma exponent"),
        (MIN_GAMMA_TENTHS, MAX_GAMMA_TENTHS, DEFAULT_GAMMA_TENTHS),
        8,
        &["0.2", "5.0"],
    )?
    .input;
    gamma_slider
        .set_attribute("aria-valuetext", &format!("{DEFAULT_GAMMA_EXPONENT:.1}"))
        .map_err(dom_err)?;
    {
        let slider = gamma_slider.clone();
        let funcs = gamma_funcs.clone();
        let caption = gamma_caption.clone();
        let on_input: DemoClosure = Closure::new(move |_: web_sys::Event| {
            let exponent = slider.value_as_number() / 10.0;
            // This uses plain Display, not a fixed `.2` precision.
            // component_transfer's own construction-time formatting also writes a bare `{v}`.
            // See discrete_table's own doc comment above for the same point about the discrete slider.
            // So this attribute's own text never shifts format between construction and interaction.
            // Neither one prints a trailing zero the other lacks.
            for func in &funcs {
                let _ = func.set_attribute("exponent", &exponent.to_string());
            }
            let _ = slider.set_attribute("aria-valuetext", &format!("{exponent:.1}"));
            caption.set_text(&gamma_caption_text(exponent));
        });
        gamma_slider
            .add_event_listener_with_callback("input", on_input.as_ref().unchecked_ref())
            .map_err(dom_err)?;
        keep_demo_closure(on_input);
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // Discrete (posterize)
    // A slider above the rectangle drives the step count of all three colour channels live.
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    let r3 = svg.rect(Point::new(RECT3_X, RECT_Y), Size::new(RECT_W, RECT_H))?;
    r3.set_fill_gradient("component-transfer-source")?;
    r3.set_filter("component-transfer-discrete")?;

    let discrete_caption = svg.text(
        Point::new(RECT3_X + RECT_W / 2.0, CAPTION_Y),
        &discrete_caption_text(DEFAULT_STEPS),
    )?;
    discrete_caption.set_fill(CAPTION)?;
    discrete_caption.set_attr("font-size", "11")?;
    discrete_caption.set_text_anchor(TextAnchor::Middle)?;

    let discrete_funcs = select_rgb_funcs(&svg, "component-transfer-discrete")?;
    let discrete_slider = super::build_h_slider(
        &svg,
        Point::new(RECT3_X, TOP),
        Size::new(RECT_W, SLIDER_ROW_H),
        ("steps", "component transfer discrete step count"),
        (MIN_STEPS, MAX_STEPS, DEFAULT_STEPS),
        1,
        &["2", "8"],
    )?
    .input;
    {
        let slider = discrete_slider.clone();
        let funcs = discrete_funcs.clone();
        let caption = discrete_caption.clone();
        let on_input: DemoClosure = Closure::new(move |_: web_sys::Event| {
            let steps = slider.value_as_number() as i32;
            let table_values = discrete_table_values_attr(steps);
            for func in &funcs {
                let _ = func.set_attribute("tableValues", &table_values);
            }
            caption.set_text(&discrete_caption_text(steps));
        });
        discrete_slider
            .add_event_listener_with_callback("input", on_input.as_ref().unchecked_ref())
            .map_err(dom_err)?;
        keep_demo_closure(on_input);
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // Alpha
    // A slider above the rectangle drives the Alpha channel's own linear slope live. Colour stays untouched.
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    let r4 = svg.rect(Point::new(RECT4_X, RECT_Y), Size::new(RECT_W, RECT_H))?;
    r4.set_fill_gradient("component-transfer-source")?;
    r4.set_filter("component-transfer-alpha")?;

    let alpha_caption = svg.text(
        Point::new(RECT4_X + RECT_W / 2.0, CAPTION_Y),
        &alpha_caption_text(DEFAULT_SLOPE),
    )?;
    alpha_caption.set_fill(CAPTION)?;
    alpha_caption.set_attr("font-size", "11")?;
    alpha_caption.set_text_anchor(TextAnchor::Middle)?;

    let alpha_func = super::select_el(&svg, "#component-transfer-alpha feFuncA")?;
    let alpha_slider = super::build_h_slider(
        &svg,
        Point::new(RECT4_X, TOP),
        Size::new(RECT_W, SLIDER_ROW_H),
        ("alpha slope", "component transfer alpha slope"),
        (MIN_SLOPE_PERCENT, MAX_SLOPE_PERCENT, DEFAULT_SLOPE_PERCENT),
        25,
        &["0%", "100%"],
    )?
    .input;
    alpha_slider
        .set_attribute("aria-valuetext", &format!("{DEFAULT_SLOPE:.1}"))
        .map_err(dom_err)?;
    {
        let slider = alpha_slider.clone();
        let func = alpha_func.clone();
        let caption = alpha_caption.clone();
        let on_input: DemoClosure = Closure::new(move |_: web_sys::Event| {
            let slope = slider.value_as_number() / 100.0;
            // This uses plain Display, not a fixed `.2` precision.
            // See the gamma slider's own handler above for why.
            let _ = func.set_attribute("slope", &slope.to_string());
            let _ = slider.set_attribute("aria-valuetext", &format!("{slope:.1}"));
            caption.set_text(&alpha_caption_text(slope));
        });
        alpha_slider
            .add_event_listener_with_callback("input", on_input.as_ref().unchecked_ref())
            .map_err(dom_err)?;
        keep_demo_closure(on_input);
    }

    Ok(())
}
