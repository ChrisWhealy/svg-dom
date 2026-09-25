# svg-dom

[![CI](https://github.com/ChrisWhealy/svg-dom/actions/workflows/ci.yml/badge.svg)](https://github.com/ChrisWhealy/svg-dom/actions)
[![crates.io](https://img.shields.io/crates/v/svg-dom.svg)](https://crates.io/crates/svg-dom)
[![Documentation](https://docs.rs/svg-dom/badge.svg)](https://docs.rs/svg-dom)
[![Rust](https://img.shields.io/badge/rust-1.85.0%2B-blue.svg?maxAge=3600)](https://github.com/ChrisWhealy/svg-dom)

A lightweight Rust/WebAssembly library for creating and mutating live SVG content directly in the browser DOM.

The crate's planned feature set is implemented.
Deliberate exclusions are documented under [Implementation Non-goals](https://github.com/ChrisWhealy/svg-dom/blob/main/docs/non-goals.md).

All reasonable, conventional steps have been taken to provide a secure, stable and robust foundation upon which to develop future functionality.

***IMPORTANT***<br>This crate targets WebAssembly only.

## [Change Log](https://github.com/ChrisWhealy/svg-dom/blob/main/changelog.md)

# Table of Contents

- [What This Crate Is NOT](#what-this-crate-is-not)
- [What This Crate Is](#what-this-crate-is)
- [Building](#building)
- [Demo Gallery](#demo-gallery)
- [Quick Start Code Examples](https://github.com/ChrisWhealy/svg-dom/blob/main/docs/quick-start.md)
- [Testing](https://github.com/ChrisWhealy/svg-dom/blob/main/docs/testing.md)
- [Supported SVG Elements](https://github.com/ChrisWhealy/svg-dom/blob/main/docs/svg_elements/README.md)
- [Design Notes](https://github.com/ChrisWhealy/svg-dom/blob/main/docs/design_notes/README.md)
- [Implementation Non-goals](https://github.com/ChrisWhealy/svg-dom/blob/main/docs/non-goals.md)

## What This Crate Is NOT

This crate makes no use of the HTML `<canvas>` element!

Whilst the `<canvas>` element offers a pixel-based, bitmap drawing API that gives you the highest performance ceiling, it also requires you to take ownership of the entire layout, the render loop and hit-testing.

Not only is the implementation cost of such functionality high, it becomes somewhat redundant in light of the fact that SVG elements are already a persistent DOM tree that can be individually updated and with which JavaScript (via `web-sys`) can already interact.

Consequently, this crate works exclusively with the SVG DOM.

### Security and Trust Boundaries

The `svg-dom` crate is a DOM-construction library, not an SVG sanitizer.
If an SVG element consumes a caller-supplied URL, you must establish the safety of the URL-derived content before adding that SVG element to the DOM

| API | Untrusted input? | Principal risk |
|---|---|---|
| [`SvgNode::set_text`](https://docs.rs/svg-dom/latest/svg_dom/node/struct.SvgNode.html#method.set_text) and other text setters | Yes | None — written via `textContent`, never interpreted as markup |
| Typed id setters (`set_id`, gradient/pattern/marker/filter/mask/clip-path/view refs) | Yes, checked by this crate's own validation | Low — rejected before reaching the DOM |
| [`SvgNode::set_attr`](https://docs.rs/svg-dom/latest/svg_dom/node/struct.SvgNode.html#method.set_attr) / [`set_attrs`](https://docs.rs/svg-dom/latest/svg_dom/node/struct.SvgNode.html#method.set_attrs) | Safe only once the caller validates it | Script or event-handler attribute injection |
| URL-bearing methods — [`SvgRoot::anchor`](https://docs.rs/svg-dom/latest/svg_dom/root/svg_root/struct.SvgRoot.html#method.anchor)'s `href`, [`SvgRoot::image`](https://docs.rs/svg-dom/latest/svg_dom/root/svg_root/struct.SvgRoot.html#method.image)'s `href`, [`SvgNode::set_href`](https://docs.rs/svg-dom/latest/svg_dom/node/struct.SvgNode.html#method.set_href) | Validate against your own application policy first | Navigation to a `javascript:` URL, or an unwanted resource fetch |
| Style/paint strings — [`SvgRoot::style`](https://docs.rs/svg-dom/latest/svg_dom/root/svg_root/struct.SvgRoot.html#method.style)'s CSS, [`SvgNode::set_fill`](https://docs.rs/svg-dom/latest/svg_dom/node/struct.SvgNode.html#method.set_fill)/[`set_stroke`](https://docs.rs/svg-dom/latest/svg_dom/node/struct.SvgNode.html#method.set_stroke)'s paint value | Validate against your own application policy first | A CSS or SVG `url(...)` reference can still trigger a resource fetch |
| [`SvgNode::as_element`](https://docs.rs/svg-dom/latest/svg_dom/node/struct.SvgNode.html#method.as_element) / [`SvgRoot::root`](https://docs.rs/svg-dom/latest/svg_dom/root/svg_root/struct.SvgRoot.html#structfield.root) | Raw `web-sys` DOM access | None added or removed by this crate — the caller takes on ordinary browser-DOM responsibilities, such as never passing untrusted content to `set_inner_html` |

Every row already carries its own `# Security` section at the point of use.
This table is a map of where to look, not a new rule.

## What This Crate Is

The `svg-dom` crate acts as a thin wrapper for `web-sys` SVG DOM bindings that allows you to:

- Attach to an existing `<svg>` element in your HTML page
- Create a new `<svg>` element programmatically that gives you back a cheap-to-clone handle (`SvgNode`) that holds a live reference to the real DOM node
- Create the supported SVG elements listed in the [element guide](https://github.com/ChrisWhealy/svg-dom/blob/main/docs/svg_elements/README.md):
   - Helper functions exist for `<rect>`, `<circle>`, `<ellipse>`, `<line>`, `<polyline>`, `<polygon>`, `<path>`, `<text>`, `<g>`
   - Reusable assets can be defined for:
     
     | SVG Element | Type | Deferred-append Helper |
     |---|---|---|
     | `<defs>` | `SvgDefs` | `build_defs` |
     | `<marker>` | `SvgMarker` | `build_marker` |
     | `<linearGradient>` | `SvgLinearGradient` | `build_linear_gradient` |
     | `<radialGradient>` | `SvgRadialGradient` | `build_radial_gradient` |
     | `<clipPath>` | `SvgClipPath` | `build_clip_path` |
     | `<mask>` | `SvgMask` | `build_mask` |
     | `<pattern>` | `SvgPattern` | `build_pattern` |
     | `<filter>` | `SvgFilter` | `build_filter` |
     | `<symbol>` | `SvgSymbol` | `build_symbol` |
     | `<view>` | `SvgView` | `build_view` |
   
     The deferred-append helpers only commit the element to the DOM once the closure used for construction has succeeded.

     * Apply a clip path to any element with `set_clip_path_ref`
     * Apply a mask with `set_mask_ref`
     * Apply a tiled pattern fill/stroke with `set_fill_pattern_ref` / `set_stroke_pattern_ref`
  - `<use>` stamps an independently positionable and styleable copy of any element referenced by `id` without duplicating DOM nodes.
     Available through `SvgRoot::use_node` / `SvgBatch::use_node`
  - `<image>` embeds a raster image or SVG by URL or `data:` URI with full `preserveAspectRatio` control.
     Available through `SvgRoot::image` / `SvgBatch::image`
- Using the `SvgNode` handle, you can mutate the element's attributes either individually or as a batch:
   - without the need to rebuild or diff the DOM tree
   - via helpers such as `fill`, `stroke` and `d`
   - using `set_attrs` (alter multiple attributes in one call)
   - formatted values via `SvgAttrs`
- Attach managed event listeners directly to individual elements (listener event names are stored as `&'static str` making them allocation-free)
   - `mouse`
   - `pointer`
   - `wheel`
   - `touch`
   - `keyboard`
   - `focus/blur`
   - `drag-and-drop`
   - and generic `Event` handlers
- Drive reactive updates through a `requestAnimationFrame` loop via `AnimationLoop`
- Support for assistive technologies such as screen readers.

   Accessible name/description support via `<title>` / `<desc>` using `title`/`set_title` and `desc`/`set_desc`.

   These values are used when calculating the element's computed accessible name and description.
   However, when present, the name or description values supplied by ARIA will take precedence over `<title>` and `<desc>`.

   Browsers commonly expose `<title>` as a native hover tooltip.

# Building

Use [wasm-pack](https://wasm-bindgen.github.io/wasm-pack/) to build:

```sh
wasm-pack build --target web
```

# Demo Gallery

## [Demo Gallery Changelog](https://github.com/ChrisWhealy/svg-dom/blob/main/demo-app/changelog.md)

The demo gallery is not published as part of the crate's release, but once you have cloned this repo, you can build and run it locally as follows:

```sh
cargo demo
```

A demo gallery app is now available at <http://127.0.0.1:8080/demo>.
The coding used to create each demo is shown beneath each example.

To understand how the demo gallery is constructed, see [Demo Gallery Build Sequence](https://github.com/ChrisWhealy/svg-dom/blob/main/docs/demo-gallery-build.md)

# Quick Start Code Examples

To understand how to get started quickly, and see a minimal code sample, vist the [Quick Start](https://github.com/ChrisWhealy/svg-dom/blob/main/docs/quick-start.md) page.