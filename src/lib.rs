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
//! | `demo` (feature `demo`) | | Provides a set of SVG element examples.  Run `cargo demo` then visit http://localhost:8080/demo. |
//! | [`error`] | [`Error`] | Wrapper for Browser DOM errors |
//! | [`root`] | [`SvgRoot`] / [`SvgAttrs`] | Wraps the `<svg>` root; factory for all child elements; reusable attribute writing |
//! | [`root::defs`] | [`SvgDefs`] | `<defs>` container; factory for markers, gradients, clip-paths, patterns, filters, and symbols |
//! | [`root::gradient`] | [`SvgLinearGradient`] / [`SvgRadialGradient`] | Gradient paint servers defined in `<defs>` |
//! | [`root::clip_path`] | [`SvgClipPath`] / [`ClipPathUnits`] | Clipping region defined in `<defs>`, applied with `set_clip_path_ref` |
//! | [`root::mask`] | [`SvgMask`] / [`MaskUnits`] / [`MaskType`] | Luminance/alpha mask defined in `<defs>`, applied with `set_mask_ref` |
//! | [`root::filter`] | [`SvgFilter`] / [`FilterUnits`] | Raster-effect filter (`<feGaussianBlur>`, ...) defined in `<defs>`, applied with `set_filter_ref` |
//! | [`root::marker`] | [`SvgMarker`] / [`MarkerUnits`] | Path-decoration markers (`<marker>`) with shape factories |
//! | [`root::path`] | [`PathDef`] | Type-safe `<path>` `d`-attribute builder from a sequence of typed segments |
//! | [`root::pattern`] | [`SvgPattern`] / [`PatternUnits`] | Tiled pattern paint server defined in `<defs>`, applied with `set_fill_pattern_ref` |
//! | [`root::symbol`] | [`SvgSymbol`] | Reusable scaled viewport defined in `<defs>`, stamped via `<use>` |
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
//! The SVG-building API is also safe by construction against script injection: all text content set through the public
//! API uses `textContent`, never `innerHTML`, and there is no use of `eval`.
//! The fact that the `demo` module makes use of `innerHTML` is simply an implementation detail of the showcase module
//! and forms no part of the library contract.
//!
//! The one thing to be aware of is [`SvgNode::set_attr`](crate::SvgNode::set_attr) (and [`set_attrs`](crate::SvgNode::set_attrs)),
//! write attribute names and values **verbatim** via `setAttribute`.
//!
//! Passing attacker-controlled input there can introduce script — for example an `onclick` attribute, or an `href`
//! whose value is `javascript:...`. Treat attribute names and values as you would any HTML sink: do not pass untrusted
//! data without validating it first.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

/// The [`AnimationLoop`] `requestAnimationFrame` driver and its per-frame [`AnimationFrame`] scratch buffer.
pub mod animate;
/// The crate's [`Error`] type, covering every failure mode of the DOM-facing API.
pub mod error;
/// The live element handle [`SvgNode`] and its attribute, text, transform, event, and tree-operation API.
pub mod node;
/// The `<svg>` root [`SvgRoot`], the element factories, batching, and the reusable attribute writer.
pub mod root;

/// Interactive browser element gallery, compiled only under the `demo` feature.
#[cfg(feature = "demo")]
pub mod demo;

pub use animate::{anim_frame::AnimationFrame, anim_loop::AnimationLoop};
pub use error::Error;
pub(crate) use error::dom_err;
pub use node::{
    CachedAttr, DominantBaseline, SvgNode, TextAnchor, TextPathMethod, TextPathSide, TextPathSpacing, WeakSvgNode,
};
pub use root::{
    attrs::{AttrWriter, SvgAttrs},
    batch::SvgBatch,
    clip_path::{ClipPathUnits, SvgClipPath},
    defs::SvgDefs,
    filter::{
        BlendMode, Channel, ColorMatrixType, CompositeOperator, FilterUnits, MorphologyOperator, SvgFilter,
        TransferFunction, TurbulenceType,
    },
    gradient::{GradientUnits, SpreadMethod, linear::SvgLinearGradient, radial::SvgRadialGradient},
    marker::{MarkerUnits, SvgMarker},
    mask::{MaskType, MaskUnits, SvgMask},
    path::{
        PathDef, PathDefAbsolute, PathDefRelative, build_d, build_d_fixed,
        elliptical_arc::{ArcSize, ArcSweep, EllipticalArc},
        write_d, write_d_fixed,
    },
    pattern::{PatternUnits, SvgPattern},
    svg_root::SvgRoot,
    symbol::SvgSymbol,
};
