use crate::{
    error::Error,
    node::SvgNode,
    root::{
        path::path_def::{PathDef, validate_starts_with_moveto, write_d, write_d_fixed},
        utils::{Point, write_points},
    },
};
use std::fmt::{self, Write};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Scratch storage made available to an animation callback.
///
/// Since this code typically lies on the hot-path, calls to `format!(...)` or `value.to_string()` inside the
/// `set_attr...()` functions should always be avoided as this repeatedly allocates a short-lived `String` and thus
/// introduce a noticeable performance hit.
///
/// The scratch buffer starts empty and grows only as needed with subsequent calls always reusing the existing buffer.
/// Reallocation only takes place if the next value exceeds the existing buffer's capacity.
///
/// Use this with [`AnimationLoop::start_with_frame`](crate::AnimationLoop::start_with_frame) when a callback needs to
/// format SVG attribute values every frame.
///
/// # Example
///
/// ```rust,no_run
/// use std::cell::RefCell;
/// use svg_dom::{AnimationLoop, SvgRoot, root::utils::{Point, Size}};
///
/// // One page-lifetime slot to hold the running loop (this DOM-facing code runs on the page's main thread).
/// thread_local! {
///     static ANIM: RefCell<Option<AnimationLoop>> = const { RefCell::new(None) };
/// }
///
/// let svg = SvgRoot::attach("diagram").unwrap();
/// let rect = svg.rect(Point::new(10.0, 10.0), Size::new(80.0, 40.0)).unwrap();
///
/// let anim = AnimationLoop::start_with_frame(move |ts, frame| {
///     let alpha = 0.4 + 0.6 * (ts / 800.0).sin().abs();
///     let _ = frame.set_attr_fmt(&rect, "opacity", format_args!("{alpha:.3}"));
/// }).unwrap();
///
/// // Keep the loop alive for the page's lifetime. Dropping it would stop it via `Drop`.
/// ANIM.with(|slot| *slot.borrow_mut() = Some(anim));
/// ```
pub struct AnimationFrame {
    /// The reusable formatting buffer, exposed so callers can also write into it directly via [`scratch`](Self::scratch).
    pub scratch: String,
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl Default for AnimationFrame {
    fn default() -> Self {
        Self::new()
    }
}
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl AnimationFrame {
    /// Creates an `AnimationFrame` with a scratch buffer arbitrarily sized at 16 bytes.
    pub fn new() -> Self {
        Self {
            scratch: String::with_capacity(16),
        }
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Returns the reusable backing buffer used by this frame.
    ///
    /// This is useful when you want to build a value manually with `write!` and then pass it to one of the existing
    /// `SvgNode` setters. Callers should normally `clear()` the buffer before writing a new value.
    pub fn scratch(&mut self) -> &mut String {
        &mut self.scratch
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Formats `args` into the reusable buffer and sets `name` on `node`.
    ///
    /// This is the allocation-light equivalent of:
    ///
    /// ```rust,no_run
    /// # use svg_dom::{SvgRoot, root::utils::{Point, Size}};
    /// # let svg = SvgRoot::attach("diagram").unwrap();
    /// # let node = svg.rect(Point::origin(), Size::new(10.0, 10.0)).unwrap();
    /// # let x = 1.0;
    /// node.set_attr("transform", &format!("translate({x:.1}, 0)"));
    /// ```
    pub fn set_attr_fmt(&mut self, node: &SvgNode, name: &str, args: fmt::Arguments<'_>) -> Result<(), Error> {
        self.scratch.clear();
        self.scratch.write_fmt(args)?;
        node.set_attr(name, &self.scratch)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Writes a displayable value into the reusable buffer and sets `name` on `node`.
    pub fn set_attr<T: fmt::Display>(&mut self, node: &SvgNode, name: &str, value: T) -> Result<(), Error> {
        self.set_attr_fmt(node, name, format_args!("{value}"))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Formats and sets the `fill` attribute.
    pub fn set_fill_fmt(&mut self, node: &SvgNode, args: fmt::Arguments<'_>) -> Result<(), Error> {
        self.set_attr_fmt(node, "fill", args)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Formats and sets the `d` path-data attribute.
    pub fn set_d_fmt(&mut self, node: &SvgNode, args: fmt::Arguments<'_>) -> Result<(), Error> {
        self.set_attr_fmt(node, "d", args)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Writes `defs` into the reusable buffer via [`write_d`] and sets the node's `d` attribute.
    ///
    /// The per-frame counterpart to [`SvgAttrs::d_from_defs`](crate::SvgAttrs::d_from_defs): use it to morph a `<path>`
    /// from inside an [`AnimationLoop::start_with_frame`](crate::AnimationLoop::start_with_frame) callback from typed
    /// [`PathDef`] segments, without allocating a fresh string each frame.
    ///
    /// # Errors
    ///
    /// - [`Error::InvalidPathData`] — `defs` is non-empty and its first command is not a `MoveTo`. This is an O(1)
    ///   check of `defs[0]`, not a traversal, so it costs nothing extra per frame.
    pub fn set_d_from_defs(&mut self, node: &SvgNode, defs: &[PathDef]) -> Result<(), Error> {
        validate_starts_with_moveto(defs)?;
        write_d(&mut self.scratch, defs);
        node.set_attr("d", &self.scratch)
    }

    /// Like [`set_d_from_defs`](Self::set_d_from_defs), but writes each coordinate, length, and rotation angle with
    /// `dps` fixed decimal places (clamped to 20).
    ///
    /// The per-frame counterpart to [`SvgAttrs::d_from_defs_fixed`](crate::SvgAttrs::d_from_defs_fixed).
    /// It produces shorter per-frame output for a path whose coordinates come from a calculation, where the
    /// full-precision string would otherwise dominate the data crossing the WASM/JS boundary each frame.
    ///
    /// # Errors
    ///
    /// - [`Error::InvalidPathData`] — `defs` is non-empty and its first command is not a `MoveTo`.
    pub fn set_d_from_defs_fixed(&mut self, node: &SvgNode, defs: &[PathDef], dps: usize) -> Result<(), Error> {
        validate_starts_with_moveto(defs)?;
        write_d_fixed(&mut self.scratch, defs, dps);
        node.set_attr("d", &self.scratch)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Formats and replaces the node's text content.
    pub fn set_text_fmt(&mut self, node: &SvgNode, args: fmt::Arguments<'_>) -> Result<(), Error> {
        self.scratch.clear();
        self.scratch.write_fmt(args)?;
        node.set_text(&self.scratch);
        Ok(())
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Formats `points` into the reusable buffer and sets the node's `points` attribute (`"x,y x,y ..."`).
    ///
    /// The per-frame counterpart to [`SvgAttrs::points`](crate::SvgAttrs::points): use it to animate the vertices of a
    /// `<polyline>` or `<polygon>` from inside an [`AnimationLoop::start_with_frame`](crate::AnimationLoop::start_with_frame)
    /// callback without allocating a fresh string each frame.
    pub fn set_points(&mut self, node: &SvgNode, points: &[Point]) -> Result<(), Error> {
        write_points(&mut self.scratch, points, None);
        node.set_attr("points", &self.scratch)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Like [`set_points`](Self::set_points), but writes each coordinate with `decimals` fixed decimal places
    /// (clamped to 20).
    ///
    /// This gives shorter per-frame output for large animated `<polyline>`/`<polygon>` geometry, where the
    /// full-precision string would otherwise dominate the data crossing the WASM/JS boundary each frame. See
    /// [`SvgAttrs::points_fixed`](crate::SvgAttrs::points_fixed).
    pub fn set_points_fixed(&mut self, node: &SvgNode, points: &[Point], decimals: usize) -> Result<(), Error> {
        write_points(&mut self.scratch, points, Some(decimals));
        node.set_attr("points", &self.scratch)
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[cfg(test)]
mod unit_tests;
