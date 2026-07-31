use std::{cell::RefCell, rc::Rc};

use crate::{H, PAD_Y, W, cached_label, caption, colours::*, keep_demo_node};
use svg_dom::{
    CachedAttr, Error, SvgRoot,
    root::utils::{Point, Size},
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Events — pointer and mouse lifecycle wrappers
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-events-pointer", Size::new(W, H))?;

    let target = svg.rect(Point::new(54.0, 24.0 + PAD_Y), Size::new(230.0, 84.0))?;
    target.set_fill(DROP_ZONE_FILL)?;
    target.set_stroke(ACCENT_BLUE)?;
    target.set_stroke_width(2.0)?;
    target.set_attrs([("rx", "10"), ("style", "cursor:crosshair; touch-action:none")])?;

    let title = svg.text(Point::new(169.0, 58.0 + PAD_Y), "pointer target")?;
    title.set_fill(TEXT)?;
    title.set_attrs([
        ("font-size", "15"),
        ("font-weight", "bold"),
        ("text-anchor", "middle"),
        ("style", "pointer-events:none"),
    ])?;

    let readout = svg.text(Point::new(330.0, 58.0 + PAD_Y), "last: none")?;
    readout.set_fill(TEXT)?;
    readout.set_attr("font-size", "14")?;

    let coords = svg.text(Point::new(330.0, 84.0 + PAD_Y), "move inside the target")?;
    coords.set_fill(TEXT_MUTED)?;
    coords.set_attr("font-size", "12")?;

    // Every "last: ..." readout write goes through one shared CachedAttr: a burst of identical labels (a stream of
    // pointermove events) then skips the DOM write after the first. Routing *all* writers through the same cache is
    // what keeps it from going stale when the event type changes.
    let label_cache = Rc::new(RefCell::new(CachedAttr::new()));

    // Discrete transitions go through the shared cache via the module-level `cached_label` helper.
    target.on_pointerover(cached_label(readout.clone(), label_cache.clone(), "pointerover"))?;
    target.on_pointerenter(cached_label(readout.clone(), label_cache.clone(), "pointerenter"))?;
    target.on_pointerdown(cached_label(readout.clone(), label_cache.clone(), "pointerdown"))?;
    target.on_pointerup(cached_label(readout.clone(), label_cache.clone(), "pointerup"))?;
    target.on_pointercancel(cached_label(readout.clone(), label_cache.clone(), "pointercancel"))?;
    target.on_pointerout(cached_label(readout.clone(), label_cache.clone(), "pointerout"))?;
    target.on_pointerleave(cached_label(readout.clone(), label_cache.clone(), "pointerleave"))?;
    target.on_mouseenter(cached_label(readout.clone(), label_cache.clone(), "mouseenter"))?;
    target.on_mouseleave(cached_label(readout.clone(), label_cache.clone(), "mouseleave"))?;
    target.on_dblclick(cached_label(readout.clone(), label_cache.clone(), "dblclick"))?;

    let move_readout = readout.clone();
    let move_coords = coords.clone();
    // The `last: ...` readout is shared with the discrete handlers above, so its cache stays in an `Rc<RefCell<...>>`.
    // The coordinate buffer is used only here, so this `FnMut` handler can simply own it.
    let move_cache = label_cache.clone();
    let mut coords_buf = String::new();
    target.on_pointermove(move |e| {
        // Constant label through the shared cache: no allocation, and the DOM write is skipped on repeat moves.
        let _ = move_cache.borrow_mut().set_text(&move_readout, "last: pointermove");
        // Coordinates change every move: format them through the owned scratch buffer.
        let _ = move_coords.set_text_fmt(
            &mut coords_buf,
            format_args!(
                "x: {}  y: {}  id: {}  type: {}",
                e.offset_x(),
                e.offset_y(),
                e.pointer_id(),
                e.pointer_type(),
            ),
        );
    })?;

    caption(
        &svg,
        400.0,
        "managed pointerover/enter/down/move/up/cancel/out/leave plus mouseenter/leave/dblclick",
    )?;
    keep_demo_node(target);
    Ok(())
}
