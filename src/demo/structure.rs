use crate::{AnimationLoop, Error, SvgRoot, root::utils::{Point, Size}};
use super::colours::*;
use super::{W, H, BAND, PAD_Y, caption, keep_demo_anim};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// group (<g>)
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(super) fn demo_group() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-group", Size::new(W, H))?;

    // Group A — steelblue block, positioned with translate.
    //
    // `build_batch_into` creates the block and label straight inside the <g> via a detached fragment, so they never
    // touch the root and are not re-parented afterwards — unlike `svg.rect(...)` + `g.append(...)`, which would append
    // each child to the root first and then move it.
    let g1 = svg.group()?;
    svg.build_batch_into(&g1, |b| {
        let block = b.rect(Point::new(0.0, 0.0), Size::new(150.0, 80.0))?;
        block.set_fill(STEELBLUE)?;
        let label = b.text(Point::new(75.0, 47.0), "Group A")?;
        label.set_fill(WHITE)?;
        label.set_attrs([("font-size", "15"), ("text-anchor", "middle")])?;
        Ok(())
    })?;
    g1.set_attr("transform", &format!("translate(40, {})", 25.0 + PAD_Y))?;

    // Dashed connector
    let conn = svg.line(Point::new(190.0, 65.0 + PAD_Y), Point::new(280.0, 65.0 + PAD_Y))?;
    conn.set_stroke(GUIDE)?;
    conn.set_stroke_width(2.0)?;
    conn.set_attr("stroke-dasharray", "5 4")?;

    // Group B — darkorange block, different translate (built the same batched way)
    let g2 = svg.group()?;
    svg.build_batch_into(&g2, |b| {
        let block = b.rect(Point::new(0.0, 0.0), Size::new(150.0, 80.0))?;
        block.set_fill(DARK_ORANGE)?;
        let label = b.text(Point::new(75.0, 47.0), "Group B")?;
        label.set_fill(WHITE)?;
        label.set_attrs([("font-size", "15"), ("text-anchor", "middle")])?;
        Ok(())
    })?;
    g2.set_attr("transform", &format!("translate(280, {})", 25.0 + PAD_Y))?;

    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// AnimationLoop
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(super) fn demo_anim() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-anim", Size::new(W, H))?;

    // Pulsing circle — radius oscillates
    let pulse = svg.circle(Point::new(80.0, 55.0 + PAD_Y), 20.0)?;
    pulse.set_fill(MEDIUM_ORCHID)?;
    caption(&svg, 80.0, "radius pulse")?;

    // Travelling circle — cx oscillates horizontally
    let travel = svg.circle(Point::new(400.0, 55.0 + PAD_Y), 14.0)?;
    travel.set_fill(LIGHT_SKY_BLUE)?;
    caption(&svg, 400.0, "horizontal travel")?;

    // Hue-rotating rectangle
    let hue_rect = svg.rect(Point::new(600.0, 10.0 + PAD_Y), Size::new(185.0, 90.0))?;
    caption(&svg, 693.0, "hue rotation")?;

    let anim = AnimationLoop::start_with_frame(move |ts, frame| {
        // r: 10..48 at ~0.7 Hz
        let r = 10.0 + 38.0 * ((ts / 700.0).sin().abs());
        let _ = frame.set_attr_fmt(&pulse, "r", format_args!("{r:.1}"));

        // cx: 300..500 at ~0.6 Hz
        let cx = 400.0 + 100.0 * (ts / 1050.0).sin();
        let _ = frame.set_attr_fmt(&travel, "cx", format_args!("{cx:.1}"));

        // hue: full rotation every 9 s
        let hue = (ts / 25.0) % 360.0;
        let _ = frame.set_fill_fmt(&hue_rect, format_args!("hsl({hue:.0},70%,50%)"));
    })?;

    // The loop must outlive this function. Park it in thread-local state so it keeps running for the page's lifetime;
    // because `AnimationLoop` stops on `Drop`, this stays cancellable, unlike `mem::forget`.
    keep_demo_anim(anim);
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// defs / marker (arrowhead)
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(super) fn demo_marker() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-marker", Size::new(W, H))?;

    // Build a <defs> container with a named arrowhead <marker> inside it.
    // build_marker appends to <defs> only when the closure returns Ok, so a partially-built
    // marker is never visible in the DOM if construction fails partway through.
    let defs = svg.defs()?;
    let arrow = defs.build_marker("arrow", |m| {
        m.set_ref_x(10.0)?;
        m.set_ref_y(3.5)?;
        m.set_marker_width(10.0)?;
        m.set_marker_height(7.0)?;
        m.set_orient("auto")?;
        let head = m.polygon(&[Point::new(0.0, 0.0), Point::new(10.0, 3.5), Point::new(0.0, 7.0)])?;
        head.set_fill(ACCENT_BLUE)?;
        Ok(())
    })?;

    // Horizontal
    let l1 = svg.line(Point::new(20.0, 55.0 + PAD_Y), Point::new(240.0, 55.0 + PAD_Y))?;
    l1.set_stroke(ACCENT_BLUE)?;
    l1.set_stroke_width(2.0)?;
    l1.set_marker_end_ref(&arrow)?;
    caption(&svg, 130.0, "marker-end")?;

    // Diagonal — orient="auto" rotates the arrowhead to track the path tangent
    let l2 = svg.line(Point::new(280.0, 20.0 + PAD_Y), Point::new(490.0, 100.0 + PAD_Y))?;
    l2.set_stroke(ACCENT_BLUE)?;
    l2.set_stroke_width(2.0)?;
    l2.set_marker_end_ref(&arrow)?;
    caption(&svg, 385.0, r#"orient="auto""#)?;

    // Thick — same marker reused across all three lines
    let l3 = svg.line(Point::new(530.0, 55.0 + PAD_Y), Point::new(770.0, 55.0 + PAD_Y))?;
    l3.set_stroke(ACCENT_BLUE)?;
    l3.set_stroke_width(4.0)?;
    l3.set_marker_end_ref(&arrow)?;
    caption(&svg, 650.0, "set_marker_end_ref")?;

    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// use — stamp copies of a <defs> shape
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(super) fn demo_use() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-use", Size::new(W, H))?;

    // Define a diamond-shaped path once inside <defs>; it is not rendered until referenced.
    svg.build_defs(|d| {
        let gem = d.path("M 0,-28 L 22,0 L 0,28 L -22,0 Z")?;
        gem.set_attr("id", "gem")?;
        gem.set_fill(ACCENT_BLUE)?;
        gem.set_stroke(WHITE)?;
        gem.set_stroke_width(2.0)?;
        Ok(())
    })?;

    // Stamp five independent copies of the same path using <use>.
    // Positioning is done entirely through the transform attribute so that x and y stay at zero
    // and the rotation centres fall exactly on each copy's visual midpoint.
    let cy = PAD_Y + BAND / 2.0;
    let mut buf = String::new();
    for i in 0..5usize {
        let cx = 80.0 + i as f64 * 160.0;
        let angle = i as f64 * 18.0;
        let u = svg.use_node("#gem", Point::origin())?;
        u.set_transform_fmt(&mut buf, format_args!("translate({cx},{cy}) rotate({angle})"))?;
    }

    caption(&svg, W / 2.0, "one <defs> path stamped five times with <use>")?;
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// image — embed a raster or SVG image
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(super) fn demo_image() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-image", Size::new(W, H))?;

    // A 60×40 four-quadrant colour grid embedded as a base64 SVG data URI.
    // Base64 avoids having to percent-encode '<', '>' and '#' that appear in raw SVG data URIs.
    // The 3:2 source aspect ratio differs from the 1:1 display boxes below, making the three
    // preserveAspectRatio modes visually distinct.
    const SRC: &str = "data:image/svg+xml;base64,\
        PHN2ZyB4bWxucz0naHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmcnIHdpZHRoPSc2MCcgaGVpZ2h0\
        PSc0MCc+PHJlY3Qgd2lkdGg9JzMwJyBoZWlnaHQ9JzIwJyBmaWxsPSdzdGVlbGJsdWUnLz48cmVj\
        dCB4PSczMCcgd2lkdGg9JzMwJyBoZWlnaHQ9JzIwJyBmaWxsPSdjb3JhbCcvPjxyZWN0IHk9JzIw\
        JyB3aWR0aD0nMzAnIGhlaWdodD0nMjAnIGZpbGw9J2dvbGQnLz48cmVjdCB4PSczMCcgeT0nMjAn\
        IHdpZHRoPSczMCcgaGVpZ2h0PScyMCcgZmlsbD0nbWVkaXVtc2VhZ3JlZW4nLz48L3N2Zz4=";

    // Alternative image: slate-blue with a centred white circle (used in the set_href demo slot).
    const ALT: &str = "data:image/svg+xml;base64,\
        PHN2ZyB4bWxucz0naHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmcnIHdpZHRoPSc2MCcgaGVpZ2h0\
        PSc0MCc+PHJlY3Qgd2lkdGg9JzYwJyBoZWlnaHQ9JzQwJyBmaWxsPSdzbGF0ZWJsdWUnLz48Y2ly\
        Y2xlIGN4PSczMCcgY3k9JzIwJyByPScxNCcgZmlsbD0nd2hpdGUnIG9wYWNpdHk9Jy44NScvPjwv\
        c3ZnPg==";

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
