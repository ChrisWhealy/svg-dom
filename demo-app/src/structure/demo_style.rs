use crate::{BAND, H, PAD_Y, W, caption};
use svg_dom::{
    Error, SvgRoot,
    root::utils::{Point, Size},
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// style — a document-wide CSS stylesheet, including a :hover rule set_attr cannot express
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-style", Size::new(W, H))?;

    // A :hover rule is the point of this demo: set_attr/the presentation-property setters can express every state
    // a shape is *currently* in, but not a state transition triggered purely by the pointer entering/leaving —
    // that needs either a real CSS rule (declarative, no event listener at all) or an on_pointerenter/leave pair
    // wired up by hand. <style> is what makes the declarative route available from Rust.
    //
    // Three different dots, three different CSS features a :hover rule can drive, so this isn't the same
    // demonstration repeated three times: recolouring (fill/stroke), a geometric transform (scale), and a CSS
    // `filter` (a glow) — the last one deliberately distinct from this crate's own SvgFilter/<feGaussianBlur> etc.,
    // since `filter: drop-shadow(...)` here is a CSS *property*, not an SVG `<filter>` element reference.
    svg.style(
        ".dot-fill { fill: steelblue; stroke: white; stroke-width: 0; transition: fill 0.2s ease, stroke-width 0.2s ease; } \
         .dot-fill:hover { fill: coral; stroke-width: 6; }\n\
         .dot-scale { fill: steelblue; transform-box: fill-box; transform-origin: center; transition: transform 0.2s ease; } \
         .dot-scale:hover { transform: scale(1.3); }\n\
         .dot-glow { fill: steelblue; transition: filter 0.2s ease; } \
         .dot-glow:hover { filter: drop-shadow(0 0 10px gold); }",
    )?;

    let cy = PAD_Y + BAND / 2.0;
    for (cx, class) in [(150.0, "dot-fill"), (400.0, "dot-scale"), (650.0, "dot-glow")] {
        let dot = svg.circle(Point::new(cx, cy), 40.0)?;
        dot.set_attrs([("class", class), ("style", "cursor:pointer")])?;
    }

    caption(
        &svg,
        W / 2.0,
        "hover over a dot — fill+stroke, transform: scale, and filter: drop-shadow, all driven by :hover rules in <style>",
    )?;
    Ok(())
}
