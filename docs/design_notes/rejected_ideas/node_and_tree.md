# Node identity, ownership, and tree navigation

[← Back to rejected ideas](README.md)

**Contents**

- [Splitting `SvgNode` into passive and interactive types](#splitting-svgnode-into-passive-and-interactive-types)
- [Adding `parent_element()` for lighter hot-path parent access](#adding-parent_element-for-lighter-hot-path-parent-access)
- [Restricting or removing `SvgNode::parent()` to prevent split listener state](#restricting-or-removing-svgnodeparent-to-prevent-split-listener-state)

## Splitting `SvgNode` into passive and interactive types

One proposal suggested splitting `SvgNode` into passive and interactive types, for a memory benefit.
Only the interactive type would carry the listener storage:

```rust
struct SvgNode {
    element: SvgElement
}

struct InteractiveSvgNode {
    node: SvgNode,
    listeners: RefCell<Vec<EventListener>>
}
```

The motivation was to stop passive geometry nodes from carrying listener state they never use.

The memory win is tiny because the common case is already optimised.

The listeners field is `RefCell<Option<Box<ListenerStore>>>`.
`store_listener` only allocates on the first `on_*` call.
A passive node therefore allocates **no** listener storage at all.
It pays only for the inline field.
On `wasm32`, that is the `RefCell` borrow flag (4 bytes) plus a niche-optimised `Option<Box<...>>` pointer that is `null` when empty (4 bytes).
The saving adds up to only ~8 bytes.

`ListenerStore` is a `One(EventListener)`/`Many(Vec<EventListener>)` enum.
The first listener is held inline in the `Box`.
A single-listener node therefore makes one heap allocation, rather than the two an empty `Vec` would (the `Box<Vec>` itself plus the element buffer on first push).
A second listener upgrades `One` to `Many`.
Registration is a setup-time path, so this is a modest leanness win rather than a hot-path one.

Splitting removes those ~8 inline bytes per node, and zero heap allocations.
That is negligible next to the `Rc` strong/weak counts and allocation header every node carries regardless.

Against that small saving sit real costs:

* **API surface.**<br>
   Callers must choose passive vs interactive up front.

   Every factory (`rect`, `circle`, `line`, `path`, `text`, `group`) lives in the shared `SvgFactory` used by both `SvgRoot` and `SvgBatch`.
   Splitting the type would force a choice: duplicate each factory and add an `.interactive()` upgrade step, or make the factory generic and ripple that change through two factory surfaces.

   To avoid re-declaring every attribute setter, `InteractiveSvgNode` would also need to `Deref` to `SvgNode`, which is `Deref`-as-inheritance.

* **It breaks the single-identity model.**<br>
  `SvgNode` is `Rc<SvgNodeInner>`.
  All clones therefore share one ownership root.
  The listener-lifetime contract ("keep at least one handle alive and the listeners stay alive") depends on that shared root.

  Putting `listeners` on the outer `InteractiveSvgNode` places it *outside* the shared `Rc`.
  So an "upgrade" forks ownership: the interactive handle owns the listeners independently of any passive clone of the same element.
  Drop the interactive handle while a passive clone is still alive, and the listeners die.
  That is exactly the footgun the single-type design eliminates.
  Restoring shared semantics would require a second `Rc` layer.

So the structurally "trivial" upgrade is semantically a fork of the very ownership the library deliberately unifies, in exchange for ~8 bytes per node.
The lightweight-passive-node property is better served by the existing lazy `Option<Box<Vec>>`.
Any need to signal interactivity is also cheaper to meet with documentation than with a second concrete type.

## Adding `parent_element()` for lighter hot-path parent access

`SvgNode::parent()` creates a fresh managed `SvgNode` around the parent element, with its own independent listener store.
One suggestion was a lighter sibling method: `pub fn parent_element(&self) -> Option<SvgElement>`.
It would suit hot-path callers, such as those inside `pointermove`, `wheel`, or RAF callbacks.
Those callers only need to inspect or compare the parent, not construct a new managed node.

The suggestion explicitly frames this as a future addition ("if parent traversal becomes common in hot paths").
It recommends not replacing the current `parent()` behaviour.
For those reasons, and the following, it is not adopted now.

* **No current consumer demonstrates a realistic need.**<br>
  The crate targets simple SVG diagrams.
  Hot-path parent traversal has not, so far, appeared in any demo or real use case.
  Extending the API surface for a hypothetical future requirement conflicts with the crate's principle of not designing for speculative requirements.

* **`as_element()` already provides the raw-DOM escape hatch.**<br>
  A caller who needs the parent element for inspection can call `node.parent().map(|n| n.as_element().clone())`.
  It is verbose but accurate for the rare case.
  It also avoids widening the public API for a path that has no demonstrated users.

* **Returning `SvgElement` directly would break the crate's abstraction layer.**<br>
  Every other navigation method (`parent`, `downgrade`/`upgrade`) returns a crate-managed type.
  Introducing `Option<SvgElement>` as a return type on a navigation method would be the first raw `web-sys` type surfaced from a traversal API.
  It would set an inconsistent precedent.

If a future profile shows parent-traversal allocation as a measured bottleneck in a real hot path, the method can be added then with evidence to justify the API surface cost.

## Restricting or removing `SvgNode::parent()` to prevent split listener state

`SvgNode::parent()` wraps the raw parent DOM element in a brand-new `SvgNode` with its own independent `Rc<SvgNodeInner>` and an empty listener store.
The external review flagged this as a "serious consistency issue."
The same SVG element now has two unrelated managed handles.
Listeners registered through the `parent()` handle are neither visible to, nor cleaned up by, the factory handle for the same element.
Four alternatives were proposed: remove `parent()`, return a read-only `SvgElementRef<'a>`, return a raw `SvgElement`, or maintain a canonical registry so the same DOM element always maps to the same `Rc<SvgNodeInner>`.

None of these changes will be adopted.

### The reviewer's premise is incorrect

The crate's invariant is **per-lineage**, not per-DOM-element.

A *lineage* is an `SvgNode` and all its `clone()`s.
They all share the same `Rc<SvgNodeInner>`, the same listener store, and the same lifetime contract ("keep one handle alive, listeners stay alive").
The crate never claims that every `SvgNode` wrapping a given DOM element belongs to the same lineage.

The doc comments for `parent()` make this explicit.
They state that listener tracking is "per handle lineage (a handle together with its clones) and **not** per DOM element."
They also state that the returned handle's listener store is independent of any factory handle for the same element.
The handle should be treated as read-only navigation.
The same caution is extended to discourage registering listeners through a `parent()` handle.
This is not a consistency violation — it is the documented, intended behaviour of the per-lineage model.

### Why the proposed alternatives do not improve the situation

**Option A — Remove `parent()`.**<br>
This removes the legitimate use cases that motivated the method in the first place.
One such case: walking up the tree to inspect or modify an ancestor attribute from inside an event callback, without retaining every ancestor handle at construction time.
A caller who needs to navigate upward, but cannot or does not want to hold the ancestor handle, would be left with two options.
Either `node.as_element().parent_node()` (a raw DOM call), or `SvgNode::as_element()` plus `dyn_ref` casting.
Both are worse than the current API.

**Option B — `SvgElementRef<'a>`, a lifetime-bound read-only wrapper.**<br>
The wrapper would need to re-expose a meaningful subset of `SvgNode`'s API (attribute reads, `tag_name`, bounding-box queries) without any of the mutation or listener methods.
That is a large parallel surface, with viral lifetime annotations that flow into any closure or data structure that holds the ref.
The underlying DOM element is still reachable, since there is no way to make the underlying `SvgElement` truly read-only.
So the wrapper would only provide a "soft" guarantee, backed by `unsafe` at the boundary.

The doc comments already discourage this pattern.
There is therefore no benefit in adding a second public type that introduces a parallel API surface and allows unpredictable lifetimes.

**Option C — Return `SvgElement` directly.**<br>
Evaluated and rejected [above](#adding-parent_element-for-lighter-hot-path-parent-access), for the same reasons.
It is the first raw `web-sys` type surfaced from a traversal API.
It denies access to the typed attribute setters and transform helpers that make `SvgNode` useful.
It also sets an inconsistent precedent relative to every other navigation method (`downgrade`/`upgrade`, which return crate-managed types).

**Option D — A canonical registry mapping DOM elements to `Rc<SvgNodeInner>`.**<br>
This would require a `HashMap<JsValue, Weak<SvgNodeInner>>` (or similar), a strategy for removing stale entries when handles are dropped, and logic to detect an already-registered element.

Such a change would require a fundamental change to the ownership model, since every factory call would need to consult the registry.
It would also add allocations and hash lookups to what is currently an allocation-free construction.
The problem it aims to solve — ensuring that `parent()` returns a handle in the same lineage as the factory handle — is not a problem callers actually need solved.
The correct use of `parent()` (read-only navigation and attribute mutation) does not require shared listener state.

### Conclusion

The per-lineage model is the correct invariant.
`parent()` returns an independent handle that forms its own lineage.
That is documented behaviour, not a bug.
The doc comment is the appropriate and sufficient fix.
It was tightened to explicitly state "Do not register listeners through a handle obtained from `parent()`" alongside the existing explanation of why listener state is not shared.
