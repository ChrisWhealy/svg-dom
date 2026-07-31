use std::{
    cell::{Cell, RefCell},
    fmt::Write,
    rc::Rc,
};

use crate::{H, PAD_Y, W, cached_label, caption, colours::*, keep_demo_node};
use svg_dom::{
    CachedAttr, Error, SvgRoot,
    root::utils::{Point, Size},
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Events — passive wheel, touchstart, and touchmove listeners
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-events-passive", Size::new(W, H))?;

    let box_y = PAD_Y + 5.0;
    let box_h = 100.0;
    let box_w = 360.0;
    let left_cx = 20.0 + box_w / 2.0; // 200
    let right_cx = 420.0 + box_w / 2.0; // 600

    // --- Passive wheel ---
    let wheel_area = svg.rect(Point::new(20.0, box_y), Size::new(box_w, box_h))?;
    wheel_area.set_fill(SLATE_BLUE)?;
    wheel_area.set_attrs([("rx", "8"), ("style", "cursor:ns-resize")])?;

    let wh_name = svg.text(Point::new(left_cx, box_y + 25.0), "on_wheel_passive")?;
    wh_name.set_fill(WHITE)?;
    wh_name.set_attrs([
        ("font-size", "14"),
        ("font-weight", "bold"),
        ("text-anchor", "middle"),
        ("style", "pointer-events:none"),
    ])?;

    let wh_hint = svg.text(Point::new(left_cx, box_y + 47.0), "wheel here — page scroll is not blocked")?;
    wh_hint.set_fill(WHITE)?;
    wh_hint.set_attrs([("font-size", "11"), ("text-anchor", "middle"), ("style", "pointer-events:none")])?;

    let wh_readout = svg.text(Point::new(left_cx, box_y + 74.0), "delta: 0")?;
    wh_readout.set_fill(WHITE)?;
    wh_readout.set_attrs([("font-size", "15"), ("text-anchor", "middle"), ("style", "pointer-events:none")])?;

    let wh_total = Rc::new(Cell::new(0i32));
    let wh_read = wh_readout.clone();
    let mut wh_buf = String::new();
    wheel_area.on_wheel_passive(move |e| {
        let n = wh_total.get() + if e.delta_y() < 0.0 { 1 } else { -1 };
        wh_total.set(n);
        wh_buf.clear();
        let _ = write!(wh_buf, "delta: {n}");
        wh_read.set_text(&wh_buf);
    })?;

    // --- Passive touch ---
    let touch_area = svg.rect(Point::new(420.0, box_y), Size::new(box_w, box_h))?;
    touch_area.set_fill(TEAL)?;
    touch_area.set_attrs([("rx", "8")])?;

    let tc_name1 = svg.text(Point::new(right_cx, box_y + 20.0), "on_touchstart_passive")?;
    tc_name1.set_fill(WHITE)?;
    tc_name1.set_attrs([
        ("font-size", "13"),
        ("font-weight", "bold"),
        ("text-anchor", "middle"),
        ("style", "pointer-events:none"),
    ])?;

    let tc_name2 = svg.text(Point::new(right_cx, box_y + 38.0), "on_touchmove_passive")?;
    tc_name2.set_fill(WHITE)?;
    tc_name2.set_attrs([
        ("font-size", "13"),
        ("font-weight", "bold"),
        ("text-anchor", "middle"),
        ("style", "pointer-events:none"),
    ])?;

    let tc_hint = svg.text(
        Point::new(right_cx, box_y + 58.0),
        "touch here (mobile) — scroll is not blocked",
    )?;
    tc_hint.set_fill(WHITE)?;
    tc_hint.set_attrs([("font-size", "11"), ("text-anchor", "middle"), ("style", "pointer-events:none")])?;

    let tc_readout = svg.text(Point::new(right_cx, box_y + 80.0), "last: none")?;
    tc_readout.set_fill(WHITE)?;
    tc_readout.set_attrs([("font-size", "15"), ("text-anchor", "middle"), ("style", "pointer-events:none")])?;

    let tc_cache = Rc::new(RefCell::new(CachedAttr::new()));
    touch_area.on_touchstart_passive(cached_label(tc_readout.clone(), tc_cache.clone(), "touchstart"))?;
    touch_area.on_touchmove_passive(cached_label(tc_readout, tc_cache, "touchmove"))?;

    caption(
        &svg,
        400.0,
        "passive listeners do not block browser scroll or touch · prevent_default() inside is silently ignored",
    )?;

    keep_demo_node(wheel_area);
    keep_demo_node(touch_area);
    Ok(())
}
