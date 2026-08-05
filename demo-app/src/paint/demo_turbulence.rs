use crate::{DemoClosure, colours::*, dom_err, keep_demo_closure};
use svg_dom::{
    Error, SvgRoot, TextAnchor,
    root::{
        filter::{Channel, TurbulenceType},
        utils::{Point, Size},
    },
};
use wasm_bindgen::{JsCast, prelude::*};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// feTurbulence / feDisplacementMap
// One slider drives both noise rectangles' own baseFrequency live.
// Three more controls drive the displacement circle: its own scale, its own x channel, and its own y channel.
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// `turbulence` and `displacement_map` each return their own primitive's `SvgNode` directly.
// So every filter here retains its own node, the same retained-primitive pattern `demo_filter`'s sliders use.
//
// The two noise rectangles keep their own separate TurbulenceType each: FractalNoise and Turbulence. A single shared
// slider drives both of their own baseFrequency values at once. Showing both types side by side, at the same live
// frequency, is more useful here than collapsing them into one dropdown: it lets the two types be compared directly,
// not just switched between.
//
// `baseFrequency`'s own useful range is small, roughly 0.000 to 0.050. So the frequency slider drives an integer number
// of thousandths, the same scaled-slider technique `demo_color_matrix`'s own Saturate slider uses. The displacement
// circle's own scale slider needs no such scaling: its raw integer value already is the real scale, in the same
// coordinate system `displacement_map`'s own `scale` argument uses.
//
// The distorted circle's own x-channel and y-channel controls each pick one of four `Channel` variants, not a
// continuous value. `Channel` already derives `Copy`/`PartialEq`, so no local stand-in enum is needed the way
// `demo_color_matrix`'s own `MatrixVariant` was for the non-`Copy` `ColorMatrixType`. Each control is still a slider,
// not a dropdown or radio group: a 4-position integer slider (`CHANNELS[index]`) keeps this circle's own three controls
// (scale, x, y) visually grouped the same way, rather than mixing control types around one shape.
// `Channel::selector_str` supplies each `xChannelSelector`/`yChannelSelector` keyword directly, the same way
// `demo_blend` already uses `BlendMode::as_str`.
//
// The distorted circle's own control zone stacks two horizontal sliders above it: scale, then x channel. Its own
// y-channel slider sits beside the circle itself instead, lined up with the circle's own diameter rather than with the
// stacked sliders above it. This differs from `demo_filter`'s own drop-shadow banner, whose dy slider spans its whole
// control column: that banner sits between two stacked sliders, so a track spanning both lines up with the/ banner too.
// Here the circle sits below both stacked sliders, not between them, so a track of that same taller span would float
// above the circle instead of beside it. All four shapes in this demo still share one row: the distorted circle's own
// three-control zone is taller than the frequency slider's single row, so that shorter zone simply stays partly empty,
// the same "shared row, unused control zone" layout `demo_color_matrix`'s own row uses.
//
// The library's own `displacement_map` doc comment warns that selecting the same channel for both `x`/`y` reproduces a
// single-diagonal constraint. Making both selectors live lets that warning be demonstrated directly: pick `Alpha` for
// both to see it happen, not just read about it.
//
// The displacement filter's own region is widened once, by a fixed margin, at construction. It is never recomputed from
// any slider's own live value.
// MAX_SCALE is chosen to stay safely inside that fixed margin at every slider position, not just at this
// demo's own default (see MAX_SCALE's own comment for the exact numbers).
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

const CIRCLE_R: f64 = 50.0;
const RECT_W: f64 = 160.0;
const RECT_H: f64 = 100.0;
const ITEM_GAP: f64 = 60.0; // gap between items

const LEFT_MARGIN: f64 = 20.0;
const RECT1_X: f64 = LEFT_MARGIN + CIRCLE_R * 2.0 + ITEM_GAP;
const RECT2_X: f64 = RECT1_X + RECT_W + ITEM_GAP;
const DISTORTED_CX: f64 = RECT2_X + RECT_W + ITEM_GAP + CIRCLE_R;

const ORIGINAL_CX: f64 = LEFT_MARGIN + CIRCLE_R;

