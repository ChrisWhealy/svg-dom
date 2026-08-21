# `requestAnimationFrame` self-rescheduling pattern

[← Back to design notes](README.md)

`AnimationLoop` uses the standard WASM self-referencing closure pattern.
The closure holds an `Rc` to itself, so it can re-register with `requestAnimationFrame` after each frame.

Calling `stop()` (or simply dropping the `AnimationLoop`) cancels the pending handle and immediately clears the `Rc` slot.
If the closure is not currently executing, that immediately releases both it and everything it has captured.

## Stopping from inside the running callback

Consider `stop()` called from *inside* the running callback — for example, a one-shot animation that stops itself on the first frame.
`stop()` clears the closure slot synchronously in every case, including this one, which drops the very closure whose invocation is still on the call stack.
But clearing the slot and releasing the closure's data are not the same moment here.

This relies on a guarantee that `wasm_bindgen::Closure` makes for owned closures: dropping the Rust-side handle from within its own currently-executing invocation keeps the closure's data (and everything it has captured) alive for the rest of that call, only freeing it once the call returns.
So captures are never retained merely because the `AnimationLoop` handle itself stays alive; they are released as soon as the currently executing callback returns, which is normally moments after `stop()` was called, well before the handle is ever touched again.

`tests/animation_loop.rs`'s `closure_can_drop_itself_from_within_its_own_invocation` isolates and verifies this directly, in a real browser, independently of `AnimationLoop`.

The closure slot itself is the single source of truth for whether `stop()` ran during the call: after `callback(ts)` returns, the RAF wrapper re-borrows the same slot to schedule the next frame.
If `stop()` ran inside `callback(ts)`, the slot is already `None`, so the wrapper simply has nothing to re-register and skips re-scheduling — no separate dispatch-state flag is needed to detect that case.

Because `stop()` is unconditional and synchronous, repeated calls during the same dispatch (either caused by an explicit second call, or `Drop` firing because the handle was dropped inside the callback) are trivially idempotent: each one finds no pending RAF id to cancel and re-clears an already-empty slot.
No deferred cleanup, no scheduling failure mode, and no leak path exists for this case.
