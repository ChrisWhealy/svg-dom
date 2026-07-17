# `requestAnimationFrame` self-rescheduling pattern

[← Back to design notes](README.md)

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
