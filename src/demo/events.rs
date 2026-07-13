use std::{
    cell::{Cell, RefCell},
    fmt::Write,
    rc::Rc,
};

use crate::{CachedAttr, Error, SvgAttrs, SvgNode, SvgRoot, root::utils::{Point, Size}};
use super::colours::*;
use super::{W, H, PAD_Y, caption, cached_label, keep_demo_node};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Events — click counter + reset button (two on_click handlers over shared state)
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
//
// Counting and resetting live on two *separate* buttons on purpose.  A "double-click to reset" on the counter itself
// would misbehave: the browser always fires two `click` events before a `dblclick`, so any quick pair of clicks would
// increment twice and then immediately reset to zero.
pub(super) fn demo_events_click() -> Result<(), Error> {
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

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Events — colour wheel (managed on_pointermove drives a second element's fill)
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
//
// A single transparent rect on *top* of everything captures the pointer, and every decoration below it carries
// `pointer-events:none`.  That keeps exactly one element under the pointer at all times, so the cursor never flickers
// as it moves (the earlier crosshair version flickered because the moving guides stole hover from the surface).
pub(super) fn demo_events_colour() -> Result<(), Error> {
    const CX: f64 = 90.0; // wheel centre (horizontal)
    const CY: f64 = H / 2.0; // wheel centre (vertical) — middle of the canvas
    const R: f64 = H / 2.0 - 13.0; // wheel radius, scaled to the canvas height (keeps a ~13px margin)
    const STEP: f64 = 2.0; // angular width of each wedge, in degrees

    let svg = SvgRoot::create_in("demo-events-colour", Size::new(W, H))?;

    // The wheel is built from thin pie wedges, each filled with its own hue.  Grouping them lets a single
    // `pointer-events:none` on the <g> apply to every wedge at once.
    let wheel = svg.group()?;
    wheel.set_attr("pointer-events", NONE)?;

    // Build all ~180 wedges straight into the <g> through a detached fragment, committed in one DOM operation.
    // Creating each with `svg.path(...)` and then `wheel.append(...)` would instead append every wedge to the live
    // root and immediately move it into the group — a lot of avoidable setup-time DOM churn.
    svg.build_batch_into(&wheel, |b| {
        let mut a: f64 = 0.0;
        while a < 360.0 {
            let (r0, r1) = (a.to_radians(), (a + STEP).to_radians());
            let wedge = b.path(&format!(
                "M {CX} {CY} L {:.2} {:.2} A {R} {R} 0 0 1 {:.2} {:.2} Z",
                CX + R * r0.cos(),
                CY + R * r0.sin(),
                CX + R * r1.cos(),
                CY + R * r1.sin(),
            ))?;
            wedge.set_fill(&format!("hsl({:.0},90%,50%)", a + STEP / 2.0))?;
            a += STEP;
        }
        Ok(())
    })?;

    // A hollow ring that marks the sampled point on the wheel; parked off-canvas until the pointer arrives.
    let marker = svg.circle(Point::new(-20.0, -20.0), 6.0)?;
    marker.set_fill(NONE)?;
    marker.set_stroke(WHITE)?;
    marker.set_stroke_width(2.0)?;
    marker.set_attr("pointer-events", NONE)?;

    // The "second object": its fill follows whatever hue the pointer is over.
    let swatch = svg.rect(Point::new(210.0, 18.0 + PAD_Y), Size::new(250.0, 94.0))?;
    swatch.set_fill(SWATCH_EMPTY)?;
    swatch.set_stroke(GUIDE)?;
    swatch.set_attrs([("rx", "12"), ("pointer-events", NONE)])?;

    let readout = svg.text(Point::new(485.0, 70.0 + PAD_Y), "move over the wheel →")?;
    readout.set_fill(TEXT)?;
    readout.set_attrs([("font-size", "15"), ("pointer-events", NONE)])?;

    caption(
        &svg,
        450.0,
        "Managed on_pointermove over the wheel sets the swatch fill (hue from pointer angle)",
    )?;

    // The pointer-capture surface goes on last so it sits on top of everything above. `touch-action:none` lets a
    // finger-drag sample the wheel instead of scrolling the page, so the pointer handler works for touch and pen too.
    let surface = svg.rect(Point::origin(), Size::new(W, H))?;
    surface.set_fill(TRANSPARENT)?;
    surface.set_attr("style", "cursor:crosshair; touch-action:none")?;

    let mv_marker = marker.clone();
    let mv_swatch = swatch.clone();
    let mv_readout = readout.clone();

    // Managed handlers are `FnMut`, so this per-move handler can *own* its reusable buffers directly — no
    // `Rc<RefCell<...>>`, no runtime borrow on every `pointermove`. `SvgAttrs` formats the attributes; a scratch `String`
    // backs the readout text.
    let mut attrs = SvgAttrs::new();
    let mut text = String::new();
    // Avoids writing cx/cy on every outside-wheel event when the marker is already parked.
    let parked = Cell::new(false);

    surface.on_pointermove(move |e| {
        let (x, y) = (f64::from(e.offset_x()), f64::from(e.offset_y()));
        let (dx, dy) = (x - CX, y - CY);

        if dx * dx + dy * dy <= R * R {
            // Inside the wheel: hue is the pointer's angle about the centre.
            parked.set(false);
            let hue = (dy.atan2(dx).to_degrees() + 360.0) % 360.0;
            let _ = attrs.fmt(&mv_swatch, "fill", format_args!("hsl({hue:.0},90%,50%)"));
            let _ = attrs.fmt(&mv_marker, "cx", format_args!("{x:.1}"));
            let _ = attrs.fmt(&mv_marker, "cy", format_args!("{y:.1}"));
            let _ = mv_readout.set_text_fmt(&mut text, format_args!("hsl({hue:.0},90%,50%)"));
        } else if !parked.get() {
            // Outside the wheel: park the marker once; skip the DOM writes on subsequent outside events.
            parked.set(true);
            let _ = attrs.set(&mv_marker, "cx", "-20");
            let _ = attrs.set(&mv_marker, "cy", "-20");
        }
    })?;

    keep_demo_node(surface);
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Events — modifier keys (on_click) + right-click (on_contextmenu, preventDefault)
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(super) fn demo_events_modifiers() -> Result<(), Error> {
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

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Events — press state (managed mousedown / mouseup / pointerleave)
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(super) fn demo_events_press() -> Result<(), Error> {
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

    // A context menu (right-click, or ctrl+click on macOS) interrupts the gesture consuming the mouseup event, so treat
    // it as a release so the button can never get stuck in the pressed state, whatever the platform.
    let release_ctx = release.clone();
    pad.on_contextmenu(move |_| release_ctx())?;

    // If the pointer leaves while still held, treat it as a release so the button cannot get stuck in the pressed state
    pad.on_pointerleave(move |_| release())?;

    caption(
        &svg,
        400.0,
        "managed mousedown / mouseup / pointerleave · pressed-state tracking · reports held modifier keys",
    )?;
    keep_demo_node(pad);
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Events — pointerenter wrappers on groups
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(super) fn demo_events_group() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-events-group", Size::new(W, H))?;

    // Builds a bordered container group holding two child shapes, translated to `x`.  The filled background rect is
    // the group's visible boundary AND its hit area: a <g> has no geometry of its own, so without a filled background
    // the gaps between the children would not count as "inside" the group.  Its fill matches the canvas colour so
    // only the coloured border shows.  Returns the group node.
    let build = |x: f64, border: &str| -> Result<SvgNode, Error> {
        let g = svg.group()?;

        // Build the boundary and children straight into the <g> via a detached fragment, instead of creating them on
        // the root and re-parenting each one with append.
        svg.build_batch_into(&g, |b| {
            let boundary = b.rect(Point::new(0.0, 0.0), Size::new(300.0, 80.0))?;
            boundary.set_fill(CANVAS_BG)?; // == canvas background, so only the stroke is visible
            boundary.set_stroke(border)?;
            boundary.set_stroke_width(2.0)?;
            boundary.set_attr("rx", "8")?;

            let child_a = b.circle(Point::new(75.0, 40.0), 22.0)?;
            child_a.set_fill(LEAF_ORANGE)?;

            let child_b = b.rect(Point::new(160.0, 18.0), Size::new(110.0, 44.0))?;
            child_b.set_fill(LEAF_GREEN)?;
            child_b.set_attr("rx", "4")?;
            Ok(())
        })?;
        g.set_attr("transform", &format!("translate({x}, {})", 26.0 + PAD_Y))?;
        Ok(g)
    };

    // Builds the title + counter labels for a group and returns the (clonable) counter text node.
    let labels = |x: f64, colour: &str, title: &str| -> Result<SvgNode, Error> {
        let t = svg.text(Point::new(x, 18.0 + PAD_Y), title)?;
        t.set_fill(colour)?;
        t.set_attr("font-size", "12")?;
        let count = svg.text(Point::new(x, 124.0 + PAD_Y), "fires: 0")?;
        count.set_fill(TEXT)?;
        count.set_attr("font-size", "14")?;
        Ok(count)
    };

    // group 1 — on_pointerenter. This uses the non-bubbling pointerenter event, so the handler fires once when the
    // pointer enters the group boundary and ignores child-to-child movement inside the group.
    let g1_count = labels(40.0, ACCENT_BLUE, "group 1: on_pointerenter")?;
    let group1 = build(40.0, ACCENT_BLUE)?;
    let c1 = Rc::new(Cell::new(0u32));
    group1.on_pointerenter(move |_| {
        let n = c1.get() + 1;
        c1.set(n);
        g1_count.set_text(&format!("fires: {n}"));
    })?;
    // Managed listeners are removed when their owning SvgNode is dropped, so keep this interactive node alive for the
    // page lifetime.
    keep_demo_node(group1);

    // group 2 — the same wrapper on another group, showing the behaviour is independent of the child shapes.
    let g2_count = labels(440.0, ACCENT_AMBER, "group 2: on_pointerenter")?;
    let group2 = build(440.0, ACCENT_AMBER)?;
    let c2 = Rc::new(Cell::new(0u32));
    group2.on_pointerenter(move |_| {
        let n = c2.get() + 1;
        c2.set(n);
        g2_count.set_text(&format!("fires: {n}"));
    })?;
    // Managed listeners are removed when their owning SvgNode is dropped, so keep this interactive node alive for the
    // page lifetime.
    keep_demo_node(group2);

    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Events — pointer and mouse lifecycle wrappers
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(super) fn demo_events_pointer_lifecycle() -> Result<(), Error> {
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

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Events — keyboard, focus and wheel wrappers
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(super) fn demo_events_keyboard_wheel() -> Result<(), Error> {
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

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Events — browser drag/drop, touch and generic Event wrappers
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(super) fn demo_events_drag_drop_touch() -> Result<(), Error> {
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
    // pointer listeners only fire when one of its children is hittable — hence the card background must NOT opt out of
    // pointer events.  Only the text label below carries `pointer-events:none`.
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
    // Repeated attempts to write the same label text do not touch the DOM after the first write. Routing *all* writers
    // through the same cache is what keeps it coherent; partial caching (some writers bypassing it) could lead to it
    // skipping a needed write.
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

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Events — passive wheel, touchstart, and touchmove listeners
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(super) fn demo_events_passive() -> Result<(), Error> {
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
