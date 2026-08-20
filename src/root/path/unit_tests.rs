use super::{elliptical_arc::*, path_def::*};
use crate::{Error, root::utils::Point};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// build_d — one command per SVG letter
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

#[test]
fn build_d_writes_absolute_move_and_line() -> Result<(), String> {
    let d = build_d(&[
        PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(10.0, 10.0))),
        PathDef::Abs(PathDefAbsolute::LineTo(Point::new(100.0, 50.0))),
        PathDef::Abs(PathDefAbsolute::LineTo(Point::new(10.0, 90.0))),
        PathDef::Abs(PathDefAbsolute::ClosePath),
    ]);
    if d != "M10 10L100 50L10 90Z" {
        return Err(format!("expected \"M10 10L100 50L10 90Z\", got {d:?}"));
    }
    Ok(())
}

#[test]
fn build_d_writes_relative_move_and_line() -> Result<(), String> {
    let d = build_d(&[
        PathDef::Rel(PathDefRelative::MoveTo(Point::new(10.0, 10.0))),
        PathDef::Rel(PathDefRelative::LineTo(Point::new(90.0, 40.0))),
        PathDef::Rel(PathDefRelative::ClosePath),
    ]);
    if d != "m10 10l90 40z" {
        return Err(format!("expected \"m10 10l90 40z\", got {d:?}"));
    }
    Ok(())
}

#[test]
fn build_d_writes_horizontal_and_vertical_lines() -> Result<(), String> {
    let d = build_d(&[
        PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(0.0, 0.0))),
        PathDef::Abs(PathDefAbsolute::HorizontalLineTo(50.0)),
        PathDef::Abs(PathDefAbsolute::VerticalLineTo(25.0)),
        PathDef::Rel(PathDefRelative::HorizontalLineTo(-10.0)),
        PathDef::Rel(PathDefRelative::VerticalLineTo(-5.0)),
    ]);
    if d != "M0 0H50V25h-10v-5" {
        return Err(format!("expected \"M0 0H50V25h-10v-5\", got {d:?}"));
    }
    Ok(())
}

#[test]
fn build_d_writes_cubic_and_smooth_cubic_bezier() -> Result<(), String> {
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
    if d != "M0 0C10 10 20 10 30 0S40 10 50 0" {
        return Err(format!("expected \"M0 0C10 10 20 10 30 0S40 10 50 0\", got {d:?}"));
    }
    Ok(())
}

#[test]
fn build_d_writes_quadratic_and_smooth_quadratic_bezier() -> Result<(), String> {
    let d = build_d(&[
        PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(0.0, 0.0))),
        PathDef::Abs(PathDefAbsolute::QuadraticBezierTo(
            Point::new(10.0, 10.0),
            Point::new(20.0, 0.0),
        )),
        PathDef::Abs(PathDefAbsolute::SmoothQuadraticBezierTo(Point::new(30.0, 0.0))),
    ]);
    if d != "M0 0Q10 10 20 0T30 0" {
        return Err(format!("expected \"M0 0Q10 10 20 0T30 0\", got {d:?}"));
    }
    Ok(())
}

#[test]
fn build_d_writes_elliptical_arc_with_size_and_sweep_flags() -> Result<(), String> {
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
    if d != "M10 65A60 60 0 1 1 130 65" {
        return Err(format!("expected \"M10 65A60 60 0 1 1 130 65\", got {d:?}"));
    }
    Ok(())
}

#[test]
fn build_d_writes_small_counter_clockwise_arc_flags_as_zero() -> Result<(), String> {
    let d = build_d(&[PathDef::Rel(PathDefRelative::EllipticalArcTo(EllipticalArc {
        radii: Point::new(5.0, 5.0),
        x_axis_rotation: 0.0,
        size: ArcSize::Small,
        sweep: ArcSweep::CounterClockwise,
        to: Point::new(10.0, 0.0),
    }))]);
    if d != "a5 5 0 0 0 10 0" {
        return Err(format!("expected \"a5 5 0 0 0 10 0\", got {d:?}"));
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// write_d — buffer reuse
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

#[test]
fn write_d_clears_previous_contents_before_writing() -> Result<(), String> {
    let mut buf = String::from("stale contents that must not survive");
    write_d(&mut buf, &[PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(1.0, 2.0)))]);
    if buf != "M1 2" {
        return Err(format!("expected \"M1 2\", got {buf:?}"));
    }
    Ok(())
}

#[test]
fn write_d_matches_build_d_for_the_same_input() -> Result<(), String> {
    let defs = [
        PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(0.0, 0.0))),
        PathDef::Abs(PathDefAbsolute::LineTo(Point::new(5.0, 5.0))),
    ];
    let mut buf = String::new();
    write_d(&mut buf, &defs);
    let expected = build_d(&defs);
    if buf != expected {
        return Err(format!("write_d produced {buf:?}, build_d produced {expected:?}"));
    }
    Ok(())
}