const CONTROL_ROW_H: f64 = 62.0; // one HTML slider block: label + track + tick row + endpoint-value row
const CONTROL_GAP: f64 = 6.0; // gap between a slider and the shape(s) it drives
// Neither noise filter sets its own region, so both use SVG's own default: 10% of the referencing rect's own
// bounding box on every side. feTurbulence reads no `in`, so it paints that whole widened region, not just the
// rect itself, and the rendered noise bleeds 10px below RECT_H (100px) before any caption text starts. 14px was
// enough clearance for a shape with no such bleed. This adds that 10px back, plus keeps the same clearance past
// the true, widened bottom edge.
const CAPTION_OFFSET: f64 = 24.0; // caption sits this far below its shape's own bottom edge
const STACK_GAP: f64 = 4.0; // gap between the distorted circle's own stacked scale/x-channel sliders

const TOP: f64 = 10.0;

// The distorted circle's own scale and x-channel sliders stack directly above it, top to bottom.
const SCALE_SLIDER_Y: f64 = TOP;
const X_SLIDER_Y: f64 = TOP + CONTROL_ROW_H + STACK_GAP;
const DISTORTED_CONTROL_H: f64 = CONTROL_ROW_H * 2.0 + STACK_GAP;

// All four shapes share the same row. Every caption below still lines up.
const RECT_Y: f64 = TOP + DISTORTED_CONTROL_H + CONTROL_GAP;
const CIRCLE_CY: f64 = RECT_Y + CIRCLE_R;
const CAPTION_Y: f64 = RECT_Y + RECT_H + CAPTION_OFFSET;

// The y-channel track sits beside the distorted circle, lined up with the circle itself rather than with the
// scale/x-channel sliders stacked above it: it spans the same vertical range as the circle's own diameter, not
// the taller control zone above.
const Y_TRACK_W: f64 = 16.0;
// Wide enough that the vertical slider's own rotated thumb, roughly 24px across, never overlaps the scale/
// x-channel sliders' own endpoint labels to its left (confirmed empirically: 16px left the thumb visibly
// overlapping their "60"/"A" text).
const Y_TRACK_GAP: f64 = 30.0; // gap between the circle's own right edge and the y track
const Y_LABEL_GAP: f64 = 10.0; // gap between the y track and its own per-tick channel-name labels
const Y_TRACK_X: f64 = DISTORTED_CX + CIRCLE_R + Y_TRACK_GAP;
const Y_LABEL_X: f64 = Y_TRACK_X + Y_TRACK_W + Y_LABEL_GAP;
const Y_TRACK_Y: f64 = RECT_Y;
const Y_TRACK_LENGTH: f64 = RECT_H;

// Wider than the shared W (see lib.rs): the four shapes' own wider gaps, plus the RGBA channel labels to the
// right of the y-channel track, need more room than the gallery's own default canvas width gives every other
// demo.
const TURBULENCE_W: f64 = 850.0;
const TURBULENCE_H: f64 = CAPTION_Y + 12.0;

const MIN_FREQUENCY_THOUSANDTHS: i32 = 0;
const MAX_FREQUENCY_THOUSANDTHS: i32 = 50;
const DEFAULT_FREQUENCY_THOUSANDTHS: i32 = 15; // 0.015, this demo's own original default
const DEFAULT_FREQUENCY: f64 = DEFAULT_FREQUENCY_THOUSANDTHS as f64 / 1000.0;

const MIN_SCALE: i32 = 0;
// The displacement filter's own region is widened by a fixed -50%/-50%/200%/200% (see `widen_filter_region`),
// a margin of 50% of the circle's own bounding-box size on each side: 50px, for this demo's own 100px-diameter
// circle. `displacement_map`'s own maximum displacement is `scale / 2`. So `MAX_SCALE` must stay at or below
// 100 for that worst case to stay within the fixed 50px margin. 60 keeps 20px of headroom below that limit,
// while still reaching well past this demo's own original fixed scale (24).
const MAX_SCALE: i32 = 60;
const DEFAULT_SCALE: i32 = 24; // this demo's own original default

const CHANNEL_SLIDER_W: f64 = 100.0; // shared by the scale and x-channel sliders, both centred over the circle

