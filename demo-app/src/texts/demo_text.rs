use super::{foreign_object_document, radio_group};
use crate::{H, W, caption, colours::*, dom_err, keep_demo_node};

use svg_dom::{
    DominantBaseline, Error, SvgRoot, TextAnchor,
    root::utils::{Point, Size},
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// text-anchor (interactive)
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-text", Size::new(W, H))?;
    let mut demo_x: f64 = 6.0;

    // A single text element sits on the dashed vertical guide at x=100. A `radio_group` of three real HTML
    // `<input type="radio">` elements, laid out inside a `<foreignObject>` within the same SVG-space position, sets
    // its `text-anchor` live on change.
    //
    // The use of native HTML radio buttons was chosen over hand-drawn SVG ones specifically because of what they
    // give for free: screen-reader semantics, keyboard navigation, and browser-enforced mutual exclusivity via the
    // shared `name` attribute — all of which a custom-drawn SVG control would have had to implement by hand.
    let vguide = svg.line(Point::new(100.0, 30.0), Point::new(100.0, 66.0))?;
    vguide.set_stroke(GUIDE)?;
    vguide.set_attr("stroke-dasharray", "4 3")?;

    let anchor_text = svg.text(Point::new(100.0, 56.0), "sample text")?;
    anchor_text.set_fill(STEELBLUE)?;
    anchor_text.set_font_size(13.0)?;
    anchor_text.set_text_anchor(TextAnchor::Start)?;

    let fo = svg.foreign_object(Point::new(demo_x, 76.0), Size::new(180.0, 72.0))?;
    let document = foreign_object_document(&fo)?;

    const RADIO_OPTIONS: [(TextAnchor, &str); 3] = [
        (TextAnchor::Start, "start"),
        (TextAnchor::Middle, "middle"),
        (TextAnchor::End, "end"),
    ];

    let target = anchor_text.clone();
    let group = radio_group(
        &document,
        "text-anchor",
        "demo-text-anchor",
        &RADIO_OPTIONS,
        TextAnchor::Start,
        move |value| {
            let _ = target.set_text_anchor(value);
        },
    )?;
    fo.as_element().append_child(&group).map_err(dom_err)?;
    keep_demo_node(fo);

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // dominant-baseline (interactive)
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // A single text element sits on the dashed horizontal guide at y=45 — the same "one text element, one fixed
    // anchor coordinate" shape the text-anchor demo above uses, just on the other axis. A second `radio_group`, in
    // its own `<foreignObject>` below it, sets its `dominant-baseline` live on change — see the text-anchor demo
    // above for why native radios were chosen over hand-drawn SVG ones, and `SvgRoot::foreign_object`'s own doc
    // comment for the raw web-sys escape hatch both of these `<foreignObject>`s use.
    demo_x += 209.0;
    let hguide = svg.line(Point::new(demo_x, 45.0), Point::new(demo_x + 150.0, 45.0))?;
    hguide.set_stroke(GUIDE)?;
    hguide.set_attr("stroke-dasharray", "4 3")?;

    let baseline_text = svg.text(Point::new(demo_x + 45.0, 45.0), "baseline")?;
    baseline_text.set_fill(STEELBLUE)?;
    baseline_text.set_font_size(13.0)?;
    baseline_text.set_text_anchor(TextAnchor::Middle)?;
    baseline_text.set_dominant_baseline(DominantBaseline::Alphabetic)?;

    let db_fo = svg.foreign_object(Point::new(demo_x, 76.0), Size::new(185.0, 78.0))?;
    let db_document = foreign_object_document(&db_fo)?;

    const BASELINE_OPTIONS: [(DominantBaseline, &str); 3] = [
        (DominantBaseline::Alphabetic, "alphabetic"),
        (DominantBaseline::Middle, "middle"),
        (DominantBaseline::Hanging, "hanging"),
    ];

    let target = baseline_text.clone();
    let db_group = radio_group(
        &db_document,
        "dominant-baseline",
        "demo-dominant-baseline",
        &BASELINE_OPTIONS,
        DominantBaseline::Alphabetic,
        move |value| {
            let _ = target.set_dominant_baseline(value);
        },
    )?;
    db_fo.as_element().append_child(&db_group).map_err(dom_err)?;
    keep_demo_node(db_fo);

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // font-family
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // The same word rendered in the three CSS generic font families.
    demo_x += 180.0;
    let ff_serif = svg.text(Point::new(demo_x, 30.0), "Serif")?;
    ff_serif.set_fill(PLAIN_TEXT)?;
    ff_serif.set_font_size(18.0)?;
    ff_serif.set_font_family("serif")?;

    let ff_sans = svg.text(Point::new(demo_x, 77.0), "Sans-serif")?;
    ff_sans.set_fill(STEELBLUE)?;
    ff_sans.set_font_size(18.0)?;
    ff_sans.set_font_family("sans-serif")?;

    let ff_mono = svg.text(Point::new(demo_x, 128.0), "Monospace")?;
    ff_mono.set_fill(CORAL)?;
    ff_mono.set_font_size(18.0)?;
    ff_mono.set_font_family("monospace")?;

    caption(&svg, demo_x + 50.0, "font-family")?;

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // font-size
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // Three labels at the same x, each rendered at a different font size.
    demo_x += 150.0;
    let fs_s = svg.text(Point::new(demo_x, 30.0), "small — 11px")?;
    fs_s.set_fill(PLAIN_TEXT)?;
    fs_s.set_font_size(11.0)?;

    let fs_m = svg.text(Point::new(demo_x, 77.0), "medium — 17px")?;
    fs_m.set_fill(PLAIN_TEXT)?;
    fs_m.set_font_size(17.0)?;

    let fs_l = svg.text(Point::new(demo_x, 128.0), "large — 26px")?;
    fs_l.set_fill(PLAIN_TEXT)?;
    fs_l.set_font_size(26.0)?;

    caption(&svg, demo_x + 50.0, "font-size")?;

    Ok(())
}