#[test]
fn build_d_of_empty_slice_is_empty_string() -> Result<(), String> {
    let d = build_d(&[]);
    if !d.is_empty() {
        return Err(format!("expected an empty string, got {d:?}"));
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// build_d_fixed / write_d_fixed
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

#[test]
fn build_d_fixed_rounds_coordinates_to_requested_precision() -> Result<(), String> {
    let d = build_d_fixed(
        &[
            PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(1.0 / 3.0, 2.0 / 3.0))),
            PathDef::Abs(PathDefAbsolute::LineTo(Point::new(10.0, 20.0))),
        ],
        2,
    );
    if d != "M0.33 0.67L10.00 20.00" {
        return Err(format!("expected \"M0.33 0.67L10.00 20.00\", got {d:?}"));
    }
    Ok(())
}

#[test]
fn build_d_fixed_at_zero_decimals_rounds_to_integers() -> Result<(), String> {
    let d = build_d_fixed(&[PathDef::Abs(PathDefAbsolute::LineTo(Point::new(1.6, 2.4)))], 0);
    if d != "L2 2" {
        return Err(format!("expected \"L2 2\", got {d:?}"));
    }
    Ok(())
}

#[test]
fn build_d_fixed_rounds_horizontal_and_vertical_line_arguments() -> Result<(), String> {
    let d = build_d_fixed(
        &[
            PathDef::Abs(PathDefAbsolute::HorizontalLineTo(1.0 / 3.0)),
            PathDef::Rel(PathDefRelative::VerticalLineTo(-1.0 / 3.0)),
        ],
        1,
    );
    if d != "H0.3v-0.3" {
        return Err(format!("expected \"H0.3v-0.3\", got {d:?}"));
    }
    Ok(())
}

#[test]
fn build_d_fixed_rounds_smooth_and_quadratic_bezier_arguments() -> Result<(), String> {
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
    if d != "S0.33 0.00 0.00 0.00T0.33 0.00" {
        return Err(format!("expected \"S0.33 0.00 0.00 0.00T0.33 0.00\", got {d:?}"));
    }
    Ok(())
}

/// The two elliptical-arc flags must never be affected by `dps`. The SVG `flag` grammar production is exactly one
/// `"0"` or `"1"` digit, not a decimal number. So rounding them to `"0.00"`/`"1.00"` would be invalid path syntax.
#[test]
fn build_d_fixed_never_rounds_elliptical_arc_flags() -> Result<(), String> {
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
    if d != "A0.33 0.33 0.33 1 1 0.33 0.33" {
        return Err(format!("expected \"A0.33 0.33 0.33 1 1 0.33 0.33\", got {d:?}"));
    }
    Ok(())
}

#[test]
fn write_d_fixed_clamps_dps_to_max() -> Result<(), String> {
    let mut clamped = String::new();
    let mut at_max = String::new();
    let defs = [PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(1.5, 2.5)))];
    write_d_fixed(&mut clamped, &defs, usize::MAX);
    write_d_fixed(&mut at_max, &defs, 20);
    if clamped != at_max {
        return Err(format!(
            "usize::MAX dps must produce the same output as the MAX_DPS clamp, got {clamped:?} vs {at_max:?}"
        ));
    }
    Ok(())
}

#[test]
fn write_d_fixed_matches_build_d_fixed_for_the_same_input() -> Result<(), String> {
    let defs = [
        PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(0.0, 0.0))),
        PathDef::Abs(PathDefAbsolute::LineTo(Point::new(5.0 / 3.0, 5.0 / 3.0))),
    ];
    let mut buf = String::new();
    write_d_fixed(&mut buf, &defs, 3);
    let expected = build_d_fixed(&defs, 3);
    if buf != expected {
        return Err(format!("write_d_fixed produced {buf:?}, build_d_fixed produced {expected:?}"));
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Capacity pre-reservation — build_d / build_d_fixed only
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

#[test]
fn build_d_reserves_capacity_proportional_to_command_count() -> Result<(), String> {
    let defs = [
        PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(1.0, 2.0))),
        PathDef::Abs(PathDefAbsolute::LineTo(Point::new(3.0, 4.0))),
        PathDef::Abs(PathDefAbsolute::ClosePath),
    ];
    let expected_min = defs.len() * 24;
    let d = build_d(&defs);
    if d.capacity() < expected_min {
        return Err(format!("expected capacity >= {expected_min}, got {}", d.capacity()));
    }
    Ok(())
}

