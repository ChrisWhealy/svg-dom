use crate::{
    W,
    colours::*,
    dom_err,
    foreign_html::{foreign_object_document, select_dropdown},
    keep_demo_node,
};
use svg_dom::{
    Error, SvgRoot, TextAnchor,
    root::{
        filter::{BlendMode, CompositeOperator},
        utils::{Point, Size},
    },
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// feBlend — the same gradient source, flooded with the same orange tint, blended against BlendMode, chosen live by
// a dropdown
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// `BlendMode` has sixteen members. `foreign_html::radio_group` (the control `demo_text`/`demo_image`/
// `demo_radial_gradient` each use for a handful of options) would need sixteen rows, dwarfing the circle it sits
// beside. `foreign_html::select_dropdown` collapses every option into one native `<select>` instead — this file is
// its first caller. See that function's own doc comment for the shared control itself; this file supplies only the
// BlendMode-specific option list and the live-update handler that keeps `feBlend`'s own `mode` attribute in sync.
// `BlendMode::as_str` (public since blend_mode.rs) supplies the attribute keyword directly, so this file keeps no
// copy of that mapping of its own.
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

const CIRCLE_R: f64 = 55.0;
const LEFT_MARGIN: f64 = 60.0;
const ITEM_GAP: f64 = 160.0; // gap between the two circles

const SELECT_ROW_H: f64 = 46.0; // one HTML select block: label + <select>
const CONTROL_GAP: f64 = 10.0; // gap between the select and the circle it drives
const CAPTION_OFFSET: f64 = 14.0; // caption sits this far below the circle's own bottom edge

const TOP: f64 = 10.0;

// Both circles share the same vertical centre, even though only the blended circle has a control above it — the
// same "shared row, one control zone unused" layout `demo_radial_gradient`'s own centred/spreadMethod pair uses,
// so the two captions below still line up.
const RECT_Y: f64 = TOP + SELECT_ROW_H + CONTROL_GAP;
const CY: f64 = RECT_Y + CIRCLE_R;
const CAPTION_Y: f64 = RECT_Y + CIRCLE_R * 2.0 + CAPTION_OFFSET;

const ORIGINAL_CX: f64 = LEFT_MARGIN + CIRCLE_R;
const BLENDED_CX: f64 = ORIGINAL_CX + CIRCLE_R * 2.0 + ITEM_GAP;

const SELECT_W: f64 = 170.0; // wide enough for "Color Dodge"/"Color Burn", this list's longest labels

const BLEND_H: f64 = CAPTION_Y + 12.0;

const DEFAULT_MODE: BlendMode = BlendMode::Multiply;

// Every `BlendMode` member, paired with the display text its own `<option>` (and the live caption below the
// blended circle) shows. Declaration order here becomes the dropdown's own option order.
const BLEND_MODE_OPTIONS: [(BlendMode, &str); 16] = [
    (BlendMode::Normal, "Normal"),
    (BlendMode::Multiply, "Multiply"),
    (BlendMode::Screen, "Screen"),
    (BlendMode::Darken, "Darken"),
    (BlendMode::Lighten, "Lighten"),
    (BlendMode::Overlay, "Overlay"),
    (BlendMode::ColorDodge, "Color Dodge"),
    (BlendMode::ColorBurn, "Color Burn"),
    (BlendMode::HardLight, "Hard Light"),
    (BlendMode::SoftLight, "Soft Light"),
    (BlendMode::Difference, "Difference"),
    (BlendMode::Exclusion, "Exclusion"),
    (BlendMode::Hue, "Hue"),
    (BlendMode::Saturation, "Saturation"),
    (BlendMode::Color, "Color"),
    (BlendMode::Luminosity, "Luminosity"),
];

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Formats the blended circle's own live caption, shown below it.
fn blend_caption_text(label: &str) -> String {
    format!("mode: {label}")
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-blend", Size::new(W, BLEND_H))?;

    let defs = svg.defs()?;

    // Same multi-hue gradient technique as the feColorMatrix demo: a flat source colour would make every mode
    // collapse to a single flat result, hiding how differently each one actually treats colour.
    defs.build_linear_gradient("blend-source", |g| {
        g.add_stop(0.0, STEELBLUE)?;
        g.add_stop(0.5, GOLD)?;
        g.add_stop(1.0, CRIMSON)?;
        Ok(())
    })?;

    // One filter, not one per mode: the dropdown below updates this filter's own `feBlend` mode attribute live,
    // via the retained `SvgNode` `blend` returns, the same retained-primitive pattern `demo_filter`'s
    // `gaussian_blur`/`drop_shadow` use for their own sliders.
    //
    // flood paints its colour opaquely across the *entire* filter region — a rectangle, unrelated to this circle's
    // own round shape — and feBlend's result alpha is the union of its two inputs' alpha. Without a final
    // composite(In) clipping back to the source's own alpha coverage, the opaque flood would leak straight through
    // the fully transparent corners of the circle's bounding box, visibly staining them with the flood colour.
    // Using circles here rather than rectangles is deliberate, for exactly this reason: a rectangle has no
    // transparency in its own bounding box for a leaking flood to show through, which would make this mistake
    // invisible. See `SvgFilter::blend`'s own doc comment for the full explanation.
    let filter = defs.filter("demo-blend-filter")?;
    filter.flood(LEAF_ORANGE, 1.0)?.set_attr("result", "tint")?;
    let blend = filter.blend("tint", DEFAULT_MODE)?;
    blend.set_attrs([("in", "SourceGraphic"), ("result", "tinted")])?;
    filter
        .composite("SourceGraphic", CompositeOperator::In)?
        .set_attr("in", "tinted")?;

    // The final composite(In) step above clips the blended result back to the source circle's own rendered
    // pixels, so — unlike feGaussianBlur/feOffset, which genuinely spread pixels beyond the source's own shape —
    // no filter region widening is needed here.
    let original = svg.circle(Point::new(ORIGINAL_CX, CY), CIRCLE_R)?;
    original.set_fill_gradient("blend-source")?;
    super::row_caption(&svg, ORIGINAL_CX, CAPTION_Y, "original")?;

    let blended = svg.circle(Point::new(BLENDED_CX, CY), CIRCLE_R)?;
    blended.set_fill_gradient("blend-source")?;
    blended.set_filter("demo-blend-filter")?;

    let blend_caption = svg.text(
        Point::new(BLENDED_CX, CAPTION_Y),
        &blend_caption_text(BLEND_MODE_OPTIONS[1].1), // DEFAULT_MODE is Multiply, index 1 above
    )?;
    blend_caption.set_fill(CAPTION)?;
    blend_caption.set_attr("font-size", "11")?;
    blend_caption.set_text_anchor(TextAnchor::Middle)?;

    // select_dropdown builds a native HTML <select>, not sixteen hand-drawn or radio-based options, for the same
    // screen-reader semantics, keyboard navigation, and browser-native behaviour `radio_group`'s own callers rely
    // on — just collapsed into one control, since sixteen radio rows would dwarf the circle beside it. It sits
    // above the blended circle, the same "control above shape" layout `demo_filter`'s blur circle uses.
    let select_fo =
        svg.foreign_object(Point::new(BLENDED_CX - SELECT_W / 2.0, TOP), Size::new(SELECT_W, SELECT_ROW_H))?;
    let select_document = foreign_object_document(&select_fo)?;

    let on_select_blend = blend.clone();
    let on_select_caption = blend_caption.clone();
    let control = select_dropdown(
        &select_document,
        ("blend mode", "feBlend blend mode"),
        &BLEND_MODE_OPTIONS,
        DEFAULT_MODE,
        move |mode: BlendMode| {
            let _ = on_select_blend.set_attr("mode", mode.as_str());
            let label = BLEND_MODE_OPTIONS
                .iter()
                .find(|&&(m, _)| m == mode)
                .map_or("", |&(_, label)| label);
            on_select_caption.set_text(&blend_caption_text(label));
        },
    )?;
    select_fo.as_element().append_child(&control).map_err(dom_err)?;
    keep_demo_node(select_fo);

    Ok(())
}
