use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use svg_dom::AnimationLoop;
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use wasm_bindgen_test::*;

mod common;

wasm_bindgen_test_configure!(run_in_browser);

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Helpers
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Waits for `n` animation frames by scheduling `requestAnimationFrame` callbacks sequentially via `Promise`.
/// Each `await` yields to the browser event loop, giving any already-scheduled RAF callbacks (e.g. from `AnimationLoop`)
/// a chance to fire.
///
/// Requires `js-sys` and `wasm-bindgen-futures` in `[dev-dependencies]`.
async fn wait_for_frames(n: u32) {
    for _ in 0..n {
        // Promise::new provides `resolve` as a js_sys::Function; RAF calls it with the timestamp, fulfilling the
        // promise and resuming our await.
        let promise = js_sys::Promise::new(&mut |resolve, _| {
            web_sys::window().unwrap().request_animation_frame(&resolve).unwrap();
        });
        wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// wasm-bindgen `Closure` self-drop guarantee
//
// AnimationLoop's stop() drops the RAF closure synchronously, even when called from inside that closure's own
// currently-running invocation. That is only sound if dropping a wasm-bindgen `Closure` from inside its own invocation
// is actually safe. This test isolates that one claim, independent of AnimationLoop, by dropping a bare `Closure` from
// inside its own body and checking that its captures are released only after the call returns — not during the call,
// and not with a crash/trap either way.
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Dropping a `Closure` from inside its own currently-executing invocation (by clearing the `Rc<RefCell<Option<_>>>`
/// slot that is its only owner) must not crash, and must not free its captures until the call returns.
///
/// `marker`'s `Rc::strong_count` is the observable: one strong reference is held by the test itself, a second by the
/// clone the closure captured. While the closure body is running (even after it clears its own slot), that captured
/// clone is still logically part of the still-executing call frame, so the count must still read 2 at that point.
/// Only once `call0` returns — proving wasm-bindgen kept the closure's data alive for the duration of the call —
/// does the count fall back to 1.
#[wasm_bindgen_test]
fn closure_can_drop_itself_from_within_its_own_invocation() -> Result<(), String> {
    type SelfDropSlot = Rc<RefCell<Option<Closure<dyn FnMut()>>>>;

    let marker = Rc::new(());
    let slot: SelfDropSlot = Rc::new(RefCell::new(None));
    let ran = Rc::new(Cell::new(false));
    let count_during_call = Rc::new(Cell::new(0usize));

    let slot_inner = slot.clone();
    let marker_inner = marker.clone();
    let ran_inner = ran.clone();
    let count_during_call_inner = count_during_call.clone();

    let closure: Closure<dyn FnMut()> = Closure::new(move || {
        // Self-drop: clear the slot holding this very closure, from inside its own invocation.
        *slot_inner.borrow_mut() = None;
        // `marker_inner`'s clone is still captured by this still-running call frame at this point, so the shared
        // count must still include it.
        count_during_call_inner.set(Rc::strong_count(&marker_inner));
        ran_inner.set(true);
    });

    *slot.borrow_mut() = Some(closure);

    // Invoke through the JS-visible function reference, borrowed out for the call only — mirrors AnimationLoop's own
    // borrow-then-release-then-call pattern, avoiding a BorrowMutError when the callback re-borrows `slot`.
    let js_fn = {
        let borrow = slot.borrow();
        borrow
            .as_ref()
            .ok_or("closure not stored")?
            .as_ref()
            .unchecked_ref::<js_sys::Function>()
            .clone()
    };
    js_fn
        .call0(&JsValue::NULL)
        .map_err(|e| format!("calling the self-dropping closure failed: {e:?}"))?;

    common::check(ran.get(), "the closure body did not run")?;
    common::check(slot.borrow().is_none(), "the closure did not actually clear its own slot")?;
    common::check_eq(count_during_call.get(), 2usize)?; // captured clone still alive mid-call
    common::check_eq(Rc::strong_count(&marker), 1usize) // released once the call returned
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// stop() tests
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `stop()` called before yielding to the event loop cancels the pending RAF, so the callback is never invoked.
///
/// JS is single-threaded: the RAF callback can only fire when we yield via `await`.
/// Calling `stop()` synchronously cancels the handle before we ever yield.
#[wasm_bindgen_test]
async fn should_stop_all_callbacks_before_first_frame() -> Result<(), String> {
    let count = Rc::new(Cell::new(0u32));
    let count_c = count.clone();

    let anim = AnimationLoop::start(move |_| {
        count_c.set(count_c.get() + 1);
    })
    .map_err(|e| e.to_string())?;

    anim.stop(); // cancel before any frame fires
    wait_for_frames(2).await; // yield — callback must not fire

    common::check_eq(count.get(), 0u32)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `stop()` called after the loop has been running freezes the callback count at its current value; subsequent frames
/// do not increment it further.
///
/// After we yield, `AnimationLoop` has fired and re-scheduled itself.
/// `stop()` calls `cancelAnimationFrame` on that handle and sets the closure slot to `None`, preventing any further
/// re-scheduling even if the cancellation races with a frame boundary.
#[wasm_bindgen_test]
async fn should_freeze_callback_count_when_stop_called_after_running() -> Result<(), String> {
    let count = Rc::new(Cell::new(0u32));
    let count_c = count.clone();

    let anim = AnimationLoop::start(move |_| {
        count_c.set(count_c.get() + 1);
    })
    .map_err(|e| e.to_string())?;

    wait_for_frames(2).await; // let the loop fire at least once

    common::check(count.get() > 0, "loop should have fired at least once before stop()")?;

    anim.stop();
    let frozen = count.get();

    wait_for_frames(2).await; // yield again — count must not change

    common::check_eq(count.get(), frozen)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Drop tests
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Dropping an `AnimationLoop` immediately (before any frame fires) invokes the `Drop` impl which calls `stop()`,
/// cancelling the pending RAF.  The callback is never invoked.
#[wasm_bindgen_test]
async fn should_inhibit_all_callbacks_if_dropped_before_first_frame() -> Result<(), String> {
    let count = Rc::new(Cell::new(0u32));
    let count_c = count.clone();

    let anim = AnimationLoop::start(move |_| {
        count_c.set(count_c.get() + 1);
    })
    .map_err(|e| e.to_string())?;

    drop(anim); // Drop impl must call stop()
    wait_for_frames(2).await; // yield — callback must not fire

    common::check_eq(count.get(), 0u32)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Dropping an `AnimationLoop` that has been running stops it at that point; the callback count does not increase
/// after the drop, confirming that `Drop` calls `stop()`.
#[wasm_bindgen_test]
async fn should_freeze_callback_count_when_drop_called_after_running() -> Result<(), String> {
    let count = Rc::new(Cell::new(0u32));
    let count_c = count.clone();

    let anim = AnimationLoop::start(move |_| {
        count_c.set(count_c.get() + 1);
    })
    .map_err(|e| e.to_string())?;

    wait_for_frames(2).await; // let the loop fire at least once

    common::check(count.get() > 0, "loop should have fired at least once before drop")?;

    drop(anim);
    let frozen = count.get();

    wait_for_frames(2).await; // yield again — count must not change

    common::check_eq(count.get(), frozen)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// stop() from inside the callback
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Dropping the `AnimationLoop` handle from inside its own callback must not leak the closure or its captures.
///
/// `Drop` calls `stop()`, which clears the closure slot synchronously — even though this drops the very closure
/// whose invocation is still on the call stack.
///
/// `wasm_bindgen::Closure`'s self-drop-during-invocation guarantee (see `stop()`'s doc comment) is what makes this
/// safe.
///
/// A `DropFlag` (a helper that increments a counter when dropped) lets the test observe whether the closure was freed.
#[wasm_bindgen_test]
async fn should_not_leak_when_animloop_dropped_from_within_callback() -> Result<(), String> {
    struct DropFlag(Rc<Cell<u32>>);
    impl Drop for DropFlag {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }

    let drop_count = Rc::new(Cell::new(0u32));
    let slot: Rc<RefCell<Option<AnimationLoop>>> = Rc::new(RefCell::new(None));

    let flag = DropFlag(drop_count.clone());
    let slot_cb = slot.clone();

    *slot.borrow_mut() = Some(
        AnimationLoop::start(move |_| {
            let _ = &flag; // ensure `flag` is captured; it is dropped when the closure is freed
            slot_cb.borrow_mut().take(); // drop the AnimationLoop from inside its own callback
        })
        .map_err(|e| e.to_string())?,
    );

    // RAF fires → handle dropped → stop() clears the slot synchronously, but the closure (and DropFlag inside it) is
    // still executing at that point, so wasm-bindgen keeps it alive until this callback invocation returns — which
    // happens moments later, still within the same frame. Three frames is more than enough margin to confirm nothing
    // further happens afterward.
    wait_for_frames(3).await;

    common::check_eq(drop_count.get(), 1u32)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Calling `stop()` from inside the running callback must not retain captured values for the lifetime of the
/// `AnimationLoop` handle, even when that handle is kept alive after the stop.
///
/// `stop()` clears the closure slot synchronously, but the closure is still executing at that point, so
/// `wasm_bindgen::Closure` keeps its captures alive until the callback returns — see `stop()`'s doc comment.
/// The important property this test isolates is that release does not additionally wait on the handle: the captures
/// are freed as soon as the currently executing callback returns, not whenever the handle itself is later dropped.
///
/// A `DropFlag` proves the captures are freed independently of handle lifetime.
#[wasm_bindgen_test]
async fn should_not_retain_captures_after_stop_from_within_callback() -> Result<(), String> {
    struct DropFlag(Rc<Cell<u32>>);

    impl Drop for DropFlag {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }

    let drop_count = Rc::new(Cell::new(0u32));
    let slot: Rc<RefCell<Option<AnimationLoop>>> = Rc::new(RefCell::new(None));

    let flag = DropFlag(drop_count.clone());
    let slot_cb = slot.clone();

    *slot.borrow_mut() = Some(
        AnimationLoop::start(move |_| {
            let _ = &flag; // ensure `flag` is captured; it is dropped when the closure is freed
            if let Some(anim) = slot_cb.borrow().as_ref() {
                anim.stop(); // stop without dropping the handle
            }
        })
        .map_err(|e| e.to_string())?,
    );

    // RAF fires → stop() called → closure freed once the callback returns, within the same frame → DropFlag dropped.
    wait_for_frames(3).await;

    // Captures must already be released by the time we check, well before touching the handle.
    common::check_eq(drop_count.get(), 1u32)?;

    // The handle itself is still alive, proving this is the stop()-from-callback path, not drop()-from-callback.
    common::check(slot.borrow().is_some(), "handle should still be alive after stop()")
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Calling `stop()` twice from inside the running callback must not crash and must prevent re-scheduling.
///
/// The first call clears the closure slot synchronously — dropping the very closure whose invocation is still
/// running. The second call is a plain idempotent no-op: it re-cancels an already-cancelled handle and re-clears an
/// already-`None` slot.
///
/// A `DropFlag` proves the closure is freed exactly once.
#[wasm_bindgen_test]
async fn should_allow_stop_twice_from_within_callback() -> Result<(), String> {
    struct DropFlag(Rc<Cell<u32>>);
    impl Drop for DropFlag {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }

    let count = Rc::new(Cell::new(0u32));
    let drop_count = Rc::new(Cell::new(0u32));
    let slot: Rc<RefCell<Option<AnimationLoop>>> = Rc::new(RefCell::new(None));

    let flag = DropFlag(drop_count.clone());
    let count_cb = count.clone();
    let slot_cb = slot.clone();

    *slot.borrow_mut() = Some(
        AnimationLoop::start(move |_| {
            let _ = &flag; // ensure `flag` is captured so its drop is observable
            count_cb.set(count_cb.get() + 1);
            if let Some(anim) = slot_cb.borrow().as_ref() {
                anim.stop();
                anim.stop(); // second call must not free the closure mid-dispatch
            }
        })
        .map_err(|e| e.to_string())?,
    );

    wait_for_frames(3).await;

    common::check_eq(count.get(), 1u32)?; // fired exactly once
    common::check_eq(drop_count.get(), 1u32) // captures freed exactly once
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Calling `stop()` then immediately dropping the `AnimationLoop` handle from inside its own callback must not crash.
///
/// The explicit `stop()` call clears the closure slot synchronously.  The subsequent `Drop` on the handle calls
/// `stop()` again — a plain idempotent no-op, since the slot is already `None` and the handle already cancelled.
///
/// A `DropFlag` proves the captures are freed exactly once.
#[wasm_bindgen_test]
async fn should_allow_stop_then_drop_from_within_callback() -> Result<(), String> {
    struct DropFlag(Rc<Cell<u32>>);
    impl Drop for DropFlag {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }

    let count = Rc::new(Cell::new(0u32));
    let drop_count = Rc::new(Cell::new(0u32));
    let slot: Rc<RefCell<Option<AnimationLoop>>> = Rc::new(RefCell::new(None));

    let flag = DropFlag(drop_count.clone());
    let count_cb = count.clone();
    let slot_cb = slot.clone();

    *slot.borrow_mut() = Some(
        AnimationLoop::start(move |_| {
            let _ = &flag;
            count_cb.set(count_cb.get() + 1);
            if let Some(anim) = slot_cb.borrow().as_ref() {
                anim.stop(); // first: clears the slot synchronously
            }
            // Drop the handle — Drop calls stop() again, a no-op since the slot is already None.
            slot_cb.borrow_mut().take();
        })
        .map_err(|e| e.to_string())?,
    );

    wait_for_frames(3).await;

    common::check_eq(count.get(), 1u32)?; // fired exactly once
    common::check_eq(drop_count.get(), 1u32) // captures freed exactly once
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Calling `stop()` from inside the running callback must not crash and must prevent re-scheduling — the loop must fire
/// exactly once.
///
/// `stop()` drops the closure slot while the RAF wrapper is still executing past `callback(ts)`; after `callback(ts)`
/// returns, the wrapper re-borrows that same slot, finds it empty, and skips re-scheduling.
#[wasm_bindgen_test]
async fn should_allow_stop_from_within_callback() -> Result<(), String> {
    let count = Rc::new(Cell::new(0u32));
    let slot: Rc<RefCell<Option<AnimationLoop>>> = Rc::new(RefCell::new(None));

    let count_cb = count.clone();
    let slot_cb = slot.clone();

    *slot.borrow_mut() = Some(
        AnimationLoop::start(move |_| {
            count_cb.set(count_cb.get() + 1);
            if let Some(anim) = slot_cb.borrow().as_ref() {
                anim.stop();
            }
        })
        .map_err(|e| e.to_string())?,
    );

    // Wait a few frames; the callback fires (and stops itself) on the first one.
    wait_for_frames(3).await;

    common::check_eq(count.get(), 1u32)
}
