# Gap Analysis

This crate offers a working foundation for generating SVG content driven by a `requestAnimationFrame` (RAF) loop.
It supports nested groups, `<defs>`, `<marker>`, `<linearGradient>`, `<radialGradient>`, batch building, and a full set of managed event wrappers.
However, it cannot yet produce anything with filters, clipping, or reusable symbols.

These gaps will be filled in time, but for now, this crate must be treated as a work-in-progress, not a general-purpose SVG library.

## Supported SVG elements

The following SVG elements are supported:

* `rect`
* `circle`
* `ellipse`
* `line`
* `polyline`
* `polygon`
* `path`
* `text`
* `g`
* `defs`
* `marker`
* `use`
* `image`
* `linearGradient` (with `stop`)
* `radialGradient` (with `stop`)

### `<defs>`

`<defs>` is the standard SVG container for reusable assets and can be obtained from `SvgRoot::defs()`.
All shape factory methods are available on `SvgDefs` for building inner content.

### `<marker>`

`<marker>` defines a reusable graphic (e.g. an arrowhead or a dot etc) rendered at the start, mid-point, or end of a stroked path and can be obtained from `SvgDefs::marker(id)`.

Apply it to any stroked element — `<line>`, `<path>`, `<polyline>`, `<polygon>` — via `SvgNode::set_marker_start`, `set_marker_mid`, or `set_marker_end`.
The `MarkerUnits` enum controls whether `markerWidth`/`markerHeight` are relative to `strokeWidth` (default) or user coordinates.

### `<image>`

`<image>` embeds a raster image (PNG, JPEG, WebP etc) or another SVG into the current document.
Obtain a handle via `SvgRoot::image(href, top_left, size)` or `SvgBatch::image(href, top_left, size)`.

- `href` accepts any URL the browser can fetch: a relative path, an absolute URL, or a `data:` URI.
  When using `data:image/svg+xml`, use base64 encoding to avoid percent-encoding `<`, `>`, and `#`.
- `top_left` and `size` define the display rectangle.
  Both width and height must be set; omitting either makes the image invisible.
- Control aspect-ratio handling with `set_attr("preserveAspectRatio", value)`:
  - `"xMidYMid meet"` — fit the whole image inside the box, adding letterbox bars if needed (default).
  - `"none"` — stretch to fill the box exactly, ignoring the source aspect ratio.
  - `"xMidYMid slice"` — scale up to fill the box and clip any overflow.
- To swap the image source after creation, call `SvgNode::set_href`.

### `<use>`

`<use>` stamps a copy of any element — typically one defined inside `<defs>` — into the rendered tree without duplicating the DOM node.
Obtain a handle via `SvgRoot::use_node(href, at)` or `SvgBatch::use_node(href, at)`.

- `href` is a local fragment reference such as `"#my-shape"` (the `id` attribute of the target element).
- `at` is an `(x, y)` offset in the parent coordinate system; pass `Point::origin()` to control positioning entirely through `transform`.
- Each returned `SvgNode` is independent: `transform`, `opacity`, `fill`, and other presentation attributes can be set per-copy without affecting the original.
- To change the referenced element after creation, call `SvgNode::set_href("#other-shape")`.

Any change to the original definition is immediately visible in all copies.
`<symbol>` is not yet supported; for now, define reusable content directly inside `<defs>` with a shape or group that carries an `id`.

### `<linearGradient>` / `<radialGradient>`

Gradient paint servers defined inside `<defs>` and referenced by shape `fill` or `stroke` attributes.
You can obtain such a paint server either from `SvgDefs::build_linear_gradient` or `build_radial_gradient`.
The live-append variants are `linear_gradient` and `radial_gradient`.

**`<linearGradient>`** paints a colour transition along a straight line.

- The axis runs from (`x1`, `y1`) to (`x2`, `y2`).
  Under the default `gradientUnits="objectBoundingBox"` these are fractions in the range `0.0` to `1.0` of the element's bounding box.
  If omitted, the default is a horizontal left-to-right gradient (SVG defaults: `x1=0`, `y1=0`, `x2=1`, `y2=0`).
- Use `set_gradient_transform("rotate(45, 0.5, 0.5)")` for a diagonal gradient without the need to compute trigonometric endpoint coordinates.
- A linear gradient can be applied to a shape's `fill` or `stroke` attributes using `SvgNode::set_fill_linear_gradient` or `SvgNode::set_stroke_linear_gradient`.

**`<radialGradient>`** radiates outward from some focal point at `fx / fy` through an outer circle centered ar `cx / cy` and having a radius of `r`.

