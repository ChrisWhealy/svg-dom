use std::{cell::Cell, fmt::Write, rc::Rc};

use crate::{H, PAD_Y, W, caption, colours::*, keep_demo_node};
use svg_dom::{
    Error, SvgRoot,
    root::utils::{Point, Size},
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Events — keyboard, focus and wheel wrappers
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-events-keyboard-wheel", Size::new(W, H))?;

    let pad = svg.rect(Point::new(50.0, 24.0 + PAD_Y), Size::new(270.0, 84.0))?;
    pad.set_fill(SLATE_GRAY)?;
    pad.set_stroke(ACCENT_AMBER)?;
    pad.set_stroke_width(2.0)?;
    pad.set_attrs([("rx", "10"), ("tabindex", "0"), ("style", "cursor:pointer; outline:none")])?;

    let label = svg.text(Point::new(185.0, 58.0 + PAD_Y), "click, type, or wheel")?;
    label.set_fill(WHITE)?;
    label.set_attrs([
        ("font-size", "15"),
        ("font-weight", "bold"),
        ("text-anchor", "middle"),
        ("style", "pointer-events:none"),
    ])?;

    let readout = svg.text(Point::new(360.0, 58.0 + PAD_Y), "focus: no · key: — · wheel: 0")?;
    readout.set_fill(TEXT)?;
    readout.set_attr("font-size", "14")?;

    let wheel_total = Rc::new(Cell::new(0i32));

    {
        let readout = readout.clone();
        pad.on_focus(move |_| readout.set_text("focus: yes · key: — · wheel: 0"))?;
    }
    {
        let readout = readout.clone();
        pad.on_blur(move |_| readout.set_text("focus: no · key: — · wheel: 0"))?;
    }
    {
        let readout = readout.clone();
        let wheel_total = wheel_total.clone();
        pad.on_keydown(move |e| {
            readout.set_text(&format!("focus: yes · keydown: {} · wheel: {}", e.key(), wheel_total.get(),));
        })?;
    }
    {
        let readout = readout.clone();
        let wheel_total = wheel_total.clone();
        pad.on_keyup(move |e| {
            readout.set_text(&format!("focus: yes · keyup: {} · wheel: {}", e.key(), wheel_total.get(),));
        })?;
    }
    {
        let readout = readout.clone();
        let wheel_total = wheel_total.clone();
        // Wheel events fire rapidly during a continuous scroll/trackpad gesture, so the closure passed to `on_wheel`
        // will genuinely lie on the hot path.  Therefore, it is beneficial to capture a reusable buffer and format
        // into it rather than allocating a fresh String each tick.
        //
        // The discrete focus/blur/keydown/keyup handlers above deliberately keep the simpler `set_text(&format!(...))`
        // idiom since that coding does not lie on a hot path.
        //
        // Buffering is worthwhile for code lying on the hot path, but is not a blanket rule for every handler.
        let mut buf = String::new();
        pad.on_wheel(move |e| {
            e.prevent_default();
            let delta = if e.delta_y() < 0.0 { 1 } else { -1 };
            let next = wheel_total.get() + delta;
            wheel_total.set(next);
            buf.clear();
            let _ = write!(buf, "focus: yes · key: — · wheel: {next}");
            readout.set_text(&buf);
        })?;
    }

    caption(&svg, 400.0, "managed focus/blur · keydown/keyup · wheel with preventDefault()")?;
    keep_demo_node(pad);
    Ok(())
}
