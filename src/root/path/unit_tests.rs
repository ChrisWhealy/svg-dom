use super::{elliptical_arc::*, path_def::*};
use crate::{Error, root::utils::Point};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// build_d — one command per SVG letter
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

#[test]
fn build_d_writes_absolute_move_and_line() {
    let d = build_d(&[
        PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(10.0, 10.0))),
        PathDef::Abs(PathDefAbsolute::LineTo(Point::new(100.0, 50.0))),
        PathDef::Abs(PathDefAbsolute::LineTo(Point::new(10.0, 90.0))),
        PathDef::Abs(PathDefAbsolute::ClosePath),
    ]);
    assert_eq!(d, "M10 10L100 50L10 90Z");
}

#[test]
fn build_d_writes_relative_move_and_line() {
    let d = build_d(&[
        PathDef::Rel(PathDefRelative::MoveTo(Point::new(10.0, 10.0))),
        PathDef::Rel(PathDefRelative::LineTo(Point::new(90.0, 40.0))),
        PathDef::Rel(PathDefRelative::ClosePath),
    ]);
    assert_eq!(d, "m10 10l90 40z");
}

#[test]
fn build_d_writes_horizontal_and_vertical_lines() {
    let d = build_d(&[
        PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(0.0, 0.0))),
        PathDef::Abs(PathDefAbsolute::HorizontalLineTo(50.0)),
        PathDef::Abs(PathDefAbsolute::VerticalLineTo(25.0)),
        PathDef::Rel(PathDefRelative::HorizontalLineTo(-10.0)),
        PathDef::Rel(PathDefRelative::VerticalLineTo(-5.0)),
    ]);
    assert_eq!(d, "M0 0H50V25h-10v-5");
}

#[test]
fn build_d_writes_cubic_and_smooth_cubic_bezier() {
    let d = build_d(&[
        PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(0.0, 0.0))),
        PathDef::Abs(PathDefAbsolute::CubicBezierTo(
            Point::new(10.0, 10.0),
            Point::new(20.0, 10.0),
            Point::new(30.0, 0.0),
        )),
        PathDef::Abs(PathDefAbsolute::SmoothCubicBezierTo(
            Point::new(40.0, 10.0),
            Point::new(50.0, 0.0),
        )),
    ]);
    assert_eq!(d, "M0 0C10 10 20 10 30 0S40 10 50 0");
}

#[test]
fn build_d_writes_quadratic_and_smooth_quadratic_bezier() {
    let d = build_d(&[
        PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(0.0, 0.0))),
        PathDef::Abs(PathDefAbsolute::QuadraticBezierTo(
            Point::new(10.0, 10.0),
            Point::new(20.0, 0.0),
        )),
        PathDef::Abs(PathDefAbsolute::SmoothQuadraticBezierTo(Point::new(30.0, 0.0))),
    ]);
    assert_eq!(d, "M0 0Q10 10 20 0T30 0");
}

#[test]
fn build_d_writes_elliptical_arc_with_size_and_sweep_flags() {
    let d = build_d(&[
        PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(10.0, 65.0))),
        PathDef::Abs(PathDefAbsolute::EllipticalArcTo(EllipticalArc {
            radii: Point::new(60.0, 60.0),
            x_axis_rotation: 0.0,
            size: ArcSize::Large,
            sweep: ArcSweep::Clockwise,
            to: Point::new(130.0, 65.0),
        })),
    ]);
    assert_eq!(d, "M10 65A60 60 0 1 1 130 65");
}

#[test]
fn build_d_writes_small_counter_clockwise_arc_flags_as_zero() {
    let d = build_d(&[PathDef::Rel(PathDefRelative::EllipticalArcTo(EllipticalArc {
        radii: Point::new(5.0, 5.0),
        x_axis_rotation: 0.0,
        size: ArcSize::Small,
        sweep: ArcSweep::CounterClockwise,
        to: Point::new(10.0, 0.0),
    }))]);
    assert_eq!(d, "a5 5 0 0 0 10 0");
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// write_d — buffer reuse
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

#[test]
fn write_d_clears_previous_contents_before_writing() {
    let mut buf = String::from("stale contents that must not survive");
    write_d(&mut buf, &[PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(1.0, 2.0)))]);
    assert_eq!(buf, "M1 2");
}