- SVG uses the defaults `cx=0.5`, `cy=0.5`, `r=0.5`.
  This positions the focal point at the centre of the outer circle.
- Move the focal point with `set_fx` / `set_fy` to create asymmetric "hot spot" or spotlight effects.
- `set_fr` sets the focal circle radius (SVG 2) for a hollow centre.
- Apply with `SvgNode::set_fill_radial_gradient` / `SvgNode::set_stroke_radial_gradient`.

**Shared API** for both gradient types:

| Function | Description
|---|---|
| `add_stop(offset, color)` | Add a `<stop>` at `offset` (0.0–1.0) with `stop-color` and full opacity.
| `add_stop_opacity(offset, color, opacity)` | As above, but with explicit `stop-opacity`.
| `set_gradient_units(GradientUnits)` | Switch between `ObjectBoundingBox` (default) and `UserSpaceOnUse`.
| `set_spread_method(SpreadMethod)` | `Pad` (default), `Reflect`, or `Repeat` outside the stop range.
| `set_gradient_transform(transform)` | Arbitrary SVG transform applied to the gradient coordinate system.
| `set_attr` / `set_attrs` / `set_attr_display` | Generic escape hatches for other attributes.
| `id()` / `set_id()` | Cached id management; renaming does not retroactively update shape references.

***IMPORTANT***<br>

* All gradient ids must match the pattern `[A-Za-z_][A-Za-z0-9_-]*`.
* The ids used by the SVG paint-server are document-scoped not SVG-element-scoped; therefore, they must be globally unique across all `<svg>` elements on the page (using `url(#id)`)

A fully qualified prefix such as `"my-app-sky-gradient"` is a practical guard against collisions.

## Missing SVG elements

The following SVG elements still need to be implemented:

| Missing Element | Why it matters
|---|---|
| `<pattern>` | Tiled fill patterns
| `<clipPath>` | Masking regions
| `<symbol>` | Named reusable viewport; the companion to `<use>` for scaled/clipped stamp copies
| `<tspan>` | Multi-line or mixed-style text within a `<text>`
| `<textPath>` | Allows text to follow a curve
| `<filter>` and primitives | Drop shadows, blur, colour matrix, compositing etc

# Tree operations

Implemented:

- `remove()` — detach a node from the DOM
- `insert_before()` — z-order control without rebuilding
- `clear()` — remove all children of a node (e.g. to redraw a `<g>` from scratch)
- `replace_with()` — swap one node for another in place
- `parent()` — navigate up to the containing SVG element (returns an independent, non-factory handle)

Still missing:

- No downward/child navigation (`children()`, `first_child`, ...)
- No way to query the tree or find a node by attribute (`query_selector` and friends)

# Event coverage

Managed wrappers now cover the SVG interaction events expected by ordinary application code:

* click/double-click/context menu,
* mouse movement and button state,
* pointer lifecycle,
* wheel,
* touch,
* keyboard,
* focus/blur,
* drag-and-drop,
* a generic `on_event` escape hatch for event types not covered by a named wrapper, and
* `on_event_once` — a generic one-shot variant; accepts any event type `E` via an `instanceof` cast at runtime.
* Typed one-shot wrappers for every named event: `on_click_once`, `on_pointerdown_once`, `on_pointerenter_once`, `on_pointerleave_once`, and equivalents for all other named events.
  These bake in the correct event type so the `instanceof` mismatch footgun cannot occur.
* Passive variants for the three high-frequency scroll events — `on_wheel_passive`, `on_touchstart_passive`, and `on_touchmove_passive` — registered with `{ passive: true }` so the compositor thread is never blocked.
  Any `prevent_default()` call made inside a passive handler is silently ignored by the browser.
  If you do need to suppress the default scroll or touch behaviour, then use the non-passive sibling instead.

Prefer `pointerenter` / `pointerleave` for hover behaviour because they do not bubble through child elements.
The legacy `mouseover` / `mouseout` wrappers remain available for compatibility reasons, but have been marked as deprecated.

# Performance patterns

## High-frequency event coalescing

On some modern devices, the events generated by `pointermove`, `touchmove`, and `wheel` can arrive at the hardware polling rate, which could be as high as 1000 Hz (i.e. one event per millisecond); while the various browser events arrive at a rate between 60 and 120 Hz.

A handler that is called at the hardware polling rate could potentially call `set_translate` or `set_attr` on every delivered event, even though all but the last position before the next paint is immediately discarded.

