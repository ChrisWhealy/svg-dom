use crate::{H, PAD_Y, W, caption, colours::*, keep_demo_node};
use svg_dom::{
    Error, SvgRoot,
    root::utils::{Point, Size},
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Geometry — bounding_box / bounding_client_rect (an auto-fit frame around a composite icon)
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    // None of the three shapes below carry a `transform`, so each one's `bounding_box()` (local, pre-transform
    // user-space) lines up exactly with where it has actually been rendered. The dashed overlay can be drawn
    // directly from the returned coordinates with no further conversion.
    //
    // ***NOTE***
    // `bounding_box()` deliberately excludes an element's own `transform`, confirmed empirically before writing this
    // demo - so a transformed element's overlay would need to share that same transform, which this demo sidesteps
    // entirely by not using one.
    //
    // The measurement happens inside the `on_click` handler, not once at setup.
    //
    // `getBBox()` and `getBoundingClientRect()` both require the element to actually be rendered — having a
    // `display:none` ancestor (which is exactly the state of every non-active demo panel: see `.section`/
    // `.section.active` in `style.css`) makes them report an all-zero rect. This panel's own build function only
    // ever runs once, the first time a user selects it (see `lib.rs`'s `init_panel`), by which point its section is
    // already the active, visible one — so measuring at setup time would in fact be safe here now. Measuring inside
    // a click handler instead is kept anyway: it is the more natural shape for a "measure on demand" demo, and it
    // means this demo makes no assumption about *when* its own panel becomes visible relative to when it runs — an
    // assumption worth not depending on regardless of whether the current lazy-initialisation approach happens to
    // guarantee it.
    let svg = SvgRoot::create_in("demo-geometry-bbox", Size::new(W, H))?;

    // A full-canvas, invisible-but-hit-testable surface placed *behind* the icon (added to the DOM first, so it
    // renders underneath). A click that lands on the icon hits the icon's own opaque shapes instead of this
    // surface, so the two `on_click` handlers below never fire for the same click — this one only ever catches
    // clicks outside the icon, which is exactly what "clear the overlay" should mean.
    let clear_zone = svg.rect(Point::origin(), Size::new(W, H))?;
    clear_zone.set_fill(TRANSPARENT)?;

    // A small cluster of three overlapping shapes — no single one of them reveals the whole composite's extent.
    let group = svg.group()?;
    group.set_attr("style", "cursor:pointer")?;
    svg.build_batch_into(&group, |b| {
        let base = b.rect(Point::new(60.0, 20.0 + PAD_Y), Size::new(110.0, 60.0))?;
        base.set_fill(LEAF_GREEN)?;
        base.set_attr("rx", "8")?;

        let badge = b.circle(Point::new(165.0, 25.0 + PAD_Y), 20.0)?;
        badge.set_fill(GOLD)?;

        let flag = b.polygon(&[
            Point::new(40.0, 82.0 + PAD_Y),
            Point::new(90.0, 100.0 + PAD_Y),
            Point::new(30.0, 100.0 + PAD_Y),
        ])?;
        flag.set_fill(LEAF_ORANGE)?;
        Ok(())
    })?;

    // Placeholder overlay and readouts; both are filled in for real on the first click.
    let overlay = svg.rect(Point::origin(), Size::new(0.0, 0.0))?;
    overlay.set_fill(NONE)?;
    overlay.set_stroke(WHITE)?;
    overlay.set_stroke_width(1.5)?;
    overlay.set_attr("stroke-dasharray", "4 3")?;

    const LOCAL_PLACEHOLDER: &str = "bounding_box (local):    click the icon →";
    const CLIENT_PLACEHOLDER: &str = "bounding_client_rect (CSS px):    click the icon →";

    let local_readout = svg.text(Point::new(280.0, 45.0 + PAD_Y), LOCAL_PLACEHOLDER)?;
    local_readout.set_fill(TEXT)?;
    local_readout.set_attr("font-size", "13")?;

    let client_readout = svg.text(Point::new(280.0, 70.0 + PAD_Y), CLIENT_PLACEHOLDER)?;
    client_readout.set_fill(TEXT_MUTED)?;
    client_readout.set_attr("font-size", "13")?;

    let click_group = group.clone();
    let click_overlay = overlay.clone();
    let click_local = local_readout.clone();
    let click_client = client_readout.clone();
    group.on_click(move |_| {
        let Ok(bbox) = click_group.bounding_box() else { return };
        let client_rect = click_group.bounding_client_rect();

        let _ = click_overlay.set_attr("x", &bbox.origin.x.to_string());
        let _ = click_overlay.set_attr("y", &bbox.origin.y.to_string());
        let _ = click_overlay.set_attr("width", &bbox.size.width.to_string());
        let _ = click_overlay.set_attr("height", &bbox.size.height.to_string());

        click_local.set_text(&format!(
            "bounding_box (local):    x={:.0} y={:.0} w={:.0} h={:.0}",
            bbox.origin.x, bbox.origin.y, bbox.size.width, bbox.size.height
        ));
        click_client.set_text(&format!(
            "bounding_client_rect (CSS px):    x={:.0} y={:.0} w={:.0} h={:.0}",
            client_rect.origin.x, client_rect.origin.y, client_rect.size.width, client_rect.size.height
        ));
    })?;

    // Clicking anywhere outside the icon clears the overlay and readouts back to their placeholder state.
    let clear_overlay = overlay.clone();
    let clear_local = local_readout.clone();
    let clear_client = client_readout.clone();
    clear_zone.on_click(move |_| {
        let _ = clear_overlay.set_attr("width", "0");
        let _ = clear_overlay.set_attr("height", "0");
        clear_local.set_text(LOCAL_PLACEHOLDER);
        clear_client.set_text(CLIENT_PLACEHOLDER);
    })?;

    caption(
        &svg,
        400.0,
        "Click the icon — bounding_box() drives the dashed overlay (local user-space) · bounding_client_rect() readout shows the same element in viewport CSS pixels — different numbers, different coordinate spaces. Click outside the icon to clear it.",
    )?;
    keep_demo_node(group);
    keep_demo_node(clear_zone);
    Ok(())
}