// `xChannelSelector`/`yChannelSelector` each pick one of these four, by index: `Red` = 0 through `Alpha` = 3,
// the same order `Channel`'s own variants are declared in.
const CHANNELS: [Channel; 4] = [Channel::Red, Channel::Green, Channel::Blue, Channel::Alpha];
const DEFAULT_X_INDEX: i32 = 0; // Red
const DEFAULT_Y_INDEX: i32 = 1; // Green

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Formats a baseFrequency value via plain `Display`, the same formatting `turbulence`'s own construction-time
/// path uses internally for the `baseFrequency` attribute itself.
/// Every raw slider position is an exact multiple of `0.001`, so this is always lossless: unlike Saturate's own
/// 1%-step slider (see `demo_color_matrix`'s own `saturate_value_text`), no two distinct positions here can ever
/// format to the same text.
/// Using the same formatting for the caption and `aria-valuetext` as for the attribute itself keeps all three in
/// agreement, rather than the attribute showing `0.05` while the caption shows a differently formatted `0.050`.
fn frequency_value_text(frequency: f64) -> String {
    frequency.to_string()
}

fn fractal_caption_text(frequency: f64) -> String {
    format!("FractalNoise ({})", frequency_value_text(frequency))
}

fn marbled_caption_text(frequency: f64) -> String {
    format!("Turbulence ({})", frequency_value_text(frequency))
}

fn channel_name(channel: Channel) -> &'static str {
    match channel {
        Channel::Red => "Red",
        Channel::Green => "Green",
        Channel::Blue => "Blue",
        Channel::Alpha => "Alpha",
    }
}