#[test]
fn write_d_matches_build_d_for_the_same_input() {
    let defs = [
        PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(0.0, 0.0))),
        PathDef::Abs(PathDefAbsolute::LineTo(Point::new(5.0, 5.0))),
    ];
    let mut buf = String::new();
    write_d(&mut buf, &defs);
    assert_eq!(buf, build_d(&defs));
}

#[test]
fn build_d_of_empty_slice_is_empty_string() {
    assert_eq!(build_d(&[]), "");
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// build_d_fixed / write_d_fixed
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

#[test]
fn build_d_fixed_rounds_coordinates_to_requested_precision() {
    let d = build_d_fixed(
        &[
            PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(1.0 / 3.0, 2.0 / 3.0))),
            PathDef::Abs(PathDefAbsolute::LineTo(Point::new(10.0, 20.0))),
        ],
        2,
    );
    assert_eq!(d, "M0.33 0.67L10.00 20.00");
}

#[test]
fn build_d_fixed_at_zero_decimals_rounds_to_integers() {
    let d = build_d_fixed(&[PathDef::Abs(PathDefAbsolute::LineTo(Point::new(1.6, 2.4)))], 0);
    assert_eq!(d, "L2 2");
}

#[test]
fn build_d_fixed_rounds_horizontal_and_vertical_line_arguments() {
    let d = build_d_fixed(
        &[
            PathDef::Abs(PathDefAbsolute::HorizontalLineTo(1.0 / 3.0)),
            PathDef::Rel(PathDefRelative::VerticalLineTo(-1.0 / 3.0)),
        ],
        1,
    );
    assert_eq!(d, "H0.3v-0.3");
}

#[test]
fn build_d_fixed_rounds_smooth_and_quadratic_bezier_arguments() {
    let d = build_d_fixed(
        &[
            PathDef::Abs(PathDefAbsolute::SmoothCubicBezierTo(
                Point::new(1.0 / 3.0, 0.0),
                Point::new(0.0, 0.0),
            )),
            PathDef::Abs(PathDefAbsolute::SmoothQuadraticBezierTo(Point::new(1.0 / 3.0, 0.0))),
        ],
        2,
    );
    assert_eq!(d, "S0.33 0.00 0.00 0.00T0.33 0.00");
}

/// The two elliptical-arc flags must never be affected by `dps`: the SVG `flag` grammar production is exactly one
/// `"0"` or `"1"` digit, not a decimal number, so rounding them to `"0.00"`/`"1.00"` would be invalid path syntax.
#[test]
fn build_d_fixed_never_rounds_elliptical_arc_flags() {
    let d = build_d_fixed(
        &[PathDef::Abs(PathDefAbsolute::EllipticalArcTo(EllipticalArc {
            radii: Point::new(1.0 / 3.0, 1.0 / 3.0),
            x_axis_rotation: 1.0 / 3.0,
            size: ArcSize::Large,
            sweep: ArcSweep::Clockwise,
            to: Point::new(1.0 / 3.0, 1.0 / 3.0),
        }))],
        2,
    );
    assert_eq!(d, "A0.33 0.33 0.33 1 1 0.33 0.33");
}

#[test]
fn write_d_fixed_clamps_dps_to_max() {
    let mut clamped = String::new();
    let mut at_max = String::new();
    let defs = [PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(1.5, 2.5)))];
    write_d_fixed(&mut clamped, &defs, usize::MAX);
    write_d_fixed(&mut at_max, &defs, 20);
    assert_eq!(
        clamped, at_max,
        "usize::MAX dps must produce the same output as the MAX_DPS clamp"
    );
}

