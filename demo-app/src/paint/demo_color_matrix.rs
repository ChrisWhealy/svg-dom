use crate::{
    DemoClosure, W,
    colours::*,
    dom_err,
    foreign_html::{foreign_object_document, radio_group},
    keep_demo_closure, keep_demo_node,
};
use svg_dom::{
    Error, SvgRoot, TextAnchor,
    root::{
        filter::ColorMatrixType,
        utils::{Point, Size},
    },
};
use wasm_bindgen::{JsCast, prelude::*};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// feColorMatrix
// One gradient source feeds three filters.
// Saturate and HueRotate each have their own live slider.
// The fourth rectangle's own filter switches between Matrix (a fixed sepia tone) and LuminanceToAlpha, via a
// radio group.
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// `color_matrix` returns the `<feColorMatrix>` primitive's own `SvgNode` directly, unlike `component_transfer`.
// So each filter here retains its own node, the same retained-primitive pattern `demo_filter`'s sliders use.
//
// `Matrix` and `LuminanceToAlpha` need different attributes.
// `Matrix` needs its own `values` attribute: twenty space-separated numbers.
// `LuminanceToAlpha` needs no `values` attribute at all.
// So the radio group's own `on_select` sets or removes `values` to match, alongside `type`.
// `ColorMatrixType::as_str` supplies each `type` keyword directly, the same way `demo_blend` already uses
// `BlendMode::as_str`.
// `ColorMatrixType` itself has no public accessor for `Matrix`'s own formatted `values` text.
// `sepia_values_attr` below supplies that instead, once, so the toggle can restore it after `LuminanceToAlpha`
// clears it.
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

const RECT_W: f64 = 160.0;
const RECT_H: f64 = 100.0;
const LEFT_MARGIN: f64 = 20.0;
const ITEM_GAP: f64 = 30.0; // gap between rectangles

const RECT2_X: f64 = LEFT_MARGIN + RECT_W + ITEM_GAP;
const RECT3_X: f64 = RECT2_X + RECT_W + ITEM_GAP;
const RECT4_X: f64 = RECT3_X + RECT_W + ITEM_GAP;

const CONTROL_ROW_H: f64 = 62.0; // one control block: label plus either a slider or a two-row radio group
const CONTROL_GAP: f64 = 6.0; // gap between a control and the rectangle it drives
const CAPTION_OFFSET: f64 = 14.0; // caption sits this far below its rectangle's own bottom edge

const TOP: f64 = 10.0;

// All four rectangles share the same row, even though only three of them have a control above them.
// `demo_blend`'s own original/blended pair, and `demo_component_transfer`'s own row, both use this same layout:
// one shared row, any unused control zones left empty.
// Every caption below still lines up.
const RECT_Y: f64 = TOP + CONTROL_ROW_H + CONTROL_GAP;
const CAPTION_Y: f64 = RECT_Y + RECT_H + CAPTION_OFFSET;

const COLOR_MATRIX_H: f64 = CAPTION_Y + 12.0;

const MIN_SATURATE_PERCENT: i32 = 0;
const MAX_SATURATE_PERCENT: i32 = 100;
const DEFAULT_SATURATE_PERCENT: i32 = 0; // fully desaturated (greyscale), this demo's own original default
const DEFAULT_SATURATE: f64 = DEFAULT_SATURATE_PERCENT as f64 / 100.0;

const MIN_HUE_DEGREES: i32 = 0;
const MAX_HUE_DEGREES: i32 = 360;
const DEFAULT_HUE_DEGREES: i32 = 180; // this demo's own original default

// Classic "sepia tone" colour matrix: well-known fixed coefficients, not derived from anything else in this
// crate.
// Included to demonstrate the fully custom `Matrix` variant, alongside the two single-value ones above.
#[rustfmt::skip]
const SEPIA_MATRIX: [f64; 20] = [
    0.393, 0.769, 0.189, 0.0, 0.0,
    0.349, 0.686, 0.168, 0.0, 0.0,
    0.272, 0.534, 0.131, 0.0, 0.0,
    0.0,   0.0,   0.0,   1.0, 0.0,
];

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Formats `SEPIA_MATRIX`'s own values as the space-separated string the `values` attribute needs.
/// This uses the same plain `Display` formatting `color_matrix`'s own construction-time path uses internally.
/// So this attribute's own text never shifts format between construction and a later toggle back to `Matrix`.
fn sepia_values_attr() -> String {
    SEPIA_MATRIX.iter().map(f64::to_string).collect::<Vec<_>>().join(" ")
}

/// Formats a Saturate value at the slider's own real precision.
/// The slider moves in 1% steps, so its own real value moves in steps of 0.01, not 0.1.
/// A fixed one-decimal format would collapse those 101 distinct positions down to just 11 displayed values.
/// This formats to two decimal places instead, then strips a single trailing zero.
/// That keeps the full precision an intermediate position like `0.25` needs, while still printing an endpoint
/// like `0.0` or `1.0`, not `0.00`/`1.00`.
fn saturate_value_text(value: f64) -> String {
    let formatted = format!("{value:.2}");
    formatted.strip_suffix('0').unwrap_or(&formatted).to_owned()
}

