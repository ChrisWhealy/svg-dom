use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use crate::{H, W, caption, colours::*, keep_demo_node};
use svg_dom::{
    Error, SvgNode, SvgRoot,
    root::utils::{Point, Size},
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Tree navigation — step through first_child/next_sibling on click, query_selector on demand
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// A static, build-once render of a tree walk is over before a viewer can follow what happened — every bar is already
// coloured and the outline is already there, so there is nothing left to observe. Driving each step from a click
// instead makes the traversal itself the thing being shown: one click is one `next_sibling()` call, visibly.
pub(crate) fn demo() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-tree-nav", Size::new(W, H))?;

    // Bar heights for a small skyline; index 3 is tagged data-role="target" so query_selector has something
    // distinctive to find later. Kept short so the row, the readout, and three buttons all fit above the caption.
    let heights: [f64; 7] = [18.0, 32.0, 26.0, 48.0, 30.0, 40.0, 22.0];
    let target_index = 3usize;
    let bar_w = 60.0_f64;
    let gap = 14.0_f64;
    let total_w = heights.len() as f64 * bar_w + (heights.len() - 1) as f64 * gap;
    let x0 = (W - total_w) / 2.0;
    let base_y = 78.0_f64; // shared baseline every bar grows up from

    // Every bar is built inside one <g> via a single batch; only the group handle is kept afterwards, not a handle
    // per bar — the point of this demo is that the bars can still be found and styled without one.
    let group = svg.group()?;
    svg.build_batch_into(&group, |b| {
        for (i, h) in heights.iter().enumerate() {
            let bar = b.rect(Point::new(x0 + i as f64 * (bar_w + gap), base_y - h), Size::new(bar_w, *h))?;
            bar.set_fill(TEXT_MUTED)?;
            if i == target_index {
                bar.set_attr("data-role", "target")?;
            }
        }
        Ok(())
    })?;

    let readout = svg.text(
        Point::new(W / 2.0, 96.0),
        "click Walk to step through first_child, then next_sibling",
    )?;
    readout.set_fill(TEXT)?;
    readout.set_attrs([("font-size", "13"), ("text-anchor", "middle")])?;

    // A small helper so the three buttons below share one layout — a pointer-styled rounded rect plus a centred,
    // click-through label, the same two-element shape `demo_events_click`'s buttons use.
    let button = |x: f64, w: f64, label: &str, fill: &str| -> Result<SvgNode, Error> {
        let rect = svg.rect(Point::new(x, 100.0), Size::new(w, 26.0))?;
        rect.set_fill(fill)?;
        rect.set_attrs([("rx", "6"), ("style", "cursor:pointer")])?;
        let text = svg.text(Point::new(x + w / 2.0, 117.0), label)?;
        text.set_fill(WHITE)?;
        text.set_attrs([("font-size", "13"), ("text-anchor", "middle"), ("style", "pointer-events:none")])?;
        Ok(rect)
    };

    let btns_w = 110.0 + 150.0 + 90.0 + 2.0 * gap;
    let bx0 = (W - btns_w) / 2.0;
    let walk_btn = button(bx0, 110.0, "Walk →", STEELBLUE)?;
    let find_btn = button(bx0 + 110.0 + gap, 150.0, "Find target", ACCENT_AMBER)?;
    let reset_btn = button(bx0 + 110.0 + 150.0 + 2.0 * gap, 90.0, "Reset", RESET_IDLE)?;

    // `cursor` tracks the last-visited bar (`None` means the walk has not started, or was just reset), so each click
    // knows whether to call `first_child()` or `next_sibling()` on the one before it.
    let cursor: Rc<RefCell<Option<SvgNode>>> = Rc::new(RefCell::new(None));
    let step = Rc::new(Cell::new(0usize));
    let bar_count = heights.len();

    let walk_group = group.clone();
    let walk_readout = readout.clone();
    let walk_cursor = cursor.clone();
    let walk_step = step.clone();
    walk_btn.on_click(move |_| {
        let next = match walk_cursor.borrow().as_ref() {
            Some(bar) => bar.next_sibling().map(|n| (n, "next_sibling()")),
            None => walk_group.first_child().map(|n| (n, "first_child()")),
        };
        match next {
            Some((bar, method)) => {
                let i = walk_step.get();
                let hue = 360.0 * i as f64 / bar_count as f64;
                let _ = bar.set_fill(&format!("hsl({hue:.0},60%,55%)"));
                walk_readout.set_text(&format!("visiting bar {} via {method}", i + 1));
                walk_step.set(i + 1);
                *walk_cursor.borrow_mut() = Some(bar);
            },
            None => walk_readout.set_text("next_sibling() returned None — walk finished; click Reset to replay"),
        }
    })?;

    let find_group = group.clone();
    let find_readout = readout.clone();
    find_btn.on_click(move |_| match find_group.query_selector("[data-role='target']") {
        Ok(Some(bar)) => {
            let _ = bar.set_stroke(GOLD);
            let _ = bar.set_stroke_width(4.0);
            find_readout.set_text("found via query_selector(\"[data-role='target']\")");
        },
        _ => find_readout.set_text("query_selector found no match"),
    })?;

    // Reset walks the bars again — via the same first_child/next_sibling pair the Walk button uses — purely to
    // restore each one's resting colour and clear the outline, then rewinds the walk state to the start.
    let reset_group = group.clone();
    let reset_readout = readout.clone();
    let reset_cursor = cursor.clone();
    let reset_step = step.clone();
    reset_btn.on_click(move |_| {
        let mut current = reset_group.first_child();
        while let Some(bar) = current {
            let _ = bar.set_fill(TEXT_MUTED);
            let _ = bar.remove_attr("stroke");
            let _ = bar.remove_attr("stroke-width");
            current = bar.next_sibling();
        }
        *reset_cursor.borrow_mut() = None;
        reset_step.set(0);
        reset_readout.set_text("click Walk to step through first_child, then next_sibling");
    })?;

    caption(
        &svg,
        W / 2.0,
        "Walk steps through first_child → next_sibling one bar per click; Find target runs query_selector",
    )?;

    keep_demo_node(walk_btn);
    keep_demo_node(find_btn);
    keep_demo_node(reset_btn);
    Ok(())
}
