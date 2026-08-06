use crate::{DemoClosure, colours::*, dom_err, keep_demo_closure};
use svg_dom::{
    Error, SvgRoot, TextAnchor,
    root::{
        filter::{CompositeOperator, LightSource},
        utils::{Point, Size},
    },
};
use wasm_bindgen::{JsCast, prelude::*};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// feDiffuseLighting / feSpecularLighting — matte + shiny bump-mapped lighting, and the combined bevel recipe
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// `diffuse_lighting_with_light`/`specular_lighting_with_light` each return a `LightingNodes`: the primitive's own
// `SvgNode`, plus a second `SvgNode` for its required light-source child. So every lit column here retains both,
// the same retained-primitive pattern `demo_morphology`'s own Erode/Dilate sliders use for their own single node.
// This file used to build each filter through `build_defs`/`build_filter`'s own closures instead. That closure
// form returns no handle to any primitive built inside it. So it could not support a live slider, and this file
// now builds each filter directly through `defs.filter(id)` instead, matching `demo_morphology`'s own
// construction style.
//
// Two shared sliders drive every lit column at once: `surfaceScale` and `azimuth`.
// `demo_morphology`'s own radius slider drives its Erode, Dilate, and outline columns together the same way.
//
// `surfaceScale` is set directly on each primitive's own retained node.
// `azimuth` is set on its own retained light-source node instead — the plain `diffuse_lighting`/`specular_lighting`
// constructors append their own `<feDistantLight>` child internally, through `append_light_source`, and return no
// handle to it at all. `diffuse_lighting_with_light`/`specular_lighting_with_light` exist for exactly this case:
// an interactive application that needs to reach that child again, without falling back to a raw CSS-selector
// query outside `svg-dom`'s own typed API — the same escape hatch `demo_component_transfer`'s own
// `<feFuncR>`/`<feFuncG>`/`<feFuncB>`/`<feFuncA>` children still need, since `component_transfer` builds those
// internally with no `_with_light`-style alternative of its own.
//
// The combined "bevel-highlight" filter holds two lighting primitives, one diffuse and one specular, each with
// its own light source child. Each `_with_light` call keeps its own light-source node paired with its own
// primitive node directly, so the two never need disambiguating by tag name or selector the way a CSS query
// would.
//
// `elevation` stays fixed at its own original value throughout.
// Only `surfaceScale` and `azimuth` are interactive here.
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

const W: f64 = 780.0;

const RECT_W: f64 = 160.0; // column width, used only to centre each column's own text and caption
const LEFT_MARGIN: f64 = 20.0;
const ITEM_GAP: f64 = 30.0; // gap between columns

const COL1_X: f64 = LEFT_MARGIN; // original
const COL2_X: f64 = COL1_X + RECT_W + ITEM_GAP; // feDiffuseLighting
const COL3_X: f64 = COL2_X + RECT_W + ITEM_GAP; // feSpecularLighting
const COL4_X: f64 = COL3_X + RECT_W + ITEM_GAP; // combined bevel

const CONTROL_ROW_H: f64 = 62.0; // one HTML slider block: label + track + tick row + endpoint-value row
const CONTROL_GAP: f64 = 6.0; // gap between a slider block and the text it drives
const STACK_GAP: f64 = 4.0; // gap between the stacked surfaceScale/azimuth sliders
const SHAPE_ZONE_H: f64 = 60.0; // tall enough for one line of 34px bold text
const CAPTION_OFFSET: f64 = 14.0; // caption sits this far below the shape zone's own bottom edge
const VALUES_ROW_H: f64 = 14.0; // one small text line: the shared "surfaceScale N · azimuth N°" readout

const TOP: f64 = 10.0;
const SCALE_SLIDER_Y: f64 = TOP;
const AZIMUTH_SLIDER_Y: f64 = TOP + CONTROL_ROW_H + STACK_GAP;
const LIGHTING_CONTROL_H: f64 = CONTROL_ROW_H * 2.0 + STACK_GAP;

