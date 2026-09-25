# `svg-dom` Security

The `svg-dom` crate is a DOM-construction library, not an SVG sanitizer.

If an SVG element consumes a caller-supplied URL, you must establish the safety of the fetched content before adding that element to the DOM

## Trust Boundaries

| API | Untrusted input? | Principal risk |
|---|---|---|
| [`SvgNode::set_text`](https://docs.rs/svg-dom/latest/svg_dom/node/struct.SvgNode.html#method.set_text) and other text setters | Yes | None. This value is written via `textContent` so it is never interpreted as markup |
| Typed id setters (`set_id`, gradient, pattern, marker, filter, mask, clip-path, view refs) | Yes, checked by this crate's own validation | Low — rejected before reaching the DOM |
| [`SvgNode::set_attr`](https://docs.rs/svg-dom/latest/svg_dom/node/struct.SvgNode.html#method.set_attr) / [`set_attrs`](https://docs.rs/svg-dom/latest/svg_dom/node/struct.SvgNode.html#method.set_attrs) | Safety must be established by the caller | Script or event-handler attribute injection |
| URL-bearing methods such as [`SvgRoot::anchor`](https://docs.rs/svg-dom/latest/svg_dom/root/svg_root/struct.SvgRoot.html#method.anchor)'s `href`, [`SvgRoot::image`](https://docs.rs/svg-dom/latest/svg_dom/root/svg_root/struct.SvgRoot.html#method.image)'s `href`, [`SvgNode::set_href`](https://docs.rs/svg-dom/latest/svg_dom/node/struct.SvgNode.html#method.set_href) | Validate against your own application policy first | Navigation to a `javascript:` URL, or an unwanted resource fetch |
| Style/paint strings such as [`SvgRoot::style`](https://docs.rs/svg-dom/latest/svg_dom/root/svg_root/struct.SvgRoot.html#method.style)'s CSS, [`SvgNode::set_fill`](https://docs.rs/svg-dom/latest/svg_dom/node/struct.SvgNode.html#method.set_fill)/[`set_stroke`](https://docs.rs/svg-dom/latest/svg_dom/node/struct.SvgNode.html#method.set_stroke)'s paint value | Validate against your own application policy first | A CSS or SVG `url(...)` reference can still trigger a resource fetch |
| [`SvgNode::as_element`](https://docs.rs/svg-dom/latest/svg_dom/node/struct.SvgNode.html#method.as_element) / [`SvgRoot::root`](https://docs.rs/svg-dom/latest/svg_dom/root/svg_root/struct.SvgRoot.html#structfield.root) | Raw `web-sys` DOM access | None added or removed by this crate — the caller takes on ordinary browser-DOM responsibilities, such as never passing untrusted content to `set_inner_html` |

Every row already carries its own `# Security` section at the point of use.
This table is a map of where to look, not a new rule.

## Resource Limits for an Attacker-controlled Scene Description

This crate places no limit on element counts, path lengths, filter/mask region sizes, or convolution kernel dimensions.
This is simply because SVG itself leaves all of these limits unbounded; therefore, it would be inappropriate for this crate to attempt to second-guess the caller.

Some individual APIs already document their own performance cost:

* [`SvgFilter::set_width`](https://docs.rs/svg-dom/latest/svg_dom/root/filter/struct.SvgFilter.html#method.set_width) and [`set_height`](https://docs.rs/svg-dom/latest/svg_dom/root/filter/struct.SvgFilter.html#method.set_height) cover filter-region size.
* [`SvgFilter::convolve_matrix`](https://docs.rs/svg-dom/latest/svg_dom/root/filter/struct.SvgFilter.html#method.convolve_matrix) covers kernel order.

Those notes describe cost, not any limit enforced by this crate.

An untrusted, externally generated/supplied scene description can drive the same excesses as oversized hand-written SVG markup: for example, unbounded DOM growth, oversized filter regions, expensive convolution kernels.

A typical example of this is a third-party diagram format that has been converted to SVG by making calls on this crate.

Apply your own budget constraints before constructing such a scene!
You must decide what sized scene your application can acceptably render, then impose your own limits on things like:

* Element and path segment counts
* String lengths
* Filter primitives per filter
* Kernel dimensions
* Image dimensions
* Animation frame update rate
* etc...

This does not apply when your own Rust code decides what SVG to build: there, the scene's shape is already under your control.