/// Formats the distorted circle's own live readout, shared by the scale/x/y sliders' `on_input` handlers so
/// each one reports the current value of the other two, not just the one it just moved.
fn distorted_caption_text(scale: f64, x_channel: Channel, y_channel: Channel) -> String {
    format!(
        "scale {scale:.0} · x {} · y {}",
        channel_name(x_channel),
        channel_name(y_channel)
    )
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-turbulence", Size::new(TURBULENCE_W, TURBULENCE_H))?;

    let defs = svg.defs()?;

    // feTurbulence reads no `in` at all: it fabricates its own noise image from nothing, so applying it with no
    // other primitive replaces the referencing element's content with a rectangular patch of noise covering the
    // whole filter region, not clipped to the element's own shape (see SvgFilter::turbulence's doc comment).
    //
    // These two filters exist purely to show the raw noise pattern each TurbulenceType produces, so a plain
    // rect (already rectangular) is used rather than a circle, which would look broken. The fill colour on the
    // rects themselves is irrelevant, since the noise output covers all of it.
    let fractal_filter = defs.filter("turbulence-fractal")?;
    let fractal_node = fractal_filter.turbulence(DEFAULT_FREQUENCY, 4, 3.0, TurbulenceType::FractalNoise)?;

    let marbled_filter = defs.filter("turbulence-marbled")?;
    let marbled_node = marbled_filter.turbulence(DEFAULT_FREQUENCY, 4, 3.0, TurbulenceType::Turbulence)?;

    // The main showcase: feTurbulence -> feDisplacementMap warps a circle's edge into a hand-drawn, organic
    // outline instead of a perfect geometric one — the standard pairing these two primitives exist for.
    // Red/Green (rather than Alpha/Alpha) give the displacement two independent axes at this demo's own
    // default: feTurbulence generates each colour channel separately, so dx and dy are no longer forced equal
    // at every pixel the way they would be reading the same channel for both selectors, which would confine
    // every displacement vector to a single diagonal. The x-channel/y-channel sliders below let that
    // constraint be demonstrated directly, by choosing the same channel for both. The displaced sampling can
    // read pixels just outside the circle's own bounding box, so the region is widened the same way demo_filter
    // widens it for a wide blur.
    let displace_filter = defs.filter("turbulence-displace")?;
    super::widen_filter_region(&displace_filter)?;
    let displace_noise = displace_filter.turbulence(0.02, 3, 5.0, TurbulenceType::FractalNoise)?;
    displace_noise.set_attr("result", "noise")?;
    let displace_map = displace_filter.displacement_map(
        "noise",
        f64::from(DEFAULT_SCALE),
        CHANNELS[DEFAULT_X_INDEX as usize],
        CHANNELS[DEFAULT_Y_INDEX as usize],
    )?;
    displace_map.set_attr("in", "SourceGraphic")?;

    let original = svg.circle(Point::new(ORIGINAL_CX, CIRCLE_CY), CIRCLE_R)?;
    original.set_fill(STEELBLUE)?;
    super::row_caption(&svg, ORIGINAL_CX, CAPTION_Y, "original")?;

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // FractalNoise / Turbulence
    // One slider, spanning both rectangles, drives both filters' own baseFrequency live.
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    let r1 = svg.rect(Point::new(RECT1_X, RECT_Y), Size::new(RECT_W, RECT_H))?;
    r1.set_filter("turbulence-fractal")?;

    let fractal_caption = svg.text(
        Point::new(RECT1_X + RECT_W / 2.0, CAPTION_Y),
        &fractal_caption_text(DEFAULT_FREQUENCY),
    )?;
    fractal_caption.set_fill(CAPTION)?;
    fractal_caption.set_attr("font-size", "11")?;
    fractal_caption.set_text_anchor(TextAnchor::Middle)?;

    let r2 = svg.rect(Point::new(RECT2_X, RECT_Y), Size::new(RECT_W, RECT_H))?;
    r2.set_filter("turbulence-marbled")?;

    let marbled_caption = svg.text(
        Point::new(RECT2_X + RECT_W / 2.0, CAPTION_Y),
        &marbled_caption_text(DEFAULT_FREQUENCY),
    )?;
    marbled_caption.set_fill(CAPTION)?;
    marbled_caption.set_attr("font-size", "11")?;
    marbled_caption.set_text_anchor(TextAnchor::Middle)?;

    let frequency_slider_w = RECT2_X + RECT_W - RECT1_X;
    let frequency_slider = super::build_h_slider(
        &svg,
        Point::new(RECT1_X, TOP),
        Size::new(frequency_slider_w, CONTROL_ROW_H),
        ("baseFrequency", "turbulence base frequency"),
        (
            MIN_FREQUENCY_THOUSANDTHS,
            MAX_FREQUENCY_THOUSANDTHS,
            DEFAULT_FREQUENCY_THOUSANDTHS,
        ),
        10,
        ("0.000", "0.050"),
    )?
    .input;
    frequency_slider
        .set_attribute("aria-valuetext", &frequency_value_text(DEFAULT_FREQUENCY))
        .map_err(dom_err)?;
    {
        let slider = frequency_slider.clone();
        let fractal = fractal_node.clone();
        let marbled = marbled_node.clone();
        let fractal_caption = fractal_caption.clone();
        let marbled_caption = marbled_caption.clone();
        let on_input: DemoClosure = Closure::new(move |_: web_sys::Event| {
            let frequency = slider.value_as_number() / 1000.0;
            let _ = fractal.set_attr("baseFrequency", &frequency.to_string());
            let _ = marbled.set_attr("baseFrequency", &frequency.to_string());
            let _ = slider.set_attribute("aria-valuetext", &frequency_value_text(frequency));
            fractal_caption.set_text(&fractal_caption_text(frequency));
            marbled_caption.set_text(&marbled_caption_text(frequency));
        });
        frequency_slider
            .add_event_listener_with_callback("input", on_input.as_ref().unchecked_ref())
            .map_err(dom_err)?;
        keep_demo_closure(on_input);
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // organic edge (feDisplacementMap) — scale (above), x channel (stacked below scale), y channel (beside)
    // sliders drive feDisplacementMap live
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    let distorted = svg.circle(Point::new(DISTORTED_CX, CIRCLE_CY), CIRCLE_R)?;
    distorted.set_fill(STEELBLUE)?;
    distorted.set_filter("turbulence-displace")?;

    let distorted_caption = svg.text(
        Point::new(DISTORTED_CX, CAPTION_Y),
        &distorted_caption_text(
            f64::from(DEFAULT_SCALE),
            CHANNELS[DEFAULT_X_INDEX as usize],
            CHANNELS[DEFAULT_Y_INDEX as usize],
        ),
    )?;
    distorted_caption.set_fill(CAPTION)?;
    distorted_caption.set_attr("font-size", "11")?;
    distorted_caption.set_text_anchor(TextAnchor::Middle)?;

    let scale_slider = super::build_h_slider(
        &svg,
        Point::new(DISTORTED_CX - CHANNEL_SLIDER_W / 2.0, SCALE_SLIDER_Y),
        Size::new(CHANNEL_SLIDER_W, CONTROL_ROW_H),
        ("scale", "displacement map scale"),
        (MIN_SCALE, MAX_SCALE, DEFAULT_SCALE),
        20,
        ("0", "60"),
    )?
    .input;

    let x_slider = super::build_h_slider(
        &svg,
        Point::new(DISTORTED_CX - CHANNEL_SLIDER_W / 2.0, X_SLIDER_Y),
        Size::new(CHANNEL_SLIDER_W, CONTROL_ROW_H),
        ("x channel", "displacement map x channel"),
        (0, 3, DEFAULT_X_INDEX),
        1,
        ("R", "A"),
    )?
    .input;
    x_slider
        .set_attribute("aria-valuetext", channel_name(CHANNELS[DEFAULT_X_INDEX as usize]))
        .map_err(dom_err)?;

    // y channel: see build_v_slider's own doc comment (paint/mod.rs) for the CSS rotation and keyboard remap it
    // needs. Plain SVG text and line elements decorate it here, not HTML, the same reason demo_filter's own dy
    // slider does. The track spans the circle's own diameter, lined up with the image rather than the stacked
    // scale/x-channel sliders above it, so its own tick labels name all four channels directly, rather than only
    // the two endpoints a plain build_h_slider call would show.
    //
    // Red (index 0) sits at the top of the track, Alpha (index 3) at the bottom — matching demo_filter's own
    // dy convention of "up is smaller", applied here to the channel's own index rather than a signed offset.
    for (index, &channel) in CHANNELS.iter().enumerate() {
        let fraction = index as f64 / (CHANNELS.len() - 1) as f64;
        let y = Y_TRACK_Y + fraction * Y_TRACK_LENGTH;
        let mark = svg.line(Point::new(Y_TRACK_X - 3.0, y), Point::new(Y_TRACK_X + Y_TRACK_W + 3.0, y))?;
        mark.set_stroke(GUIDE)?;
        mark.set_stroke_width(1.0)?;
        super::side_label(&svg, Y_LABEL_X, y + 4.0, channel_name(channel))?;
    }

    let y_slider = super::build_v_slider(
        &svg,
        Point::new(Y_TRACK_X, Y_TRACK_Y),
        Y_TRACK_W,
        Y_TRACK_LENGTH,
        "displacement map y channel",
        (0, 3, DEFAULT_Y_INDEX),
    )?;
    y_slider
        .set_attribute("aria-valuetext", channel_name(CHANNELS[DEFAULT_Y_INDEX as usize]))
        .map_err(dom_err)?;

    {
        let scale = scale_slider.clone();
        let x = x_slider.clone();
        let y = y_slider.clone();
        let map = displace_map.clone();
        let caption = distorted_caption.clone();
        let on_input: DemoClosure = Closure::new(move |_: web_sys::Event| {
            let value = scale.value_as_number();
            let _ = map.set_attr("scale", &value.to_string());
            let x_channel = CHANNELS[x.value_as_number() as usize];
            let y_channel = CHANNELS[y.value_as_number() as usize];
            caption.set_text(&distorted_caption_text(value, x_channel, y_channel));
        });
        scale_slider
            .add_event_listener_with_callback("input", on_input.as_ref().unchecked_ref())
            .map_err(dom_err)?;
        keep_demo_closure(on_input);
    }
    {
        let scale = scale_slider.clone();
        let x = x_slider.clone();
        let y = y_slider.clone();
        let map = displace_map.clone();
        let caption = distorted_caption.clone();
        let on_input: DemoClosure = Closure::new(move |_: web_sys::Event| {
            let channel = CHANNELS[x.value_as_number() as usize];
            let _ = map.set_attr("xChannelSelector", channel.selector_str());
            let _ = x.set_attribute("aria-valuetext", channel_name(channel));
            let y_channel = CHANNELS[y.value_as_number() as usize];
            caption.set_text(&distorted_caption_text(scale.value_as_number(), channel, y_channel));
        });
        x_slider
            .add_event_listener_with_callback("input", on_input.as_ref().unchecked_ref())
            .map_err(dom_err)?;
        keep_demo_closure(on_input);
    }
    {
        let scale = scale_slider.clone();
        let x = x_slider.clone();
        let y = y_slider.clone();
        let map = displace_map.clone();
        let caption = distorted_caption.clone();
        let on_input: DemoClosure = Closure::new(move |_: web_sys::Event| {
            let channel = CHANNELS[y.value_as_number() as usize];
            let _ = map.set_attr("yChannelSelector", channel.selector_str());
            let _ = y.set_attribute("aria-valuetext", channel_name(channel));
            let x_channel = CHANNELS[x.value_as_number() as usize];
            caption.set_text(&distorted_caption_text(scale.value_as_number(), x_channel, channel));
        });
        y_slider
            .add_event_listener_with_callback("input", on_input.as_ref().unchecked_ref())
            .map_err(dom_err)?;
        keep_demo_closure(on_input);
    }

    Ok(())
}
