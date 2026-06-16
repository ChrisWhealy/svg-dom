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
//! | [`root`] | [`SvgRoot`] | Wraps the `<svg>` root; factory for all child elements |
//! | [`animate`] | [`AnimationLoop`] | `requestAnimationFrame` loop |
//! | [`node`] | [`SvgNode`] | Live element handle that provides access to attributes, events and tree operations |
//!
//! # Minimal example
//!
//! ```rust,no_run
//! use svg_dom::{AnimationLoop, SvgRoot};
//!
//! // Attach to <svg id="vis"> in the page, add a rect, animate its colour.
//! let svg  = SvgRoot::attach("vis").unwrap();
//! let rect = svg.rect(Point::new(10.0, 10.0), Size::new(80.0, 40.0)).unwrap();
//! rect.set_fill("steelblue").unwrap();
//!
//! let _loop = AnimationLoop::start(move |ts| {
//!     let lightness = 30 + ((ts / 1000.0).sin().abs() * 40.0) as u8;
//!     let _ = rect.set_fill(&format!("hsl(210,70%,{lightness}%)"));
//! }).unwrap();
//! ```

pub mod animate;
pub mod error;
pub mod node;
pub mod root;

#[cfg(feature = "demo")]
pub mod demo;

pub use animate::AnimationLoop;
pub use error::Error;
pub use node::SvgNode;
pub use root::svg_root::SvgRoot;
