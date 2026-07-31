use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use crate::{H, PAD_Y, W, cached_label, caption, colours::*, keep_demo_node};
use svg_dom::{
    CachedAttr, Error, SvgRoot,
    root::utils::{Point, Size},
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Events — browser drag/drop, touch and generic Event wrappers
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    const CARD_W: f64 = 130.0;
    const CARD_H: f64 = 58.0;
    const MIN_X: f64 = 34.0;
    const MAX_X: f64 = 600.0;
    const MIN_Y: f64 = 24.0 + PAD_Y;
    const MAX_Y: f64 = 96.0 + PAD_Y;

    // Drop-zone bounds — shared by the rect that draws it and the drop test that decides whether the card stays put.
    const ZONE_X: f64 = 245.0;
    const ZONE_Y: f64 = 24.0 + PAD_Y;
    const ZONE_W: f64 = 220.0;
    const ZONE_H: f64 = 84.0;

    let svg = SvgRoot::create_in("demo-events-drag-drop-touch", Size::new(W, H))?;

    let zone = svg.rect(Point::new(ZONE_X, ZONE_Y), Size::new(ZONE_W, ZONE_H))?;
    zone.set_fill(DROP_ZONE_FILL)?;
    zone.set_stroke(DROP_ZONE_BORDER)?;
    zone.set_stroke_width(2.0)?;
    zone.set_attrs([("rx", "10"), ("stroke-dasharray", "6 4")])?;

    let zone_label = svg.text(Point::new(355.0, 72.0 + PAD_Y), "native drop zone")?;
    zone_label.set_fill(TEXT_MUTED)?;
    zone_label.set_attrs([
        ("font-size", "14"),
        ("text-anchor", "middle"),
        ("style", "pointer-events:none; user-select:none"),
    ])?;

    // The blue rectangle is the hit target for the drag gesture.  A <g> has no geometry of its own, so the group's
    // pointer listeners only fire when one of its children is hittable — hence the card background must NOT opt out
    // of pointer events.  Only the text label below carries `pointer-events:none`.
    let card = svg.group()?;
    svg.build_batch_into(&card, |b| {
        let card_bg = b.rect(Point::origin(), Size::new(CARD_W, CARD_H))?;
        card_bg.set_fill(ACCENT_BLUE)?;
        card_bg.set_attr("rx", "8")?;

        let card_label = b.text(Point::new(CARD_W / 2.0, CARD_H / 2.0 + 5.0), "drag / touch")?;
        card_label.set_fill(INK)?;
        card_label.set_attrs([
            ("font-size", "13"),
            ("font-weight", "bold"),
            ("text-anchor", "middle"),
            ("style", "pointer-events:none; user-select:none"),
        ])?;
        Ok(())
    })?;
    card.set_attrs([("style", "cursor:grab; touch-action:none; user-select:none")])?;

    let start = (50.0, 36.0 + PAD_Y);
    card.set_attr("transform", &format!("translate({:.1}, {:.1})", start.0, start.1))?;

    let readout = svg.text(Point::new(500.0, 48.0 + PAD_Y), "last: none")?;
    readout.set_fill(TEXT)?;
    readout.set_attr("font-size", "14")?;

    // Every "last: ..." readout write (the hot pointermove/touchmove/dragover streams *and* the discrete handlers
    // below) goes through this one shared CachedAttr as the pointer-lifecycle demo does.
    //
    // Repeated attempts to write the same label text do not touch the DOM after the first write. Routing *all*
    // writers through the same cache is what keeps it coherent; partial caching (some writers bypassing it) could
    // lead to it skipping a needed write.
    let label_cache = Rc::new(RefCell::new(CachedAttr::new()));

    let coords = svg.text(Point::new(500.0, 74.0 + PAD_Y), &format!("box: {:.0}, {:.0}", start.0, start.1))?;
    coords.set_fill(TEXT_MUTED)?;
    coords.set_attr("font-size", "12")?;

    let pos = Rc::new(Cell::new(start));
    let last_pointer: Rc<Cell<Option<(i32, i32)>>> = Rc::new(Cell::new(None));

    {
        let listener = card.clone();
        let card = card.clone();
        let readout = readout.clone();
        let label_cache = label_cache.clone();
        let last_pointer = last_pointer.clone();
        listener.on_pointerdown(move |e| {
            e.prevent_default();
            let _ = card.as_element().set_pointer_capture(e.pointer_id());
            last_pointer.set(Some((e.client_x(), e.client_y())));
            let _ = card.set_attr("style", "cursor:grabbing; touch-action:none; user-select:none");
            let _ = label_cache.borrow_mut().set_text(&readout, "last: pointerdown — moving box");
        })?;
    }

    {
        let listener = card.clone();
        let card = card.clone();
        let coords = coords.clone();
        let readout = readout.clone();
        let label_cache = label_cache.clone();
        let pos = pos.clone();
        let last_pointer = last_pointer.clone();
        let mut scratch = String::new();
        listener.on_pointermove(move |e| {
            if let Some((last_x, last_y)) = last_pointer.get() {
                e.prevent_default();
                let dx = f64::from(e.client_x() - last_x);
                let dy = f64::from(e.client_y() - last_y);
                let (x, y) = pos.get();
                let nx = (x + dx).clamp(MIN_X, MAX_X);
                let ny = (y + dy).clamp(MIN_Y, MAX_Y);
                pos.set((nx, ny));
                last_pointer.set(Some((e.client_x(), e.client_y())));
                let _ = card.set_translate(&mut scratch, nx, ny);
                let _ = coords.set_text_fmt(&mut scratch, format_args!("box: {nx:.0}, {ny:.0}"));
                // Constant label through the shared cache: the DOM write is skipped on repeated moves.
                let _ = label_cache.borrow_mut().set_text(&readout, "last: pointermove — moving box");
            }
        })?;
    }

    {
        let listener = card.clone();
        let card = card.clone();
        let readout = readout.clone();
        let label_cache = label_cache.clone();
        let coords = coords.clone();
        let pos = pos.clone();
        let last_pointer = last_pointer.clone();
        let mut scratch = String::new();
        let finish = move |e: web_sys::PointerEvent| {
            e.prevent_default();
            let _ = card.as_element().release_pointer_capture(e.pointer_id());
            last_pointer.set(None);
            let _ = card.set_attr("style", "cursor:grab; touch-action:none; user-select:none");

            // The card only counts as dropped if it is *fully* inside the zone; otherwise it snaps back to its
            // original position.
            let (x, y) = pos.get();
            let fully_inside =
                x >= ZONE_X && x + CARD_W <= ZONE_X + ZONE_W && y >= ZONE_Y && y + CARD_H <= ZONE_Y + ZONE_H;

            if fully_inside {
                let _ = label_cache.borrow_mut().set_text(&readout, "last: pointerup — dropped in zone");
            } else {
                pos.set(start);
                let _ = card.set_translate(&mut scratch, start.0, start.1);
                let _ = coords.set_text_fmt(&mut scratch, format_args!("box: {:.0}, {:.0}", start.0, start.1));
                let _ = label_cache
                    .borrow_mut()
                    .set_text(&readout, "last: pointerup — outside zone, returned to start");
            }
        };
        listener.on_pointerup(finish)?;
    }

    {
        let listener = card.clone();
        let card = card.clone();
        let readout = readout.clone();
        let label_cache = label_cache.clone();
        let last_pointer = last_pointer.clone();
        listener.on_pointercancel(move |e| {
            let _ = card.as_element().release_pointer_capture(e.pointer_id());
            last_pointer.set(None);
            let _ = card.set_attr("style", "cursor:grab; touch-action:none; user-select:none");
            let _ = label_cache.borrow_mut().set_text(&readout, "last: pointercancel");
        })?;
    }

    // The blue card is moved using pointer events because native browser drag/drop reports a DragEvent but does not
    // reposition SVG content for you.  These DragEvent wrappers are still attached so the demo logs any native drag
    // events a browser chooses to emit for the element.
    card.on_dragstart(cached_label(readout.clone(), label_cache.clone(), "dragstart"))?;
    card.on_drag(cached_label(readout.clone(), label_cache.clone(), "drag"))?;
    card.on_dragend(cached_label(readout.clone(), label_cache.clone(), "dragend"))?;
    {
        let readout = readout.clone();
        let label_cache = label_cache.clone();
        card.on_touchstart(move |e| {
            e.prevent_default();
            let _ = label_cache.borrow_mut().set_text(&readout, "last: touchstart");
        })?;
    }
    {
        let readout = readout.clone();
        let label_cache = label_cache.clone();
        card.on_touchmove(move |e| {
            e.prevent_default();
            let _ = label_cache.borrow_mut().set_text(&readout, "last: touchmove");
        })?;
    }
    card.on_touchend(cached_label(readout.clone(), label_cache.clone(), "touchend"))?;
    card.on_touchcancel(cached_label(readout.clone(), label_cache.clone(), "touchcancel"))?;

    zone.on_dragenter(cached_label(readout.clone(), label_cache.clone(), "dragenter"))?;
    zone.on_dragleave(cached_label(readout.clone(), label_cache.clone(), "dragleave"))?;
    {
        let readout = readout.clone();
        let label_cache = label_cache.clone();
        zone.on_dragover(move |e| {
            e.prevent_default();
            let _ = label_cache.borrow_mut().set_text(&readout, "last: dragover (drop enabled)");
        })?;
    }
    {
        let readout = readout.clone();
        let label_cache = label_cache.clone();
        zone.on_drop(move |e| {
            e.prevent_default();
            let _ = label_cache.borrow_mut().set_text(&readout, "last: drop");
        })?;
    }

    // Generic Event wrapper: auxclick is deliberately handled as a plain Event, proving that callers are not forced
    // back to raw Closure management when a typed convenience method is absent.
    card.on_event(
        "auxclick",
        cached_label(readout.clone(), label_cache.clone(), "generic auxclick"),
    )?;

    caption(
        &svg,
        400.0,
        "managed pointer drag moves the box · touch wrappers prevent scrolling · drag/drop wrappers are logged",
    )?;
    keep_demo_node(card);
    keep_demo_node(zone);
    Ok(())
}
