use super::{Point, Size};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// An axis-aligned rectangle, returned by [`SvgNode::bounding_box`](crate::SvgNode::bounding_box) and
/// [`SvgNode::bounding_client_rect`](crate::SvgNode::bounding_client_rect).
///
/// # Two producers, two different coordinate spaces
///
/// These two methods both return a `Rect`, but the coordinates are **not interchangeable**:
///
/// * [`bounding_box`](crate::SvgNode::bounding_box) wraps the no-argument form of `getBBox()` and reports **local,
///   user-space** coordinates — the same coordinate system the element's own `x`/`y`/`d`/`points` attributes are
///   authored in, unaffected by any transform applied to the element or its ancestors. It is also the **object/fill**
///   bounding box only: stroke width, markers, and clipping are not included (see
///   [`bounding_box`](crate::SvgNode::bounding_box)'s own doc comment). Empirically, in Chromium at least,
///   `getBoundingClientRect()` reports this same fill-only extent for SVG shape elements too — a wide stroke does
///   not necessarily widen either box, so do not assume `bounding_client_rect` is the "include everything painted"
///   alternative to `bounding_box`; verify against the specific browsers you target if that distinction matters.
/// * [`bounding_client_rect`](crate::SvgNode::bounding_client_rect) wraps `getBoundingClientRect()` and reports
///   **rendered CSS pixels**, relative to the browser viewport, after every transform, `viewBox` scale, and CSS zoom
///   has been applied.
///
/// The two will differ whenever any transform, `viewBox`, or CSS scaling is in play. Do not feed one method's `Rect`
/// into code that assumes the other's coordinate space — see `docs/design_notes/rejected_ideas/geometry.md`
/// ("Provide a rendered-size fallback...") for a worked example of exactly this mistake.
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    /// The rectangle's origin (top-left corner) — see the coordinate-space note above for which space this is in,
    /// depending on which method produced this `Rect`.
    pub origin: Point,
    /// The rectangle's size — see the coordinate-space note above.
    pub size: Size,
}
