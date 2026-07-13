use super::colours::*;
use super::{H, PAD_Y, W, caption};
use crate::{
    DominantBaseline, Error, SvgRoot, TextAnchor,
    root::utils::{Point, Size},
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// text
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(super) fn demo_text() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-text", Size::new(W, H))?;

    // ── text-anchor ───────────────────────────────────────────────────────────────
    // Vertical dashed guide at x=100; three labels anchored at that x position each
    // using a different TextAnchor variant so the visual spread makes the effect clear.
    let vguide = svg.line(Point::new(100.0, 35.0), Point::new(100.0, 138.0))?;
    vguide.set_stroke(GUIDE)?;
    vguide.set_attr("stroke-dasharray", "4 3")?;

    let ta_s = svg.text(Point::new(100.0, 58.0), "start")?;
    ta_s.set_fill(PLAIN_TEXT)?;
    ta_s.set_font_size(13.0)?;
    ta_s.set_text_anchor(TextAnchor::Start)?;

    let ta_m = svg.text(Point::new(100.0, 90.0), "middle")?;
    ta_m.set_fill(STEELBLUE)?;
    ta_m.set_font_size(13.0)?;
    ta_m.set_text_anchor(TextAnchor::Middle)?;

    let ta_e = svg.text(Point::new(100.0, 122.0), "end")?;
    ta_e.set_fill(CORAL)?;
    ta_e.set_font_size(13.0)?;
    ta_e.set_text_anchor(TextAnchor::End)?;

    caption(&svg, 100.0, "text-anchor")?;

    // ── font-size ─────────────────────────────────────────────────────────────────
    // Three labels at the same x, each rendered at a different font size.
    let fs_s = svg.text(Point::new(215.0, 60.0), "small — 11px")?;
    fs_s.set_fill(PLAIN_TEXT)?;
    fs_s.set_font_size(11.0)?;

    let fs_m = svg.text(Point::new(215.0, 92.0), "medium — 17px")?;
    fs_m.set_fill(PLAIN_TEXT)?;
    fs_m.set_font_size(17.0)?;

    let fs_l = svg.text(Point::new(215.0, 128.0), "large — 26px")?;
    fs_l.set_fill(PLAIN_TEXT)?;
    fs_l.set_font_size(26.0)?;

    caption(&svg, 300.0, "font-size")?;

    // ── dominant-baseline ─────────────────────────────────────────────────────────
    // Horizontal dashed guide at y=90; three labels share that y but use different
    // DominantBaseline values so the guide passes through a different part of each.
    let hguide = svg.line(Point::new(415.0, 90.0), Point::new(588.0, 90.0))?;
    hguide.set_stroke(GUIDE)?;
    hguide.set_attr("stroke-dasharray", "4 3")?;

    let db_h = svg.text(Point::new(432.0, 90.0), "hanging")?;
    db_h.set_fill(PLAIN_TEXT)?;
    db_h.set_font_size(13.0)?;
    db_h.set_text_anchor(TextAnchor::Middle)?;
    db_h.set_dominant_baseline(DominantBaseline::Hanging)?;

    let db_m = svg.text(Point::new(503.0, 90.0), "middle")?;
    db_m.set_fill(STEELBLUE)?;
    db_m.set_font_size(13.0)?;
    db_m.set_text_anchor(TextAnchor::Middle)?;
    db_m.set_dominant_baseline(DominantBaseline::Middle)?;

    let db_a = svg.text(Point::new(573.0, 90.0), "alphabetic")?;
    db_a.set_fill(CORAL)?;
    db_a.set_font_size(13.0)?;
    db_a.set_text_anchor(TextAnchor::Middle)?;
    db_a.set_dominant_baseline(DominantBaseline::Alphabetic)?;

    caption(&svg, 500.0, "dominant-baseline")?;

    // ── font-family ───────────────────────────────────────────────────────────────
    // The same word rendered in the three CSS generic font families.
    let ff_serif = svg.text(Point::new(618.0, 60.0), "Serif")?;
    ff_serif.set_fill(PLAIN_TEXT)?;
    ff_serif.set_font_size(16.0)?;
    ff_serif.set_font_family("serif")?;

    let ff_sans = svg.text(Point::new(618.0, 91.0), "Sans-serif")?;
    ff_sans.set_fill(STEELBLUE)?;
    ff_sans.set_font_size(16.0)?;
    ff_sans.set_font_family("sans-serif")?;

    let ff_mono = svg.text(Point::new(618.0, 124.0), "Monospace")?;
    ff_mono.set_fill(CORAL)?;
    ff_mono.set_font_size(16.0)?;
    ff_mono.set_font_family("monospace")?;

    caption(&svg, 700.0, "font-family")?;

    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// tspan — multi-line and inline mixed-style text
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(super) fn demo_tspan() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-tspan", Size::new(W, H))?;

    // ── multi-line (dy) ───────────────────────────────────────────────────────
    // A <text> with three <tspan> children, each advancing 22px down via dy.
    // The first tspan inherits x/y from the parent; subsequent ones use dy to
    // step to the next line without needing absolute y coordinates.
    const LINE_DY: f64 = 22.0;
    let ml = svg.text(Point::new(50.0, 50.0 + PAD_Y), "")?;
    ml.set_fill(PLAIN_TEXT)?;
    ml.set_font_size(15.0)?;

    ml.tspan("The quick brown fox")?;
    ml.tspan_dy(LINE_DY, "jumps over the")?;
    ml.tspan_dy(LINE_DY, "lazy dog.")?;

    caption(&svg, 200.0, "multi-line (tspan dy)")?;

    // ── inline mixed styles ───────────────────────────────────────────────────
    // A single <text> element whose <tspan> children each override fill and
    // font-size, producing a mixed-style run on one baseline.
    let mx = svg.text(Point::new(420.0, 90.0 + PAD_Y), "")?;

    let w1 = mx.tspan("small ")?;
    w1.set_fill(PLAIN_TEXT)?;
    w1.set_font_size(12.0)?;
    w1.set_dominant_baseline(DominantBaseline::Middle)?;

    let w2 = mx.tspan("MEDIUM ")?;
    w2.set_fill(STEELBLUE)?;
    w2.set_font_size(18.0)?;
    w2.set_dominant_baseline(DominantBaseline::Middle)?;

    let w3 = mx.tspan("LARGE")?;
    w3.set_fill(CORAL)?;
    w3.set_font_size(26.0)?;
    w3.set_dominant_baseline(DominantBaseline::Middle)?;

    caption(&svg, 600.0, "inline mixed styles (tspan)")?;

    Ok(())
}
