use std::{cell::Cell, rc::Rc};

use crate::{H, PAD_Y, W, caption, colours::*, keep_demo_node};
use svg_dom::{
    Error, SvgRoot,
    root::utils::{Point, Size},
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Events — click counter + reset button (two on_click handlers over shared state)
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    // Counting and resetting live on two *separate* buttons on purpose. A "double-click to reset" on the counter
    // itself would misbehave: the browser always fires two `click` events before a `dblclick`, so any quick pair of
    // clicks would increment twice and then immediately reset to zero.
    let svg = SvgRoot::create_in("demo-events-click", Size::new(W, H))?;

    // Counter button.  Its colour cycles on every click so repeated presses are visible.
    let btn = svg.rect(Point::new(40.0, 30.0 + PAD_Y), Size::new(150.0, 60.0))?;
    btn.set_fill(STEELBLUE)?;
    btn.set_attrs([("rx", "8"), ("style", "cursor:pointer")])?;

    // The label sits on top of the button; `pointer-events:none` lets clicks fall through to the rect beneath.
    let btn_label = svg.text(Point::new(115.0, 66.0 + PAD_Y), "click me")?;
    btn_label.set_fill(WHITE)?;
    btn_label.set_attrs([("font-size", "16"), ("text-anchor", "middle"), ("style", "pointer-events:none")])?;

    // Reset button — greyed out until there is actually something to reset.
    let reset = svg.rect(Point::new(210.0, 30.0 + PAD_Y), Size::new(110.0, 60.0))?;
    reset.set_fill(RESET_IDLE)?;
    reset.set_attrs([("rx", "8"), ("style", "cursor:pointer")])?;

    let reset_label = svg.text(Point::new(265.0, 66.0 + PAD_Y), "reset")?;
    reset_label.set_fill(WHITE)?;
    reset_label.set_attrs([("font-size", "15"), ("text-anchor", "middle"), ("style", "pointer-events:none")])?;

    let readout = svg.text(Point::new(350.0, 66.0 + PAD_Y), "clicks: 0")?;
    readout.set_fill(TEXT)?;
    readout.set_attr("font-size", "15")?;

    let count = Rc::new(Cell::new(0u32));

    // Counter click → increment.  The closures also capture clones of other demo nodes (cross-captures, no cycle).
    // inc_btn is a self-capture of btn — a harmless cycle because keep_demo_node(btn) below already holds it alive.
    let inc_btn = btn.clone();
    let inc_reset = reset.clone();
    let inc_readout = readout.clone();
    let inc_count = count.clone();
    btn.on_click(move |_| {
        let n = inc_count.get() + 1;
        inc_count.set(n);
        let _ = inc_btn.set_fill(&format!("hsl({},60%,45%)", (n * 40) % 360));
        let _ = inc_reset.set_fill(TOMATO); // reset now has something to do
        inc_readout.set_text(&format!("clicks: {n}"));
    })?;

    // Reset click → zero the count and restore the resting colours.
    let rst_btn = btn.clone();
    let rst_reset = reset.clone();
    let rst_readout = readout.clone();
    let rst_count = count.clone();
    reset.on_click(move |_| {
        rst_count.set(0);
        let _ = rst_btn.set_fill(STEELBLUE);
        let _ = rst_reset.set_fill(RESET_IDLE);
        rst_readout.set_text("clicks: 0");
    })?;

    caption(&svg, 400.0, "two on_click handlers sharing one Rc<Cell> counter")?;
    keep_demo_node(btn);
    keep_demo_node(reset);
    Ok(())
}
