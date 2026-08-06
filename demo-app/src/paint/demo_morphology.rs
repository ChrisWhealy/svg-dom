use crate::{DemoClosure, W, colours::*, dom_err, keep_demo_closure};
use svg_dom::{
    Error, SvgRoot, TextAnchor,
    root::{
        filter::MorphologyOperator,
        utils::{Point, Size},
    },
};
use wasm_bindgen::{JsCast, prelude::*};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// feMorphology — erode/dilate a shape's silhouette, plus dilate + merge for a bold text outline
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// `morphology` returns the `<feMorphology>` primitive's own `SvgNode` directly.
// So both the Erode and Dilate filters here retain their own node.
// This is the same retained-primitive pattern `demo_turbulence`'s sliders use.
//
// A single shared slider drives both the Erode and Dilate radius live.
// `demo_turbulence`'s own baseFrequency slider drives both its noise rectangles the same way.
// Showing both operators side by side, at the same live radius, lets them be compared directly, not just
// switched between.
//
// `radius` is a plain `f64`, with no natural integer step of its own.
// So the slider drives an integer number of tenths, the same scaled-slider technique `demo_color_matrix`'s own
// Saturate slider uses.
// Every raw slider position is an exact multiple of `0.1`, so `radius_value_text` below is always lossless.
// `demo_turbulence`'s own `frequency_value_text` needs no more than plain `Display` formatting, for the same
// reason.
//
// At radius 0, `SvgFilter::morphology`'s own doc comment says the primitive disables itself entirely.
// `in` passes through unchanged.
// So at that end of the slider, the Erode and Dilate columns both show the same unfiltered text as the original
// column beside them.
// They do not show a maximally-thinned or maximally-thickened result.
// `demo_turbulence`'s own scale slider has this same property at `0`: its own minimum reproduces the identity
// case too.
//
// The same slider also drives the bold-outline column's own dilate radius.
// Only the original column stays fixed, as an unchanging reference point for the other three.
// The bold-outline column still demonstrates the dilate-then-merge compositing technique itself.
// Sharing the one slider does not change that: what varies is only how thick the resulting fringe is.
//
// Dilate's own filter region is widened, the same way `demo_filter` widens it for a wide blur.
// This slider's own maximum radius would otherwise clip the dilated glyphs against the SVG default region's own
// narrow 10% margin.
// Erode never expands past the source's own bounding box, so its own filter needs no such widening.
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

const RECT_W: f64 = 160.0; // column width, used only to centre each column's own text and caption
const LEFT_MARGIN: f64 = 20.0;
const ITEM_GAP: f64 = 30.0; // gap between columns

const COL1_X: f64 = LEFT_MARGIN; // original
const COL2_X: f64 = COL1_X + RECT_W + ITEM_GAP; // Erode
const COL3_X: f64 = COL2_X + RECT_W + ITEM_GAP; // Dilate
const COL4_X: f64 = COL3_X + RECT_W + ITEM_GAP; // bold outline

const CONTROL_ROW_H: f64 = 62.0; // one HTML slider block: label + track + tick row + endpoint-value row
const CONTROL_GAP: f64 = 6.0; // gap between the slider and the text it drives
const SHAPE_ZONE_H: f64 = 60.0; // tall enough for one line of 34px bold text
const CAPTION_OFFSET: f64 = 14.0; // caption sits this far below the shape zone's own bottom edge

const TOP: f64 = 10.0;

// All four columns share the same row, even though only two of them (Erode, Dilate) have a control above them.
// `demo_color_matrix`'s own row uses this same "shared row, unused control zone" layout.
const TEXT_ZONE_Y: f64 = TOP + CONTROL_ROW_H + CONTROL_GAP;
const TEXT_BASELINE_Y: f64 = TEXT_ZONE_Y + SHAPE_ZONE_H / 2.0 + 12.0; // vertically centres the 34px text
const CAPTION_Y: f64 = TEXT_ZONE_Y + SHAPE_ZONE_H + CAPTION_OFFSET;

const MORPHOLOGY_H: f64 = CAPTION_Y + 12.0;

