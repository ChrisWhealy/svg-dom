# `requestAnimationFrame` self-rescheduling pattern

[← Back to design notes](README.md)

`AnimationLoop` uses the standard WASM self-referencing closure pattern.
The closure holds an `Rc` to itself, so it can re-register with `requestAnimationFrame` after each frame.

Calling `stop()` (or dropping the `AnimationLoop`) cancels the pending handle and clears the `Rc` slot immediately, releasing the closure and everything it captured.

## Stopping from inside the running callback

Consider `stop()` called from *inside* the running callback — for example, a one-shot animation that stops itself on the first frame.
`stop()` clears the closure slot synchronously in every case, including this one, which drops the very closure whose invocation is still on the call stack.

This relies on a guarantee that `wasm_bindgen::Closure` makes for owned closures: dropping the Rust-side handle from within its own currently-executing invocation keeps the closure's data alive for the rest of that call, and only frees it once the call returns.

`tests/animation_loop.rs`'s `closure_can_drop_itself_from_within_its_own_invocation` isolates and verifies this directly, in a real browser, independently of `AnimationLoop`.

`AnimationLoop` tracks the dispatch lifecycle via the enum `AnimLoopState` (with members `Idle`, `Dispatching` and `Stopped`), purely so that after `callback(ts)` returns, the RAF wrapper can tell whether `stop()` ran during that call and already cleared the slot.
When it did, the wrapper skips re-scheduling instead of trying to re-register a closure that no longer exists.

Because `stop()` is unconditional and synchronous, repeated calls during the same dispatch — an explicit second call, or `Drop` firing because the handle was dropped inside the callback — are trivially idempotent: each one re-cancels an already-cancelled RAF handle and re-clears an already-empty slot.
No deferred cleanup, no scheduling failure mode, and no leak path exists for this case.
