use crate::{DemoClosure, W, colours::*, dom_err, keep_demo_closure};
use svg_dom::{
    Error, SvgRoot, TextAnchor,
    root::utils::{Point, Size},
};
use wasm_bindgen::{JsCast, prelude::*};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// filter — one circle whose feGaussianBlur stdDeviation is driven live by a slider, plus a tinted drop shadow
// whose feDropShadow dx/dy/stdDeviation are each driven live by their own slider, applied via set_filter
//
// `select_el`, `build_h_slider`, `build_v_slider`, `side_label`, and `widen_filter_region` (called below via
// `super::`) live in `paint/mod.rs`. The blurred circle uses the same slider-above-shape layout
// `demo_radial_gradient`'s own "centred" row uses. The drop-shadow banner uses the same dx-above/dy-beside
// layout `demo_radial_gradient`'s own "off-centre focal" row uses for fx/fy, plus a third slider below for
// stdDeviation, with its dy track on the banner's left rather than the rectangle's right, spanning the whole
// control column rather than just the height of the banner's own text box.
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

const LEFT_MARGIN: f64 = 40.0; // the blur circle's own slider/circle column starts here
const SLIDER_W: f64 = 140.0;
const SLIDER_ROW_H: f64 = 62.0; // one HTML slider block: label + track + tick row + endpoint-value row
const CONTROL_GAP: f64 = 6.0; // gap between a slider and the shape it controls
const CAPTION_OFFSET: f64 = 14.0; // caption sits this far below its shape's bottom edge

const TOP: f64 = 10.0;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Blurred circle
const CIRCLE_R: f64 = 45.0;
const CIRCLE_RECT_Y: f64 = TOP + SLIDER_ROW_H + CONTROL_GAP;
const CIRCLE_CX: f64 = LEFT_MARGIN + SLIDER_W / 2.0;
const CIRCLE_CY: f64 = CIRCLE_RECT_Y + CIRCLE_R;
const CIRCLE_CAPTION_Y: f64 = CIRCLE_RECT_Y + CIRCLE_R * 2.0 + CAPTION_OFFSET;

const MIN_BLUR: i32 = 0;
const MAX_BLUR: i32 = 20;
const DEFAULT_BLUR: i32 = 3;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Drop-shadow banner
const SHADOW_BOX_W: f64 = 280.0; // wide enough for "DROP SHADOW" at font-size 42
const SHADOW_BOX_H: f64 = 60.0; // banner text's own line height
const SHADOW_BOX_X: f64 = 300.0; // left edge of the text bounding box
const SHADOW_CX: f64 = SHADOW_BOX_X + SHADOW_BOX_W / 2.0; // banner text's own horizontal centre

const SHADOW_DY_TRACK_W: f64 = 16.0;
const SHADOW_DY_TRACK_GAP: f64 = 16.0; // gap between the dy track and the text box
const SHADOW_DY_LABEL_GAP: f64 = 40.0; // room for the dy track's own endpoint labels to its left
const SHADOW_DY_TRACK_X: f64 = SHADOW_BOX_X - SHADOW_DY_TRACK_GAP - SHADOW_DY_TRACK_W;
const SHADOW_DY_LABEL_X: f64 = SHADOW_DY_TRACK_X - SHADOW_DY_LABEL_GAP;

const SHADOW_BOX_Y: f64 = TOP + SLIDER_ROW_H + CONTROL_GAP;
const SHADOW_CY: f64 = SHADOW_BOX_Y + SHADOW_BOX_H / 2.0;
const SHADOW_STDDEV_TOP: f64 = SHADOW_BOX_Y + SHADOW_BOX_H + CONTROL_GAP;
const SHADOW_CAPTION_Y: f64 = SHADOW_STDDEV_TOP + SLIDER_ROW_H + CAPTION_OFFSET;

// The dy track spans the whole control column, from the dx slider's own top edge down to the stdDeviation
// slider's own bottom edge, rather than just alongside the banner's own text box.
const SHADOW_DY_TRACK_Y: f64 = TOP;
const SHADOW_DY_TRACK_LENGTH: f64 = SHADOW_STDDEV_TOP + SLIDER_ROW_H - SHADOW_DY_TRACK_Y;