// All four columns share the same row, even though only three of them (diffuse, specular, bevel) have sliders
// above them. `demo_color_matrix`'s own row uses this same "shared row, unused control zone" layout.
const VALUES_Y: f64 = TOP + LIGHTING_CONTROL_H + CONTROL_GAP + VALUES_ROW_H;
const TEXT_ZONE_Y: f64 = VALUES_Y + CONTROL_GAP;
const TEXT_BASELINE_Y: f64 = TEXT_ZONE_Y + SHAPE_ZONE_H / 2.0 + 12.0; // vertically centres the 34px text
const CAPTION_Y: f64 = TEXT_ZONE_Y + SHAPE_ZONE_H + CAPTION_OFFSET;

const LIGHTING_H: f64 = CAPTION_Y + 12.0;

const MIN_SCALE: i32 = 0;
const MAX_SCALE: i32 = 20;
const DEFAULT_SCALE: i32 = 6; // this demo's own original fixed surfaceScale

const MIN_AZIMUTH: i32 = 0;
const MAX_AZIMUTH: i32 = 360;
const DEFAULT_AZIMUTH: i32 = 235; // this demo's own original fixed azimuth
const ELEVATION: f64 = 55.0; // fixed throughout, not part of this demo's own interactive controls

const DIFFUSE_CONSTANT: f64 = 1.0;
const SPECULAR_CONSTANT: f64 = 1.0;
const SPECULAR_EXPONENT: f64 = 20.0;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Formats a `surfaceScale` value via plain `Display`.
/// Every slider position here is a whole number, so this never needs a decimal point.
/// Using this same formatting for the attribute, the readout caption, and `aria-valuetext` keeps all three in
/// agreement.
fn scale_value_text(scale: f64) -> String {
    scale.to_string()
}

/// Formats an `azimuth` value with its own unit spelled out.
/// The readout caption and endpoint labels already show degrees.
/// The slider's own raw `value` does not carry that unit on its own.
/// This supplies `aria-valuetext`, so a screen reader announces "235 degrees", not the bare number 235.
fn azimuth_value_text(azimuth: f64) -> String {
    format!("{azimuth:.0} degrees")
}

