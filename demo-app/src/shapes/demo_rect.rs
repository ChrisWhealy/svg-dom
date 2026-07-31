use std::{cell::Cell, rc::Rc};

use crate::{H, PAD_Y, W, caption, colours::*};
use svg_dom::{
    Error, SvgAttrs, SvgRoot,
    root::utils::{Point, Size},
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// rect
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-rect", Size::new(W, H))?;

    // 1. Plain fill
    let r1 = svg.rect(Point::new(10.0, 10.0 + PAD_Y), Size::new(130.0, 90.0))?;
    r1.set_fill(STEELBLUE)?;
    caption(&svg, 75.0, "fill")?;

    // 2. Stroke-only (no fill)
    let r2 = svg.rect(Point::new(155.0, 10.0 + PAD_Y), Size::new(130.0, 90.0))?;
    let mut attrs = SvgAttrs::new();
    r2.attrs(&mut attrs).fill(NONE)?.stroke(CORAL)?.stroke_width(3.0)?;
    caption(&svg, 220.0, "stroke")?;

    // 3. Rounded corners via rx attribute
    let r3 = svg.rect(Point::new(300.0, 10.0 + PAD_Y), Size::new(130.0, 90.0))?;
    r3.set_fill(MEDIUM_SEA_GREEN)?;
    r3.set_attr("rx", "20")?;
    caption(&svg, 365.0, "rounded (rx)")?;

    // 4. Hover: fill swaps on pointerenter / pointerleave
    // Strong self-captures are intentional here: these demo nodes live for the page lifetime,
    // so the reference cycle is harmless.  In application code prefer `downgrade()`/`upgrade()`.
    let r4 = svg.rect(Point::new(445.0, 10.0 + PAD_Y), Size::new(130.0, 90.0))?;
    r4.set_fill(GOLDENROD)?;
    r4.set_attr("style", "cursor:pointer")?;
    let r4b = r4.clone();
    r4.on_pointerenter(move |_| {
        let _ = r4b.set_fill(GOLD);
    })?;
    let r4c = r4.clone();
    r4.on_pointerleave(move |_| {
        let _ = r4c.set_fill(GOLDENROD);
    })?;
    caption(&svg, 510.0, "hover")?;

    // 5. Click: toggles between two fills
    let r5 = svg.rect(Point::new(590.0, 10.0 + PAD_Y), Size::new(130.0, 90.0))?;
    r5.set_fill(SLATE_GRAY)?;
    r5.set_attr("style", "cursor:pointer")?;
    let toggled = Rc::new(Cell::new(false));
    let r5b = r5.clone();
    r5.on_click(move |_| {
        let next = !toggled.get();
        toggled.set(next);
        let _ = r5b.set_fill(if next { CORAL } else { SLATE_GRAY });
    })?;
    caption(&svg, 655.0, "click (toggle)")?;

    Ok(())
}
