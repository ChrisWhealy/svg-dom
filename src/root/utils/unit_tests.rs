use super::*;

#[test]
fn write_points_clamps_dps_to_max() -> Result<(), String> {
    let p = Point::new(1.5, 2.5);
    let mut clamped = String::new();
    let mut at_max = String::new();
    write_points(&mut clamped, &[p], Some(usize::MAX));
    write_points(&mut at_max, &[p], Some(MAX_DPS));
    if clamped != at_max {
        return Err(format!(
            "usize::MAX dps must produce the same output as MAX_DPS, got {clamped:?} vs {at_max:?}"
        ));
    }
    Ok(())
}

#[test]
fn write_points_fixed_precision_rounds_correctly() -> Result<(), String> {
    let p = Point::new(1.0 / 3.0, 2.0 / 3.0);
    let mut out = String::new();
    write_points(&mut out, &[p], Some(3));
    if out != "0.333,0.667" {
        return Err(format!("expected \"0.333,0.667\", got {out:?}"));
    }
    Ok(())
}
