use crate::{H, PAD_Y, W, caption, colours::*, keep_demo_node};
use svg_dom::{
    Error, SvgRoot,
    root::utils::{Point, Size},
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Events — press state (managed mousedown / mouseup / pointerleave)
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-events-press", Size::new(W, H))?;

    let pad = svg.rect(Point::new(60.0, 25.0 + PAD_Y), Size::new(200.0, 80.0))?;
    pad.set_fill(TEAL)?;
    pad.set_attrs([("rx", "8"), ("style", "cursor:pointer")])?;

    let label = svg.text(Point::new(160.0, 70.0 + PAD_Y), "press & hold")?;
    label.set_fill(WHITE)?;
    label.set_attrs([("font-size", "15"), ("text-anchor", "middle"), ("style", "pointer-events:none")])?;

    let readout = svg.text(Point::new(320.0, 70.0 + PAD_Y), "state: idle")?;
    readout.set_fill(TEXT)?;
    readout.set_attr("font-size", "14")?;

    // Closures are `Clone` when everything they capture is `Clone` (SvgNode is), so we can build `press`/`release`
    // once and reuse them across several listeners.
    let press = {
        let pad = pad.clone();
        let readout = readout.clone();
        move |mods: &str| {
            let _ = pad.set_fill(TEAL_PRESSED); // darken while held
            let _ = pad.set_attr("transform", "translate(2,2)");
            readout.set_text(&format!("state: pressed{mods}"));
        }
    };
    let release = {
        let pad = pad.clone();
        let readout = readout.clone();
        move || {
            let _ = pad.set_fill(TEAL);
            let _ = pad.set_attr("transform", "translate(0,0)");
            readout.set_text("state: idle");
        }
    };

    // Only the primary button starts a press, so a plain right-click never engages one. That guard is not enough on
    // its own: on macOS a ctrl+click is reported as a *primary* mousedown (button 0) yet still opens the context
    // menu, which swallows the matching mouseup and would leave the state stuck on "pressed".
    // The `contextmenu` listener below is an OS-agnostic fix, which treats any context-menu trigger as a release.
    // The readout also lists any modifier keys held during the press.
    pad.on_mousedown(move |e| {
        if e.button() != 0 {
            return;
        }

        let mut held = Vec::new();
        if e.shift_key() {
            held.push("shift");
        }
        if e.ctrl_key() {
            held.push("ctrl");
        }
        if e.alt_key() {
            held.push("alt");
        }
        if e.meta_key() {
            held.push("meta");
        }
        let mods = if held.is_empty() {
            String::new()
        } else {
            format!("  ·  {}", held.join(" + "))
        };
        press(&mods);
    })?;

    let release_up = release.clone();
    pad.on_mouseup(move |_| release_up())?;

    // A context menu (right-click, or ctrl+click on macOS) interrupts the gesture consuming the mouseup event, so
    // treat it as a release so the button can never get stuck in the pressed state, whatever the platform.
    let release_ctx = release.clone();
    pad.on_contextmenu(move |_| release_ctx())?;

    // If the pointer leaves while still held, treat it as a release so the button cannot get stuck in the pressed
    // state.
    pad.on_pointerleave(move |_| release())?;

    caption(
        &svg,
        400.0,
        "managed mousedown / mouseup / pointerleave · pressed-state tracking · reports held modifier keys",
    )?;
    keep_demo_node(pad);
    Ok(())
}
