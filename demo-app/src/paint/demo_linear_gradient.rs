use crate::{DemoClosure, W, colours::*, dom_err, keep_demo_closure};
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
// `row_caption`, `side_label`, `select_el`, `build_h_slider`, and `build_v_slider` (called below via `super::`)
// live in `paint/mod.rs`, shared with `demo_radial_gradient` — the other demo whose own controls need this same
// custom row layout, rather than the shared W/H/PAD_Y/BAND/caption convention every other demo uses. See their
// own doc comments there for what each one does and why.
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
/// Updates stop 3's own live lower bound to follow stop 2's current value: its `min` attribute, its own endpoint
/// label, and its own tick marks all move together, so none of the three describes a different range than the
/// other two. `99` and `25` are stop 3's own fixed absolute max and tick step (see its own `build_h_slider` call
/// below), unaffected by stop 2's movement.
fn sync_s3_min(s3: &super::HSlider, s2_value: f64) -> Result<(), Error> {
    let new_min = (s2_value + 1.0).round() as i32;
    s3.input.set_min(&new_min.to_string());
    s3.lo_label.set_text_content(Some(&format!("{new_min}%")));
    super::fill_ticks(&s3.ticks_row, new_min, 99, 25)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Updates stop 2's own live upper bound to follow stop 3's current value, the same way [`sync_s3_min`] updates
/// stop 3's. `1` and `25` are stop 2's own fixed absolute min and tick step.
fn sync_s2_max(s2: &super::HSlider, s3_value: f64) -> Result<(), Error> {
    let new_max = (s3_value - 1.0).round() as i32;
    s2.input.set_max(&new_max.to_string());
    s2.hi_label.set_text_content(Some(&format!("{new_max}%")));
    super::fill_ticks(&s2.ticks_row, 1, new_max, 25)
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

        // 5. Gradient stroke: a colour sweep applied to a stroke rather than a fill.
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
    super::row_caption(&svg, ROW1_H_X + RECT_W / 2.0, ROW1_CAPTION_Y, "horizontal")?;

    let h_stop = super::select_el(&svg, "#demo-lg-h stop:nth-child(2)")?;
    let h_slider = super::build_h_slider(
        &svg,
        Point::new(ROW1_H_X, ROW1_TOP),
        Size::new(RECT_W, SLIDER_ROW_H),
        ("shift stop 2", "horizontal gradient stop 2"),
        (10, 100, 100),
        30,
        &["10%", "100%"],
    )?
    .input;
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
    super::row_caption(&svg, ROW1_V_X + RECT_W / 2.0, ROW1_CAPTION_Y, "vertical")?;

    // Vertical-shift control: see `build_v_slider`'s own doc comment (paint/mod.rs) for the CSS rotation and
    // keyboard remapping it needs.
    // Plain SVG text and line elements decorate it here, not HTML: once the control is rotated, an HTML tick row
    // or endpoint-label row would need extra CSS positioning these plain SVG elements do not.
    let v_stop = super::select_el(&svg, "#demo-lg-v stop:nth-child(2)")?;
    let v_track_x = ROW1_V_X + RECT_W + V_TRACK_GAP;

    super::row_caption(&svg, v_track_x + V_TRACK_W / 2.0, ROW1_TOP + 10.0, "shift stop 2")?;

    // Endpoint values: min (10%) sits at the top of the track, max (100%) at the bottom.
    // This matches the track's own top-is-min orientation, set by `build_v_slider` below.
    // Up therefore moves the stop up, the same spatial logic the horizontal slider's left-is-smaller framing uses.
    super::side_label(&svg, v_track_x + V_TRACK_W + 6.0, ROW1_RECT_Y + 4.0, "10%")?;
    super::side_label(&svg, v_track_x + V_TRACK_W + 6.0, ROW1_RECT_Y + RECT_H - 2.0, "100%")?;

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

    let v_slider = super::build_v_slider(
        &svg,
        Point::new(v_track_x, ROW1_RECT_Y),
        V_TRACK_W,
        RECT_H,
        "shift the vertical gradient's second stop",
        (10, 100, 100),
    )?;
    // Matches the slider's own default value (100) above, set once here for the same reason the horizontal
    // slider's own aria-valuetext is set at construction, not only after the first input event.
    v_slider.set_attribute("aria-valuetext", "100%").map_err(dom_err)?;
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

    let d_gradient = super::select_el(&svg, "#demo-lg-d")?;
    // The slider's own min/max/value all share one coordinate system: the total angle applied to the gradient,
    // not a displacement from some other base folded in afterwards. `45` (the initial rotation) sits 90° in from
    // each end, giving the same ±90° of travel the earlier relative-displacement version offered, without the
    // slider's own numeric range and its spoken aria-valuetext describing two different scales.
    let rotate_slider = super::build_h_slider(
        &svg,
        Point::new(ROW2_D_X, ROW2_SINGLE_SLIDER_Y),
        Size::new(RECT_W, SLIDER_ROW_H),
        ("rotate", "diagonal gradient rotation"),
        (-45, 135, 45),
        45,
        &["-45°", "135°"],
    )?
    .input;
    // Matches the slider's own default value (45) above, set once here so a screen reader announces the real
    // starting value, not "no value", before any interaction happens.
    rotate_slider.set_attribute("aria-valuetext", "rotate 45°").map_err(dom_err)?;
    {
        let slider = rotate_slider.clone();
        let gradient = d_gradient.clone();
        let label = rotate_label.clone();
        let mut buf = String::new();
        let on_input: DemoClosure = Closure::new(move |_: web_sys::Event| {
            let total = slider.value_as_number();
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
    super::row_caption(&svg, ROW2_S_X + RECT_W / 2.0, ROW2_CAPTION_Y, "4-stop spectrum")?;

    // The middle two stops start at 0.35 (index 1) and 0.65 (index 2).
    // These two stops stay ordered by construction, both against each other and against the fixed outer stops
    // (0.0 and 1.0).
    //
    // stop2/stop3 keep a live one-point gap between each other. `s2.input`'s own `max` attribute always tracks
    // `s3.input.value - 1`. `s3.input`'s own `min` attribute always tracks `s2.input.value + 1`. The browser
    // itself then refuses to move either thumb past the other, keyboard included, before this demo's own
    // `on_input` handler ever runs. Neither handler clamps or writes back a value any more: each slider's native
    // sanitisation already guarantees the value it reports is in range.
    //
    // `(1, 98)` and `(2, 99)` below, and the "1%"/"98%"/"2%"/"99%" endpoint labels and tick marks
    // `build_h_slider` draws from them, describe each slider's absolute possible range against the fixed outer
    // stops, not its live range against the other slider. Those stay fixed on purpose, as a description of the
    // total range each stop could ever reach, distinct from the live `max`/`min` this section maintains.
    let s2_stop = super::select_el(&svg, "#demo-lg-s stop:nth-child(2)")?;
    let s3_stop = super::select_el(&svg, "#demo-lg-s stop:nth-child(3)")?;
    let s2 = super::build_h_slider(
        &svg,
        Point::new(ROW2_S_X, ROW2_TOP),
        Size::new(RECT_W, SLIDER_ROW_H),
        ("stop 2", "spectrum gradient stop 2"),
        (1, 98, 35),
        25,
        &["1%", "98%"],
    )?;
    // Matches this slider's own default value (35) above, set once here rather than only after the first input
    // event, the same reason every other slider in this file sets its own initial aria-valuetext.
    s2.input.set_attribute("aria-valuetext", "35%").map_err(dom_err)?;
    let s3 = super::build_h_slider(
        &svg,
        Point::new(ROW2_S_X, ROW2_TOP + SLIDER_ROW_H + SLIDER_GAP),
        Size::new(RECT_W, SLIDER_ROW_H),
        ("stop 3", "spectrum gradient stop 3"),
        (2, 99, 65),
        25,
        &["2%", "99%"],
    )?;
    // Matches this slider's own default value (65) above, for the same reason.
    s3.input.set_attribute("aria-valuetext", "65%").map_err(dom_err)?;

    // Establishes the live one-point gap from each slider's own default value, before either has fired an
    // `on_input` event of its own — syncing each other's live bound, endpoint label, and tick marks together,
    // the same [`sync_s3_min`]/[`sync_s2_max`] every later `on_input` event below also calls.
    sync_s2_max(&s2, s3.input.value_as_number())?;
    sync_s3_min(&s3, s2.input.value_as_number())?;
    {
        let this_slider = s2.input.clone();
        let other = s3.clone();
        let stop = s2_stop.clone();
        let on_input: DemoClosure = Closure::new(move |_: web_sys::Event| {
            let value = this_slider.value_as_number();
            let _ = stop.set_attribute("offset", &format!("{:.3}", value / 100.0));
            let _ = this_slider.set_attribute("aria-valuetext", &format!("{value:.0}%"));
            let _ = sync_s3_min(&other, value);
        });
        s2.input
            .add_event_listener_with_callback("input", on_input.as_ref().unchecked_ref())
            .map_err(dom_err)?;
        keep_demo_closure(on_input);
    }
    {
        let this_slider = s3.input.clone();
        let other = s2.clone();
        let stop = s3_stop.clone();
        let on_input: DemoClosure = Closure::new(move |_: web_sys::Event| {
            let value = this_slider.value_as_number();
            let _ = stop.set_attribute("offset", &format!("{:.3}", value / 100.0));
            let _ = this_slider.set_attribute("aria-valuetext", &format!("{value:.0}%"));
            let _ = sync_s2_max(&other, value);
        });
        s3.input
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
    super::row_caption(&svg, ROW3_X + RECT_W / 2.0, ROW3_CAPTION_Y, "gradient stroke")?;

    Ok(())
}
