use crate::{H, PAD_Y, W, caption, colours::*};

use svg_dom::{
    DominantBaseline, Error, SvgRoot,
    root::utils::{Point, Size},
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// tspan — multi-line and inline mixed-style text
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-tspan", Size::new(W, H))?;

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
