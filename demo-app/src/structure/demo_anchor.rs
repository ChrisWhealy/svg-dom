use crate::{BAND, H, PAD_Y, W, caption, colours::*};
use svg_dom::{
    Error, SvgRoot,
    root::utils::{Point, Size},
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// a — <g>-like wrapper that turns its children into one hyperlink
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-anchor", Size::new(W, H))?;
    let cy = PAD_Y + BAND / 2.0;

    // Fragment hrefs are used here purely so clicking inside this demo does not navigate away from the gallery; a
    // real application would pass whatever URL it actually wants to link to. Each link targets a different demo
    // panel's id rather than "#demo-anchor" (this panel's own id) or the page header: linking to an element already
    // on screen — this panel, or the always-visible header — is indistinguishable from the link doing nothing.
    // demo/index.html's hashchange listener is what makes a bare fragment link actually switch the visible panel;
    // without it, following a link to a hidden `.section` does nothing at all, since there is nothing to scroll to.
    //
    // Both the circle and its label become part of the same hyperlink, the same way an HTML <a> around several
    // elements would — clicking either one navigates.
    let link1 = svg.anchor("#panel-image")?;
    let icon1 = svg.circle(Point::new(150.0, cy), 40.0)?;
    icon1.set_fill(ACCENT_BLUE)?;
    let label1 = svg.text(Point::new(150.0, cy + 5.0), "A")?;
    label1.set_fill(WHITE)?;
    label1.set_attrs([("text-anchor", "middle"), ("font-size", "20"), ("font-weight", "bold")])?;
    link1.append(&icon1)?;
    link1.append(&label1)?;
    caption(&svg, 150.0, "<a>: one href wraps both children")?;

    // `target` is not wrapped by a named parameter — every meaningful use of <a> supplies href, but target is only
    // occasionally needed — so it goes through the generic set_attr escape hatch instead.
    let link2 = svg.anchor("#panel-marker")?;
    link2.set_attr("target", "_blank")?;
    let icon2 = svg.circle(Point::new(450.0, cy), 40.0)?;
    icon2.set_fill(DARK_ORANGE)?;
    let label2 = svg.text(Point::new(450.0, cy + 5.0), "B")?;
    label2.set_fill(WHITE)?;
    label2.set_attrs([("text-anchor", "middle"), ("font-size", "20"), ("font-weight", "bold")])?;
    link2.append(&icon2)?;
    link2.append(&label2)?;
    caption(&svg, 450.0, "target=\"_blank\" set via set_attr")?;

    Ok(())
}
