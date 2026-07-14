use crate::root::utils::{MAX_DPS, Point};
use std::fmt::Write;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Which of the two candidate elliptical arcs to draw between the start and end points (the SVG `large-arc-flag`).
///
/// An elliptical arc between two points with a given radius has exactly two geometric solutions; `ArcSize` picks
/// between them, and [`ArcSweep`] then picks the rotation direction for the chosen one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArcSize {
    /// `large-arc-flag = 0` — The arc spans 180° or less.
    Small,
    /// `large-arc-flag = 1` — The arc spans more than 180°.
    Large,
}

impl ArcSize {
    fn flag(self) -> u8 {
        match self {
            ArcSize::Small => 0,
            ArcSize::Large => 1,
        }
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Controls the direction in which the elliptical arc sweeps (the SVG `sweep-flag`).
///
/// See [`ArcSize`] for the other half of the arc-selection pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArcSweep {
    /// `sweep-flag = 0` — the arc is drawn counter-clockwise (negative-angle direction).
    CounterClockwise,
    /// `sweep-flag = 1` — the arc is drawn clockwise (positive-angle direction).
    Clockwise,
}

impl ArcSweep {
    fn flag(self) -> u8 {
        match self {
            ArcSweep::CounterClockwise => 0,
            ArcSweep::Clockwise => 1,
        }
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Parameters for an elliptical arc path segment (the SVG `A`/`a` command).
///
/// Bundled into a named-field struct rather than a five-element tuple variant so that the two SVG boolean flags
/// (`large-arc-flag`, `sweep-flag`) become the self-documenting [`ArcSize`] and [`ArcSweep`] enums instead of two
/// adjacent `bool`s, which are easy to transpose by mistake and give no clue at the call site which is which.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EllipticalArc {
    /// Ellipse x/y radii.
    pub radii: Point,
    /// Rotation of the ellipse's x-axis, in degrees.
    pub x_axis_rotation: f64,
    /// Which of the two candidate arcs to draw.
    pub size: ArcSize,
    /// Which direction the arc sweeps.
    pub sweep: ArcSweep,
    /// The arc's end point — absolute or relative to the current point, depending on whether this value is wrapped
    /// in a [`super::PathDefAbsolute::EllipticalArcTo`] or a [`super::PathDefRelative::EllipticalArcTo`].
    pub to: Point,
}

impl EllipticalArc {
    /// Outputs a path segment for this elliptical arc using the given command character.
    ///
    /// `dps` (clamped to `MAX_DPS` = 20) fixes the decimal precision of the radii, rotation, and end point — `None`
    /// uses the default shortest round-trip representation.
    ///
    /// The two flags are deliberately never subject to `dps`: the SVG path grammar's `flag` production is exactly
    /// one `"0"` or `"1"` digit, not a decimal number, so they are always written via `ArcSize`/`ArcSweep`'s `u8`
    /// `Display` regardless of the requested precision — rounding them would either be a no-op (they are already
    /// integral) or, worse, invalid path syntax if ever formatted with a decimal point.
    ///
    /// `cmd` is restricted to `'A'`/`'a'` by construction, not by validation: `pub(super)` keeps this callable only
    /// from the two known-correct call sites in `path_def.rs`, so an external caller can never pass an arbitrary
    /// `char` here and produce an invalid command letter — the same guarantee `PathDef` gives the `d` string as a
    /// whole would otherwise leak right back out through this one method.
    pub(super) fn write(&self, out: &mut String, cmd: char, dps: Option<usize>) {
        match dps {
            Some(n) => {
                let n = n.min(MAX_DPS);
                let _ = write!(
                    out,
                    "{cmd}{:.n$} {:.n$} {:.n$} {} {} {:.n$} {:.n$}",
                    self.radii.x,
                    self.radii.y,
                    self.x_axis_rotation,
                    self.size.flag(),
                    self.sweep.flag(),
                    self.to.x,
                    self.to.y,
                );
            },
            None => {
                let _ = write!(
                    out,
                    "{cmd}{} {} {} {} {} {} {}",
                    self.radii.x,
                    self.radii.y,
                    self.x_axis_rotation,
                    self.size.flag(),
                    self.sweep.flag(),
                    self.to.x,
                    self.to.y,
                );
            },
        }
    }
}
