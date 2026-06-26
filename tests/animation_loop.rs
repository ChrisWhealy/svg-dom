use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use svg_dom::AnimationLoop;
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
/// Without the deferred-cleanup `setTimeout(0)` in `Drop`, the self-referencing `Rc` cycle is never broken and the
/// `FrameClosure` (along with all the values it has captured) leaks permanently.
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

    // RAF fires → handle dropped → setTimeout(0) scheduled → setTimeout fires (between frames) →
    // slot cleared → DropFlag freed.  Three frames is more than enough margin.
    wait_for_frames(3).await;

    common::check_eq(drop_count.get(), 1u32)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Calling `stop()` from inside the running callback must release captured values promptly via the deferred timer,
/// even when the `AnimationLoop` handle is kept alive after the stop.
///
/// Without the `setTimeout(0)` in `stop()`, the closure and its captures would be retained as long as the handle
/// lives, which equates to a memory-retention bug for long-lived handles.
///
/// A `DropFlag` proves the captures are freed by the next-tick timer, independently of handle lifetime.
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

    // RAF fires → stop() called → setTimeout(0) scheduled → setTimeout fires → closure freed → DropFlag dropped.
    wait_for_frames(3).await;

    // Captures must be released by the deferred timer before we even touch the handle.
    common::check_eq(drop_count.get(), 1u32)?;

    // The handle itself is still alive, proving this is the stop()-from-callback path, not drop()-from-callback.
    common::check(slot.borrow().is_some(), "handle should still be alive after stop()")
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Calling `stop()` from inside the running callback must not crash and must prevent re-scheduling — the loop must fire
/// exactly once.
///
/// This guards against creating a use-after-free: without the `AnimLoopState` dispatch guard, `stop()` would drop the
/// closure slot while the inner RAF closure was still executing past `callback(ts)`.
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
