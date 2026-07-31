use std::{cell::Cell, rc::Rc};

use crate::{H, PAD_Y, W, colours::*, keep_demo_node};
use svg_dom::{
    Error, SvgNode, SvgRoot,
    root::utils::{Point, Size},
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Events — non-bubbling pointerenter vs bubbling mouseover, on identically-shaped groups
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-events-group", Size::new(W, H))?;

    // Builds a bordered container group holding two child shapes, translated to `x`.  The filled background rect is
    // the group's visible boundary AND its hit area: a <g> has no geometry of its own, so without a filled
    // background the gaps between the children would not count as "inside" the group.  Its fill matches the canvas
    // colour so only the coloured border shows.  Returns the group node.
    let build = |x: f64, border: &str| -> Result<SvgNode, Error> {
        let g = svg.group()?;

        // Build the boundary and children straight into the <g> via a detached fragment, instead of creating them
        // on the root and re-parenting each one with append.
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
    // Managed listeners are removed when their owning SvgNode is dropped, so keep this interactive node alive for
    // the page lifetime.
    keep_demo_node(group1);

    // group 2 — the same shape, but wired to the bubbling on_mouseover event instead. Unlike group 1's
    // pointerenter, mouseover re-fires on every child-to-child crossing inside the group: entering child A bubbles
    // a mouseover up from the group's boundary, and then crossing from child A to child B fires *another*
    // mouseover (child A's mouseout is immediately followed by child B's mouseover, both bubbling up to the same
    // group handler) even though the pointer never left the group. That is the entire contrast this demo exists to
    // show — group 1's counter only rises on a genuine group-boundary crossing, group 2's rises on every child
    // boundary crossed as well.
    //
    // on_mouseover is `#[deprecated]` in favour of on_pointerenter precisely because of this bubbling quirk; it is
    // used here deliberately, as the demonstration of the quirk it warns about, not as a recommendation to prefer
    // it for real hover handling.
    let g2_count = labels(440.0, ACCENT_AMBER, "group 2: on_mouseover (bubbles)")?;
    let group2 = build(440.0, ACCENT_AMBER)?;
    let c2 = Rc::new(Cell::new(0u32));
    #[allow(deprecated)]
    group2.on_mouseover(move |_| {
        let n = c2.get() + 1;
        c2.set(n);
        g2_count.set_text(&format!("fires: {n}"));
    })?;
    // Managed listeners are removed when their owning SvgNode is dropped, so keep this interactive node alive for
    // the page lifetime.
    keep_demo_node(group2);

    Ok(())
}
