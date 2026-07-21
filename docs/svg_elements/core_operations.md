# Core operations

[← Back to supported elements](README.md)

**Contents**

- [Implemented Tree operations](#implemented-tree-operations)
- [Event coverage](#event-coverage)
- [Implemented Attribute helpers](#implemented-attribute-helpers)
- [Implemented geometry helpers](#implemented-geometry-helpers)
- [Implemented accessibility helpers](#implemented-accessibility-helpers)

These capabilities apply to every `SvgNode` regardless of the underlying element type.

# Implemented Tree operations

| Method | Description |
| --- | --- |
| `remove()` | Detach a node from the DOM |
| `insert_before()` | Z-order control without rebuilding |
| `clear()` | Remove all children of a node (e.g. to redraw a `<g>` from scratch) |
| `replace_with()` | Swap one node for another in place |
| `parent()` | Navigate up to the containing SVG element (returns an independent, non-factory handle) |
| `first_child()`, `last_child()`, `next_sibling()` `previous_sibling()` | Navigate down/across without having kept a handle to the target (returns independent, non-factory handles, like `parent()`) |
| `children()` | Every SVG child element, in document order (independent, non-factory handles) |
| `query_selector()` `query_selector_all()` | Find descendant(s) anywhere in the subtree by CSS selector, including by attribute (independent, non-factory handles) |

***IMPORTANT***

Every handle returned by the tree navigation and query methods above is a **fresh, independent** owner of its element, not a reference to whatever handle originally created it.
This is the same caveat that applies to the use of `parent()`.

In particular, you should not register event listeners for the element obtained via one of these handles; see `SvgNode::parent`'s doc comment for the full explanation.

All non-SVG matches (for example HTML content inside a `<foreignObject>`) are silently skipped rather than returned.

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
* Passive variants for the three high-frequency scroll events (`on_wheel_passive`, `on_touchstart_passive`, and `on_touchmove_passive`) registered in the DOM with `{ passive: true }` so the compositor thread is never blocked.

  Any `prevent_default()` call made inside a passive handler is silently ignored by the browser.

  If you do need to suppress the default scroll or touch behaviour, then use the non-passive sibling instead.

Prefer the use of `pointerenter` and `pointerleave` for hover behaviour because these events do not bubble through child elements.
The legacy `mouseover` / `mouseout` wrappers remain available for compatibility reasons, but have been marked as deprecated.

# Implemented Attribute helpers

## Transform helpers

`set_translate`, `set_rotate`, `set_rotate_about`, `set_scale`, `set_scale_xy`, `set_translate_scale`, `set_matrix`/`set_matrix_precise`

For skew/shear or any transform not expressible via the named helpers listed above, you can create a 2D affine matrix via the role-named `Matrix2D { h_scale, v_scale, h_skew, v_skew, h_trans, v_trans }`.

The arguments to `set_matrix` are quantised at 3 and 1 decimal places for compact hot-path output; however, due to the possibility that this quantisation might create rounding errors, the use of `set_matrix` might produce jerky animation effects particularly when slow or precise control is needed.
To avoid this, `set_matrix_precise` uses exact shortest-round-trip formatting for matrices computed elsewhere, and `set_transform_fmt` for anything else (all reuse a caller-owned scratch buffer).

## `<text>`

To update `<text>` content after creation, use `set_text`, plus the buffer-reusing methods `set_text_fmt` and `set_text_display`

## Allocation-light Numeric Attribute Writes

`set_attr_display` and the redundant-write helpers `set_attr_if_changed` / `CachedAttr`

## View Box

`SvgRoot::set_view_box(x, y, width, height)` sets the root `<svg>`'s internal coordinate system, independent of `set_viewport`'s `width`/`height`.

`SvgSymbol`, `SvgPattern`, and `SvgMarker` have the same method for their own `viewBox`; see [`<symbol>`](structural_elements.md#symbol), [`<pattern>`](paint_servers.md#pattern), and [`<marker>`](structural_elements.md#marker).

## CSS Class Manipulation

To manipulate CSS classes on `SvgNode`, use `add_class`, `remove_class`, `toggle_class`, `set_class_enabled` (deterministic set/clear via `classList.toggle(token, force)`), `has_class`, backed by the DOM `classList` API.

# Implemented Geometry Helpers

Read-only geometry queries on `SvgNode`.

Each call crosses into the browser and potentially triggers synchronous style or layout calculation if the relevant geometry is not already current:

- `bounding_box()`

  A no-argument form of `getBBox()` that returns a local, user-space bounding box, i.e. the **object/fill** bounding box only (invoked with`fill=true`, `stroke=false`, `markers=false`, `clipped=false`) where the stroke width, markers, and clipping are not included.
  Consequently, the returned bounding box can be visibly smaller than the painted contents.

  `Err` might be returned if:

  - the browser rejects the call
  - the element does not implement `SVGGraphicsElement`

  Most rendered shapes do implement `SVGGraphicsElement`; however, there are some non-rendering elements (such as the filter primitives e.g. `SvgFilter::gaussian_blur`, `offset`, `merge` etc.) that also return a plain `SvgNode`, so this is a reachable case, not just a defensive one.

- `bounding_client_rect()`

  Rendered bounding rectangle whose size is given in viewport CSS pixels (`getBoundingClientRect()`).
  This reflects every transform, `viewBox` scale and CSS zoom currently in effect.

  This method is infallible and is available on every element.

  **IMPORTANT**<br>
  This does not use the same coordinate space as `bounding_box()` — see `Rect`'s own doc comment.

- `ctm()` and `screen_ctm()`

  Returns the current transformation matrix as the same role-named `Matrix2D` used by `set_matrix` and `set_matrix_precise`.

  `ctm()` accumulates every ancestor transform up to the nearest *viewport* ancestor; whereas, `screen_ctm()` continues all the way to the document viewport's CSS-pixel coordinates additionally carrying the root `<svg>`'s own position on the page.

  In spite of its name, this method does not use the physical screen/monitor coordinates.

  Both return accumulated coordinate-conversion matrices, which is not generally this element's own local transform.

  Writing the ctm straight back via `set_matrix` or `set_matrix_precise` is only correct when there is a one-to-one scaling relationship between the parent and the viewport (i.e., the parent-to-viewport transform is the identity matrix).

  Converting a *point* between viewport coordinates and the element's local coordinates (which requires you to invert this element's own `screen_ctm()`) and recovering this element's own writable local *transform* (compare this element's `ctm()` against its parent's) are two different operations — see [`design_notes/geometry.md`](../design_notes/geometry.md#ctmscreen_ctm-are-accumulated-matrices-not-generally-the-elements-own-local-transform) for both.

  Both `ctm()` and `screen_ctm()` will return `None` if the element is not currently rendered.

- `total_length()` and `point_at_length(distance)`

  Path measurement (`getTotalLength()` and `getPointAtLength()`) are only meaningful for `<rect>`, `<circle>`, `<ellipse>`, `<line>`, `<polyline>`, `<polygon>`, and `<path>`.

  **WARNING**<br>
  `total_length()` will return `None`, and `point_at_length()` will return `Err` if called on an element that does not implement `SVGGeometryElement` (such as `<text>`, `<tspan>`, `<textPath>`, `<use>`, `<image>`, `<g>`, the root `<svg>`).

# Implemented accessibility helpers

`<title>` and `<desc>` child elements are supported generically on `SvgNode`, so they work on any element such as a shape or a group.

The root `<svg>` element itself is not an `SvgNode`, it is wrapped by the separate `SvgRoot` type.
So `SvgRoot` forwards the same six methods (`set_title`, `title`, `remove_title`, `set_desc`, `desc`, `remove_desc`) directly onto the root element, since naming the whole document/diagram is one of the principal use cases for this API, not an edge case:

```rust,no_run
use svg_dom::SvgRoot;
let svg = SvgRoot::attach("diagram")?;
svg.set_title("Quarterly sales chart")?;
svg.set_desc("A bar chart comparing sales across four regions")?;
Ok::<(), svg_dom::Error>(())
```

***IMPORTANT***<br>
Use these methods judiciously: not every element needs a name or description.

Adding a non-empty `<title>` or `<desc>` can cause an otherwise purely decorative or presentational element to be exposed to assistive technology as its own separate object in the accessibility tree.
That is exactly the point for meaningful icons, controls, diagrams, and diagram components — but naming every individual decorative path or primitive produces a noisy, cumbersome accessibility tree that works against the users it is meant to help.

Because `set_title`/`set_desc` are generic on `SvgNode`, they are callable on almost any element this crate hands back, which makes it easy to over-apply them.
As a rule of thumb: attach `<title>`/`<desc>` to elements that are meaningful on their own — icons, controls, whole diagrams, or a `<g>` representing one logical idea — and leave purely decorative geometry (the individual paths/shapes that only exist to render a larger meaningful group) unnamed, so it is not individually exposed.

A `<title>`/`<desc>` also does not, by itself, make an element interactive: it makes a graphic *describable*, not a control.
If an icon is meant to be operable (clickable, focusable, activatable from the keyboard), that behaviour has to be built independently — a suitable `role`, a `tabindex`, and keyboard event handling — none of which these two methods provide.

| Method | Effect |
|---|---|
| `set_title(text)` | Creates (or updates) the *first* direct `<title>` child. |
| `title()` | Returns the text of the *first* direct `<title>` child, or `None` if there isn't one. |
| `remove_title()` | Idempotent removal of the *first* direct `<title>` child. |
| `set_desc(text)` | Creates (or updates) the *first* direct `<desc>` child. |
| `desc()` | Returns the text of the *first* direct `<desc>` child, or `None` if there isn't one. |
| `remove_desc()` | Idempotent removal of the *first* direct `<desc>` child. |

***IMPORTANT — this is a first-child convenience API, not a full DOM manager***<br>
All six methods listed above operate on whichever `<title>`/`<desc>` happens to be this element's *first* matching direct child.
They are a simple, single-value convenience for the common case, not a general manager of every `<title>`/`<desc>` an element might have.

SVG 2 deliberately permits **multiple `<title>`/`<desc>` siblings on one element, one per language**, with the user agent selecting the most appropriate one via `lang`/`xml:lang`.
This crate does not implement that selection, and these methods make **no attempt to enforce singularity** on DOM they did not build from scratch: if an element already has more than one `<title>` (for example, attached from externally authored markup, or a multilingual set built by hand), `set_title`/`title()`/`remove_title()` only ever read, write, or remove the first one — every other `<title>` sibling is left completely untouched.

The same applies to `<desc>`.

Build or manage multilingual `<title>`/`<desc>` sets through the underlying DOM directly; a `lang`-aware API remains a possible future addition, not something these methods provide today.

`title()`/`desc()` read the DOM child directly — they do **not** compute the element's *accessible name* or *accessible description*, and the value they return is not always the same thing.

Per the accessible-name-and-description computation algorithm, the values held in `aria-labelledby` and `aria-label` take precedence over a `<title>` child for the accessible name, and `aria-describedby` takes precedence over a `<desc>` child for the accessible description.

A `<title>` child only supplies the accessible name (and only then does it also drive the browser's native hover tooltip) when neither ARIA attribute is present on the element; `<desc>` is otherwise never rendered as a tooltip by any browser.

`remove_title()`/`remove_desc()` remove only the first direct child; accessible names are not inherited from an ancestor, so removing a `<title>` does not cause "fallback" to some ancestor's name — the practical effect on the accessibility tree depends on what else, if anything, supplies a name (ARIA attributes, other content, or nothing at all).

A newly created `<title>` (i.e. when the element had none at all) is always inserted as this element's first child.

A newly created `<desc>` (i.e. when the element had none at all) is inserted immediately after an existing `<title>` (or as the first child if there is no `<title>` yet) — so `<title>` always precedes `<desc>` once both are set, regardless of which one you call first.

**Example**:

```rust,no_run
use svg_dom::{SvgRoot, root::utils::{Point, Size}};
let svg  = SvgRoot::attach("diagram")?;
let icon = svg.rect(Point::origin(), Size::new(24.0, 24.0))?;
icon.set_title("Close dialog")?;
icon.set_desc("Discards unsaved changes and closes this dialog.")?;
Ok::<(), svg_dom::Error>(())
```