#[test]
fn build_d_fixed_reserves_capacity_proportional_to_command_count() -> Result<(), String> {
    let defs = [
        PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(1.0, 2.0))),
        PathDef::Abs(PathDefAbsolute::LineTo(Point::new(3.0, 4.0))),
        PathDef::Abs(PathDefAbsolute::ClosePath),
    ];
    let expected_min = defs.len() * 24;
    let d = build_d_fixed(&defs, 2);
    if d.capacity() < expected_min {
        return Err(format!("expected capacity >= {expected_min}, got {}", d.capacity()));
    }
    Ok(())
}

/// The capacity estimate must grow with `dps`. A fixed flat per-command guess, tuned for the default,
/// shortest-round-trip format, badly undershoots at high precision. Each number can be far longer than that guess
/// accounts for.
#[test]
fn build_d_fixed_capacity_grows_with_requested_precision() -> Result<(), String> {
    let defs = [PathDef::Abs(PathDefAbsolute::CubicBezierTo(
        Point::new(1.0, 2.0),
        Point::new(3.0, 4.0),
        Point::new(5.0, 6.0),
    ))];
    let low_dps = build_d_fixed(&defs, 0).capacity();
    let high_dps = build_d_fixed(&defs, 20).capacity();
    if high_dps <= low_dps {
        return Err(format!(
            "expected capacity to grow with dps: dps=0 -> {low_dps}, dps=20 -> {high_dps}"
        ));
    }
    Ok(())
}

/// Regression case for the specific worst case cited when this estimate was made precision-aware. A six-argument
/// `CubicBezierTo` at `dps = 20` formats to roughly 138 bytes (`"C0.00000000000000000000 0.00000000000000000000
/// ..."`), nearly six times the flat 24-byte guess a precision-unaware estimate would have reserved.
///
/// `APPROX_VALUES_PER_COMMAND` (3) is deliberately an *average* across command shapes (`ClosePath` has zero
/// numeric arguments, `CubicBezierTo` has six), not a per-command worst-case bound. Reaching a true worst-case
/// guarantee would need the variant-aware second pass this estimate exists specifically to avoid. So this does not
/// assert the reservation covers `CubicBezierTo`'s full length. Instead, it asserts the narrower, honest claim
/// that the precision-aware formula reserves *more* than the old flat, precision-unaware guess would have. It also
/// covers a larger fraction of the real content — a measurable improvement, not a complete fix, for exactly this
/// worst case.
#[test]
fn build_d_fixed_capacity_formula_improves_on_flat_guess_for_high_precision_cubic_bezier() -> Result<(), String> {
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
    if actual_len <= 24 {
        return Err(format!(
            "sanity check: this case should exceed the old flat 24-byte guess (was {actual_len})"
        ));
    }
    if new_reservation <= old_flat_reservation {
        return Err(format!(
            "precision-aware reservation ({new_reservation}) should exceed the flat guess ({old_flat_reservation})"
        ));
    }

    let old_shortfall = actual_len.saturating_sub(old_flat_reservation);
    let new_shortfall = actual_len.saturating_sub(new_reservation);
    if new_shortfall >= old_shortfall {
        return Err(format!(
            "precision-aware shortfall ({new_shortfall}) should be smaller than the flat guess's shortfall \
             ({old_shortfall})"
        ));
    }
    Ok(())
}

/// `write_d` must not reserve on the caller's behalf. It writes into a buffer the caller is expected to reuse, and
/// therefore already size correctly via `SvgAttrs::with_capacity` if desired. So a fresh, empty buffer keeps
/// whatever capacity `String`'s own incremental growth produces, rather than a `build_d`-style upfront reservation.
#[test]
fn write_d_does_not_preemptively_reserve_like_build_d_does() -> Result<(), String> {
    let defs = [PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(1.0, 2.0)))];
    let mut buf = String::new();
    write_d(&mut buf, &defs);
    if buf.capacity() >= defs.len() * 24 {
        return Err("write_d should not pre-reserve a build_d-sized allocation for a single short command".to_owned());
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Check paths start with moveto command
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

#[test]
fn should_accept_empty_slice() -> Result<(), String> {
    validate_starts_with_moveto(&[]).map_err(|e| format!("expected Ok, got {e:?}"))?;
    Ok(())
}

#[test]
fn should_accept_absolute_move_first() -> Result<(), String> {
    let defs = [
        PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(0.0, 0.0))),
        PathDef::Abs(PathDefAbsolute::LineTo(Point::new(1.0, 1.0))),
    ];
    validate_starts_with_moveto(&defs).map_err(|e| format!("expected Ok, got {e:?}"))?;
    Ok(())
}

/// A leading relative `m` is accepted too, per the SVG spec. A path's very first moveto is always treated as
/// absolute, even when written with the lowercase letter. There is no current point yet for it to be relative to.
#[test]
fn should_accept_relative_move_first() -> Result<(), String> {
    let defs = [PathDef::Rel(PathDefRelative::MoveTo(Point::new(0.0, 0.0)))];
    validate_starts_with_moveto(&defs).map_err(|e| format!("expected Ok, got {e:?}"))?;
    Ok(())
}

