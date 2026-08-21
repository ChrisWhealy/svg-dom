# Animation API

[← Back to rejected ideas](README.md)

**Contents**

- [Making `AnimationLoop::start_with_frame` the canonical animation API](#making-animationloopstart_with_frame-the-canonical-animation-api)
- [Optional shared RAF scheduler (`AnimationScheduler`)](#optional-shared-raf-scheduler-animationscheduler)

## Making `AnimationLoop::start_with_frame` the canonical animation API

The external review observed that `AnimationLoop` exposes two starting styles:

```rust
// Plain callback — timestamp only.
AnimationLoop::start(|ts| { /* ... */ })?;

// Frame callback — timestamp plus a reusable scratch buffer.
AnimationLoop::start_with_frame(|ts, frame| { /* ... */ })?;
```

The recommendation was to make `start_with_frame` canonical.
This implies that `start` should be removed or deprecated.
The stated reason: it is *"more consistent with the crate's current performance direction."*

This idea will not be adopted.

### The `start` and `start_with_frame` function are layered, not competing

`start_with_frame` is implemented as a thin wrapper around `start_inner`, the same private entry point used by `start`:

```rust
pub fn start_with_frame<F: FnMut(f64, &mut AnimationFrame) + 'static>(mut callback: F) -> Result<Self, Error> {
    let mut frame = AnimationFrame::new();
    Self::start_inner(move |ts| callback(ts, &mut frame))
}
```

`AnimationFrame` is an adapter that adds one allocation (the scratch `String`, made once) and no other overhead.
The two constructors are not alternatives at the same abstraction level.
One wraps the other.

### `start` is the right API for a large class of callbacks

Not every animation callback needs to format numeric attributes.
A callback that reads a game state, calls `set_transform()` on a pre-computed matrix string, or invokes a typed setter (`set_cx`, `set_cy`) has no use for `AnimationFrame`.
Forcing the `AnimationFrame` parameter onto every caller adds an unused parameter for callers that do not need it.
It also adds the mental overhead of an API whose purpose is irrelevant to the callback ("why is there a frame here?").

### The relationship exactly mirrors the `defs` / `batch` API pairs

`defs()`/`build_defs()`, `marker()`/`build_marker()`, `batch()`/`build_batch()`, and `start()`/`start_with_frame()` all follow the same pattern.
Each pair offers a simpler form for the common case and a richer form that adds behaviour, such as atomicity or a scratch buffer, for callers that need it.
[Canonicalising on one construction model for `defs`, `marker`, and `batch`](api_surface.md#canonicalising-on-one-construction-model-for-defs-marker-and-batch) already rejected the same idea: collapsing the `defs`/`batch` pairs into a single canonical form.

### The doc already guides callers to the right form

The `start` doc comment includes the remark "For a crate-managed buffer, see `start_with_frame`."
Its example also shows how to manage a manually-owned buffer when the crate's buffer is not wanted.
Callers are directed to the performance path without it being forced on them.

Removing `start` would be a breaking change.
It delivers no benefit for the majority of animation callbacks.
It would also add a mandatory, unused parameter for every caller that does not need to format attributes.

## Optional shared RAF scheduler (`AnimationScheduler`)

An external review proposed an `AnimationScheduler` abstraction that owns one `requestAnimationFrame` registration and dispatches `N` registered callbacks through it each frame.
The motivation: `N` independent `AnimationLoop` values make `N` JS → WASM boundary crossings per frame.
They also issue `N` RAF registrations, all carrying essentially the same timestamp.
The proposal argued a shared scheduler would reduce both to one.

None of the proposed changes will be adopted.

### The aggregation is already available and the crate already demonstrates it

`AnimationLoop::start` and `start_with_frame` both accept a single `FnMut` callback that can call any number of sub-functions.
The existing animation demo drives three geometrically independent animations (a pulsing circle, a travelling circle, and a hue-rotating rectangle) from one callback, with one RAF registration.
This is already the idiomatic pattern: if several animations must run concurrently, put all their updates in one closure.
No library change is needed for the common case.
The scheduler would only add infrastructure to solve a problem the API does not actually have.

### Mutation during dispatch multiplies a problem already shown to be subtle

`AnimationLoop` already required careful handling for the "stop from inside the callback" case (see the [`requestAnimationFrame` self-rescheduling pattern](../animation.md) design note).
`stop()` drops the closure synchronously (even when called from inside that closure's own currently-running invocation).
This relies on a `wasm_bindgen::Closure` guarantee verified by a dedicated regression test.

The RAF wrapper still has to re-borrow the closure slot after the callback returns to tell whether `stop()` already cleared it, and skip re-scheduling into it when it has.
A scheduler with `N` callbacks inherits this per-callback borrow discipline, and multiplies it.
Any callback can deregister *itself*, deregister *another* callback, add new callbacks, or drop the whole scheduler.

The RFC acknowledges this directly and proposes a slot table with tombstones followed by post-dispatch compaction.

A tombstone-based slot table is the correct solution, but every frame then pays:

- a linear scan of the slot table to collect live indices before dispatch (to avoid iterating while callbacks mutate the collection)
- a second pass to compact tombstoned slots
- `RefCell` borrow management around each callback invocation to keep the collection accessible for mutation

This introduces O(N) bookkeeping per frame, even when the slot table is perfectly stable.
The borrow hygiene is at least as finicky as `AnimationLoop`'s existing single-slot-as-source-of-truth trick, but with more interacting mutation paths.
A panic or use-after-free in this code in production would cause a silent WASM failure with no stack trace.

### The feature is not SVG-specific

A multi-callback RAF multiplexer does not interact with the SVG DOM, `SvgNode`, `SvgRoot`, attributes, or any other part of the crate.
It is a general-purpose WASM/browser utility.
Its correct home is a dedicated crate (`raf-scheduler` or similar), where it can be composed with any browser WASM project.
Adding it here conflates the library's scope without offering any SVG-specific advantage.

### The hosting model has no good answer

A standalone `AnimationScheduler` value can be instantiated many times.
Multiple schedulers in the same application would simply recreate the N-loop problem.
For the scheduler to actually unify all animations, it must be treated as a shared singleton.
In WASM that means `thread_local!` state, or `Rc<RefCell<...>>` indirection passed through every code module that registers a callback.
That burden falls on the application developer, who would not have needed it at all had they put their animations in one `AnimationLoop` callback from the start.

Hosting the scheduler on `SvgRoot` (one per SVG) is a cleaner ownership model.
But it adds surface to `SvgRoot` that has nothing to do with SVG element creation or mutation.

### The overhead is marginal at realistic `N`

* One JS → WASM boundary crossing at 60 Hz costs roughly 1–2 µs on modern hardware.

* For `N = 3` concurrent loops, the "waste" over a unified scheduler is 2 extra crossings per frame.
  That is approximately 120–240 µs per second — well under 0.02% of CPU time on any current device.

* The benefit is only significant at large `N` (the RFC uses ten as its example).
  No profile of a downstream application with that many concurrent `AnimationLoop` values has been produced.

As with previous rejections, the measurement gate applies here as well.
This is the same gate consistently applied to speculative performance work throughout this document.
See [Performance & binary size](performance.md) and, within the event system, [feature-gating event families](events.md#feature-gate-event-families-and-specialised-svg-functionality).