Even though all but the last position before the next paint is discarded, such an architecture involves performing a Rust → JavaScript crossing, then a possible `setAttribute` DOM call, and a potential SVG layout invalidation for each event.

The `AnimationFrame` scratch buffer (see `AnimationLoop::start_with_frame`) removes per-event *allocation*, but it does not reduce the *count* of those crossings.

The fix is standard: record the latest value in the event handler and apply it at most once per animation frame.

Modern browsers partially automate this for pointer events via `getCoalescedEvents()`, but the pattern below works uniformly across all high-frequency event types.

The crate does not yet provide a built-in coalescer type, but the pattern can be built from existing primitives:

```rust
use std::{cell::{Cell, RefCell}, rc::Rc};
use svg_dom::{SvgRoot, WeakSvgNode, root::utils::{Point, Size}};
use wasm_bindgen::prelude::*;

let svg  = SvgRoot::attach("diagram")?;
let node = svg.rect(Point::origin(), Size::new(60.0, 60.0))?;
node.set_fill("steelblue")?;

// --- coalescer state ---
// `pending` holds the latest position submitted by the event handler.
// `scheduled` is true while a RAF has been requested but not yet dispatched.
let pending:   Rc<Cell<Option<Point>>> = Rc::new(Cell::new(None));
let scheduled: Rc<Cell<bool>>          = Rc::new(Cell::new(false));

// Clones for the RAF closure.
let pending_raf   = pending.clone();
let scheduled_raf = scheduled.clone();

// Use a weak handle so the RAF closure does not keep `node` alive, avoiding a
// reference cycle (listener store → closure → Rc<node> → listener store).
// See `WeakSvgNode` for the full explanation.
let node_weak = node.downgrade();

// The RAF callback: read the latest position, clear the scheduled flag, apply.
let raf_cb = Closure::<dyn FnMut()>::new(move || {
    scheduled_raf.set(false);
    if let Some(pt) = pending_raf.take() {
        if let Some(n) = node_weak.upgrade() {
            let _ = n.set_translate(pt.x, pt.y);
        }
    }
});

// Clones for the event handler.
let pending_ev   = pending.clone();
let scheduled_ev = scheduled.clone();
let window       = web_sys::window().unwrap();

node.on_pointermove(move |evt| {
    // Always overwrite with the freshest position.
    pending_ev.set(Some(Point::new(evt.client_x() as f64, evt.client_y() as f64)));
    // Request a frame only if one is not already queued.
    if !scheduled_ev.get() {
        scheduled_ev.set(true);
        let _ = window.request_animation_frame(raf_cb.as_ref().unchecked_ref());
    }
})?;

Ok::<(), svg_dom::Error>(())
```

Key points:

- `pending.set(...)` replaces whatever intermediate position was stored; only the last one matters.
- The `scheduled` flag ensures at most one `requestAnimationFrame` is queued at a time.
  When the RAF fires it clears the flag, so the next event will queue a fresh one.
- The RAF closure uses a `WeakSvgNode` to avoid a reference cycle.
  A strong `SvgNode` clone inside both the event handler and the RAF closure would keep the node (and its listeners) alive indefinitely.
- If you also need the `AnimationFrame` scratch buffer for formatted attribute writes, allocate one `AnimationFrame` outside both closures and put it behind an `Rc<RefCell<AnimationFrame>>`, then borrow it inside the RAF closure.

# Attribute helpers

Already implemented:

- Transform helpers — `set_translate`, `set_rotate`, `set_rotate_about`, `set_scale`, `set_scale_xy`, `set_translate_scale`, and `set_transform_fmt` for arbitrary transforms (all reuse a caller-owned scratch buffer)
- Updating `<text>` content after creation — `set_text`, plus the buffer-reusing `set_text_fmt` / `set_text_display`
- Allocation-light numeric attribute writes — `set_attr_display`, and the redundant-write helpers `set_attr_if_changed` / `CachedAttr`

Still missing:

- No `matrix(...)` transform helper specifically (use `set_transform_fmt` for now)
- No `viewBox` helper (only `set_viewport`, which sets `width`/`height`)
- No `classList` / CSS class manipulation
- No `text_anchor`, `dominant_baseline`, `font_family`, `font_size` helpers

# Missing geometry access

Geometry read-back is mostly absent.
The crate exposes text advance measurement through `SvgNode::computed_text_length`, but does not currently expose broader geometry APIs:

- `getBBox()` — bounding box in local coordinates
- `getTotalLength()` / `getPointAtLength()` — path measurement
- `getCTM()` / `getScreenCTM()` — coordinate system transforms
- `getBoundingClientRect()` — position relative to viewport
