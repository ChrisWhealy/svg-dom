//! Allocation-free redundant-write elision for high-frequency attribute updates.
//!
//! [`SvgNode::set_attr_if_changed`](crate::SvgNode::set_attr_if_changed) skips the DOM write when the value has not
//! changed, but to decide that it calls `get_attribute` — which **allocates a fresh Rust `String` for the current value
//! and crosses the wasm/JS boundary on every call**, even when it then writes nothing. On the exact "value usually
//! repeats" hot path it is meant for (a cursor style or `opacity` flag touched on every `pointermove`), that is a
//! per-event allocation plus a round-trip that buys only a comparison.
//!
//! `CachedAttr` removes both costs by remembering the last value it wrote on the **Rust** side. The no-op case becomes a
//! plain `&str` comparison against an owned `String`: no allocation, and no call into JS at all. The DOM is touched only
//! when the value genuinely changes, and even then the backing buffer is reused (`clear` + `push_str`) rather than
//! reallocated.
//!
//! The same cache also covers text content via [`CachedAttr::set_text`] — a status readout rewritten with the same
//! string on every `pointermove` is exactly the kind of redundant `set_text_content` it elides.
//!
//! Like the transform scratch buffer, a `CachedAttr` is **caller-owned** and deliberately not stored inside `SvgNode`:
//! passive nodes never animate, so they should not carry caching state. Keep one `CachedAttr` per value you update
//! frequently — typically captured in an event handler's state — and dedicate it to a single attribute (or the text
//! content) of a single node.
//!
//! # Example
//!
//! ```rust,no_run
//! use svg_dom::{root::utils::{Point, Size}, CachedAttr, SvgRoot};
//! let svg     = SvgRoot::attach("diagram")?;
//! let surface = svg.rect(Point::origin(), Size::new(100.0, 50.0))?;
//!
//! // Lives in the handler's captured state, reused across every event.
//! let mut cursor = CachedAttr::new();
//!
//! // Called many times per second; only the first call (and any real change) touches the DOM,
//! // and the unchanged case allocates nothing and never calls into JS.
//! cursor.set(&surface, "style", "cursor:grab")?;
//! cursor.set(&surface, "style", "cursor:grab")?; // no-op: cheap Rust string compare
//! Ok::<(), svg_dom::Error>(())
//! ```

use crate::{Error, SvgNode};
use std::fmt::{self, Write};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// A caller-owned cache of the last value written to one attribute, used to elide redundant DOM writes.
///
/// Prefer this to [`SvgNode::set_attr_if_changed`](crate::SvgNode::set_attr_if_changed) on genuinely high-frequency
/// paths: it remembers the last value on the Rust side, so the unchanged case is a plain string comparison with no
/// allocation and no call into JS at all. See the module-level notes above for the full rationale.
#[derive(Debug, Default)]
pub struct CachedAttr {
    last: String,
    written: bool,
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl CachedAttr {
    /// Creates an empty cache that has not yet written a value.
    ///
    /// The first [`set`](Self::set) call always writes, since there is no cached value to compare against.
    pub fn new() -> Self {
        Self::default()
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Writes `value` to `name` on `node`, but only if it differs from the value this cache last wrote.
    ///
    /// When the value is unchanged this returns `Ok(())` without allocating and without calling into the browser. When
    /// it changes, the attribute is written and the cached value is updated by reusing the existing buffer rather than
    /// allocating a new one.
    ///
    /// Use one `CachedAttr` per attribute. Reusing a single cache for several different attributes (or several nodes)
    /// would make its remembered value meaningless, since it tracks only the most recent write.
    pub fn set(&mut self, node: &SvgNode, name: &str, value: &str) -> Result<(), Error> {
        if self.written && self.last == value {
            // The value is unchanged, so bail out
        } else {
            node.set_attr(name, value)?;
            self.last.clear();
            self.last.push_str(value);
            self.written = true;
        }

        Ok(())
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Replaces the text content of `node`, but only if it differs from the value this cache last wrote.
    ///
    /// The text-content analogue of [`set`](Self::set): a status readout updated on every `pointermove` typically shows
    /// the same string frame after frame, and `set_text_content` marshals the string across the wasm/JS boundary on every
    /// call. Caching the last value turns the unchanged case into a plain string comparison with no allocation and no
    /// DOM work.
    ///
    /// Dedicate a `CachedAttr` to *either* an attribute (via [`set`](Self::set)) *or* text content — not both, since the
    /// two would share, and so clobber, the single remembered value.
    pub fn set_text(&mut self, node: &SvgNode, value: &str) -> Result<(), Error> {
        if self.written && self.last == value {
            // The value is unchanged, so bail out
        } else {
            node.set_text(value);
            self.last.clear();
            self.last.push_str(value);
            self.written = true;
        }

        Ok(())
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Formats `args` into the caller-owned `scratch` buffer and writes it via [`set`](Self::set).
    ///
    /// This is the allocation-light counterpart to `set(node, name, &format!(...))`: the candidate value is formatted
    /// into a reused buffer instead of a fresh `String` each call, so a frequently-touched but rarely-changing
    /// *formatted* attribute (a grid-snapped coordinate, a zoom percentage, etc) costs no per-call allocation **and**
    /// no DOM write while it is unchanged.
    ///
    /// `scratch` must be a buffer you own *separately* from the cache: the cache's own buffer holds the last-written
    /// value it compares against, so the new value needs somewhere else to be built. Reuse one `scratch` across calls
    /// (typically captured alongside the cache in the handler's state).
    pub fn set_fmt(
        &mut self,
        node: &SvgNode,
        name: &str,
        scratch: &mut String,
        args: fmt::Arguments<'_>,
    ) -> Result<(), Error> {
        scratch.clear();
        scratch.write_fmt(args)?;
        self.set(node, name, scratch)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Formats `args` into the caller-owned `scratch` buffer and writes it via [`set_text`](Self::set_text).
    ///
    /// The text-content counterpart to [`set_fmt`](Self::set_fmt): use it for a formatted status readout updated on
    /// every event but usually showing the same text, to avoid both the per-call `format!` allocation and the redundant
    /// `set_text_content`. See [`set_fmt`](Self::set_fmt) for the caller-owned `scratch` requirement.
    pub fn set_text_fmt(
        &mut self,
        node: &SvgNode,
        scratch: &mut String,
        args: fmt::Arguments<'_>,
    ) -> Result<(), Error> {
        scratch.clear();
        scratch.write_fmt(args)?;
        self.set_text(node, scratch)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Forgets the cached value so the next [`set`](Self::set) is guaranteed to write.
    ///
    /// Call this if the attribute was changed by some other code path (for example a plain `set_attr`, or an animation),
    /// so the cache does not wrongly believe a stale value is still current and skip a needed write. The backing buffer's
    /// capacity is retained.
    pub fn invalidate(&mut self) {
        self.last.clear();
        self.written = false;
    }
}
