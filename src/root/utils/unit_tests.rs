use super::*;

#[test]
fn write_points_clamps_dps_to_max() {
    let p = Point::new(1.5, 2.5);
    let mut clamped = String::new();
    let mut at_max = String::new();
    write_points(&mut clamped, &[p], Some(usize::MAX));
    write_points(&mut at_max, &[p], Some(MAX_DPS));
    assert_eq!(clamped, at_max, "usize::MAX dps must produce the same output as MAX_DPS");
}

#[test]
fn write_points_fixed_precision_rounds_correctly() {
    let p = Point::new(1.0 / 3.0, 2.0 / 3.0);
    let mut out = String::new();
    write_points(&mut out, &[p], Some(3));
    assert_eq!(out, "0.333,0.667");
}