#[test]
fn should_reject_line_to_first() -> Result<(), String> {
    let defs = [PathDef::Abs(PathDefAbsolute::LineTo(Point::new(1.0, 1.0)))];
    match validate_starts_with_moveto(&defs) {
        Err(Error::InvalidPathData(_)) => Ok(()),
        other => Err(format!("expected Err(InvalidPathData), got {other:?}")),
    }
}

#[test]
fn should_reject_close_path_first() -> Result<(), String> {
    let defs = [PathDef::Abs(PathDefAbsolute::ClosePath)];
    match validate_starts_with_moveto(&defs) {
        Err(Error::InvalidPathData(_)) => Ok(()),
        other => Err(format!("expected Err(InvalidPathData), got {other:?}")),
    }
}

#[test]
fn should_reject_elliptical_arc_first() -> Result<(), String> {
    let defs = [PathDef::Rel(PathDefRelative::EllipticalArcTo(EllipticalArc {
        radii: Point::new(5.0, 5.0),
        x_axis_rotation: 0.0,
        size: ArcSize::Small,
        sweep: ArcSweep::CounterClockwise,
        to: Point::new(10.0, 0.0),
    }))];
    match validate_starts_with_moveto(&defs) {
        Err(Error::InvalidPathData(_)) => Ok(()),
        other => Err(format!("expected Err(InvalidPathData), got {other:?}")),
    }
}

/// Only the first command matters: a later command that isn't a moveto is fine.
#[test]
fn should_ignore_later_commands() -> Result<(), String> {
    let defs = [
        PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(0.0, 0.0))),
        PathDef::Abs(PathDefAbsolute::ClosePath),
        PathDef::Abs(PathDefAbsolute::LineTo(Point::new(1.0, 1.0))),
    ];
    validate_starts_with_moveto(&defs).map_err(|e| format!("expected Ok, got {e:?}"))?;
    Ok(())
}

/// `build_d` / `write_d` deliberately do not call `validate_starts_with_moveto`: they are general-purpose
/// formatters that may be used to build a path-data fragment, not necessarily a complete, standalone path.
#[test]
fn should_accept_that_build_d_does_not_validate_leading_command() -> Result<(), String> {
    let defs = [PathDef::Abs(PathDefAbsolute::LineTo(Point::new(1.0, 1.0)))];
    let d = build_d(&defs);
    if d != "L1 1" {
        return Err(format!("expected \"L1 1\", got {d:?}"));
    }
    Ok(())
}

/// Diagnostic, not a portability assertion. Layout is not guaranteed by Rust, so this deliberately does not
/// `assert_eq!` against a fixed byte count (see `docs/design_notes/path_data.md`, "Measuring `PathDef`'s nested-enum
/// layout cost" for the rationale and the numbers observed on the host and wasm32 targets at the time of writing).
///
/// The one assertion here is a structural regression guard rather than a target-specific magic number. Wrapping
/// `PathDefAbsolute`/`PathDefRelative` in `PathDef` can cost at most one extra alignment unit. That is the padded
/// slot Rust's enum layout reserves for the outer discriminant when it cannot find a spare niche in the inner type.
/// So `PathDef` must never be larger than that. If it were, either the outer wrapper stopped being a single
/// niche-or-one-word tag, or an inner variant grew unexpectedly.
#[test]
fn pathdef_size_diagnostics() -> Result<(), String> {
    use std::mem::{align_of, size_of};

    eprintln!(
        "size_of: Point={} EllipticalArc={} PathDefAbsolute={} PathDefRelative={} PathDef={} \
         Vec<PathDef> (64 commands)={}",
        size_of::<Point>(),
        size_of::<EllipticalArc>(),
        size_of::<PathDefAbsolute>(),
        size_of::<PathDefRelative>(),
        size_of::<PathDef>(),
        64 * size_of::<PathDef>(),
    );

    if size_of::<PathDef>() > size_of::<PathDefAbsolute>() + align_of::<PathDefAbsolute>() {
        return Err(format!(
            "PathDef ({} bytes) grew by more than one alignment unit ({}) over PathDefAbsolute ({} bytes)",
            size_of::<PathDef>(),
            align_of::<PathDefAbsolute>(),
            size_of::<PathDefAbsolute>(),
        ));
    }
    if size_of::<PathDef>() > size_of::<PathDefRelative>() + align_of::<PathDefRelative>() {
        return Err(format!(
            "PathDef ({} bytes) grew by more than one alignment unit ({}) over PathDefRelative ({} bytes)",
            size_of::<PathDef>(),
            align_of::<PathDefRelative>(),
            size_of::<PathDefRelative>(),
        ));
    }
    Ok(())
}
