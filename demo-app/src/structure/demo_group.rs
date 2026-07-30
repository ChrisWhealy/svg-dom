use crate::{H, PAD_Y, W, colours::*};
use svg_dom::{
    Error, SvgRoot,
    root::utils::{Matrix2D, Point, Size},
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// group (<g>)
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
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
