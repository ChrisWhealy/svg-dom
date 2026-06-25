use crate::{error::Error, node::SvgNode};
use std::fmt::{self, Write};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Scratch storage made available to an animation callback.
///
/// Use this with [`AnimationLoop::start_with_frame`](crate::AnimationLoop::start_with_frame) when a callback needs to
/// format SVG attribute values every frame.
/// The internal `String` is allocated once and then reused, avoiding the repeated short-lived allocations caused by
/// `format!(...)` or `value.to_string()` inside a `requestAnimationFrame` loop.
///
/// # Example
///
/// ```rust,no_run
/// use svg_dom::{AnimationLoop, SvgRoot, root::utils::{Point, Size}};
///
/// let svg = SvgRoot::attach("diagram").unwrap();
/// let rect = svg.rect(Point::new(10.0, 10.0), Size::new(80.0, 40.0)).unwrap();
///
/// let anim = AnimationLoop::start_with_frame(move |ts, frame| {
///     let alpha = 0.4 + 0.6 * (ts / 800.0).sin().abs();
///     let _ = frame.set_attr_fmt(&rect, "opacity", format_args!("{alpha:.3}"));
/// }).unwrap();
///
/// std::mem::forget(anim);
/// ```
#[derive(Default)]
pub struct AnimationFrame {
    /// The reusable formatting buffer, exposed so callers can also write into it directly via [`scratch`](Self::scratch).
    pub scratch: String,
}

impl AnimationFrame {
    /// Creates an `AnimationFrame` with an empty scratch buffer.
    pub fn new() -> Self {
        Self { scratch: String::new() }
    }

    /// Returns the reusable backing buffer used by this frame.
    ///
    /// This is useful when you want to build a value manually with `write!` and then pass it to one of the existing
    /// `SvgNode` setters. Callers should normally `clear()` the buffer before writing a new value.
    pub fn scratch(&mut self) -> &mut String {
        &mut self.scratch
    }

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
        self.scratch
            .write_fmt(args)
            .map_err(|_| Error::Dom("failed to format SVG attribute".into()))?;
        node.set_attr(name, &self.scratch)
    }

    /// Writes a displayable value into the reusable buffer and sets `name` on `node`.
    pub fn set_attr<T: fmt::Display>(&mut self, node: &SvgNode, name: &str, value: T) -> Result<(), Error> {
        self.set_attr_fmt(node, name, format_args!("{value}"))
    }

    /// Formats and sets the `fill` attribute.
    pub fn set_fill_fmt(&mut self, node: &SvgNode, args: fmt::Arguments<'_>) -> Result<(), Error> {
        self.set_attr_fmt(node, "fill", args)
    }

    /// Formats and sets the `d` path-data attribute.
    pub fn set_d_fmt(&mut self, node: &SvgNode, args: fmt::Arguments<'_>) -> Result<(), Error> {
        self.set_attr_fmt(node, "d", args)
    }

    /// Formats and replaces the node's text content.
    pub fn set_text_fmt(&mut self, node: &SvgNode, args: fmt::Arguments<'_>) -> Result<(), Error> {
        self.scratch.clear();
        self.scratch
            .write_fmt(args)
            .map_err(|_| Error::Dom("failed to format SVG text".into()))?;
        node.set_text(&self.scratch);
        Ok(())
    }
}