fn saturate_caption_text(value: f64) -> String {
    format!("Saturate({})", saturate_value_text(value))
}

fn hue_caption_text(degrees: f64) -> String {
    format!("HueRotate({degrees:.0})")
}

fn matrix_caption_text(variant: MatrixVariant) -> &'static str {
    match variant {
        MatrixVariant::Sepia => "Matrix (sepia)",
        MatrixVariant::LuminanceToAlpha => "LuminanceToAlpha",
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// The two `ColorMatrixType` variants the fourth rectangle's own radio group switches between.
/// A local, `Copy` stand-in for `ColorMatrixType` itself: `radio_group` requires `T: Copy`, and
/// `ColorMatrixType` deliberately does not implement `Copy` (see its own doc comment).
/// `Matrix`'s own 160 bytes of coefficients are exactly why not.
#[derive(Clone, Copy, PartialEq, Eq)]
enum MatrixVariant {
    Sepia,
    LuminanceToAlpha,
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-color-matrix", Size::new(W, COLOR_MATRIX_H))?;

    let defs = svg.defs()?;

    // A multi-hue gradient source: a single flat colour would not show saturation or hue-rotation changing
    // anything.
    // Both operate on hue and chroma, which a flat fill barely has.
    defs.build_linear_gradient("cm-source", |g| {
        g.add_stop(0.0, STEELBLUE)?;
        g.add_stop(0.5, GOLD)?;
        g.add_stop(1.0, CRIMSON)?;
        Ok(())
    })?;

    // feColorMatrix transforms colour in place.
    // Unlike feGaussianBlur/feOffset, it never spreads pixels beyond the source's own bounding box.
    // So, unlike the demo_filter panel, no filter region needs widening here.
    let r1 = svg.rect(Point::new(LEFT_MARGIN, RECT_Y), Size::new(RECT_W, RECT_H))?;
    r1.set_fill_gradient("cm-source")?;
    super::row_caption(&svg, LEFT_MARGIN + RECT_W / 2.0, CAPTION_Y, "original")?;

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // Saturate
    // A slider above the rectangle drives feColorMatrix's own single Saturate value live.
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    let r2 = svg.rect(Point::new(RECT2_X, RECT_Y), Size::new(RECT_W, RECT_H))?;
    r2.set_fill_gradient("cm-source")?;
    r2.set_filter("cm-filter-saturate")?;

    let saturate_caption = svg.text(
        Point::new(RECT2_X + RECT_W / 2.0, CAPTION_Y),
        &saturate_caption_text(DEFAULT_SATURATE),
    )?;
    saturate_caption.set_fill(CAPTION)?;
    saturate_caption.set_attr("font-size", "11")?;
    saturate_caption.set_text_anchor(TextAnchor::Middle)?;

    let saturate_filter = defs.filter("cm-filter-saturate")?;
    let saturate_matrix = saturate_filter.color_matrix(ColorMatrixType::Saturate(DEFAULT_SATURATE))?;

    let saturate_slider = super::build_h_slider(
        &svg,
        Point::new(RECT2_X, TOP),
        Size::new(RECT_W, CONTROL_ROW_H),
        ("saturate", "colour matrix saturate"),
        (MIN_SATURATE_PERCENT, MAX_SATURATE_PERCENT, DEFAULT_SATURATE_PERCENT),
        25,
        ("0%", "100%"),
    )?
    .input;
    saturate_slider
        .set_attribute("aria-valuetext", &saturate_value_text(DEFAULT_SATURATE))
        .map_err(dom_err)?;
    {
        let slider = saturate_slider.clone();
        let matrix = saturate_matrix.clone();
        let caption = saturate_caption.clone();
        let on_input: DemoClosure = Closure::new(move |_: web_sys::Event| {
            let value = slider.value_as_number() / 100.0;
            let _ = matrix.set_attr("values", &value.to_string());
            let _ = slider.set_attribute("aria-valuetext", &saturate_value_text(value));
            caption.set_text(&saturate_caption_text(value));
        });
        saturate_slider
            .add_event_listener_with_callback("input", on_input.as_ref().unchecked_ref())
            .map_err(dom_err)?;
        keep_demo_closure(on_input);
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // HueRotate
    // A slider above the rectangle drives feColorMatrix's own single HueRotate value live.
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    let r3 = svg.rect(Point::new(RECT3_X, RECT_Y), Size::new(RECT_W, RECT_H))?;
    r3.set_fill_gradient("cm-source")?;
    r3.set_filter("cm-filter-hue")?;

    let hue_caption = svg.text(
        Point::new(RECT3_X + RECT_W / 2.0, CAPTION_Y),
        &hue_caption_text(f64::from(DEFAULT_HUE_DEGREES)),
    )?;
    hue_caption.set_fill(CAPTION)?;
    hue_caption.set_attr("font-size", "11")?;
    hue_caption.set_text_anchor(TextAnchor::Middle)?;

    let hue_filter = defs.filter("cm-filter-hue")?;
    let hue_matrix = hue_filter.color_matrix(ColorMatrixType::HueRotate(f64::from(DEFAULT_HUE_DEGREES)))?;

    let hue_slider = super::build_h_slider(
        &svg,
        Point::new(RECT3_X, TOP),
        Size::new(RECT_W, CONTROL_ROW_H),
        ("hue rotate", "colour matrix hue rotate"),
        (MIN_HUE_DEGREES, MAX_HUE_DEGREES, DEFAULT_HUE_DEGREES),
        90,
        ("0°", "360°"),
    )?
    .input;
    {
        let slider = hue_slider.clone();
        let matrix = hue_matrix.clone();
        let caption = hue_caption.clone();
        let on_input: DemoClosure = Closure::new(move |_: web_sys::Event| {
            let degrees = slider.value_as_number();
            let _ = matrix.set_attr("values", &degrees.to_string());
            caption.set_text(&hue_caption_text(degrees));
        });
        hue_slider
            .add_event_listener_with_callback("input", on_input.as_ref().unchecked_ref())
            .map_err(dom_err)?;
        keep_demo_closure(on_input);
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // Matrix / LuminanceToAlpha
    // A radio group above the rectangle switches feColorMatrix's own type live, between a fixed sepia Matrix and
    // LuminanceToAlpha.
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // LuminanceToAlpha zeroes red, green, and blue. Alpha is the only signal left. `Matrix`'s own sepia result
    // stays fully opaque, so it always hides this backing rectangle completely.
    // LuminanceToAlpha's own result does not: without this backing rectangle, its near-transparent, near-black pixels
    // would blend into this gallery's own near-black canvas background and the luminance signal this filter wants to
    // demonstrate would be almost invisible.
    // Therefore, a plain white backing rectangle, drawn once here and left untouched by the radio group, makes that
    // signal read as a visible greyscale gradient instead.
    let r4_backing = svg.rect(Point::new(RECT4_X, RECT_Y), Size::new(RECT_W, RECT_H))?;
    r4_backing.set_fill(WHITE)?;

    let r4 = svg.rect(Point::new(RECT4_X, RECT_Y), Size::new(RECT_W, RECT_H))?;
    r4.set_fill_gradient("cm-source")?;
    r4.set_filter("cm-filter-matrix")?;

    let matrix_caption = svg.text(
        Point::new(RECT4_X + RECT_W / 2.0, CAPTION_Y),
        matrix_caption_text(MatrixVariant::Sepia),
    )?;
    matrix_caption.set_fill(CAPTION)?;
    matrix_caption.set_attr("font-size", "11")?;
    matrix_caption.set_text_anchor(TextAnchor::Middle)?;

    let matrix_filter = defs.filter("cm-filter-matrix")?;
    let matrix_node = matrix_filter.color_matrix(ColorMatrixType::Matrix(SEPIA_MATRIX))?;

    // radio_group builds native HTML radio buttons, not a slider: neither option here has a numeric range for a
    // slider to sweep, so this reuses the same shared control `demo_text`'s own text-anchor demo uses, rather
    // than `select_dropdown`, which exists for a long option list this two-option choice does not have.
    let matrix_fo = svg.foreign_object(Point::new(RECT4_X, TOP), Size::new(RECT_W, CONTROL_ROW_H))?;
    let matrix_document = foreign_object_document(&matrix_fo)?;

    const MATRIX_OPTIONS: [(MatrixVariant, &str); 2] =
        [(MatrixVariant::Sepia, "sepia"), (MatrixVariant::LuminanceToAlpha, "luminance")];

    let matrix = matrix_node.clone();
    let caption = matrix_caption.clone();
    let group = radio_group(
        &matrix_document,
        "matrix type",
        "demo-color-matrix-variant",
        &MATRIX_OPTIONS,
        MatrixVariant::Sepia,
        move |variant: MatrixVariant| {
            match variant {
                MatrixVariant::Sepia => {
                    let _ = matrix.set_attr("type", ColorMatrixType::Matrix(SEPIA_MATRIX).as_str());
                    let _ = matrix.set_attr("values", &sepia_values_attr());
                },
                MatrixVariant::LuminanceToAlpha => {
                    let _ = matrix.set_attr("type", ColorMatrixType::LuminanceToAlpha.as_str());
                    let _ = matrix.remove_attr("values");
                },
            }
            caption.set_text(matrix_caption_text(variant));
        },
    )?;
    matrix_fo.as_element().append_child(&group).map_err(dom_err)?;
    keep_demo_node(matrix_fo);

    Ok(())
}
