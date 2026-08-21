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
//! | [`error`] | [`Error`] | Crate-wide error type for DOM and validation failures |
//! | [`root`] | [`SvgRoot`] / [`SvgAttrs`] | Wraps the `<svg>` root; factory for all child elements; reusable attribute writing |
//! | [`root::defs`] | [`SvgDefs`] | `<defs>` container; factory for markers, gradients, clip-paths, masks, patterns, filters, symbols, and views |
//! | [`root::gradient`] | [`SvgLinearGradient`] / [`SvgRadialGradient`] | Gradient paint servers defined in `<defs>` |
//! | [`root::clip_path`] | [`SvgClipPath`] / [`ClipPathUnits`] | Clipping region defined in `<defs>`, applied with `set_clip_path_ref` |
//! | [`root::mask`] | [`SvgMask`] / [`MaskUnits`] / [`MaskType`] | Luminance/alpha mask defined in `<defs>`, applied with `set_mask_ref` |
//! | [`root::filter`] | [`SvgFilter`] / [`FilterUnits`] | Raster-effect filter (`<feGaussianBlur>`, ...) defined in `<defs>`, applied with `set_filter_ref` |
//! | [`root::marker`] | [`SvgMarker`] / [`MarkerUnits`] | Path-decoration markers (`<marker>`) with shape factories |
//! | [`root::path`] | [`PathDef`] | Type-safe `<path>` `d`-attribute builder from a sequence of typed segments |
//! | [`root::pattern`] | [`SvgPattern`] / [`PatternUnits`] | Tiled pattern paint server defined in `<defs>`, applied with `set_fill_pattern_ref` |
//! | [`root::symbol`] | [`SvgSymbol`] | Reusable scaled viewport defined in `<defs>`, stamped via `<use>` |
//! | [`root::view`] | [`SvgView`] | Named `viewBox`/`preserveAspectRatio` defined in `<defs>`, navigated to via a `#id` URL fragment |
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
//! Text APIs avoid markup/script injection: text is written with `textContent`, never `innerHTML`, and the crate does
//! not use `eval`.
//! The interactive demo gallery (the separate `svg-dom-demo` workspace crate, built via `cargo demo`) does make use of
//! `innerHTML` for its own syntax-highlighted source-code panels; that is an implementation detail of that showcase
//! crate and forms no part of this library's own contract.
//!
//!
//! ⚠️ Caveat ⚠️
//!
//! APIs that accept raw URLs or attribute values remain trust boundaries.
//!
//! [`SvgNode::set_attr`](crate::SvgNode::set_attr) and [`set_attrs`](crate::SvgNode::set_attrs) are deliberate escape
//! hatches: they write attribute names and values **verbatim** via `setAttribute`.
//! [`SvgRoot::anchor`](crate::SvgRoot::anchor) is a typed API with the same exposure: SVG `<a>` is a genuine
//! navigation target, so its `href` argument — written verbatim, unvalidated — can be a `javascript:` URL that runs
//! on activation, exactly as an HTML `<a href="javascript:...">` would.
//!
//! Do not pass untrusted values to [`SvgRoot::anchor`](crate::SvgRoot::anchor)'s `href`, or to
//! [`SvgNode::set_attr`](crate::SvgNode::set_attr)/[`set_attrs`](crate::SvgNode::set_attrs), without validating them
//! first — treat them as you would any HTML sink.

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
        BlendMode, Channel, ColorMatrixType, CompositeOperator, EdgeMode, FilterUnits, LightSource, MorphologyOperator,
        SvgFilter, TransferFunction, TurbulenceType,
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
    view::SvgView,
};