const MIN_OFFSET: i32 = -10;
const MAX_OFFSET: i32 = 10;
const DEFAULT_DX: i32 = 6;
const DEFAULT_DY: i32 = 6;
const MIN_SHADOW_BLUR: i32 = 0;
const MAX_SHADOW_BLUR: i32 = 20;
const DEFAULT_SHADOW_BLUR: i32 = 4;

// The shadow column is taller than the circle column, so the canvas height follows it.
const FILTER_H: f64 = if CIRCLE_CAPTION_Y > SHADOW_CAPTION_Y {
    CIRCLE_CAPTION_Y
} else {
    SHADOW_CAPTION_Y
} + 12.0;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Formats the drop-shadow banner's own live readout, shared by all three of its sliders' `on_input` handlers so
/// each one reports the current value of the other two, not just the one it just moved.
fn shadow_caption_text(dx: f64, dy: f64, std_deviation: f64) -> String {
    format!("dx {dx:.0} · dy {dy:.0} · stdDeviation {std_deviation:.0}")
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-filter", Size::new(W, FILTER_H))?;

    svg.build_defs(|d| {
        // Blur-only filter. Its stdDeviation starts at DEFAULT_BLUR and is then updated live by the slider
        // below. The SVG default filter region (-10%/-10%/120%/120% of the referencing element's bounding box)
        // clips visibly once stdDeviation nears MAX_BLUR, so the region is widened up front for the whole slider
        // range, not just its default.
        d.build_filter("demo-filter-blur", |f| {
            super::widen_filter_region(f)?;
            f.gaussian_blur(f64::from(DEFAULT_BLUR))?;
            Ok(())
        })?;

        // True tinted drop shadow via the feDropShadow shorthand: one primitive call blurs the source alpha,
        // floods a colour into the blurred mask, offsets it, and merges it underneath the original — no separate
        // merge() call needed, since feDropShadow's result already has the original graphic composited on top.
        // A plain black shadow would be nearly invisible against this dark canvas background, so the flood colour
        // is a saturated one instead, which also demonstrates that the shadow's colour is independently
        // controllable, not just a blurred copy of the source graphic's own fill. Its stdDeviation/dx/dy start at
        // their own DEFAULT_ constants and are then updated live by the three sliders below; the region is
        // widened for the same reason the blur filter's is, since both the offset and the blur can push the
        // shadow past the SVG default region.
        d.build_filter("demo-filter-shadow", |f| {
            super::widen_filter_region(f)?;
            f.drop_shadow(
                f64::from(DEFAULT_SHADOW_BLUR),
                f64::from(DEFAULT_DX),
                f64::from(DEFAULT_DY),
                CRIMSON,
                0.85,
            )?;
            Ok(())
        })?;
        Ok(())
    })?;

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // Blurred circle — a slider above it drives feGaussianBlur's stdDeviation live
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    let circle = svg.circle(Point::new(CIRCLE_CX, CIRCLE_CY), CIRCLE_R)?;
    circle.set_fill(STEELBLUE)?;
    circle.set_filter("demo-filter-blur")?;

    let blur_caption = svg.text(
        Point::new(CIRCLE_CX, CIRCLE_CAPTION_Y),
        &format!("stdDeviation: {DEFAULT_BLUR}"),
    )?;
    blur_caption.set_fill(CAPTION)?;
    blur_caption.set_attr("font-size", "11")?;
    blur_caption.set_text_anchor(TextAnchor::Middle)?;

    let blur = super::select_el(&svg, "#demo-filter-blur feGaussianBlur")?;
    let blur_slider = super::build_h_slider(
        &svg,
        Point::new(LEFT_MARGIN, TOP),
        Size::new(SLIDER_W, SLIDER_ROW_H),
        ("blur", "gaussian blur standard deviation"),
        (MIN_BLUR, MAX_BLUR, DEFAULT_BLUR),
        5,
        ("0", "20"),
    )?
    .input;
    // Matches the slider's own default value (DEFAULT_BLUR) above, set once here so a screen reader announces the
    // real starting value, not "no value", before any interaction happens.
    blur_slider
        .set_attribute("aria-valuetext", &format!("stdDeviation {DEFAULT_BLUR}"))
        .map_err(dom_err)?;
    {
        let slider = blur_slider.clone();
        let blur = blur.clone();
        let blur_caption = blur_caption.clone();
        let on_input: DemoClosure = Closure::new(move |_: web_sys::Event| {
            let value = slider.value_as_number();
            let _ = blur.set_attribute("stdDeviation", &format!("{value:.0}"));
            let _ = slider.set_attribute("aria-valuetext", &format!("stdDeviation {value:.0}"));
            blur_caption.set_text(&format!("stdDeviation: {value:.0}"));
        });
        blur_slider
            .add_event_listener_with_callback("input", on_input.as_ref().unchecked_ref())
            .map_err(dom_err)?;
        keep_demo_closure(on_input);
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // Drop-shadow banner — dx (above), dy (left), and stdDeviation (below) sliders drive feDropShadow live
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // Drop-shadow banner text: the feDropShadow filter applied to real text content rather than a plain shape,
    // the effect's most common real-world use. White fill with a narrow dark grey border keeps the glyphs legible
    // against the dark canvas background; independently of that, the shadow's own colour comes from
    // feDropShadow's flood-color, not from the text's fill, so it stays the same crimson regardless of what
    // colour the banner itself is set to.
    let banner = svg.text(Point::new(SHADOW_CX, SHADOW_CY + 15.0), "DROP SHADOW")?;
    banner.set_fill(WHITE)?;
    banner.set_stroke("#333333")?;
    banner.set_stroke_width(1.0)?;
    banner.set_font_size(42.0)?;
    banner.set_text_anchor(TextAnchor::Middle)?;
    banner.set_attr("font-weight", "bold")?;
    banner.set_filter("demo-filter-shadow")?;

    let shadow_caption = svg.text(
        Point::new(SHADOW_CX, SHADOW_CAPTION_Y),
        &shadow_caption_text(f64::from(DEFAULT_DX), f64::from(DEFAULT_DY), f64::from(DEFAULT_SHADOW_BLUR)),
    )?;
    shadow_caption.set_fill(CAPTION)?;
    shadow_caption.set_attr("font-size", "11")?;
    shadow_caption.set_text_anchor(TextAnchor::Middle)?;

    let shadow = super::select_el(&svg, "#demo-filter-shadow feDropShadow")?;

    let dx_slider = super::build_h_slider(
        &svg,
        Point::new(SHADOW_BOX_X, TOP),
        Size::new(SHADOW_BOX_W, SLIDER_ROW_H),
        ("dx", "drop shadow dx offset"),
        (MIN_OFFSET, MAX_OFFSET, DEFAULT_DX),
        5,
        ("-10", "10"),
    )?
    .input;
    dx_slider
        .set_attribute("aria-valuetext", &format!("dx {DEFAULT_DX}"))
        .map_err(dom_err)?;

    let stddev_slider = super::build_h_slider(
        &svg,
        Point::new(SHADOW_BOX_X, SHADOW_STDDEV_TOP),
        Size::new(SHADOW_BOX_W, SLIDER_ROW_H),
        ("blur", "drop shadow standard deviation"),
        (MIN_SHADOW_BLUR, MAX_SHADOW_BLUR, DEFAULT_SHADOW_BLUR),
        5,
        ("0", "20"),
    )?
    .input;
    stddev_slider
        .set_attribute("aria-valuetext", &format!("stdDeviation {DEFAULT_SHADOW_BLUR}"))
        .map_err(dom_err)?;

    // dy: see build_v_slider's own doc comment (paint/mod.rs) for the CSS rotation and keyboard remapping it
    // needs. Plain SVG text and line elements decorate it here, not HTML, the same reason demo_radial_gradient's
    // own fy slider does. The track spans the whole control column (dx's own top edge to stdDeviation's own
    // bottom edge), so its own "dy" label sits beside its vertical centre rather than above a short track.
    //
    // Endpoint values: min (-10, the shadow shifted up) sits at the top of the track, max (10, the shadow shifted
    // down) at the bottom — matching dy's own top-is-smaller meaning, the same "up is smaller" spatial framing
    // demo_radial_gradient's own vertical slider uses.
    super::side_label(&svg, SHADOW_DY_LABEL_X, SHADOW_DY_TRACK_Y + 4.0, "-10")?;
    super::side_label(
        &svg,
        SHADOW_DY_LABEL_X,
        SHADOW_DY_TRACK_Y + SHADOW_DY_TRACK_LENGTH / 2.0 + 4.0,
        "dy",
    )?;
    super::side_label(&svg, SHADOW_DY_LABEL_X, SHADOW_DY_TRACK_Y + SHADOW_DY_TRACK_LENGTH - 2.0, "10")?;

    // Three tick marks mark the min, middle, and max values, proportioned along the track the same way
    // build_h_slider's own tick marks are, not evenly spaced in pixels.
    for tick_percent in [0.0, 50.0, 100.0_f64] {
        let fraction = tick_percent / 100.0;
        let y = SHADOW_DY_TRACK_Y + fraction * SHADOW_DY_TRACK_LENGTH;
        let mark = svg.line(
            Point::new(SHADOW_DY_TRACK_X - 3.0, y),
            Point::new(SHADOW_DY_TRACK_X + SHADOW_DY_TRACK_W + 3.0, y),
        )?;
        mark.set_stroke(GUIDE)?;
        mark.set_stroke_width(1.0)?;
    }

    let dy_slider = super::build_v_slider(
        &svg,
        Point::new(SHADOW_DY_TRACK_X, SHADOW_DY_TRACK_Y),
        SHADOW_DY_TRACK_W,
        SHADOW_DY_TRACK_LENGTH,
        "drop shadow dy offset",
        (MIN_OFFSET, MAX_OFFSET, DEFAULT_DY),
    )?;
    dy_slider
        .set_attribute("aria-valuetext", &format!("dy {DEFAULT_DY}"))
        .map_err(dom_err)?;

    {
        let dx = dx_slider.clone();
        let dy = dy_slider.clone();
        let stddev = stddev_slider.clone();
        let shadow = shadow.clone();
        let caption = shadow_caption.clone();
        let on_input: DemoClosure = Closure::new(move |_: web_sys::Event| {
            let value = dx.value_as_number();
            let _ = shadow.set_attribute("dx", &format!("{value:.0}"));
            let _ = dx.set_attribute("aria-valuetext", &format!("dx {value:.0}"));
            caption.set_text(&shadow_caption_text(value, dy.value_as_number(), stddev.value_as_number()));
        });
        dx_slider
            .add_event_listener_with_callback("input", on_input.as_ref().unchecked_ref())
            .map_err(dom_err)?;
        keep_demo_closure(on_input);
    }
    {
        let dx = dx_slider.clone();
        let dy = dy_slider.clone();
        let stddev = stddev_slider.clone();
        let shadow = shadow.clone();
        let caption = shadow_caption.clone();
        let on_input: DemoClosure = Closure::new(move |_: web_sys::Event| {
            let value = dy.value_as_number();
            let _ = shadow.set_attribute("dy", &format!("{value:.0}"));
            let _ = dy.set_attribute("aria-valuetext", &format!("dy {value:.0}"));
            caption.set_text(&shadow_caption_text(dx.value_as_number(), value, stddev.value_as_number()));
        });
        dy_slider
            .add_event_listener_with_callback("input", on_input.as_ref().unchecked_ref())
            .map_err(dom_err)?;
        keep_demo_closure(on_input);
    }
    {
        let dx = dx_slider.clone();
        let dy = dy_slider.clone();
        let stddev = stddev_slider.clone();
        let shadow = shadow.clone();
        let caption = shadow_caption.clone();
        let on_input: DemoClosure = Closure::new(move |_: web_sys::Event| {
            let value = stddev.value_as_number();
            let _ = shadow.set_attribute("stdDeviation", &format!("{value:.0}"));
            let _ = stddev.set_attribute("aria-valuetext", &format!("stdDeviation {value:.0}"));
            caption.set_text(&shadow_caption_text(dx.value_as_number(), dy.value_as_number(), value));
        });
        stddev_slider
            .add_event_listener_with_callback("input", on_input.as_ref().unchecked_ref())
            .map_err(dom_err)?;
        keep_demo_closure(on_input);
    }

    Ok(())
}
