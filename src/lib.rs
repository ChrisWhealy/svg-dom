// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
//! Live SVG DOM manipulation for Rust/WebAssembly.
//!
//! `svg_dom` lets you create, style, and animate SVG elements directly in the browser DOM without needing either to
//! rebuild or diff a virtual tree.
//!
//! Every element you create is returned as an [`SvgNode`].  This is a cheap-to-clone `Rc`-backed handle to the real DOM
//! node — so you can update its attributes or attach event listeners at any time.
//!
//! # Crate layout
//!
//! | Module | Key type | Purpose |
//! |---|---|---|
//! | [`demo`] | | Provides a set of SVG element examples.  Run `cargo demo` then visit http://localhost:8000/demo. |
//! | [`error`] | [`Error`] | Wrapper for Browser DOM errors |
//! | [`root`] | [`SvgRoot`] / [`SvgAttrs`] | Wraps the `<svg>` root; factory for all child elements; reusable attribute writing |
//! | [`animate`] | [`AnimationLoop`] | `requestAnimationFrame` loop |
//! | [`node`] | [`SvgNode`] | Live element handle that provides access to attributes, events and tree operations |
//!
//! # Minimal example
//!
//! ```rust,no_run
//! use svg_dom::{AnimationLoop, SvgRoot, root::utils::{Point, Size}};
//!
//! // Attach to <svg id="vis"> in the page, add a rect, animate its colour.
//! let svg  = SvgRoot::attach("vis").unwrap();
//! let rect = svg.rect(Point::new(10.0, 10.0), Size::new(80.0, 40.0)).unwrap();
//! rect.set_fill("steelblue").unwrap();
//!
//! let _loop = AnimationLoop::start_with_frame(move |ts, frame| {
//!     let lightness = 30 + ((ts / 1000.0).sin().abs() * 40.0) as u8;
//!     let _ = frame.set_fill_fmt(&rect, format_args!("hsl(210,70%,{lightness}%)"));
//! }).unwrap();
//! ```
//!
//! # Safety and security
//!
//! The crate contains no `unsafe` code (this is enforced with `#![forbid(unsafe_code)]` for the library build).
//!
//! It is also safe by construction against script injection: text is written through `textContent`, never `innerHTML`,
//! and there is no use of `eval`.
//!
//! The one thing to be aware of is [`SvgNode::set_attr`](crate::SvgNode::set_attr) (and [`set_attrs`](crate::SvgNode::set_attrs)),
//! write attribute names and values **verbatim** via `setAttribute`.
//!
//! Passing attacker-controlled input there can introduce script — for example an `onclick` attribute, or an `href`
//! whose value is `javascript:...`. Treat attribute names and values as you would any HTML sink: do not pass untrusted
//! data without validating it first.

#![cfg_attr(not(feature = "demo"), forbid(unsafe_code))]

pub mod animate;
pub mod error;
pub mod node;
pub mod root;

#[cfg(feature = "demo")]
pub mod demo;

pub use animate::{anim_frame::AnimationFrame, anim_loop::AnimationLoop};
pub use error::Error;
pub use node::{CachedAttr, SvgNode};
pub use root::{
    attrs::{AttrWriter, SvgAttrs},
    batch::SvgBatch,
    svg_root::SvgRoot,
};