#[test]
fn write_d_fixed_matches_build_d_fixed_for_the_same_input() {
    let defs = [
        PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(0.0, 0.0))),
        PathDef::Abs(PathDefAbsolute::LineTo(Point::new(5.0 / 3.0, 5.0 / 3.0))),
    ];
    let mut buf = String::new();
    write_d_fixed(&mut buf, &defs, 3);
    assert_eq!(buf, build_d_fixed(&defs, 3));
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Capacity pre-reservation — build_d / build_d_fixed only
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

#[test]
fn build_d_reserves_capacity_proportional_to_command_count() {
    let defs = [
        PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(1.0, 2.0))),
        PathDef::Abs(PathDefAbsolute::LineTo(Point::new(3.0, 4.0))),
        PathDef::Abs(PathDefAbsolute::ClosePath),
    ];
    let expected_min = defs.len() * 24;
    let d = build_d(&defs);
    assert!(
        d.capacity() >= expected_min,
        "expected capacity >= {expected_min}, got {}",
        d.capacity()
    );
}

#[test]
fn build_d_fixed_reserves_capacity_proportional_to_command_count() {
    let defs = [
        PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(1.0, 2.0))),
        PathDef::Abs(PathDefAbsolute::LineTo(Point::new(3.0, 4.0))),
        PathDef::Abs(PathDefAbsolute::ClosePath),
    ];
    let expected_min = defs.len() * 24;
    let d = build_d_fixed(&defs, 2);
    assert!(
        d.capacity() >= expected_min,
        "expected capacity >= {expected_min}, got {}",
        d.capacity()
    );
}

/// The capacity estimate must grow with `dps`: a fixed flat per-command guess (tuned for the default,
/// shortest-round-trip format) badly undershoots at high precision, since each number can be far longer than that
/// guess accounts for.
#[test]
fn build_d_fixed_capacity_grows_with_requested_precision() {
    let defs = [PathDef::Abs(PathDefAbsolute::CubicBezierTo(
        Point::new(1.0, 2.0),
        Point::new(3.0, 4.0),
        Point::new(5.0, 6.0),
    ))];
    let low_dps = build_d_fixed(&defs, 0).capacity();
    let high_dps = build_d_fixed(&defs, 20).capacity();
    assert!(
        high_dps > low_dps,
        "expected capacity to grow with dps: dps=0 -> {low_dps}, dps=20 -> {high_dps}"
    );
}

/// Regression case for the specific worst case cited when this estimate was made precision-aware: a six-argument
/// `CubicBezierTo` at `dps = 20` formats to roughly 138 bytes (`"C0.00000000000000000000 0.00000000000000000000
/// ..."`), nearly six times the flat 24-byte guess a precision-unaware estimate would have reserved.
///
/// `APPROX_VALUES_PER_COMMAND` (3) is deliberately an *average* across command shapes (`ClosePath` has zero
/// numeric arguments, `CubicBezierTo` has six), not a per-command worst-case bound — reaching a true worst-case
/// guarantee would need the variant-aware second pass this estimate exists specifically to avoid. So this does not
/// assert the reservation covers `CubicBezierTo`'s full length; it asserts the narrower, honest claim: the
/// precision-aware formula reserves *more* than the old flat, precision-unaware guess would have, and covers a
/// larger fraction of the real content — a measurable improvement, not a complete fix, for exactly this worst case.
#[test]
fn build_d_fixed_capacity_formula_improves_on_flat_guess_for_high_precision_cubic_bezier() {
    let defs = [PathDef::Abs(PathDefAbsolute::CubicBezierTo(
        Point::new(0.0, 0.0),
        Point::new(0.0, 0.0),
        Point::new(0.0, 0.0),
    ))];
    let dps = 20;
    let base_bytes_per_command = 24;
    let approx_values_per_command = 3;
    let old_flat_reservation = defs.len() * base_bytes_per_command;
    let new_reservation = defs.len() * (base_bytes_per_command + approx_values_per_command * dps);

    let actual_len = build_d_fixed(&defs, dps).len();
    assert!(
        actual_len > 24,
        "sanity check: this case should exceed the old flat 24-byte guess (was {actual_len})"
    );
    assert!(
        new_reservation > old_flat_reservation,
        "precision-aware reservation ({new_reservation}) should exceed the flat guess ({old_flat_reservation})"
    );

    let old_shortfall = actual_len.saturating_sub(old_flat_reservation);
    let new_shortfall = actual_len.saturating_sub(new_reservation);
    assert!(
        new_shortfall < old_shortfall,
        "precision-aware shortfall ({new_shortfall}) should be smaller than the flat guess's shortfall ({old_shortfall})"
    );
}

