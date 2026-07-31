use std::cell::Cell;

use crate::{H, PAD_Y, W, caption, colours::*, keep_demo_node};
use svg_dom::{
    Error, SvgAttrs, SvgRoot,
    root::utils::{Point, Size},
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Events — colour wheel (managed on_pointermove drives a second element's fill)
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    // A single transparent rect on *top* of everything captures the pointer, and every decoration below it carries
    // `pointer-events:none`.  That keeps exactly one element under the pointer at all times, so the cursor never
    // flickers as it moves (the earlier crosshair version flickered because the moving guides stole hover from the
    // surface).
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
    // `Rc<RefCell<...>>`, no runtime borrow on every `pointermove`. `SvgAttrs` formats the attributes; a scratch
    // `String` backs the readout text.
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