const MIN_RADIUS_TENTHS: i32 = 0;
// 4.0 comfortably erases this font size's own thin bold strokes under Erode.
// It also fuses "MORPH"'s own letters into a near-solid shape under Dilate.
// Neither extreme needs an unreasonably long slider to reach.
const MAX_RADIUS_TENTHS: i32 = 40;
const DEFAULT_RADIUS_TENTHS: i32 = 12; // 1.2, this demo's own original fixed radius
const DEFAULT_RADIUS: f64 = DEFAULT_RADIUS_TENTHS as f64 / 10.0;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Formats a radius value via plain `Display`, the same formatting `morphology`'s own construction-time path
/// uses internally for the `radius` attribute itself.
/// Every raw slider position is an exact multiple of `0.1`, so this is always lossless.
/// No two distinct slider positions can ever format to the same text.
/// Using the same formatting for the caption and `aria-valuetext` as for the attribute itself keeps all three in
/// agreement.
/// Otherwise the attribute could show `1.2` while the caption shows a differently formatted `1.20`.
fn radius_value_text(radius: f64) -> String {
    radius.to_string()
}

fn erode_caption_text(radius: f64) -> String {
    format!("Erode({})", radius_value_text(radius))
}

fn dilate_caption_text(radius: f64) -> String {
    format!("Dilate({})", radius_value_text(radius))
}

