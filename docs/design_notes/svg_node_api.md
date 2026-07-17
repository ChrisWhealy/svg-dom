# Small `SvgNode`/`SvgRoot` attribute-helper decisions: `viewBox`, `classList`

[← Back to design notes](README.md)

**Contents**

- [`SvgRoot::set_view_box` reuses `SvgSymbol`/`SvgPattern`'s existing shape](#svgrootset_view_box-reuses-svgsymbolsvgpatterns-existing-shape)
- [`classList` helpers are scoped to `SvgNode`, not duplicated per element type](#classlist-helpers-are-scoped-to-svgnode-not-duplicated-per-element-type)

## `SvgRoot::set_view_box` reuses `SvgSymbol`/`SvgPattern`'s existing shape

`SvgSymbol` and `SvgPattern` already had a `set_view_box(x, y, width, height)` method, each writing the same `"x y width height"` string via `display_element`'s reused scratch buffer.
`SvgRoot` was the one place `docs/gaps.md` flagged as missing it — `set_viewport` covers `width`/`height`, but nothing covered `viewBox` beyond the generic `root.set_attribute(...)` escape hatch documented on the `root` field itself.

The new method is a direct copy of that existing shape, not a new design: same four positional `f64` parameters in the same order, same lack of a getter (nothing in this crate reads `viewBox` back to stay internally consistent, unlike `width`/`height`, which `set_viewport` must cache to support its skip-unchanged-writes optimisation described in [Performance patterns](performance.md)).

`viewBox` and `set_viewport`'s cached `width`/`height` are independent and do not need to agree on scale: `width`/`height` size the `<svg>` element in the surrounding page, while `viewBox` maps that rendered area onto an internal coordinate system within which child elements are positioned.
Setting one does not read, invalidate, or need to touch the other, so `set_view_box` needed no interaction with the `viewport: Cell<Size>` field at all — it is a plain, uncached attribute write, exactly like `SvgSymbol`'s and `SvgPattern`'s.

`SvgMarker` was, at this point, the one remaining SVG element in this crate's coverage with a `viewBox` attribute and no dedicated setter, deliberately left alone in this round rather than opportunistically added, since it was not the gap this round of work was scoped to close.
It got the same `set_view_box(x, y, width, height)` method in a later round, once asked for directly — the same shape again, the same shared `validate_view_box` call at the top (see below), and no new design decisions: `<marker>`'s own `refX`/`refY`/`markerWidth`/`markerHeight` already covers positioning and sizing the marker itself, so `viewBox` there plays the same "define an internal coordinate system, independent of the outer viewport's own units" role it plays for `<symbol>`/`<use>`, not a new relationship this crate had to design from scratch.

### `set_view_box` validates its four components before writing, and the validator is shared across all three setters

Copying `SvgSymbol`/`SvgPattern`'s existing shape also copied their gap: none of the three original `set_view_box` methods checked their `x`/`y`/`width`/`height` arguments before formatting and writing them.

An `f64` can hold values that SVG's own `viewBox` grammar does not accept, for example `NaN`, `+infinity`, `-infinity` make no sense in the context of SVG, and supplying a negative `width` or `height` is equally nonsensical, even though the syntax parses.

Before this, `set_view_box(0.0, 0.0, -100.0, 100.0)` or `set_view_box(f64::NAN, 0.0, 100.0, 100.0)` both silently wrote a `viewBox` string the browser would then reject or misbehave on, with no signal back to the caller that anything was wrong — exactly the class of problem `Error::InvalidMarkerId` and its five siblings already exist to catch for id strings, just not yet extended to this attribute.

The fix is a single `pub(crate) fn validate_view_box` in `src/root/utils/mod.rs`, called as the first line of every `set_view_box` method (`SvgRoot`, `SvgSymbol`, `SvgPattern`, and — added in a later round — `SvgMarker`), returning the new `Error::InvalidViewBox(&'static str)` variant before anything is written.

A shared function, rather than one copy per type, matters here for the same reason `is_valid_svg_id` (see [References](references.md)) is one function instead of six: multiple instances of the same functionality opens the door to future implementation inconsistecy or drift — a concern the fourth call site (`SvgMarker`) already validates in practice, since it needed zero new validation logic of its own, only the same one-line call the other three already had.

`x`/`y` are checked for numeracy and finiteness, not sign — an SVG viewBox origin is routinely negative (panning into negative coordinate space is normal usage, and one of `SvgRoot::set_view_box`'s own tests exercises exactly that).
Only `width`/`height` are additionally checked for sign.
A `width`/`height` of exactly `0.0` is deliberately still accepted: it is valid `viewBox` syntax, and per the SVG spec, disables rendering of the element it's set on rather than being an error.
This amounts to a real, if unusual, way to hide content without removing it.
That distinction (finite-and-non-negative is required, zero is allowed) is documented on `Error::InvalidViewBox` itself and each setter's own doc comment, so a caller does not have to guess which zero-adjacent values are and are not accepted.

## `classList` helpers are scoped to `SvgNode`, not duplicated per element type

Since each of the types `SvgRoot`, `SvgSymbol`, `SvgPattern`, and `SvgMarker` owns its own `viewBox` attribute, each type therefore needed its own `set_view_box`.
However, CSS `class` is not type-specific: every element this crate derefs down to a plain DOM `Element`.

`SvgNode` is already the shared handle every other typed wrapper (`SvgRoot`, `SvgMarker`, `SvgPattern`, ...) hands back to callers for general-purpose attribute work (see `set_attr`/`attr`/`remove_attr` on `SvgNode` itself).
Therefore, we can simply add class methods to `SvgNode` once and cover every element type this crate produces, with no per-type duplication and no gap for a type added later.

Rather than hand-rolling `class` string parsing, five methods wrap `web_sys::Element::class_list()`'s `DomTokenList` (`add_1`, `remove_1`, `toggle`, `toggle_with_force`, `contains`):

- `add_class`
- `remove_class`
- `toggle_class`
- `set_class_enabled`
- `has_class`

Reimplementing `DomTokenList`'s whitespace-splitting and duplicate-avoidance rules on top of the raw `class` attribute string is entirely redundant because the DOM functionality of a browser already handles these details for `classList` operations.

`has_class`/`contains` is infallible and returns a plain `bool`; the other four return `Result`/`Result<bool, Error>` through the same `dom_err` mapping every other DOM-touching method in `src/node/attrs.rs` uses, since `add_1`/`remove_1`/`toggle`/`toggle_with_force` are themselves fallible at the `web_sys` boundary.

`set_class_enabled(class, enabled)` was added after the first four, once it was clear that `toggle_class`'s invert-whatever-is-there semantics is the wrong tool whenever the caller already knows the desired end state (for example, syncing a `"selected"` class to a boolean `is_selected` flag on every render).
Attempting to compute that with the first four methods required a `has_class` read followed by a conditional `add_class`/`remove_class` call: that is, two avoidable DOM round trips, one of which will be wasted whenever the state already matches.

`DomTokenList::toggle_with_force(token, force)` is the DOM's own two-argument `classList.toggle` overload built for exactly this: it sets membership to unconditionally `force` a new class value.
The wrapper (`set_class_enabled`) is both simpler at the call site and cheaper than using the `has_class`-then-branch construct.
It returns `Result<(), Error>` rather than echoing the resulting `bool` back — unlike `toggle_class`, where the caller does not know the outcome in advance, here the caller already supplied `enabled` and asking for it back would be redundant.

The demo (`demo_events_classlist` in `src/demo/events.rs`) deliberately keeps all styling in `style.css`.
Each tile's `on_click` handler calls only `toggle_class("selected")`, never `set_fill`/`set_attr` on `stroke` directly, and the gold highlight comes purely from a `.tile.selected` CSS rule.
This is the whole point of the demo: it shows that a `class` attribute change alone is enough to restyle an element, which is the actual reason `classList` helpers are useful over `set_attr("class", ...)` string-splicing.
The "selected: n / 3" readout is recomputed from `has_class` on every tile after each click, rather than tracked in a separate Rust state, so it cannot drift from what the DOM actually contains.
