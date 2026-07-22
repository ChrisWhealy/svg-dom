use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use super::colours::*;
use super::{BAND, H, PAD_Y, W, caption, keep_demo_anim, keep_demo_node};
use crate::{
    AnimationLoop, Error, PathDef, PathDefAbsolute, SvgNode, SvgRoot,
    root::utils::{Matrix2D, Point, Size},
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// symbol — reusable scaled viewport
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(super) fn demo_symbol() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-symbol", Size::new(W, H))?;

    // Define a badge icon once inside <defs> with its own 40×40 coordinate space.
    // The viewBox allows the <use> element's width/height to scale the content automatically; the same definition
    // renders at any size with no extra work — this is what separates <symbol> from a plain <g> in <defs>.
    let defs = svg.defs()?;

    defs.build_symbol("badge", |s| {
        s.set_view_box(0.0, 0.0, 40.0, 40.0)?;
        // Outer disc
        s.circle(Point::new(20.0, 20.0), 19.0)?.set_fill(STEELBLUE)?;
        // Inner 5-pointed star: 10-vertex polygon alternating outer (r≈12) and inner (r≈5) vertices
        s.polygon(&[
            Point::new(20.0, 8.0),
            Point::new(22.5, 15.0),
            Point::new(30.0, 15.5),
            Point::new(24.5, 21.0),
            Point::new(27.0, 29.0),
            Point::new(20.0, 24.5),
            Point::new(13.0, 29.0),
            Point::new(15.5, 21.0),
            Point::new(10.0, 15.5),
            Point::new(17.5, 15.0),
        ])?
        .set_fill(WHITE)?;
        Ok(())
    })?;

    // Stamp five copies at increasing sizes.
    // Even spacing: gap = (W − Σsizes) / (n+1).
    let sizes: [f64; 5] = [30.0, 50.0, 70.0, 90.0, 110.0];
    let total: f64 = sizes.iter().sum();
    let gap = (W - total) / (sizes.len() as f64 + 1.0);
    let cy = PAD_Y + BAND / 2.0;
    let mut x = gap;

    for sz in sizes {
        let u = svg.use_node("#badge", Point::new(x, cy - sz / 2.0))?;
        u.set_attr("width", &format!("{sz}"))?;
        u.set_attr("height", &format!("{sz}"))?;
        x += sz + gap;
    }

    caption(
        &svg,
        W / 2.0,
        "one <symbol> (viewBox 40×40) stamped five times at increasing sizes via <use>",
    )?;
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// SvgRoot::set_view_box — three mini canvases, identical scene, different viewBox
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(super) fn demo_view_box() -> Result<(), Error> {
    // Draws the same four coloured quadrants (each 100x100, filling a 200x200 area) plus a white landmark dot fixed
    // inside the bottom-right (mediumseagreen) quadrant — identical drawing coordinates every time this is called.
    // Only the set_view_box call made on each SvgRoot after this differs, so any visual difference between the
    // three canvases below comes entirely from the viewBox, not from what was drawn.
    let scene = |svg: &SvgRoot| -> Result<(), Error> {
        svg.rect(Point::new(0.0, 0.0), Size::new(100.0, 100.0))?.set_fill(STEELBLUE)?;
        svg.rect(Point::new(100.0, 0.0), Size::new(100.0, 100.0))?.set_fill(CORAL)?;
        svg.rect(Point::new(0.0, 100.0), Size::new(100.0, 100.0))?.set_fill(GOLD)?;
        svg.rect(Point::new(100.0, 100.0), Size::new(100.0, 100.0))?
            .set_fill(MEDIUM_SEA_GREEN)?;
        let dot = svg.circle(Point::new(170.0, 170.0), 8.0)?;
        dot.set_fill(WHITE)?;
        dot.set_stroke(INK)?;
        dot.set_stroke_width(2.0)?;
        Ok(())
    };
    let panel_size = Size::new(200.0, 200.0);

    // 1. No viewBox: the 200x200 drawing coordinates map directly onto the 200x200 pixel canvas, 1 unit = 1 pixel.
    //    All four quadrants and the landmark dot are visible.
    let svg1 = SvgRoot::create_in("demo-view-box-1", panel_size)?;
    scene(&svg1)?;

    // 2. viewBox(0, 0, 100, 100) mapped onto the same 200x200 pixel canvas magnifies everything 2x. Only the
    //    top-left (steelblue) quadrant is visible, and the landmark dot at (170, 170) falls entirely outside this
    //    viewBox, so it is clipped rather than just shrinking off-screen.
    let svg2 = SvgRoot::create_in("demo-view-box-2", panel_size)?;
    scene(&svg2)?;
    svg2.set_view_box(0.0, 0.0, 100.0, 100.0)?;

    // 3. viewBox(50, 50, 150, 150) is both panned (by the (50, 50) origin) and mildly magnified (150 viewBox units
    //    stretched across the 200px canvas), and — deliberately — its far corner (50+150, 50+150) lands exactly on
    //    the scene's own edge (200, 200), so this crops the outer ring of the other three quadrants without
    //    exposing any blank background beyond the drawn content. The bottom-right quadrant, and the landmark dot
    //    inside it, are now fully back in view.
    let svg3 = SvgRoot::create_in("demo-view-box-3", panel_size)?;
    scene(&svg3)?;
    svg3.set_view_box(50.0, 50.0, 150.0, 150.0)?;

    Ok(())
}

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

    // Dashed connector 2
    let conn2 = svg.line(Point::new(430.0, 65.0 + PAD_Y), Point::new(560.0, 65.0 + PAD_Y))?;
    conn2.set_stroke(GUIDE)?;
    conn2.set_stroke_width(2.0)?;
    conn2.set_attr("stroke-dasharray", "5 4")?;

    // Group C — mediumorchid block, sheared via set_matrix. No combination of translate/rotate/scale can produce a
    // shear, so this is the one shape that cannot be expressed by the named helpers. The matrix's e/f components
    // (560, 25 + PAD_Y) do the same positioning job as Group A/B's translate, folded into the same call as the shear
    // itself rather than needing a second transform.
    let g3 = svg.group()?;
    svg.build_batch_into(&g3, |b| {
        let block = b.rect(Point::new(0.0, 0.0), Size::new(150.0, 80.0))?;
        block.set_fill(MEDIUM_ORCHID)?;
        let label = b.text(Point::new(75.0, 47.0), "Group C")?;
        label.set_fill(WHITE)?;
        label.set_attrs([("font-size", "15"), ("text-anchor", "middle")])?;
        Ok(())
    })?;
    let mut matrix_buf = String::new();
    g3.set_matrix(
        &mut matrix_buf,
        Matrix2D {
            h_scale: 1.0,
            v_scale: 1.0,
            h_skew: 0.3,
            v_skew: 0.0,
            h_trans: 560.0,
            v_trans: 25.0 + PAD_Y,
        },
    )?;

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
// SvgMarker::set_view_box — same authored triangle, two different viewBox windows onto it
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(super) fn demo_marker_view_box() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-marker-view-box", Size::new(W, H))?;

    let defs = svg.defs()?;

    // Both markers render the *same* polygon — a triangle from (0,0) to (100,35) to (0,70) — through the same
    // markerWidth/markerHeight (24x16.8, matching the triangle's own 10:7 aspect ratio so neither viewBox below
    // needs any preserveAspectRatio letterboxing). Only the viewBox each one applies to that shared polygon
    // differs, and that alone is enough to make them look quite different.
    let triangle = [Point::new(0.0, 0.0), Point::new(100.0, 35.0), Point::new(0.0, 70.0)];

    // Marker A: viewBox covers the polygon's full 100x70 extent, so the whole sharp-pointed triangle is visible.
    let full = defs.build_marker("arrow-full", |m| {
        m.set_ref_x(100.0)?;
        m.set_ref_y(35.0)?;
        m.set_marker_width(24.0)?;
        m.set_marker_height(16.8)?;
        m.set_view_box(0.0, 0.0, 100.0, 70.0)?;
        m.set_orient("auto")?;
        m.polygon(&triangle)?.set_fill(ACCENT_BLUE)?;
        Ok(())
    })?;

    // Marker B: viewBox covers only the polygon's top-left 50x35 quarter — half the width and half the height, so
    // the content is magnified 2x, and the triangle's pointed tip (which lives outside x:50-100) is clipped away
    // entirely. What is left is a blunt, roughly trapezoidal wedge: a visibly different shape, at a visibly
    // different scale, from the same polygon data.
    let cropped = defs.build_marker("arrow-cropped", |m| {
        m.set_ref_x(50.0)?;
        m.set_ref_y(26.0)?;
        m.set_marker_width(24.0)?;
        m.set_marker_height(16.8)?;
        m.set_view_box(0.0, 0.0, 50.0, 35.0)?;
        m.set_orient("auto")?;
        m.polygon(&triangle)?.set_fill(DARK_ORANGE)?;
        Ok(())
    })?;

    let label = |cx: f64, y: f64, text: &str| -> Result<(), Error> {
        let t = svg.text(Point::new(cx, y), text)?;
        t.set_fill(TEXT)?;
        t.set_attrs([("font-size", "13"), ("text-anchor", "end")])?;
        Ok(())
    };

    let l1 = svg.line(Point::new(140.0, PAD_Y + 35.0), Point::new(650.0, PAD_Y + 35.0))?;
    l1.set_stroke(ACCENT_BLUE)?;
    l1.set_stroke_width(2.0)?;
    l1.set_marker_end_ref(&full)?;
    label(120.0, PAD_Y + 39.0, "full shape")?;

    let l2 = svg.line(Point::new(140.0, PAD_Y + 95.0), Point::new(650.0, PAD_Y + 95.0))?;
    l2.set_stroke(DARK_ORANGE)?;
    l2.set_stroke_width(2.0)?;
    l2.set_marker_end_ref(&cropped)?;
    label(120.0, PAD_Y + 99.0, "zoomed 2×")?;

    caption(
        &svg,
        W / 2.0,
        "the same figure, two different set_view_box windows: a half-size viewBox both magnifies 2× and clips the tip off",
    )?;

    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// use — stamp copies of a <defs> shape
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(super) fn demo_use() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-use", Size::new(W, H))?;

    // Define a diamond-shaped path once inside <defs>; it is not rendered until referenced.
    svg.build_defs(|d| {
        let gem = d.path_from_defs(&[
            PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(0.0, -28.0))),
            PathDef::Abs(PathDefAbsolute::LineTo(Point::new(22.0, 0.0))),
            PathDef::Abs(PathDefAbsolute::LineTo(Point::new(0.0, 28.0))),
            PathDef::Abs(PathDefAbsolute::LineTo(Point::new(-22.0, 0.0))),
            PathDef::Abs(PathDefAbsolute::ClosePath),
        ])?;
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

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Tree navigation — step through first_child/next_sibling on click, query_selector on demand
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// A static, build-once render of a tree walk is over before a viewer can follow what happened — every bar is already
// coloured and the outline is already there, so there is nothing left to observe. Driving each step from a click
// instead makes the traversal itself the thing being shown: one click is one `next_sibling()` call, visibly.
pub(super) fn demo_tree_nav() -> Result<(), Error> {
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

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// accessibility — set_title / set_desc, native tooltip, and read-back via title() / desc()
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(super) fn demo_accessibility() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-accessibility", Size::new(W, H))?;

    // Three labelled icon circles — non-interactive graphics, not buttons. Each carries its own <title> child (also
    // the native hover tooltip) and <desc> child (visible only to assistive technology), but deliberately no
    // role/tabindex/click or keyboard handling: <title>/<desc> make a graphic describable, they do not make it a
    // control, so this demo does not dress it up as one (no pointer cursor, no button semantics).
    //
    // Neither is used here as a stand-in for real accessible-name/description computation: aria-label/aria-labelledby
    // and aria-describedby would take precedence over these if present, which this demo deliberately doesn't exercise.
    //
    // The readout below echoes both back via the title()/desc() getters when a pointer enters an icon, so the invisible
    // <desc> becomes visible for the purposes of this demo — this hover-to-reveal is the same passive reveal a mouse
    // user gets from any native title tooltip, not a stand-in for clicking a control.
    let icons: [(f64, &str, &str, &str, &str); 3] = [
        (
            150.0,
            STEELBLUE,
            "Save",
            "Save icon",
            "Represents the save function: writes the current document to disk.",
        ),
        (
            400.0,
            ACCENT_AMBER,
            "Share",
            "Share icon",
            "Represents the share function: opens the share sheet for this item.",
        ),
        (
            650.0,
            CRIMSON,
            "Delete",
            "Delete icon",
            "Represents the delete function: permanently removes the selected item. This cannot be undone.",
        ),
    ];

    let icon_y = PAD_Y + 34.0;
    let readout = svg.text(
        Point::new(W / 2.0, PAD_Y + 92.0),
        "hover an icon to read its title and desc back",
    )?;
    readout.set_fill(TEXT)?;
    readout.set_attrs([("font-size", "13"), ("text-anchor", "middle")])?;

    for (cx, fill, label, title, desc) in icons {
        let icon = svg.circle(Point::new(cx, icon_y), 22.0)?;
        icon.set_fill(fill)?;
        icon.set_title(title)?;
        icon.set_desc(desc)?;

        let icon_label = svg.text(Point::new(cx, icon_y + 40.0), label)?;
        icon_label.set_fill(TEXT_MUTED)?;
        icon_label.set_attrs([("font-size", "12"), ("text-anchor", "middle"), ("style", "pointer-events:none")])?;

        let hover_readout = readout.clone();
        let hover_icon = icon.clone();
        icon.on_pointerenter(move |_| {
            let title = hover_icon.title().unwrap_or_default();
            let desc = hover_icon.desc().unwrap_or_default();
            hover_readout.set_text(&format!("title: \"{title}\"  ·  desc: \"{desc}\""));
        })?;

        keep_demo_node(icon);
    }

    caption(
        &svg,
        W / 2.0,
        "set_title() also drives the browser's native hover tooltip; set_desc() has no visible tooltip",
    )?;

    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// a — <g>-like wrapper that turns its children into one hyperlink
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(super) fn demo_anchor() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-anchor", Size::new(W, H))?;
    let cy = PAD_Y + BAND / 2.0;

    // Fragment hrefs are used here purely so clicking inside this demo does not navigate away from the gallery; a
    // real application would pass whatever URL it actually wants to link to.
    //
    // Both the circle and its label become part of the same hyperlink, the same way an HTML <a> around several
    // elements would — clicking either one navigates.
    let link1 = svg.anchor("#demo-anchor")?;
    let icon1 = svg.circle(Point::new(150.0, cy), 40.0)?;
    icon1.set_fill(ACCENT_BLUE)?;
    let label1 = svg.text(Point::new(150.0, cy + 5.0), "A")?;
    label1.set_fill(WHITE)?;
    label1.set_attrs([("text-anchor", "middle"), ("font-size", "20"), ("font-weight", "bold")])?;
    link1.append(&icon1)?;
    link1.append(&label1)?;
    caption(&svg, 150.0, "<a>: one href wraps both children")?;

    // `target` is not wrapped by a named parameter — every meaningful use of <a> supplies href, but target is only
    // occasionally needed — so it goes through the generic set_attr escape hatch instead.
    let link2 = svg.anchor("#demo-anchor")?;
    link2.set_attr("target", "_blank")?;
    let icon2 = svg.circle(Point::new(450.0, cy), 40.0)?;
    icon2.set_fill(DARK_ORANGE)?;
    let label2 = svg.text(Point::new(450.0, cy + 5.0), "B")?;
    label2.set_fill(WHITE)?;
    label2.set_attrs([("text-anchor", "middle"), ("font-size", "20"), ("font-weight", "bold")])?;
    link2.append(&icon2)?;
    link2.append(&label2)?;
    caption(&svg, 450.0, "target=\"_blank\" set via set_attr")?;

    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// switch — renders exactly one direct child, chosen by conditional-processing attributes
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(super) fn demo_switch() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-switch", Size::new(W, H))?;
    let cy = PAD_Y + BAND / 2.0;

    // Panel 1: the only conditional child carries a systemLanguage no browser ever reports, so it never matches —
    // <switch> falls through to the attribute-free fallback. This is deterministic in every browser/locale, unlike
    // testing a real systemLanguage match against whichever locale happens to be running the demo.
    let switch1 = svg.switch()?;
    let never_matches = svg.circle(Point::new(150.0, cy), 40.0)?;
    never_matches.set_fill(CORAL)?;
    never_matches.set_attr("systemLanguage", "xx")?;
    switch1.append(&never_matches)?;
    let fallback = svg.circle(Point::new(150.0, cy), 40.0)?;
    fallback.set_fill(STEELBLUE)?;
    switch1.append(&fallback)?;
    caption(&svg, 150.0, "no child matches -> fallback renders")?;

    // Panel 2: the first child has no test attributes at all, so it always matches and renders immediately — the
    // second child is never reached, even though it would otherwise be a perfectly valid alternative.
    let switch2 = svg.switch()?;
    let first_match = svg.circle(Point::new(450.0, cy), 40.0)?;
    first_match.set_fill(MEDIUM_SEA_GREEN)?;
    switch2.append(&first_match)?;
    let never_reached = svg.circle(Point::new(450.0, cy), 40.0)?;
    never_reached.set_fill(DARK_ORANGE)?;
    switch2.append(&never_reached)?;
    caption(&svg, 450.0, "attribute-free first child always matches")?;

    Ok(())
}
