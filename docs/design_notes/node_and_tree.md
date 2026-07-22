# Node identity, ownership, and tree navigation

[← Back to design notes](README.md)

**Contents**

- [`SvgNode` is a reference-counted handle](#svgnode-is-a-reference-counted-handle)
- [Event listeners are owned by the node](#event-listeners-are-owned-by-the-node)
- [Shared element factory implementation](#shared-element-factory-implementation)
- [Downward tree navigation and query-by-selector reuse `parent`'s independent-handle pattern, not a new type](#downward-tree-navigation-and-query-by-selector-reuse-parents-independent-handle-pattern-not-a-new-type)

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

## Shared element factory implementation

`SvgRoot` and `SvgBatch` expose the same basic element factories (`rect`, `circle`, `line`, `path`, `text`, and `group`).
Internally, those factories delegate to a shared `SvgFactory` implementation, so shape-specific creation logic and initial attribute writes exist in one place only.

The only difference between the two paths is the append target: `SvgRoot` appends directly to the live `<svg>`, while `SvgBatch` appends to its `DocumentFragment` until `commit()` is called.

`anchor` (`<a>`) and `switch` (`<switch>`) were added to `SvgFactory` the same way: `create_anchor` and `create_switch` follow `create_group`'s exact shape (a bare `create_svg_element` + optional attribute write + `append_node`, no geometry).
`SvgRoot::anchor`/`switch` and `SvgBatch::anchor`/`switch` are thin public wrappers, covered by the same `assert_parity` structural guard `tests/svg_root.rs` already runs across every other paired factory method.

Unlike `group`, which is also exposed on every other `SvgFactory` implementer (`SvgDefs`, `SvgSymbol`, `SvgPattern`, `SvgMask`, `SvgMarker`, `SvgClipPath`), `anchor` and `switch` are scoped to `SvgRoot`/`SvgBatch` only, matching `image`/`use_node`'s narrower precedent rather than `group`'s broader one.
A hyperlink or a language-conditional fallback is a main-document, user-facing construct; neither has an obvious use inside a non-rendered `<defs>` entry or a `<mask>`, `<clipPath>` or `<pattern>` tile, so the trait method exists (any future container can adopt it cheaply) without every container exposing it speculatively ahead of a real need.

## Downward tree navigation and query-by-selector reuse `parent`'s independent-handle pattern, not a new type

`SvgNode::parent` (`src/node/tree.rs`) already had to solve the problem this section is about: how to hand back a live `SvgNode` for a DOM element the crate did not create through one of its own factory methods.

Its solution (cast the `web_sys::Node`/`Element` to `SvgElement`, then wrap it in a brand-new `SvgNode::new(...)`) has already been established and already carries the caveat that matters:

| The returned handle is a **fresh, independent** `Rc<SvgNodeInner>` with empty listener storage, not a second reference to whatever handle originally owns the element (see `SvgNode::parent`'s doc comment for the full explanation).

`first_child`, `last_child`, `next_sibling`, `previous_sibling`, `children`, `query_selector`, and `query_selector_all` are all built the same way: they are thin wrappers over the matching `web_sys` traversal or query method, followed by the same cast-and-wrap.

No new "lightweight, non-owning" traversal type was introduced for this, even though such a function would sidestep the caveat entirely.
However, inventing a second handle type just for tree-walking would double the API surface a caller has to learn (leading to questions such as "So, which method returns which kind of handle?") to avoid a caveat that already has to be understood for `parent`.

Reusing the exact same trade-off, and pointing every new method's doc comment back at `parent`'s existing explanation rather than restating it, keeps the mental model to one rule instead of eight not-quite-identical ones.

Two further decisions followed directly from matching `parent`'s existing precedent rather than inventing new behaviour:

- **Single-result methods do not search past a non-SVG match.**

  `parent` returns `None` when the parent exists but is not an SVG element (the classic case being the root `<svg>`, whose own parent is the surrounding HTML page) — it does not walk further up looking for a usable ancestor.

  `first_child`/`last_child`/`next_sibling`/`previous_sibling`/`query_selector` copy that exactly: if the element at that specific DOM position or selector match is not an SVG element (for example HTML content inside a `<foreignObject>`), the method reports nothing found there rather than silently returning some other element the caller did not ask for.

- **Collection-returning methods filter instead of erroring or including everything.**

  `children` and `query_selector_all` skip non-SVG matches rather than failing the whole call over one stray non-SVG descendant, or returning a `Vec` mixing `SvgNode`s with some other representation.

  This is a different call from the single-result case above precisely because a collection degrading by omission (documented explicitly in each method's doc comment) is a much smaller surprise than a single lookup silently guessing at a different answer than the one the caller's selector or position asked for.