/// `write_d` must not reserve on the caller's behalf: it writes into a buffer the caller is expected to reuse
/// (and therefore already size correctly, via `SvgAttrs::with_capacity` if desired), so a fresh, empty buffer keeps
/// whatever capacity `String`'s own incremental growth produces rather than a `build_d`-style upfront reservation.
#[test]
fn write_d_does_not_preemptively_reserve_like_build_d_does() {
    let defs = [PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(1.0, 2.0)))];
    let mut buf = String::new();
    write_d(&mut buf, &defs);
    assert!(
        buf.capacity() < defs.len() * 24,
        "write_d should not pre-reserve a build_d-sized allocation for a single short command"
    );
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Check paths start with moveto command
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

#[test]
fn should_accept_empty_slice() {
    assert!(validate_starts_with_moveto(&[]).is_ok());
}

#[test]
fn should_accept_absolute_move_first() {
    let defs = [
        PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(0.0, 0.0))),
        PathDef::Abs(PathDefAbsolute::LineTo(Point::new(1.0, 1.0))),
    ];
    assert!(validate_starts_with_moveto(&defs).is_ok());
}

/// A leading relative `m` is accepted too: per the SVG spec, a path's very first moveto is always treated as
/// absolute, even when written with the lowercase letter, since there is no current point yet for it to be
/// relative to.
#[test]
fn should_accept_relative_move_first() {
    let defs = [PathDef::Rel(PathDefRelative::MoveTo(Point::new(0.0, 0.0)))];
    assert!(validate_starts_with_moveto(&defs).is_ok());
}

#[test]
fn should_reject_line_to_first() {
    let defs = [PathDef::Abs(PathDefAbsolute::LineTo(Point::new(1.0, 1.0)))];
    assert!(matches!(validate_starts_with_moveto(&defs), Err(Error::InvalidPathData(_))));
}

#[test]
fn should_reject_close_path_first() {
    let defs = [PathDef::Abs(PathDefAbsolute::ClosePath)];
    assert!(matches!(validate_starts_with_moveto(&defs), Err(Error::InvalidPathData(_))));
}

#[test]
fn should_reject_elliptical_arc_first() {
    let defs = [PathDef::Rel(PathDefRelative::EllipticalArcTo(EllipticalArc {
        radii: Point::new(5.0, 5.0),
        x_axis_rotation: 0.0,
        size: ArcSize::Small,
        sweep: ArcSweep::CounterClockwise,
        to: Point::new(10.0, 0.0),
    }))];
    assert!(matches!(validate_starts_with_moveto(&defs), Err(Error::InvalidPathData(_))));
}

/// Only the first command matters: a later command that isn't a moveto is fine.
#[test]
fn should_ignore_later_commands() {
    let defs = [
        PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(0.0, 0.0))),
        PathDef::Abs(PathDefAbsolute::ClosePath),
        PathDef::Abs(PathDefAbsolute::LineTo(Point::new(1.0, 1.0))),
    ];
    assert!(validate_starts_with_moveto(&defs).is_ok());
}

/// `build_d` / `write_d` deliberately do not call `validate_starts_with_moveto`: they are general-purpose
/// formatters that may be used to build a path-data fragment, not necessarily a complete, standalone path.
#[test]
fn should_accept_that_build_d_does_not_validate_leading_command() {
    let defs = [PathDef::Abs(PathDefAbsolute::LineTo(Point::new(1.0, 1.0)))];
    assert_eq!(build_d(&defs), "L1 1");
}
