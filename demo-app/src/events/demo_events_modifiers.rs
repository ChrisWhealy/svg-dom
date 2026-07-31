use crate::{H, PAD_Y, W, caption, colours::*, keep_demo_node};
use svg_dom::{
    Error, SvgRoot,
    root::utils::{Point, Size},
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Events — modifier keys (on_click) + right-click (on_contextmenu, preventDefault)
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-events-modifiers", Size::new(W, H))?;

    let pad = svg.rect(Point::new(40.0, 25.0 + PAD_Y), Size::new(240.0, 80.0))?;
    pad.set_fill(SLATE_BLUE)?;
    pad.set_attrs([("rx", "8"), ("style", "cursor:pointer")])?;

    let hint = svg.text(Point::new(160.0, 70.0 + PAD_Y), "click me")?;
    hint.set_fill(WHITE)?;
    hint.set_attrs([("font-size", "15"), ("text-anchor", "middle"), ("style", "pointer-events:none")])?;

    let readout = svg.text(Point::new(310.0, 70.0 + PAD_Y), "try: click · shift · ctrl · alt · right-click")?;
    readout.set_fill(TEXT)?;
    readout.set_attr("font-size", "14")?;

    // Left-click → inspect the modifier-key flags carried by the MouseEvent.
    let pad_click = pad.clone();
    let ro_click = readout.clone();
    pad.on_click(move |e| {
        let (label, colour) = if e.shift_key() {
            ("shift + click", TOMATO)
        } else if e.ctrl_key() {
            ("ctrl + click", MEDIUM_SEA_GREEN)
        } else if e.alt_key() {
            ("alt + click", GOLDENROD)
        } else if e.meta_key() {
            ("meta + click", ORCHID)
        } else {
            ("plain click", SLATE_BLUE)
        };
        let _ = pad_click.set_fill(colour);
        ro_click.set_text(&format!("last: {label}"));
    })?;

    // Right-click → suppress the browser context menu and report it.  ('click' never fires for the secondary button,
    // so the contextmenu event is the idiomatic hook for right-clicks.)
    let pad_ctx = pad.clone();
    let ro_ctx = readout.clone();
    pad.on_contextmenu(move |e| {
        e.prevent_default();
        let _ = pad_ctx.set_fill(CRIMSON);
        ro_ctx.set_text("last: right-click (context menu suppressed)");
    })?;

    caption(
        &svg,
        400.0,
        "on_click reads modifier keys · on_contextmenu calls preventDefault()",
    )?;
    keep_demo_node(pad);
    Ok(())
}
