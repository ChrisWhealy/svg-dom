# Design notes

**Contents**

- [`SvgNode` is a reference-counted handle](#svgnode-is-a-reference-counted-handle)
- [Event listeners are owned by the node](#event-listeners-are-owned-by-the-node)
- [`requestAnimationFrame` self-rescheduling pattern](#requestanimationframe-self-rescheduling-pattern)
- [Per-frame formatting uses a reusable scratch buffer](#per-frame-formatting-uses-a-reusable-scratch-buffer)
- [Transform setters reuse a caller-owned buffer](#transform-setters-reuse-a-caller-owned-buffer)
- [Redundant attribute writes are skipped on hot paths](#redundant-attribute-writes-are-skipped-on-hot-paths)
- [Caller-owned attribute cache for genuinely hot paths](#caller-owned-attribute-cache-for-genuinely-hot-paths)
- [Multi-attribute updates](#multi-attribute-updates)
- [Reusable attribute formatting](#reusable-attribute-formatting)
- [Shared element factory implementation](#shared-element-factory-implementation)
- [Typesafe Path Data Builder](#typesafe-path-data-builder)
- [`<filter>` primitives return a plain `SvgNode`](#filter-primitives-return-a-plain-svgnode)
- [Ideas Considered and Rejected](rejected_ideas.md)
- [Performance Patterns](#performance-patterns)

## `SvgNode` is a reference-counted handle

`SvgNode` wraps an `Rc`, so cloning it is cheap and all clones refer to the same underlying DOM node.
This makes it natural to share a node between an event closure and the surrounding code without the need for any `unsafe` or `Arc` shenanigans.

## Event listeners are owned by the node

Listeners registered through the managed helpers such as `on_click`, `on_mousedown`, `on_mousemove`, `on_contextmenu`, `on_pointerdown`, `on_pointermove`, `on_pointerenter`, `on_pointerleave`, `on_wheel`, `on_touchstart`, `on_keydown`, `on_focus`, and the drag-and-drop helpers are stored inside the `SvgNode`'s `Rc`.
Each stored entry keeps the event type together with its wasm-bindgen closure, so the DOM listener can be removed before the closure is dropped.

The built-in listener helpers use fixed browser event names, so event types can be stored as `&'static str` values.
They live exactly as long as the last clone of the node exists, so you never have to manage their lifetime separately or call `Closure::forget` for normal `SvgNode` interactions.

This lifetime rule is important for long-lived browser demos and applications: if a function creates a DOM node, attaches a managed listener, and then drops every `SvgNode` handle before returning, the listener is deliberately removed.
Keep at least one handle to every listener-owning node for as long as the interaction should remain active.
The demo gallery does this with a small page-lifetime owner for interactive nodes.

For uncommon browser events, `on_event` provides the same managed lifetime behaviour while using a generic `web_sys::Event`.
`on_event_once` is the one-shot counterpart: the browser removes the listener automatically after the first dispatch (via the native `{ once: true }` option), and the handler's captured values are freed on that dispatch if the `instanceof` check passes.
It accepts a generic typed event `E`; the cast from the raw `Event` is checked at runtime via `instanceof`, so a mismatched `E` silently suppresses the handler and leaves the captured values alive until the node is dropped or its listeners are cleared.
This is preferable to allowing undefined behaviour (that you hope won't do anything bad...)

Handlers are bound as `FnMut`, not `Fn`, so a handler can own and mutate captured state directly — typically a reusable `SvgAttrs` or `String` scratch buffer for a hot `pointermove`/`mousemove` path — without an `Rc<RefCell<...>>` wrapper.
The only constraint, inherited from wasm-bindgen's `Closure<dyn FnMut>`, is that a handler must not run re-entrantly (synchronously dispatching the same event to the same node from inside the handler), which will cause a panic — the same outcome a re-entrant `RefCell` borrow would produce.

Registration is otherwise append-only: the usual way to retire a listener is to drop the entire node.
However, for a long-lived node whose behaviour changes over time (e.g. one that swaps in mode-specific handlers) `clear_listeners` and `remove_listeners(event_type)` detach managed listeners without dropping the node, mirroring the detach-then-drop sequence that `SvgNodeInner::drop` performs (the browser-side callback is removed before its closure is freed).

`remove_listeners` reuses the per-listener event type already stored for cleanup, and compacts a `Many` store back to `One` when a single listener survives.
Since removing a listener frees its closure, neither method may be used to remove the listener that is currently executing; that is documented as the caller's responsibility.

## `requestAnimationFrame` self-rescheduling pattern

`AnimationLoop` uses the standard WASM self-referencing closure pattern: the closure holds an `Rc` to itself so it can re-register with `requestAnimationFrame` after each frame.

Calling `stop()` (or dropping the `AnimationLoop`) cancels the pending handle and sets the `Rc` slot to `None`, which prevents the next re-schedule and allows the closure to be freed.

When `stop()` is called from *inside* the running callback (e.g. a one-shot animation that stops itself on the first frame), freeing the closure immediately would create a use-after-free error on the still-executing closure body.

`AnimationLoop` tracks the dispatch lifecycle via the enum `AnimLoopState` (with members `Idle` / `Dispatching` / `StopPending` / `Stopped`).

When `stop()` detects the `Dispatching` state, it transitions to `StopPending` and defers the slot clear by scheduling a zero-delay `setTimeout`; by the time that timer fires the callback has fully returned and the closure (and all it has captured) are released.
The post-callback code in the RAF wrapper detects `StopPending`, transitions to `Stopped`, and skips re-scheduling.

`StopPending` exists specifically to make `stop()` **genuinely idempotent during dispatch**.
Without it, a second `stop()` call during the same dispatch — whether an explicit second call or the `Drop` impl firing because the handle is dropped inside the callback — would see `Stopped` instead of `Dispatching`, enter the synchronous cleanup branch, and drop the `FrameClosure` while the wrapper body was still executing past `callback(ts)`.
That recreates the exact use-after-free error the dispatch guard was added to prevent.

With `StopPending`, subsequent calls to `stop()` during the same dispatch see `StopPending` and collapse to a no-op: the deferred timer fires exactly once, the closure is never freed mid-execution, and both the "stop twice from inside callback" and "stop then drop from inside callback" scenarios are safe.

This mechanism is shared by the "drop from inside callback", "stop from inside callback (handle kept alive)", and "stop then drop from inside callback" paths, so captured values are released promptly without relying on when the `AnimationLoop` handle is eventually dropped.

Two rare failure paths are worth noting:

1. If `requestAnimationFrame` fails during re-scheduling (after the callback returns), the loop cannot continue; the failure path immediately sets the state to `Stopped` and clears the slot and frees any captured values at that moment rather than waiting for the `AnimationLoop` to be dropped.

   If `setTimeout` scheduling itself fails (a near-impossible browser-level error), the deferred cleanup cannot be registered.
   The post-callback code still transitions the state from `StopPending` to `Stopped`, so *if another `AnimationLoop` handle survives*, a later `stop()` or `Drop` sees `Stopped` and clears the slot synchronously, releasing the RAF closure and its captures.
   But if the handle that called `stop()` was the last `AnimationLoop` handle — i.e. it was dropped from inside the running callback — no later `stop()`/`Drop` call exists to perform that cleanup, and the RAF closure, the shared slot, and everything the user callback captured remain permanently leaked.

1. The callback created by `Closure::once_into_js` for the deferred `setTimeout` is a Rust `FnOnce` handed to JavaScript as a one-shot function; wasm-bindgen only deallocates it when it is *invoked* — an uninvoked `once_into_js` closure is not reclaimed merely by JavaScript garbage collection.

   If `setTimeout` registration fails, that callback is never invoked, so it — and its cloned `Rc` reference to the closure slot — leaks for the life of the page.
   In the recoverable case where another `AnimationLoop` handle survives (see above), that leaked callback's `Rc` ends up pointing at an already-cleared (`None`) slot, so the RAF closure and the user's captured state are not doubled up in the leak; only the one-shot closure and the empty slot allocation remain leaked.

## Per-frame formatting uses a reusable scratch buffer

`AnimationLoop::start_with_frame` supplies an `AnimationFrame` value to each RAF callback.
`AnimationFrame` owns one reusable `String` scratch buffer and exposes helpers such as `set_attr_fmt`, `set_fill_fmt`, `set_d_fmt`, `set_text_fmt`, and `set_points` / `set_points_fixed` (for animating `<polyline>`/`<polygon>` vertices, the latter at a fixed decimal precision to keep the per-frame string short).

Use these helpers for values that change every frame instead of writing `set_attr(..., &format!(...))` or `set_attr(..., &value.to_string())` inside the RAF callback.

The DOM still receives a normal `&str`, but on the Rust/WASM side, the same allocation is used across frames.

## Transform setters reuse a caller-owned buffer

`AnimationFrame`'s reusable buffer only helps callbacks that run *through* `AnimationLoop`.
Event-driven handlers such as drag, pan/zoom, sliders, knobs, follow-the-pointer cursors, resize/selection handles etc. do not, yet they update `transform` just as often.
Writing `set_attr("transform", &format!("translate({x:.1}, {y:.1})"))` inside a `pointermove` handler allocates and then drops a new `String` on every event - which adds up to unnecessary churn.

`src/node/transform.rs` adds a set of helpers that take a caller-owned `&mut String` scratch buffer, clear it, format the new transform into it, and hand it to `set_attr`.
These are

* `set_translate`
* `set_rotate`
* `set_rotate_about`
* `set_scale`
* `set_scale_xy`
* `set_translate_scale`
* `set_transform_fmt`

Reusing one buffer across calls means no new allocation happens unless the formatted text outgrows the buffer's capacity.
For shapes that the typed helpers do not cover, your escape hatch is `set_transform_fmt`: it accepts `std::fmt::Arguments` so `format_args!(...)` can build any transform string without the heap allocation that `format!` would otherwise incur.

The scratch buffer is deliberately **not** stored inside `SvgNode`.
Most nodes are passive geometry that never animate, do folding formatting state into every node would cause them all to grow while benefiting only a few.
Passive nodes can remain small by keeping the buffer external whilst hot paths can opt in explicitly.
Because managed handlers are `FnMut`, a handler that is the sole user of a buffer can simply own it (`let mut buf = String::new()` captured by the closure), as the colour-wheel demo does.
An `Rc<RefCell<String>>` is needed only when one buffer is *shared across several* closures, as the drag/touch demo does for its coordinate readout.

In spite of the fact that writing into a `String` is infallible, `write!` is typed to return `std::fmt::Error`.
`Error` implements `From<std::fmt::Error>`, mapping to the existing `Error::Dom` variant so the helpers can use `?` without a dedicated error variant.

## Redundant attribute writes are skipped on hot paths

`set_attr_if_changed` reads the current value with `get_attribute` and writes only when it differs.
This avoids a redundant DOM write in high-frequency handlers where the same value repeats between frames such as cursor style, `opacity` flags or selected state.

It is not a universal win and the cost can be bigger than it first appears: `get_attribute` **allocates a fresh `String` for the current value and crosses the wasm/JS boundary on every call**, even when it then writes nothing.
So for values that change on every call (such as a drag `transform`) calling `set_attr` remains the cheaper option, keeping `set_attr_if_changed` as best kept for *occasional* de-duplication rather than a per-event hot path.

## Caller-owned attribute cache for genuinely hot paths

For a high-frequency path where an element's attribute value usually repeats (E.G. a cursor style or `opacity` flag touched on every `pointermove`), using `CachedAttr` (in `src/node/cached.rs`) is preferable to calling `set_attr_if_changed`.

`CachedAttr` remembers the last value it wrote on the **Rust** side, so the unchanged case is a plain `&str` comparison against an owned `String`: I.E. no allocation takes place and no call into JS.

The DOM is touched only on a genuine change and even then, the `String` backing buffer is reused (`clear` + `push_str`) rather than reallocated.

This is the same design used for the transform scratch buffer: the cache is **caller-owned** and deliberately not stored inside `SvgNode`, so passive geometry nodes carry no caching state.
Keep one `CachedAttr` per frequently-updated value (typically captured in an event handler's state), dedicated to a single attribute on a single node.

If that attribute is changed by some other path, call `invalidate()` so the cache does not skip a needed write on the strength of a now-stale remembered value.

The same cache also covers text content via `CachedAttr::set_text`, for the equivalent case of a status readout rewritten with the same string on every `pointermove`.
Dedicate a given `CachedAttr` to *either* an attribute or text content, not both, since they share the single remembered value.

For a *formatted* cached value, `CachedAttr::set_fmt` / `set_text_fmt` format into a caller-owned `&mut String` scratch and then cache, so a frequently-touched but rarely-changing formatted readout (a grid-snapped coordinate, a zoom percentage) avoids both the per-call `format!` allocation and the redundant DOM write.
The scratch is a *separate* buffer from the cache, because the cache's own buffer holds the last-written value the new one is compared against.

## Multi-attribute updates

`SvgNode::set_attrs` accepts any `IntoIterator` of `(name, value)` pairs where both sides implement `AsRef<str>`.
This keeps the public API ergonomic for string literals and precomputed string values.

Use `set_attrs` when all values are already strings and you want a compact method for setting multiple attributes at once.

## Reusable attribute formatting

`SvgAttrs` owns a reusable `String` scratch buffer, then `AttrWriter` binds that buffer to a single `SvgNode` for chainable writes.
Use this in order to avoid the need to call `to_string()` or `format!` for numeric or formatted attribute values.

The browser still receives one normal SVG `setAttribute` operation per attribute, but the Rust/WASM side reuses the formatting allocation.
The built-in root and batch element factories use the same mechanism for initial numeric geometry attributes, so repeated element creation does not allocate a fresh formatting `String` per element.

For a single numeric attribute updated on a hot path, `SvgNode::set_attr_display` is the lightweight counterpart taking a caller-owned `&mut String` directly (the same shape as the transform setters), without the ceremony of binding an `AttrWriter`.
The convenience numeric setters such as `set_stroke_width` instead allocate a short-lived `String` per call; that is fine for one-off styling but should be swapped for `set_attr_display` (or an `AttrWriter`) when the value is animated.
The same caveat applies to the `Point`/`Size` `get_*_str` helpers, which each allocate; they are documented as one-off conveniences, not for per-event or per-frame use.

`SvgNode::set_text_fmt` and the `set_text_display` convenience for a single value both format into a caller-owned `&mut String` and set the result as text content.
For a label whose value changes on every event (e.g. a coordinate or status readout updated each time `pointermove` is handled), use `set_text_fmt` or `set_text_display` rather than `set_text(&format!(...))` or `set_text(&value.to_string())`, which allocate and discard a fresh `String` on every call.

When the text instead *repeats* between events, `CachedAttr::set_text` is the better fit since the DOM write only takes place when the value actually changes.
Both the pointer-lifecycle and drag/touch demos route *every* `last: ...` readout writer such as the hot `pointermove`/`touchmove`/`dragover` streams and the discrete transitions alike, through one shared `CachedAttr`, so a burst of identical label updates only touches the DOM on the first write.
The essential rule is that *all* writers should share one cache: partial caching, where some writers bypass it, is what would let the cache skip a genuinely needed write (which is why the cache is fed even from handlers, such as the native `drag` wrappers, that fire between `pointermove`s).

The drag/touch demo's live *coordinate* readout is a separate node that changes on every move, so it keeps using `set_text_fmt` with a scratch buffer shared with the card's transform rather than the cache.

## Shared element factory implementation

`SvgRoot` and `SvgBatch` expose the same basic element factories (`rect`, `circle`, `line`, `path`, `text`, and `group`).
Internally, those factories delegate to a shared `SvgFactory` implementation, so shape-specific creation logic and initial attribute writes exist in one place only.

The only difference between the two paths is the append target: `SvgRoot` appends directly to the live `<svg>`, while `SvgBatch` appends to its `DocumentFragment` until `commit()` is called.

## Typesafe Path Data Builder

`SvgRoot::path(d: &str)` (and its siblings on `SvgBatch`, `SvgDefs`, `SvgClipPath`, `SvgMarker`, `SvgPattern`, `SvgSymbol`) writes a `d` path verbatim.
A hand-written `d` string is free text, so there are no safeguards against it being malformed such as a wrong command letter, a missing argument or a transposed flag.
The SVG parser does not reject a malformed `d` string outright; it simply stops rendering at the first token it cannot parse, so the failure is silent and possibly quite difficult to debug

`PathDef` (in `root::path::path_def`, re-exported at the crate root) removes that failure mode by definition.
A `<path>`'s `d` attribute is built from an ordered `&[PathDef]` slice instead of a string; `build_d` / `write_d` do the formatting.

Since a `PathDef` can only ever represent one well-formed SVG command, there is no possibility of creating a malformed `d` string.

### Two enums, not one, wrapped in a third

`PathDefAbsolute` and `PathDefRelative` mirror each other variant-for-variant (`MoveTo`, `LineTo`, `EllipticalArcTo` etc.), differing only in whether the emitted command is upper- or lower-case.

Real SVG path data routinely mixes both within a single path: an initial absolute move command (`M`) followed by a run of relative line (`l`) or curve (`c`) commands is the idiomatic, compact way to define path data by hand.
It is commonplace for callers to mix both absolute and relative path definitions within the same `d` string.

`PathDef::{Abs, Rel}` is the thinnest possible wrapper that permits this: a single `Vec<PathDef>` (or array/slice literal) can freely interleave absolute and relative segments, exactly as hand-written path data would, while each individual segment stays unambiguous about which coordinate space it uses.

### Measuring the nested-enum layout cost, rather than assuming it

Rust does not guarantee enum layout, so whether wrapping `PathDefAbsolute`/`PathDefRelative` in `PathDef` actually costs anything beyond a single flattened enum is a question best answered by using `size_of`/`align_of`, not intuition.

The `pathdef_size_diagnostics` unit test (`src/root/path/unit_tests.rs`) measures it directly and prints the numbers on every run (`cargo nextest run --lib pathdef_size_diagnostics --no-capture`), rather than asserting a fixed byte count that could legitimately change across targets or compiler versions.

Measured on both the host target (x86_64/aarch64, `usize` = 8 bytes) and `wasm32-unknown-unknown` (`usize` = 4 bytes) — the numbers were identical on both, because every field in these types is an `f64`, `Point`, or a small fieldless enum, so alignment is driven entirely by `f64`'s 8-byte alignment, not by pointer width:

| Type | `size_of` | `align_of` |
|---|---|---|
| `Point` | 16 | 8 |
| `EllipticalArc` | 48 | 8 |
| `PathDefAbsolute` | 56 | 8 |
| `PathDefRelative` | 56 | 8 |
| `PathDef` | 64 | 8 |

`PathDef` is 8 bytes larger than either inner enum alone — a real, measured cost, not a hypothetical one.
`ArcSize`/`ArcSweep` are two-variant fieldless enums, which do have a spare-bit-pattern niche a wrapping enum's discriminant could in principle occupy, but rustc's current layout algorithm does not thread that niche out through `EllipticalArc` and then through `PathDefAbsolute`/`PathDefRelative` to `PathDef`; instead the outer discriminant gets its own padded slot, sized to the type's 8-byte alignment.
That slot is exactly one alignment unit, not an unbounded amount — `pathdef_size_diagnostics` asserts `size_of::<PathDef>() <= size_of::<PathDefAbsolute>() + align_of::<PathDefAbsolute>()` (and the `PathDefRelative` equivalent) as a structural regression guard, so a future accidental size regression (e.g. an added field, or a future rustc layout change that stops finding even this bound) fails the test rather than going unnoticed.

For a `Vec<PathDef>` holding many commands, that is a genuine ~14% (8/56) memory overhead per command versus a single flattened ~20-variant enum, which, because its own largest variant is no bigger than `PathDefAbsolute`'s, would likely pay the same one-alignment-unit discriminant cost but only once, not twice.

This difference is real and worth knowing about, but on its own, this is not a reason to flatten: it only matters if a program builds and retains large `Vec<PathDef>` arrays long-term (most callers build a `d` string once via `build_d`/`write_d` and then discard or reuse the `defs` slice), and flattening would double the variant count and duplicate the absolute/relative distinction across every command name — the API cost [`Two enums, not one, wrapped in a third`](#two-enums-not-one-wrapped-in-a-third) above already weighed against.

Revisit only if profiling (not this measurement alone) shows stored `PathDef` arrays materially affecting memory footprint or serializer dispatch time in a real caller.

### `HorizontalLineTo` / `VerticalLineTo` take `f64`, not `Point`

The SVG `H`/`h` and `V`/`v` commands each take a single coordinate.
`H` takes a bare `x` and `V` takes a bare `y`, not a full `(x, y)` coordinate pair.

### `EllipticalArc` is a named-field struct, not a five-element tuple

The SVG arc commands (`A`/`a`) take two boolean flags (`large-arc-flag`, `sweep-flag`) to select between the (up to) four geometric solutions for an arc between two points at a given radius.

As adjacent positional `bool`s in a tuple variant, they are easy to transpose — `(true, false)` vs `(false, true)` looks the same at a glance and the compiler cannot catch the swap.
`ArcSize` (`Small`/`Large`) and `ArcSweep` (`CounterClockwise`/`Clockwise`) turn each flag into a self-documenting enum, and bundling all five arc parameters into one named-field `EllipticalArc` struct (rather than a five-argument tuple variant) means every field is labelled at the construction site instead of positional.

`EllipticalArc::write` takes a `cmd: char` so the one method can serve both the `A` and `a` forms without duplicating its formatting body, but a bare `char` parameter accepts anything — nothing about the argument type stops a caller passing some nonsense value such as `'X'` and producing a command letter no SVG parser recognises.
The two real call sites (`path_def.rs`, passing the literal `'A'`/`'a'`) are the only ones that need to exist, so `write` is `pub(super)`, not `pub`, even though `EllipticalArc` itself is public: the struct's fields must stay public for callers to construct `PathDefAbsolute::EllipticalArcTo(EllipticalArc { .. })` literals, but the serialization method is purely internal machinery, and leaving it `pub` would have let a caller bypass `PathDef`'s well-formed-command guarantee through this one method while every other route into a `d` string stayed safe.

### Formatting matches the existing `write_points` convention

Coordinates are written with plain `{}` (`Display`) formatting (Rust's shortest round-trip representation) rather than a fixed decimal count, mirroring `write_points`'s default-precision path in `root::utils`.
This keeps whole-number demo coordinates compact (`"70"`, not `"70.0"`).

`write_d_fixed` / `build_d_fixed` (and the `d_from_defs_fixed` methods layered on top, mirroring `points_fixed`) do add a fixed-precision mode — but the "n decimal places for everything" knob only ever reaches the genuinely continuous arguments: coordinates, lengths, and the arc's `x_axis_rotation`.
It deliberately never reaches the two Boolean flags belonging to [`EllipticalArc`].
`large-arc-flag` and `sweep-flag` are written via `ArcSize`/`ArcSweep`'s `u8` `Display` regardless of `dps`, because the SVG `flag` grammar require Boolean `true` and `false` to be represented as `"0"` or `"1"`.

This is the concrete version of the general caution about path data mixing several different argument shapes: a uniform `dps` is safe for every numeric field *except* the numeric representation of a Boolean value.
So those two fields are simply carved out of the fixed-precision path entirely rather than trusting a caller to remember not to round them.

### Path `d` strings omit whitespace

The Backus-Naur Form (BNF) of the SVG path-data allows every command to have zero or more whitespace characters (`wsp*`) between the command letter and the first argument, not one or more (`wsp+`).
Since a command letter can never appear inside a number, that command letter unambiguously terminates whichever number preceded it, meaning the separator between a command's last argument and the next command's letter is grammatically unnecessary.
`write_d` and every per-command `write` method rely on both of these facts: thus we can write `"M{} {}"` instead of `"M {} {}"` for the command/first-argument boundary, and within the `write_d` loop, there is no need to add whitespace between commands.

`"M10 10L100 50L10 90Z"` and `"M 10 10 L 100 50 L 10 90 Z"` parse to the identical path in every conforming SVG implementation — this is a standard, lossless minification technique (the same one tools like SVGO apply), not an approximation, so there is no loss of precision or correctness trade-off.

For a path of `N` commands, of which `K` of them take arguments, this removes exactly `(N - 1) + K` bytes.
So for a long, procedurally-generated path (e.g. a fine-grained curve sampled as many `LineTo` segments), the saving is proportional to the number of commands, which is exactly the case where a smaller `d` string matters most: less data serialized means less data has to cross the WASM/JS boundary resulting in a shorter DOM attribute.

Separator elision between arguments *within* a command is deliberately not attempted (e.g. relying on a leading `-` or `.` to glue two numbers together without a space).
That trick is real per the SVG grammar too, but it depends on the sign and shape of each emitted number and requires per-value inspection to stay unambiguous
In reality, thus buys us far less than the always-safe, context-free whitespace removal described above.

**NOTE**:Eliding a repeated command letter (`"M0 0L10 10 20 20 30 30"` instead of `"M0 0L10 10L20 20L30 30"`) is also permitted by the grammar but has not been implemented as this requires stateful serialization (tracking the previous command's letter in both the absolute and relative forms across multiple iterations).
This in turn introduces a real correctness hazard specific to the move (`M`/`m`) command: a repeated move command's extra coordinate pairs are reinterpreted by the parser as implicit `L`/`l` commands, so naively eliding a repeated `M` changes the path's meaning, not just its byte count.

That complexity is only worth taking on for paths long enough that the extra savings are measurable; until then, the always-safe whitespace removal above is the better cost/benefit trade-off.

### Two allocation tiers, mirroring `points` / `set_attr_display`

An earlier version of this feature had `path_from_defs` and `SvgNode::set_d_from_defs` both call `build_d`, which allocates a fresh `String` on every call.
That included the shared `SvgFactory::create_path_from_defs` default method used by every `path_from_defs` factory sibling — nothing in the shipped API actually called `write_d` outside of `build_d`'s own body, contradicting `write_d`'s own documentation, which describes it as the buffer-reusing path for hot call sites.

The fix follows the crate's existing two-tier split for `points`, verbatim:

- **Node *creation*** (`path_from_defs` on `SvgRoot` and its factory siblings) now writes `d` through the factory's own retained `SvgAttrs` buffer — the same `self.attrs().borrow_mut()` pattern `create_rect` and friends already use — so repeated calls on one factory allocate at most once (for buffer growth), not once per call.

- **Node *updates* on a live `SvgNode`** still have two tiers, exactly as `set_font_size` (allocating) and `set_attr_display` (caller-owned buffer) do for other attributes: `SvgNode::set_d_from_defs` remains a convenience that allocates a short-lived `String` per call (which is fine for an occasional update) while `SvgAttrs::d_from_defs` / `AttrWriter::d_from_defs` and `AnimationFrame::set_d_from_defs` reuse a caller-owned buffer for a path that is morphed on every `pointermove` event or every animation frame.

`SvgNode` has no buffer of its own to reuse (it is a lightweight `Rc` handle, not a factory), which is exactly why the crate's hot-path attribute setters — `set_attr_display`, the transform setters, `AnimationFrame` — all take the scratch buffer as a parameter rather than owning one.
`d_from_defs` follows that same shape rather than inventing a new one.

### `build_d` / `build_d_fixed` pre-size their `String`; `write_d` / `write_d_fixed` deliberately do not

`build_d` and `build_d_fixed` are the one guaranteed-fresh-allocation case in the whole path API: every other entry point writes into a buffer the caller already owns and is expected to reuse.
Starting that fresh `String` from `String::new()` means hitting the usual doubling-reallocation pattern, as a path will grow from nothing.
This then incurs the cost `write_points` already avoids for a point lists by reserving a rough capacity upfront.

Both functions now reserve `defs.len() * APPROX_BYTES_PER_COMMAND` before writing.
`APPROX_BYTES_PER_COMMAND` is set to 24, the same per-entry "best guess" used by `write_points` for its default-precision path.

24 bytes is a rough, deliberately non-variant-aware estimate: `ClosePath` needs one byte, but a six-argument `CubicBezierTo` with large float coordinates needs several times that, so no single flat constant is exactly right for every path shape.
Computing a precise per-variant estimate would mean a second pass over `defs`, matching every command to sum its exact argument count and typical width — more work than the reallocations it would save, for a number that is already only ever a lower-bound guess (a `String` that undershoots just grows normally; it never produces wrong output).

`write_d` / `write_d_fixed` do not reserve anything themselves, unlike `write_points`, which calls `out.reserve(..)` on every invocation regardless of whether the caller is reusing the buffer.
The two functions serve different callers: `write_points` has no one-shot sibling to shoulder the sizing concern, so it has to do double duty.
`write_d` does have one (`build_d`), so the buffer-reusing function stays lean — clear, then append — and relies on the caller-owned buffer's capacity already being retained from a previous call (or, for a caller who cares about even the first call, constructing the buffer via `SvgAttrs::with_capacity` upfront rather than `SvgAttrs::new()`).

### `dps` is clamped once per `write_d_fixed` call, not once per command — but splitting the serializer into `write_default`/`write_fixed` was measured and rejected

Every per-command `write` originally took `dps: Option<usize>` and re-derived `n.min(MAX_DPS)` inside its own `Some(n)` arm — for `SmoothQuadraticBezierTo`/`EllipticalArcTo` specifically, more than once per arm, since each numeric argument's `{:.*}` format spec repeated the `.min(MAX_DPS)` call.

Since `dps` does not vary across a single `write_d_fixed` call, clamping is now done exactly once, before the loop and the already-clamped value is threaded down unchanged.
This part is a pure win with no downside: it is strictly less source, strictly fewer redundant comparisons, and provably produces byte-identical output (every existing fixed-precision test, including the one asserting `usize::MAX` and `MAX_DPS` clamp to the same result, passed unchanged).

A further step — splitting each `write` into separate `write_default(&self, out)` / `write_fixed(&self, out, dps: usize)` methods, so the per-command code no longer branches on `Option<usize>` at all.
This idea was also tried, but discarded after measurement rather than adopted on the strength of the argument alone.

The two versions were built and compared: full `cargo build --release --target wasm32-unknown-unknown`, then `wasm-opt -O3`, for the crate with only the clamp-hoist applied versus the crate with the full `write_default`/`write_fixed` split on top.
The resulting `.wasm` files were **byte-for-byte identical** (same MD5, both before and after `wasm-opt`) in both cases.
Rustc/LLVM already specializes `write_d`'s and `write_d_fixed`'s respective inlined call sites against the constant `None`/`Some(..)` they each always pass, so hand-writing that specialization as two separate methods produced no binary difference of any kind — no size change, and (since the generated code is identical) no possible runtime difference either.

Given that outcome, the split was reverted: it would have doubled the match-arm source for every current and future `PathDef` variant (a real, ongoing risk of the two copies drifting apart) in exchange for a measured benefit of exactly zero.
This is the concrete version of the reasoning that already kept this crate from making a dependency to `ryu`/`itoa` for numeric formatting — an optimization is only worth its complexity cost if it can provide a measurable benefit, not merely because it looks like it should.

### `build_d_fixed`'s capacity estimate scales with `dps`

`BASE_BYTES_PER_COMMAND` (24) was tuned for the *default*, shortest-round-trip format, and both `build_d` and `build_d_fixed` originally reserved `defs.len() * 24` regardless of precision.
This is fine until we encounter a high `dps` value applied to, say, a six-argument `CubicBezierTo`.
Here, for `dps = 20`, the six-argument `CubicBezierTo` formats to roughly 138 bytes (`"C0.00000000000000000000 0.00000000000000000000 ..."`), against a 24-byte reservation for the whole command, which is nearly a 6-fold shortfall, guaranteeing at least one (but usually several), reallocate-and-copy doublings for that command alone.

`build_d_fixed` now reserves `defs.len() * (BASE_BYTES_PER_COMMAND + APPROX_VALUES_PER_COMMAND * dps.min(MAX_DPS))`, with `APPROX_VALUES_PER_COMMAND = 3`.
`build_d` (no `dps`) is unaffected and keeps the flat 24-byte guess.

Setting `APPROX_VALUES_PER_COMMAND` to `3` is a deliberate *average*, not a per-command worst-case bound: real commands range from zero numeric arguments (`ClosePath`) to six (`CubicBezierTo`).

A test proved this directly (`build_d_fixed_capacity_formula_improves_on_flat_guess_for_high_precision_cubic_bezier`): for the six-argument case above, the new formula reserves 84 bytes against a real 138 — still short, but the shortfall drops from 114 bytes to 54, roughly halved, and the reservation is closer to exact for the far more common shorter commands (`MoveTo`, `LineTo`, `HorizontalLineTo`) that a real path is mostly made of.

The first version of this fix asserted the new formula fully covered the worst case; that test failed immediately (84 << 138), so the assertion was corrected to match what three-as-an-average actually promises: a measurable improvement, not a guarantee.

A second, more accurate option exists: sum each command's actual numeric-argument count via a `PathDef::numeric_arg_count` helper, in a dedicated pass over `defs` before allocating.

This has deliberately not been implemented.

It is exactly the "second pass over `defs` matching every variant" this module's capacity estimates already decline to perform elsewhere, for the same reason: the win only matters for `build_d_fixed`'s one guaranteed-fresh-allocation case (a direct call, a first use of a fresh buffer, or a workload that keeps constructing new paths rather than updating one in place — `write_d_fixed` on a retained buffer is unaffected either way), and no benchmark has shown that case to be a real bottleneck worth a second traversal by which further reallocations can be avoided.
If one ever does, the variant-aware pass is the documented next step, not a redesign.

### What "prevents malformed path data" actually covers

Early documentation for `PathDef` claimed the resulting `d` string "can never contain a mistyped command letter, a missing argument, or *any other* malformed path data."
The last clause overstated the guarantee: `PathDef` prevents malformed *commands* — spelling, argument arity, arc-flag validity — but was silent about two ways a *sequence* of individually well-formed commands can still fail to be a valid path.

**SVG requires a non-empty path to start with a moveto.**

`[PathDef::Abs(PathDefAbsolute::LineTo(..))]` formats into perfectly well-formed path *syntax* — `"L1 1"` — that is nonetheless not valid path *data*: the SVG grammar requires a non-empty path to begin with an `M`/`m`.
Not only will a conforming user agent silently render nothing for a path that starts with anything else, it will also not report an error.
This is cheap to catch (an O(1) look at `defs.first()`), so `path_from_defs`, `SvgNode::set_d_from_defs`, `SvgAttrs::d_from_defs` / `d_from_defs_fixed`, and `AnimationFrame::set_d_from_defs` / `set_d_from_defs_fixed` all call `validate_starts_with_moveto` and return `Error::InvalidPathData` if it fails — including the per-frame `SvgAttrs`/`AnimationFrame` methods, since the check costs nothing beyond that single comparison regardless of call frequency.

A leading relative moveto (`m`) is accepted because no current point yet exists to which a relative point can refer, so the SVG spec always treats a path's very first moveto command as absolute, irrespective of whether `m` or `M` is used.

`build_d` / `write_d` (and their `_fixed` siblings) deliberately do **not** call this check.
They are the lowest-level formatters in the module and may legitimately be asked to build a path-data *fragment* that isn't meant to stand alone (e.g. a caller is assembling several `PathDef` slices before concatenating them) so enforcing "must start with a moveto" at this location would reject legitimate uses.
The check exists only at the boundary where a sequence is committed to an element's actual `d` attribute.

**Coordinates are unconstrained `f64` values.**

Nothing with the definition of a `Point` field stops it from holding values such as `f64::NAN` or `f64::INFINITY`.

The SVG number grammar has no token for either, so Rust's `Display` output for them (`"NaN"`, `"inf"`, `"-inf"`) is not valid path syntax, and unlike the moveto check, catching this is *not* cheap: it means visiting every numeric argument of every command, an O(total arguments) traversal rather than an O(1) look at one element.
That cost would land squarely on `write_d`/`write_d_fixed`, the functions this whole feature exists to keep cheap for a per-frame caller, so this crate does not check for it anywhere in the path API.

⚠️ Caution ⚠️

A caller whose coordinates come from a calculation that could produce a non-finite value (division, trigonometry) is expected to validate with `f64::is_finite()` before constructing the `PathDef` — the same "caller's responsibility at the boundary" shape as the `set_attr` security caveat elsewhere in this crate.

### `create_path_from_defs` validates once, not twice

Wiring `validate_starts_with_moveto` into both `create_path_from_defs` (the shared `SvgFactory` default method behind every `path_from_defs` factory) and `SvgAttrs::d_from_defs` (the natural place to put each check in isolation) meant `path_from_defs` ran the same check twice: once before `make_node("path")`, and again inside `d_from_defs` when the factory wrote the freshly-created node's `d` attribute.

The factory's own check has to stay: it is the one that matters, since it rejects a bad `defs` slice *before* a detached `<path>` element is ever created, rather than after.
Removing it and relying solely on `SvgAttrs::d_from_defs`'s check would mean a bad path first allocates a DOM node, then discards it — wasted work on the failure path and a small window where a detached, doomed element exists for no reason.

`SvgAttrs::d_from_defs` is therefore split into two: the public method still validates (a caller reaching it directly — via `AttrWriter` or by hand — has had no earlier chance to check), then delegates to `pub(crate) fn d_from_validated_defs`, the unchecked core that just writes.
`create_path_from_defs` calls `d_from_validated_defs` directly, skipping the redundant second pass over the same three-or-so-byte slice prefix it already inspected moments earlier.
The saving is not the point since an O(1) check is not worth optimising away for its own sake, but leaving it in obscured which validation call in the sequence was the one actually performing the protection, which on clarity grounds alone, is worth fixing.

## `<filter>` primitives return a plain `SvgNode`

`SvgFilter` (`src/root/filter.rs`) is structurally identical to `SvgClipPath` and `SvgPattern`: that is, it is an id-cached container obtained from `SvgDefs::filter`/`build_filter`, applied to any element via `SvgNode::set_filter_ref`/`set_filter`, with the usual `set_attr`/`set_attrs`/`set_attr_display` escape hatch for attributes not yet wrapped by a named setter.
That much follows established precedent directly; the one new decision is what a filter-primitive *builder method* — `gaussian_blur`, and whatever `fe*` methods follow it — should hand back.

The SVG filter primitives are a large, mostly-orthogonal vocabulary: around fifteen elements (`feGaussianBlur`, `feOffset`, `feColorMatrix`, `feComposite`, `feMerge`/`feMergeNode`, `feFlood`, `feBlend`, and others), each with its own attribute grammar, but sharing two attributes across nearly all of them — `in` (identifies the upstream input or named result to be read) and `result` (the name under which this primitive's output is published, and which a later primitive's `in`/`in2` can reference).

Two designs were available for the return type of a method like `gaussian_blur`:

1. A typed wrapper per primitive (`FeGaussianBlur`, mirroring `SvgClipPath`'s own typed methods), or a `FilterPrimitive` enum in the `PathDef` style, with `in`/`result` as named fields or setters.
2. A plain `SvgNode` — the same handle already returned by every ordinary shape factory (`create_rect`, `create_circle`, ...) — relying on the existing generic `SvgNode::set_attr` for `in`, `result`, and any primitive-specific attribute not yet promoted to a named parameter.

Option 2 was chosen for this first primitive.

Unlike `PathDef`, which models a single, closed, well-understood grammar (SVG path data) that benefits from exhaustive compile-time coverage, the filter primitive vocabulary is still only one primitive deep in this crate; committing to a typed wrapper (or a `PathDef`-style enum) per primitive now would mean guessing at a shape for fourteen more elements this crate does not yet implement, several of which (`feMerge`'s ordered `feMergeNode` children, `feComponentTransfer`'s per-channel `feFunc*` children) have structure closer to `SvgClipPath`'s child-shape factories than to a flat attribute bag.

It costs nothing to add primitives around a plain `SvgNode` — `gaussian_blur` is a thin `create_svg_element` + attribute write + `append_child`, the same shape as `GradientInner::add_stop` — and does not pre-commit the crate to an API surface for primitives not yet built.

This decision will be revisited once several more primitives exist and a genuine cross-primitive pattern (such as a shared `in`/`result` typed setter, or a `feMerge`-shaped child-list builder) becomes visible from real usage rather than anticipated in advance.

### `feOffset` and `feMerge` confirm the plain-`SvgNode` decision, rather than forcing a redesign

`offset` was a second flat-attribute primitive (`dx`, `dy`), no surprises — the same shape as `gaussian_blur`.

`merge` was the first real test: `<feMerge>` has ordered `<feMergeNode>` children rather than a flat attribute bag, exactly the case flagged above as a possible reason to introduce a typed, child-list builder.

In practice it does not need one.

`merge(&["offset-blur", "SourceGraphic"])` takes the list of `in` values as a plain `&[&str]` parameter and builds the `<feMergeNode>` children internally in one pass, still handing back a plain `SvgNode` for the outer `<feMerge>` (which has nothing but `result` left to set).
There was no ordering, mutation, or per-node configuration requirement that a closure-based builder (in the `SvgClipPath`/`build_clip_path` style) would have served better.
Each `feMergeNode` is only ever an `in` value, so a slice is already the natural shape for "an ordered list of input names."

The general shape of the decision therefore still stands after three primitives: reach for a closure/child-builder API only when a primitive's children need more than one attribute each, or when they must be added incrementally rather than known upfront — neither of which has come up yet.

### `gaussian_blur_xy` shares a private `fmt::Arguments` core with `gaussian_blur`, rather than duplicating it

`stdDeviation` is one of several SVG attributes with a `<number-optional-number>` grammar: one number for an isotropic value, or two space-separated numbers (`"x y"`) for independent horizontal/vertical (anisotropic) values.

`gaussian_blur` only ever wrote the one-number form, so a caller wanting the two-number form had no direct route to it — the closest workaround was calling `gaussian_blur` (one `stdDeviation` write), then overwriting the same attribute on the returned `SvgNode` with a hand-formatted `"x y"` string (a second write, and ordinarily a `format!`-allocated `String` to supply it).

`gaussian_blur_xy` closes that gap as a second public constructor for the same `<feGaussianBlur>` element, not a new primitive.
Both public methods delegate to a private `gaussian_blur_args(&self, std_deviation: fmt::Arguments<'_>)` that does the actual element creation, single attribute write, and append; `gaussian_blur` calls it with `format_args!("{std_deviation}")` and `gaussian_blur_xy` with `format_args!("{x} {y}")`.
Passing `fmt::Arguments` rather than a `&str` means neither caller needs to pre-format a `String`: `Arguments` implements `Display`, so it flows straight through `SvgAttrs::display_element`'s existing `write!(scratch, "{value}")` into the retained scratch buffer — the same technique `SvgPattern::set_view_box` and `SvgSymbol::set_view_box` already use to combine several numbers into one attribute (see "Reusable attribute formatting" above).

This is a second data point (after `merge`'s slice-of-`&str` parameter) that a filter primitive needing a slightly richer call shape than "one flat attribute, one method" does not need a bigger abstraction — a second thin public method sharing a private core is enough as long as the underlying element is still just attributes, no child structure.

See [`docs/gaps.md`](gaps.md) for the primitives and region-attribute setters (`filterUnits`, `primitiveUnits`, typed `x`/`y`/`width`/`height`) still to be added.

# Performance Patterns

## High-frequency event coalescing

On some modern devices, the events generated by `pointermove`, `touchmove`, and `wheel` can arrive at the hardware polling rate, which could be as high as 1000 Hz (i.e. one event per millisecond); while the various browser events arrive at a rate between 60 and 120 Hz.

A handler that is called at the hardware polling rate could potentially call `set_translate` or `set_attr` on every delivered event, even though all but the last position before the next paint is immediately discarded.
This might involve performing a Rust → JavaScript crossing, then a possible `setAttribute` DOM call, and a potential SVG layout invalidation for each event.
Such an architecture is highly wasteful of computing resources and can result in jerky or laggy scrolling.

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
