use crate::{BAND, H, PAD_Y, W, caption, colours::*};
use svg_dom::{
    Error, SvgRoot,
    root::utils::{Point, Size},
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// image — embed a raster or SVG image
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-image", Size::new(W, H))?;

    // A 60×40 four-quadrant colour grid, embedded as a base64 PNG data URI (not SVG — see below).
    // The 3:2 source aspect ratio differs from the 1:1 display boxes below, making the three
    // preserveAspectRatio modes visually distinct.
    //
    // This has to be a raster image, not an embedded SVG: Chromium does not clip a nested-SVG
    // `<image>` source correctly under preserveAspectRatio="slice" — the overflow that "slice" is
    // supposed to crop away stays visible, so it ends up looking identical to "meet". The same
    // "slice" value clips correctly once the source is a raster format instead, which is what this
    // PNG (generated from four filled rects plus a circle, not hand-drawn) sidesteps the bug.
    //
    // The small white/black dot sits at x=5, just inside the 60-wide source's left edge. It is
    // what makes "slice" visually distinguishable from "none": both scale the source up to a
    // different size, but because the grid's colour boundaries sit exactly on the source's
    // centre line, a symmetric centre-crop maps that centre back onto itself — so "none" and
    // "slice" would otherwise land the quadrant colours in identical positions. The off-centre
    // dot breaks that symmetry: "slice" crops the source to its central 40 units (x=10..50) and
    // the dot falls outside that window, so it disappears; "meet" and "none" show the whole
    // source, so the dot stays visible in both.
    const SRC: &str = "data:image/png;base64,\
        iVBORw0KGgoAAAANSUhEUgAAADwAAAAoCAIAAAAt2Q6oAAAAfklEQVR42u3UwQmAMBBE0S3Ms2db\
        sQLBlrQXy1kFQQIhgiYTovzhn8O7bKyfV1E+DaIMNGjQoEGDBg0a9H/RFq119Kn0YEXcQnQsLuXW\
        oj0x0BXRvr2vW0ZRn0XfHGKj6OP11JeXI5ajL3e4THENtCLQoEGDBg0aNGjQoJ+0A+7MRi7Hd5r9\
        AAAAAElFTkSuQmCC";

    // Alternative image: slate-blue with a centred white circle (used in the set_href demo slot).
    // PNG for the same nested-SVG-in-<image> reason as SRC above.
    const ALT: &str = "data:image/png;base64,\
        iVBORw0KGgoAAAANSUhEUgAAADwAAAAoCAIAAAAt2Q6oAAAAfElEQVR42u3YwQ2AMAxDUc/JCOza\
        NZDYgCML0EsT0qb5kgd4t8TWebR0EWjQoLdD39fzmUXRPe4fdMVwfekKFru4FS+2uzVFbHRrltji\
        LoP2Eg+7a6B9xWNu0KBBgwbNRVwCzZdHCdizbmUttlknhKxjTeJZjNUUNOhi6BfLEZpQ5CIbXgAA\
        AABJRU5ErkJggg==";

    // 100×100 px square display boxes; the 3:2 source makes preserveAspectRatio effects clear.
    let img_w = 100.0_f64;
    let img_h = 100.0_f64;
    let y0 = PAD_Y + (BAND - img_h) / 2.0;
    let xs: [f64; 4] = [80.0, 250.0, 420.0, 590.0];

    // Thin guide outline showing each image's bounding box.
    let slot = |x: f64| -> Result<(), Error> {
        let r = svg.rect(Point::new(x, y0), Size::new(img_w, img_h))?;
        r.set_fill(NONE)?;
        r.set_stroke(GUIDE)?;
        r.set_stroke_width(1.0)?;
        Ok(())
    };

    // 1. xMidYMid meet (default) — fits the whole image inside the box, preserving the 3:2 ratio.
    //    Horizontal bars appear because the box is square and the source is wider than tall.
    slot(xs[0])?;
    let i1 = svg.image(SRC, Point::new(xs[0], y0), Size::new(img_w, img_h))?;
    i1.set_attr("preserveAspectRatio", "xMidYMid meet")?;
    caption(&svg, xs[0] + img_w / 2.0, "meet (default)")?;

    // 2. none — stretches to fill the exact box dimensions, squashing the 3:2 source into a square.
    slot(xs[1])?;
    let i2 = svg.image(SRC, Point::new(xs[1], y0), Size::new(img_w, img_h))?;
    i2.set_attr("preserveAspectRatio", "none")?;
    caption(&svg, xs[1] + img_w / 2.0, "none (stretch)")?;

    // 3. xMidYMid slice — scales up to fill the box and clips the sides.
    slot(xs[2])?;
    let i3 = svg.image(SRC, Point::new(xs[2], y0), Size::new(img_w, img_h))?;
    i3.set_attr("preserveAspectRatio", "xMidYMid slice")?;
    caption(&svg, xs[2] + img_w / 2.0, "slice (fill+clip)")?;

    // 4. set_href — the element is created with SRC, then the source is swapped to ALT after creation.
    slot(xs[3])?;
    let i4 = svg.image(SRC, Point::new(xs[3], y0), Size::new(img_w, img_h))?;
    i4.set_href(ALT)?;
    caption(&svg, xs[3] + img_w / 2.0, "set_href swap")?;

    Ok(())
}