fn outline_caption_text(radius: f64) -> String {
    format!("bold outline (dilate {} + merge)", radius_value_text(radius))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-morphology", Size::new(W, MORPHOLOGY_H))?;
    let defs = svg.defs()?;

    let original = svg.text(Point::new(COL1_X + RECT_W / 2.0, TEXT_BASELINE_Y), "MORPH")?;
    original.set_fill(STEELBLUE)?;
    original.set_font_size(34.0)?;
    original.set_text_anchor(TextAnchor::Middle)?;
    original.set_attr("font-weight", "bold")?;
    super::row_caption(&svg, COL1_X + RECT_W / 2.0, CAPTION_Y, "original")?;

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // Erode / Dilate — one slider, spanning both columns, drives both filters' own radius live
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    let erode_filter = defs.filter("morphology-erode")?;
    let erode_node = erode_filter.morphology(DEFAULT_RADIUS, MorphologyOperator::Erode)?;

    let eroded = svg.text(Point::new(COL2_X + RECT_W / 2.0, TEXT_BASELINE_Y), "MORPH")?;
    eroded.set_fill(STEELBLUE)?;
    eroded.set_font_size(34.0)?;
    eroded.set_text_anchor(TextAnchor::Middle)?;
    eroded.set_attr("font-weight", "bold")?;
    eroded.set_filter("morphology-erode")?;

    let erode_caption = svg.text(
        Point::new(COL2_X + RECT_W / 2.0, CAPTION_Y),
        &erode_caption_text(DEFAULT_RADIUS),
    )?;
    erode_caption.set_fill(CAPTION)?;
    erode_caption.set_attr("font-size", "11")?;
    erode_caption.set_text_anchor(TextAnchor::Middle)?;

    // Dilate expands the rendered area.
    // So its own filter region is widened, the same way demo_filter widens it for a wide blur.
    // Without this, the slider's own larger radii clip the dilated glyphs against the SVG default region's own
    // narrow 10% margin.
    let dilate_filter = defs.filter("morphology-dilate")?;
    super::widen_filter_region(&dilate_filter)?;
    let dilate_node = dilate_filter.morphology(DEFAULT_RADIUS, MorphologyOperator::Dilate)?;

    let dilated = svg.text(Point::new(COL3_X + RECT_W / 2.0, TEXT_BASELINE_Y), "MORPH")?;
    dilated.set_fill(STEELBLUE)?;
    dilated.set_font_size(34.0)?;
    dilated.set_text_anchor(TextAnchor::Middle)?;
    dilated.set_attr("font-weight", "bold")?;
    dilated.set_filter("morphology-dilate")?;

    let dilate_caption = svg.text(
        Point::new(COL3_X + RECT_W / 2.0, CAPTION_Y),
        &dilate_caption_text(DEFAULT_RADIUS),
    )?;
    dilate_caption.set_fill(CAPTION)?;
    dilate_caption.set_attr("font-size", "11")?;
    dilate_caption.set_text_anchor(TextAnchor::Middle)?;

    let radius_slider_w = COL3_X + RECT_W - COL2_X;
    let radius_slider = super::build_h_slider(
        &svg,
        Point::new(COL2_X, TOP),
        Size::new(radius_slider_w, CONTROL_ROW_H),
        ("radius", "morphology radius"),
        (MIN_RADIUS_TENTHS, MAX_RADIUS_TENTHS, DEFAULT_RADIUS_TENTHS),
        10, // a tick lands exactly on every whole-number radius, 0 through 4
        &["0.0", "4.0"],
    )?
    .input;
    radius_slider
        .set_attribute("aria-valuetext", &radius_value_text(DEFAULT_RADIUS))
        .map_err(dom_err)?;

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // bold outline (dilate + merge) — the same radius slider above drives this column's own dilate radius too
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // `SourceAlpha` has zero-valued colour channels, by construction. So the dilated fringe this filter exposes
    // is plain black, the same reason `demo_color_matrix`'s own LuminanceToAlpha rectangle needs one. Against
    // this gallery's own near-black canvas, that fringe would otherwise be nearly invisible. A plain white
    // backing rectangle, drawn once here and never touched by the slider, makes it read clearly instead.
    let outline_backing = svg.rect(Point::new(COL4_X, TEXT_ZONE_Y), Size::new(RECT_W, SHAPE_ZONE_H))?;
    outline_backing.set_fill(PAPER_WHITE)?;

    let outline_filter = defs.filter("morphology-outline")?;
    super::widen_filter_region(&outline_filter)?;
    let outline_node = outline_filter.morphology(DEFAULT_RADIUS, MorphologyOperator::Dilate)?;
    outline_node.set_attrs([("in", "SourceAlpha"), ("result", "thickened")])?;
    outline_filter.merge(&["thickened", "SourceGraphic"])?;

    let outlined = svg.text(Point::new(COL4_X + RECT_W / 2.0, TEXT_BASELINE_Y), "MORPH")?;
    outlined.set_fill(STEELBLUE)?;
    outlined.set_font_size(34.0)?;
    outlined.set_text_anchor(TextAnchor::Middle)?;
    outlined.set_attr("font-weight", "bold")?;
    outlined.set_filter("morphology-outline")?;

    let outline_caption = svg.text(
        Point::new(COL4_X + RECT_W / 2.0, CAPTION_Y),
        &outline_caption_text(DEFAULT_RADIUS),
    )?;
    outline_caption.set_fill(CAPTION)?;
    outline_caption.set_attr("font-size", "11")?;
    outline_caption.set_text_anchor(TextAnchor::Middle)?;

    {
        let slider = radius_slider.clone();
        let erode = erode_node.clone();
        let dilate = dilate_node.clone();
        let outline = outline_node.clone();
        let erode_caption = erode_caption.clone();
        let dilate_caption = dilate_caption.clone();
        let outline_caption = outline_caption.clone();
        let on_input: DemoClosure = Closure::new(move |_: web_sys::Event| {
            let radius = slider.value_as_number() / 10.0;
            let text = radius_value_text(radius);
            let _ = erode.set_attr("radius", &text);
            let _ = dilate.set_attr("radius", &text);
            let _ = outline.set_attr("radius", &text);
            let _ = slider.set_attribute("aria-valuetext", &text);
            erode_caption.set_text(&erode_caption_text(radius));
            dilate_caption.set_text(&dilate_caption_text(radius));
            outline_caption.set_text(&outline_caption_text(radius));
        });
        radius_slider
            .add_event_listener_with_callback("input", on_input.as_ref().unchecked_ref())
            .map_err(dom_err)?;
        keep_demo_closure(on_input);
    }

    Ok(())
}