fn lighting_values_text(scale: f64, azimuth: f64) -> String {
    format!("surfaceScale {} · azimuth {:.0}°", scale_value_text(scale), azimuth)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-lighting", Size::new(W, LIGHTING_H))?;
    let defs = svg.defs()?;

    let default_scale = f64::from(DEFAULT_SCALE);
    let default_azimuth = f64::from(DEFAULT_AZIMUTH);
    let light = LightSource::Distant {
        azimuth: default_azimuth,
        elevation: ELEVATION,
    };

    let original = svg.text(Point::new(COL1_X + RECT_W / 2.0, TEXT_BASELINE_Y), "LIGHT")?;
    original.set_fill(STEELBLUE)?;
    original.set_font_size(34.0)?;
    original.set_text_anchor(TextAnchor::Middle)?;
    original.set_attr("font-weight", "bold")?;
    super::row_caption(&svg, COL1_X + RECT_W / 2.0, CAPTION_Y, "original")?;

    // feDiffuseLighting alone: bump-maps SourceAlpha and lights it with a matte (Lambertian) model. The result
    // is fully opaque (A = 1.0 everywhere, per the SVG spec), so with no further compositing this renders as a
    // flat lit plate with the text embossed into it, not a transparent highlight over the canvas.
    let diffuse_filter = defs.filter("diffuse-only")?;
    let diffuse_only = diffuse_filter.diffuse_lighting_with_light(default_scale, DIFFUSE_CONSTANT, "white", light)?;
    diffuse_only.primitive.set_attr("in", "SourceAlpha")?;

    let diffuse = svg.text(Point::new(COL2_X + RECT_W / 2.0, TEXT_BASELINE_Y), "LIGHT")?;
    diffuse.set_fill(STEELBLUE)?;
    diffuse.set_font_size(34.0)?;
    diffuse.set_text_anchor(TextAnchor::Middle)?;
    diffuse.set_attr("font-weight", "bold")?;
    diffuse.set_filter("diffuse-only")?;
    super::row_caption(&svg, COL2_X + RECT_W / 2.0, CAPTION_Y, "feDiffuseLighting (opaque)")?;

    // feSpecularLighting alone: the shiny (Blinn-Phong) counterpart. Unlike feDiffuseLighting, its alpha is the
    // maximum of its own lit R/G/B, so it renders as a highlight-only glint against the dark canvas
    // background, transparent everywhere the highlight itself is zero.
    let specular_filter = defs.filter("specular-only")?;
    let specular_only = specular_filter.specular_lighting_with_light(
        default_scale,
        SPECULAR_CONSTANT,
        SPECULAR_EXPONENT,
        "white",
        light,
    )?;
    specular_only.primitive.set_attr("in", "SourceAlpha")?;

    let specular = svg.text(Point::new(COL3_X + RECT_W / 2.0, TEXT_BASELINE_Y), "LIGHT")?;
    specular.set_fill(STEELBLUE)?;
    specular.set_font_size(34.0)?;
    specular.set_text_anchor(TextAnchor::Middle)?;
    specular.set_attr("font-weight", "bold")?;
    specular.set_filter("specular-only")?;
    super::row_caption(&svg, COL3_X + RECT_W / 2.0, CAPTION_Y, "feSpecularLighting (highlight)")?;

    // The combined bevel recipe from SvgFilter::diffuse_lighting's/specular_lighting's own doc comments:
    // multiply the opaque diffuse-lit surface back over the original graphic (composite Arithmetic, k1: 1.0),
    // then add the non-opaque specular highlight on top (composite Arithmetic, k2/k3: 1.0). Using merge instead
    // of the first composite step would paint the opaque lit plate over everything, hiding the original
    // steelblue fill entirely.
    let bevel_filter = defs.filter("bevel-highlight")?;
    let bevel_diffuse = bevel_filter.diffuse_lighting_with_light(default_scale, DIFFUSE_CONSTANT, "white", light)?;
    bevel_diffuse.primitive.set_attrs([("in", "SourceAlpha"), ("result", "lit")])?;
    bevel_filter.composite("lit", CompositeOperator::Arithmetic)?.set_attrs([
        ("in", "SourceGraphic"),
        ("result", "beveled"),
        ("k1", "1"),
        ("k2", "0"),
        ("k3", "0"),
        ("k4", "0"),
    ])?;
    let bevel_specular = bevel_filter.specular_lighting_with_light(
        default_scale,
        SPECULAR_CONSTANT,
        SPECULAR_EXPONENT,
        "white",
        light,
    )?;
    bevel_specular
        .primitive
        .set_attrs([("in", "SourceAlpha"), ("result", "highlight")])?;
    bevel_filter.composite("highlight", CompositeOperator::Arithmetic)?.set_attrs([
        ("in", "beveled"),
        ("k1", "0"),
        ("k2", "1"),
        ("k3", "1"),
        ("k4", "0"),
    ])?;

    let beveled = svg.text(Point::new(COL4_X + RECT_W / 2.0, TEXT_BASELINE_Y), "LIGHT")?;
    beveled.set_fill(STEELBLUE)?;
    beveled.set_font_size(34.0)?;
    beveled.set_text_anchor(TextAnchor::Middle)?;
    beveled.set_attr("font-weight", "bold")?;
    beveled.set_filter("bevel-highlight")?;
    super::row_caption(&svg, COL4_X + RECT_W / 2.0, CAPTION_Y, "combined bevel")?;

    // The shared "surfaceScale N · azimuth N°" readout. Centred across the three lit columns it describes, since
    // no single column's own caption belongs to it more than the others.
    let values_cx = COL2_X + (COL4_X + RECT_W - COL2_X) / 2.0;
    let values_caption = svg.text(
        Point::new(values_cx, VALUES_Y),
        &lighting_values_text(default_scale, default_azimuth),
    )?;
    values_caption.set_fill(CAPTION)?;
    values_caption.set_attr("font-size", "11")?;
    values_caption.set_text_anchor(TextAnchor::Middle)?;

    let slider_w = COL4_X + RECT_W - COL2_X;

    let scale_slider = super::build_h_slider(
        &svg,
        Point::new(COL2_X, SCALE_SLIDER_Y),
        Size::new(slider_w, CONTROL_ROW_H),
        ("surface scale", "lighting surface scale"),
        (MIN_SCALE, MAX_SCALE, DEFAULT_SCALE),
        5, // a tick lands exactly on 0/5/10/15/20
        &["0", "20"],
    )?
    .input;
    scale_slider
        .set_attribute("aria-valuetext", &scale_value_text(default_scale))
        .map_err(dom_err)?;

    let azimuth_slider = super::build_h_slider(
        &svg,
        Point::new(COL2_X, AZIMUTH_SLIDER_Y),
        Size::new(slider_w, CONTROL_ROW_H),
        ("azimuth", "lighting azimuth"),
        (MIN_AZIMUTH, MAX_AZIMUTH, DEFAULT_AZIMUTH),
        90, // a tick lands exactly on 0°/90°/180°/270°/360°, the compass points a distant light's azimuth uses
        &["0°", "360°"],
    )?
    .input;
    azimuth_slider
        .set_attribute("aria-valuetext", &azimuth_value_text(default_azimuth))
        .map_err(dom_err)?;

    {
        let scale_input = scale_slider.clone();
        let azimuth_input = azimuth_slider.clone();
        let diffuse_only = diffuse_only.primitive.clone();
        let specular_only = specular_only.primitive.clone();
        let bevel_diffuse = bevel_diffuse.primitive.clone();
        let bevel_specular = bevel_specular.primitive.clone();
        let values_caption = values_caption.clone();
        let on_scale_input: DemoClosure = Closure::new(move |_: web_sys::Event| {
            let scale = scale_input.value_as_number();
            let text = scale_value_text(scale);
            let _ = diffuse_only.set_attr("surfaceScale", &text);
            let _ = specular_only.set_attr("surfaceScale", &text);
            let _ = bevel_diffuse.set_attr("surfaceScale", &text);
            let _ = bevel_specular.set_attr("surfaceScale", &text);
            let _ = scale_input.set_attribute("aria-valuetext", &text);
            values_caption.set_text(&lighting_values_text(scale, azimuth_input.value_as_number()));
        });
        scale_slider
            .add_event_listener_with_callback("input", on_scale_input.as_ref().unchecked_ref())
            .map_err(dom_err)?;
        keep_demo_closure(on_scale_input);
    }

    {
        let scale_input = scale_slider.clone();
        let azimuth_input = azimuth_slider.clone();
        let diffuse_only = diffuse_only.light.clone();
        let specular_only = specular_only.light.clone();
        let bevel_diffuse = bevel_diffuse.light.clone();
        let bevel_specular = bevel_specular.light.clone();
        let values_caption = values_caption.clone();
        let on_azimuth_input: DemoClosure = Closure::new(move |_: web_sys::Event| {
            let azimuth = azimuth_input.value_as_number();
            let text = azimuth.to_string();
            let _ = diffuse_only.set_attr("azimuth", &text);
            let _ = specular_only.set_attr("azimuth", &text);
            let _ = bevel_diffuse.set_attr("azimuth", &text);
            let _ = bevel_specular.set_attr("azimuth", &text);
            let _ = azimuth_input.set_attribute("aria-valuetext", &azimuth_value_text(azimuth));
            values_caption.set_text(&lighting_values_text(scale_input.value_as_number(), azimuth));
        });
        azimuth_slider
            .add_event_listener_with_callback("input", on_azimuth_input.as_ref().unchecked_ref())
            .map_err(dom_err)?;
        keep_demo_closure(on_azimuth_input);
    }

    Ok(())
}
