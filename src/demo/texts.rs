use super::colours::*;
use super::{H, PAD_Y, W, caption};
use crate::{
    DominantBaseline, Error, PathDef, PathDefAbsolute, SvgRoot, TextAnchor,
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

    // ── multi-line (tspan_line) ───────────────────────────────────────────────
    // A <text> with three <tspan> children.  The first inherits x from <text>. Subsequent spans use `tspan_line`, which
    // resets `x` to the same absolute coordinate and advances `y` by `dy`, so every line aligns with the left edge.
    const LINE_X: f64 = 50.0;
    const LINE_DY: f64 = 22.0;
    let ml = svg.text(Point::new(LINE_X, 50.0 + PAD_Y), "")?;
    ml.set_fill(PLAIN_TEXT)?;
    ml.set_font_size(15.0)?;

    ml.tspan("The quick brown fox")?;
    ml.tspan_line(LINE_X, LINE_DY, "jumps over the")?;
    ml.tspan_line(LINE_X, LINE_DY, "lazy dog.")?;

    caption(&svg, 200.0, "multi-line (tspan_line)")?;

    // ── inline mixed styles ───────────────────────────────────────────────────
    // A single <text> element whose <tspan> children each override fill and font-size, producing a mixed-style run on
    // one baseline.
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

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Builds an SVG path `d` string tracing `periods` cycles of a sine wave, `width` user units wide and `amplitude`
/// tall, starting at `(x0, y0)`.
///
/// Sampled as short straight-line segments (`STEP` user units apart) rather than fitted with Bézier curves — dense
/// enough to read as a smooth curve at demo scale, and far simpler than deriving cubic control points that actually
/// hug a sine function (a circular arc's Bézier approximation constant does not carry over to a sine curve).
fn sine_wave_path(x0: f64, y0: f64, width: f64, amplitude: f64, periods: f64) -> String {
    use std::fmt::Write;
    const STEP: f64 = 4.0;

    let mut path_d = format!("M {x0:.1} {y0:.1}");
    let mut x = STEP;
    while x <= width {
        let y = y0 - amplitude * (2.0 * std::f64::consts::PI * periods * x / width).sin();
        let _ = write!(path_d, " L {:.1} {:.1}", x0 + x, y);
        x += STEP;
    }
    path_d
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// textPath — gluing text to a curve
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(super) fn demo_text_path() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-text-path", Size::new(W, H))?;

    // ── follows a sine wave ──────────────────────────────────────────────────────
    // The guide wave lives in <defs> — it is never rendered itself, only referenced by `href` — which is the usual
    // way to use textPath: the geometry is a pure positioning aid for the text. The text only covers part of the
    // path's full length, so more periods are packed in than will actually be traversed, letting several full
    // up/down cycles show through the glyphs that do get drawn.
    let wave_d = sine_wave_path(20.0, 90.0, 1000.0, 40.0, 4.0);
    let defs = svg.build_defs(|d| {
        d.path(&wave_d)?.set_attr("id", "demo-tp-wave")?;
        Ok(())
    })?;
    let _ = defs; // held only to construct the wave; the id reference below is what matters

    let curve = svg.text(Point::origin(), "")?;
    let curve_path = curve.text_path("#demo-tp-wave", "Now riding a sine wave, up and down")?;
    curve_path.set_fill(PLAIN_TEXT)?;
    curve_path.set_font_size(14.0)?;

    caption(&svg, 200.0, "textPath — follows a sine wave")?;

    // ── startOffset ───────────────────────────────────────────────────────────────
    // Here the guide arc is drawn directly on the canvas (dashed) rather than hidden in <defs>, so the effect of
    // set_start_offset is visible: two independent <textPath> elements share the same path but start at different
    // distances along it.
    let guide = svg.path_from_defs(&[
        PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(430.0, 130.0))),
        PathDef::Abs(PathDefAbsolute::QuadraticBezierTo(
            Point::new(600.0, 45.0),
            Point::new(770.0, 130.0),
        )),
    ])?;
    guide.set_attr("id", "demo-tp-offset-arc")?;
    guide.set_fill(NONE)?;
    guide.set_stroke(GUIDE)?;
    guide.set_attr("stroke-dasharray", "4 3")?;

    let start = svg.text(Point::origin(), "")?;
    let start_path = start.text_path("#demo-tp-offset-arc", "offset 0")?;
    start_path.set_fill(PLAIN_TEXT)?;
    start_path.set_font_size(14.0)?;

    let shifted = svg.text(Point::origin(), "")?;
    let shifted_path = shifted.text_path("#demo-tp-offset-arc", "offset 200")?;
    shifted_path.set_fill(CORAL)?;
    shifted_path.set_font_size(14.0)?;
    shifted_path.set_start_offset(200.0)?;

    caption(&svg, 600.0, "startOffset — slides text along the path")?;

    Ok(())
}
