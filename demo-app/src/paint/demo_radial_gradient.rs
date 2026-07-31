use crate::{
    DemoClosure, W,
    colours::*,
    dom_err,
    foreign_html::{foreign_object_document, radio_group},
    keep_demo_closure, keep_demo_node,
};
use svg_dom::{
    Error, SvgRoot,
    root::{
        gradient::SpreadMethod,
        utils::{Point, Size},
    },
};
use wasm_bindgen::{JsCast, prelude::*};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// radialGradient — centred, spreadMethod, off-centre focal, and ellipse fill, laid out two rows deep
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Row 1 is centred (a slider shifts the middle stop along the radius) and spreadMethod (radio buttons choose
// Pad/Reflect/Repeat). Row 2 is off-centre focal (a pair of sliders move fx/fy) and the ellipse metallic sheen,
// unchanged from before beyond its new position.
//
// `row_caption`, `select_el`, `build_h_slider`, and `build_v_slider` (called below via `super::`) live in
// `paint/mod.rs`, shared with `demo_linear_gradient` — the other demo whose own controls need this same custom
// row layout, rather than the shared W/H/PAD_Y/BAND/caption convention every other demo uses. See their own doc
// comments there for what each one does and why.
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

const CIRCLE_R: f64 = 52.0;
const FOCAL_RECT_W: f64 = 155.0;
const FOCAL_RECT_H: f64 = 110.0;
const ELLIPSE_RX: f64 = 130.0;
const ELLIPSE_RY: f64 = 52.0;

const LEFT_MARGIN: f64 = 40.0; // every row starts here, rather than being centred in the canvas
const ITEM_GAP: f64 = 70.0; // horizontal gap between the two items sharing a row

const SLIDER_ROW_H: f64 = 62.0; // one HTML slider block: label + track + tick row + endpoint-value row
const CONTROL_GAP: f64 = 6.0; // gap between a row's control zone and the shape below it
const CAPTION_OFFSET: f64 = 14.0; // caption sits this far below its shape's bottom edge
const ROW_GAP: f64 = 26.0; // gap from one row's caption down to the next row's top

const ROW1_TOP: f64 = 10.0;

// centred: a slider sits above its circle, the same way every other single-slider demo in this file's sibling
// (demo_linear_gradient) reserves its own control zone.
const ROW1_RECT_Y: f64 = ROW1_TOP + SLIDER_ROW_H + CONTROL_GAP;
const ROW1_CY: f64 = ROW1_RECT_Y + CIRCLE_R;
const ROW1_CAPTION_Y: f64 = ROW1_RECT_Y + CIRCLE_R * 2.0 + CAPTION_OFFSET;
const ROW1_SLIDER_Y: f64 = ROW1_TOP;

const ROW1_A_X: f64 = LEFT_MARGIN + (FOCAL_RECT_W / 2.0) - CIRCLE_R;
const ROW1_B_X: f64 = ROW2_B_X + ELLIPSE_RX - CIRCLE_R;

// spreadMethod: its radio group sits beside its circle, not above it, so nothing reserves any space above the
// circle itself. Its own caption still lines up with centred's, sharing ROW1_CAPTION_Y above, so the row's
// caption line stays level even though the two circles themselves do not.
const SPREAD_RADIO_GAP: f64 = 16.0; // gap between the spreadMethod circle and its radio group
const SPREAD_RADIO_W: f64 = 110.0; // wide enough for the "reflect" row, its longest option
const SPREAD_RADIO_X: f64 = ROW1_B_X + CIRCLE_R * 2.0 + SPREAD_RADIO_GAP;

const ROW2_TOP: f64 = ROW1_CAPTION_Y + ROW_GAP;
const ROW2_RECT_Y: f64 = ROW2_TOP + SLIDER_ROW_H + CONTROL_GAP;
const ROW2_CAPTION_Y: f64 = ROW2_RECT_Y + FOCAL_RECT_H + CAPTION_OFFSET;

const ROW2_A_X: f64 = LEFT_MARGIN;
const V_TRACK_GAP: f64 = 16.0; // gap between the focal rect and its fy slider track
const V_TRACK_W: f64 = 16.0; // the rotated fy slider's own (post-rotation) thickness
const SIDE_LABEL_GAP: f64 = 46.0; // room the fy track's own side labels (e.g. "100%") need to its right
const ROW2_B_X: f64 = ROW2_A_X + FOCAL_RECT_W + V_TRACK_GAP + V_TRACK_W + SIDE_LABEL_GAP + ITEM_GAP;

const RG_H: f64 = ROW2_CAPTION_Y + 12.0;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-radial-gradient", Size::new(W, RG_H))?;

    let defs = svg.build_defs(|d| {
        // 1. Centred glow: white hot core fading to deep transparent blue (default cx/cy/r = 0.5/0.5/0.5).
        d.build_radial_gradient("demo-rg-c", |g| {
            g.add_stop_opacity(0.0, "white", 1.0)?;
            g.add_stop_opacity(0.5, STEELBLUE, 0.9)?;
            g.add_stop_opacity(1.0, NIGHT_SKY_BLUE, 1.0)?;
            Ok(())
        })?;

        // 2. spreadMethod, defaulting to reflect, with a tight radius (r=0.25) so the pattern tiles visibly.
        //    Two contrasting stops create concentric rings.
        d.build_radial_gradient("demo-rg-r", |g| {
            g.add_stop(0.0, LIGHT_SKY_BLUE)?;
            g.add_stop(1.0, "#00264d")?;
            g.set_r(0.25)?;
            g.set_spread_method(SpreadMethod::Reflect)?;
            Ok(())
        })?;

        // 3. Off-centre focal point: the first-stop colour (gold) originates from the upper-left
        //    while the outer colour (deep red) fills the rest.
        d.build_radial_gradient("demo-rg-f", |g| {
            g.add_stop(0.0, GOLDENROD)?;
            g.add_stop(1.0, "#7b0000")?;
            g.set_fx(0.25)?;
            g.set_fy(0.25)?;
            Ok(())
        })?;

        // 4. Ellipse with a compressed-Y radial (gradient in objectBoundingBox so it follows the
        //    ellipse shape automatically).  Three stops give a metallic sheen.
        d.build_radial_gradient("demo-rg-e", |g| {
            g.add_stop(0.0, "white")?;
            g.add_stop(0.4, MEDIUM_SEA_GREEN)?;
            g.add_stop(1.0, "#003d1f")?;
            Ok(())
        })?;

        Ok(())
    })?;

    // `defs` is used only to hold the gradients here.
    // Binding it to `_` below tells the compiler this is intentional.
    let _ = defs;

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // Row 1 — centred (slider moves the middle stop along the radius) and spreadMethod (radio buttons choose
    // Pad/Reflect/Repeat)
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    let c1 = svg.circle(Point::new(ROW1_A_X + CIRCLE_R, ROW1_CY), CIRCLE_R)?;
    c1.set_fill_gradient("demo-rg-c")?;
    super::row_caption(&svg, ROW1_A_X + CIRCLE_R, ROW1_CAPTION_Y, "centred")?;

    let c_stop = super::select_el(&svg, "#demo-rg-c stop:nth-child(2)")?;
    let c_slider = super::build_h_slider(
        &svg,
        Point::new(ROW1_A_X, ROW1_SLIDER_Y),
        Size::new(CIRCLE_R * 2.0, SLIDER_ROW_H),
        ("shift stop 2", "centred gradient stop 2"),
        (1, 99, 50),
        25,
        ("1%", "99%"),
    )?
    .input;
    // Matches the slider's own default value (50) above, set once here so a screen reader announces the real
    // starting value, not "no value", before any interaction happens.
    c_slider.set_attribute("aria-valuetext", "50%").map_err(dom_err)?;
    {
        let slider = c_slider.clone();
        let stop = c_stop.clone();
        let on_input: DemoClosure = Closure::new(move |_: web_sys::Event| {
            let percent = slider.value_as_number();
            let _ = stop.set_attribute("offset", &format!("{:.2}", percent / 100.0));
            let _ = slider.set_attribute("aria-valuetext", &format!("{percent:.0}%"));
        });
        c_slider
            .add_event_listener_with_callback("input", on_input.as_ref().unchecked_ref())
            .map_err(dom_err)?;
        keep_demo_closure(on_input);
    }

    let c2 = svg.circle(Point::new(ROW1_B_X + CIRCLE_R, ROW1_CY), CIRCLE_R)?;
    c2.set_fill_gradient("demo-rg-r")?;
    super::row_caption(&svg, ROW1_B_X + CIRCLE_R, ROW1_CAPTION_Y, "spreadMethod")?;

    // radio_group builds native HTML radio buttons, not hand-drawn SVG ones, for the same reason
    // `texts::demo_text`'s text-anchor/dominant-baseline demos do: screen-reader semantics, keyboard navigation,
    // and browser-enforced mutual exclusivity via the shared `name` attribute, all for free.
    // It sits beside the circle, not above it, so its own foreignObject spans the same height as the circle.
    let r_gradient = super::select_el(&svg, "#demo-rg-r")?;
    let spread_fo = svg.foreign_object(
        Point::new(SPREAD_RADIO_X, ROW1_CY - CIRCLE_R),
        Size::new(SPREAD_RADIO_W, CIRCLE_R * 2.0),
    )?;
    let spread_document = foreign_object_document(&spread_fo)?;

    const SPREAD_OPTIONS: [(SpreadMethod, &str); 3] = [
        (SpreadMethod::Pad, "pad"),
        (SpreadMethod::Reflect, "reflect"),
        (SpreadMethod::Repeat, "repeat"),
    ];

    let gradient = r_gradient.clone();
    let group = radio_group(
        &spread_document,
        "spreadMethod",
        "demo-rg-spread-method",
        &SPREAD_OPTIONS,
        SpreadMethod::Reflect,
        move |value| {
            let attr = match value {
                SpreadMethod::Pad => "pad",
                SpreadMethod::Reflect => "reflect",
                SpreadMethod::Repeat => "repeat",
            };
            let _ = gradient.set_attribute("spreadMethod", attr);
        },
    )?;
    spread_fo.as_element().append_child(&group).map_err(dom_err)?;
    keep_demo_node(spread_fo);

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // Row 2 — off-centre focal (a pair of sliders move fx/fy) and the ellipse metallic sheen, unchanged from
    // before beyond its new position
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    let rf = svg.rect(Point::new(ROW2_A_X, ROW2_RECT_Y), Size::new(FOCAL_RECT_W, FOCAL_RECT_H))?;
    rf.set_fill_gradient("demo-rg-f")?;
    super::row_caption(&svg, ROW2_A_X + FOCAL_RECT_W / 2.0, ROW2_CAPTION_Y, "off-centre focal")?;

    // A small hollow marker tracks fx/fy directly on the rectangle, at the same 0%-100% -> left/right or
    // top/bottom mapping the gradient's own objectBoundingBox coordinates use. Near the corners this marker
    // visibly sits outside #demo-rg-f's own end circle (its default cx/cy/r = 0.5/0.5/0.5) — SVG 2 accepts a
    // focal point outside the end circle, unlike SVG 1.1, which moved it back onto the circle instead. The
    // marker makes that relationship visible: it does not explain on its own why the gradient outside the cone
    // it forms then reads transparent black, which is why the panel's own figcaption spells that out too.
    let focal_marker = svg.circle(
        Point::new(ROW2_A_X + 0.25 * FOCAL_RECT_W, ROW2_RECT_Y + 0.25 * FOCAL_RECT_H),
        5.0,
    )?;
    focal_marker.set_fill(NONE)?;
    focal_marker.set_stroke(WHITE)?;
    focal_marker.set_stroke_width(2.0)?;
    focal_marker.set_attr("pointer-events", NONE)?;

    let f_gradient = super::select_el(&svg, "#demo-rg-f")?;
    let fx_slider = super::build_h_slider(
        &svg,
        Point::new(ROW2_A_X, ROW2_TOP),
        Size::new(FOCAL_RECT_W, SLIDER_ROW_H),
        ("shift fx", "off-centre focal gradient fx"),
        (0, 100, 25),
        25,
        ("0%", "100%"),
    )?
    .input;
    fx_slider.set_attribute("aria-valuetext", "25%").map_err(dom_err)?;
    {
        let slider = fx_slider.clone();
        let gradient = f_gradient.clone();
        let marker = focal_marker.clone();
        let on_input: DemoClosure = Closure::new(move |_: web_sys::Event| {
            let percent = slider.value_as_number();
            let _ = gradient.set_attribute("fx", &format!("{:.2}", percent / 100.0));
            let _ = slider.set_attribute("aria-valuetext", &format!("{percent:.0}%"));
            let cx = ROW2_A_X + percent / 100.0 * FOCAL_RECT_W;
            let _ = marker.set_attr("cx", &format!("{cx:.1}"));
        });
        fx_slider
            .add_event_listener_with_callback("input", on_input.as_ref().unchecked_ref())
            .map_err(dom_err)?;
        keep_demo_closure(on_input);
    }

    // fy: see `build_v_slider`'s own doc comment (paint/mod.rs) for the CSS rotation and keyboard remapping it
    // needs. Plain SVG text and line elements decorate it here, not HTML, for the same reason
    // `demo_linear_gradient`'s own vertical slider does.
    let fy_track_x = ROW2_A_X + FOCAL_RECT_W + V_TRACK_GAP;

    super::row_caption(&svg, fy_track_x + V_TRACK_W / 2.0, ROW2_TOP + 10.0, "shift fy")?;

    // Endpoint values: min (0%, the top of the object) sits at the top of the track, max (100%, the bottom of
    // the object) at the bottom — matching fy's own top-is-0 meaning, the same "up is smaller" spatial framing
    // `demo_linear_gradient`'s own vertical slider uses.
    super::side_label(&svg, fy_track_x + V_TRACK_W + 6.0, ROW2_RECT_Y + 4.0, "0%")?;
    super::side_label(&svg, fy_track_x + V_TRACK_W + 6.0, ROW2_RECT_Y + FOCAL_RECT_H - 2.0, "100%")?;

    // Three tick marks mark the min, middle, and max values, proportioned along [0, 100] the same way
    // build_h_slider's own tick marks are, not evenly spaced in pixels.
    for tick_percent in [0.0, 50.0, 100.0_f64] {
        let fraction = tick_percent / 100.0;
        let y = ROW2_RECT_Y + fraction * FOCAL_RECT_H;
        let mark = svg.line(Point::new(fy_track_x - 3.0, y), Point::new(fy_track_x + V_TRACK_W + 3.0, y))?;
        mark.set_stroke(GUIDE)?;
        mark.set_stroke_width(1.0)?;
    }

    let fy_slider = super::build_v_slider(
        &svg,
        Point::new(fy_track_x, ROW2_RECT_Y),
        V_TRACK_W,
        FOCAL_RECT_H,
        "off-centre focal gradient fy",
        (0, 100, 25),
    )?;
    fy_slider.set_attribute("aria-valuetext", "25%").map_err(dom_err)?;
    {
        let slider = fy_slider.clone();
        let gradient = f_gradient.clone();
        let marker = focal_marker.clone();
        let on_input: DemoClosure = Closure::new(move |_: web_sys::Event| {
            let percent = slider.value_as_number();
            let _ = gradient.set_attribute("fy", &format!("{:.2}", percent / 100.0));
            let _ = slider.set_attribute("aria-valuetext", &format!("{percent:.0}%"));
            let cy = ROW2_RECT_Y + percent / 100.0 * FOCAL_RECT_H;
            let _ = marker.set_attr("cy", &format!("{cy:.1}"));
        });
        fy_slider
            .add_event_listener_with_callback("input", on_input.as_ref().unchecked_ref())
            .map_err(dom_err)?;
        keep_demo_closure(on_input);
    }

    // Ellipse with a compressed-Y radial (gradient in objectBoundingBox so it follows the ellipse shape
    // automatically). Unchanged from before this demo's row split, beyond its new position.
    let e4 = svg.ellipse(
        Point::new(ROW2_B_X + ELLIPSE_RX, ROW2_RECT_Y + ELLIPSE_RY),
        Size::new(ELLIPSE_RX, ELLIPSE_RY),
    )?;
    e4.set_fill_gradient("demo-rg-e")?;
    super::row_caption(&svg, ROW2_B_X + ELLIPSE_RX, ROW2_CAPTION_Y, "ellipse metallic sheen")?;

    Ok(())
}
