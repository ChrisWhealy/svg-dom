use crate::{BAND, H, PAD_Y, W, caption, colours::*};
use svg_dom::{
    Error, SvgRoot,
    root::utils::{Point, Size},
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// switch — renders at most one direct child, chosen by conditional-processing attributes
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-switch", Size::new(W, H))?;
    let cy = PAD_Y + BAND / 2.0;

    // Panel 1: SVG 2 defines an empty systemLanguage value as always false — unlike a made-up language code, which
    // is not guaranteed never to match (it would match a browser actually configured with that exact preference).
    // So the only conditional child here is guaranteed to fail, and <switch> falls through to the attribute-free
    // fallback, deterministically in every browser/locale.
    let switch1 = svg.switch()?;
    let never_matches = svg.circle(Point::new(150.0, cy), 40.0)?;
    never_matches.set_fill(CORAL)?;
    never_matches.set_attr("systemLanguage", "")?;
    switch1.append(&never_matches)?;
    let fallback = svg.circle(Point::new(150.0, cy), 40.0)?;
    fallback.set_fill(STEELBLUE)?;
    switch1.append(&fallback)?;
    caption(&svg, 150.0, "no child matches -> fallback renders")?;

    // Panel 2: the first child has no test attributes at all, so it always matches and renders immediately — the
    // second child is never reached, even though it would otherwise be a perfectly valid alternative.
    let switch2 = svg.switch()?;
    let first_match = svg.circle(Point::new(450.0, cy), 40.0)?;
    first_match.set_fill(MEDIUM_SEA_GREEN)?;
    switch2.append(&first_match)?;
    let never_reached = svg.circle(Point::new(450.0, cy), 40.0)?;
    never_reached.set_fill(DARK_ORANGE)?;
    switch2.append(&never_reached)?;
    caption(&svg, 450.0, "attribute-free first child always matches")?;

    Ok(())
}
