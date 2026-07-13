use crate::{animate::anim_frame::AnimationFrame, dom_err, error::Error};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{JsCast, prelude::*};

/// The per-frame closure registered with `requestAnimationFrame`.
type FrameClosure = Closure<dyn FnMut(f64)>;
/// Shared, self-referencing slot used by the closure to re-register itself each frame; cleared on `stop`.
type SharedClosure = Rc<RefCell<Option<FrameClosure>>>;
/// Shared cell holding the pending `requestAnimationFrame` handle so it can be cancelled.
type RafHandle = Rc<Cell<i32>>;

/// Dispatch state for the RAF loop.
///
/// Tracks the dispatch lifecycle so that `stop()` is genuinely idempotent when called multiple times, or via `Drop`
/// from inside the running callback.
///
/// | State | Description |
/// | ----- | ----------- |
/// | `Idle` | The loop is running; no callback is currently executing. |
/// | `Dispatching` | The RAF wrapper is currently inside `callback(ts)`. |
/// | `StopPending` | `stop()` was called during dispatch; deferred closure cleanup is scheduled.<br><br>A subsequent call to `stop()` are no-ops, as is firing the `Drop` impl because the handle was dropped inside the callback.  This prevents a use-after-free of the still-running closure body. |
/// | `Stopped` | The loop has fully stopped; the closure slot has been (or will be) cleared.
#[derive(Clone, Copy, PartialEq)]
enum AnimLoopState {
    Idle,
    Dispatching,
    StopPending,
    Stopped,
}

/// # A running `window.requestAnimationFrame` loop.
///
/// `requestAnimationFrame` is the browser API that schedules a callback immediately before the browser paints the next
/// frame — typically 60 times per second on a 60 Hz display.  Your callback receives the frame timestamp in
/// milliseconds (the same value as `performance.now()`), which you can use to drive time-based animations that stay
/// frame-rate–independent.
///
/// The loop continues until [`stop`](Self::stop) is called or this value is dropped.  Dropping an `AnimationLoop` is
/// always safe since the `Drop` impl calls `stop()` automatically, thus cancelling any pending frame and releasing the
/// closure.
///
/// ## Keeping the loop alive
///
/// The `AnimationLoop` value **must** be kept alive for the loop to continue running.  If you drop it (e.g. by
/// assigning it to `_`), then `stop()` fires and the loop ends after the very first frame.
///
/// The `AnimationLoop` can be kept alive by storing it in a `static`, a `Closure` captured variable, or some other
/// location whose lifespan outlives your animation.
#[must_use = "dropping the AnimationLoop stops the requestAnimationFrame loop — store the handle for as long as the animation should run"]
pub struct AnimationLoop {
    window: web_sys::Window,
    handle: RafHandle,
    closure: SharedClosure,
    /// Tracks the dispatch lifecycle so that multiple `stop()` calls (including via `Drop`) during a single callback
    /// invocation are all safe and idempotent.  See [`AnimLoopState`] for the full state-transition description.
    state: Rc<Cell<AnimLoopState>>,
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl AnimationLoop {
    /// Starts a `requestAnimationFrame` loop, calling `callback(timestamp_ms)` each frame.
    ///
    /// The first frame is scheduled immediately.  Subsequent frames are re-scheduled from inside the closure using the
    /// self-referencing `Rc<RefCell<Option<Closure>>>` pattern — the closure captures a reference counted clone of its
    /// own slot and re-fills it each time it runs.
    ///
    /// # Arguments
    ///
    /// * `callback` — called once per animation frame and is passed the frame timestamp in milliseconds.  Must be
    ///   `'static` because it runs in a browser callback.
    ///
    /// # Errors
    ///
    /// - [`Error::Dom`] — Either the `window` is not available (unlikely in a WASM context), or the initial
    ///   `requestAnimationFrame` call failed for some reason.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::cell::RefCell;
    /// use std::fmt::Write;
    /// use svg_dom::{AnimationLoop, SvgRoot};
    ///
    /// // One page-lifetime slot to hold the running loop (a wasm page is single-threaded).
    /// thread_local! {
    ///     static ANIM: RefCell<Option<AnimationLoop>> = const { RefCell::new(None) };
    /// }
    ///
    /// let svg = SvgRoot::attach("vis").unwrap();
    /// let path = svg.path("M 0 50 L 200 50").unwrap();
    ///
    /// // The callback is `FnMut`, so it can own a reusable buffer and format into it rather than
    /// // allocating a fresh `String` each frame. (For a crate-managed buffer, see `start_with_frame`.)
    /// let mut d = String::new();
    /// let anim = AnimationLoop::start(move |ts| {
    ///     // Animate the midpoint of the path upward and downward.
    ///     let y = 50.0 + 30.0 * (ts / 600.0).sin();
    ///     d.clear();
    ///     let _ = write!(d, "M 0 50 Q 100 {y} 200 50");
    ///     let _ = path.set_d(&d);
    /// }).unwrap();
    ///
    /// // Keep the loop alive for the page's lifetime; dropping it would stop it via `Drop`.
    /// ANIM.with(|slot| *slot.borrow_mut() = Some(anim));
    /// ```
    pub fn start<F: FnMut(f64) + 'static>(callback: F) -> Result<Self, Error> {
        Self::start_inner(callback)
    }

    /// Starts a `requestAnimationFrame` loop and gives each callback a reusable [`AnimationFrame`] buffer.
    ///
    /// This is intended for hot animation paths that update attributes such as `x`, `y`, `transform`, `d`, or text every
    /// frame.  Instead of allocating a fresh `String` via `format!(...)` on each frame, write the formatted value into
    /// the provided buffer with methods such as [`AnimationFrame::set_attr_fmt`].
    pub fn start_with_frame<F: FnMut(f64, &mut AnimationFrame) + 'static>(mut callback: F) -> Result<Self, Error> {
        let mut frame = AnimationFrame::new();
        Self::start_inner(move |ts| callback(ts, &mut frame))
    }

    fn start_inner<F: FnMut(f64) + 'static>(mut callback: F) -> Result<Self, Error> {
        let window = web_sys::window().ok_or_else(|| Error::Dom("no window".into()))?;

        let handle: RafHandle = Rc::new(Cell::new(0));
        let closure: SharedClosure = Rc::new(RefCell::new(None));
        let state: Rc<Cell<AnimLoopState>> = Rc::new(Cell::new(AnimLoopState::Idle));

        // Clones moved into the closure so it can re-schedule itself.
        let handle_inner = handle.clone();
        let closure_inner = closure.clone();
        let window_inner = window.clone();
        let state_inner = state.clone();

        // The closure holds an Rc to its own slot so it can re-register after each frame.
        let raf_closure: FrameClosure = Closure::new(move |ts: f64| {
            state_inner.set(AnimLoopState::Dispatching);
            callback(ts);

            // If `stop()` was called from inside the callback, the state is set to `StopPending` and a setTimeout(0) is
            // scheduled to free the closure slot after this body returns.
            // Transition to `Stopped` and skip re-scheduling.
            // Any subsequent `stop()` calls during the same dispatch (a second explicit call, or Drop firing because
            // the handle was dropped inside the callback) will see `StopPending` and collapse into a no-op, so the
            // deferred timer fires exactly once and the closure is never freed while its body is still executing.
            match state_inner.get() {
                AnimLoopState::StopPending => {
                    state_inner.set(AnimLoopState::Stopped);
                    return;
                }
                AnimLoopState::Stopped => return, // should not occur; guard defensively
                AnimLoopState::Dispatching => state_inner.set(AnimLoopState::Idle),
                AnimLoopState::Idle => return, // unexpected
            }

            // Borrow, extract the RAF result, then release the borrow before potentially mutating the slot — avoids a
            // BorrowMutError on the failure path.
            let raf_result = {
                let borrow = closure_inner.borrow();
                borrow
                    .as_ref()
                    .map(|c| window_inner.request_animation_frame(c.as_ref().unchecked_ref()))
            };
            match raf_result {
                Some(Ok(h)) => handle_inner.set(h),
                Some(Err(_)) => {
                    // requestAnimationFrame failed; the loop cannot continue.
                    // Release captures immediately rather than holding them until the AnimationLoop is dropped.
                    state_inner.set(AnimLoopState::Stopped);
                    *closure_inner.borrow_mut() = None;
                },
                None => {}, // stop() already cleared the slot; nothing to do
            }
        });

        // Schedule the first frame from the local binding, then hand the closure to the shared
        // slot. Driving the initial call this way avoids re-borrowing the slot we just filled and
        // the `unwrap` that would have required; on failure `?` drops the closure before anything
        // was scheduled.
        let h = window
            .request_animation_frame(raf_closure.as_ref().unchecked_ref())
            .map_err(dom_err)?;
        handle.set(h);

        *closure.borrow_mut() = Some(raf_closure);

        Ok(AnimationLoop { window, handle, closure, state })
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Cancels the pending animation frame and stops the loop.
    ///
    /// After `stop()` returns, the callback will not be called again and the pending `requestAnimationFrame` handle is
    /// cancelled.
    ///
    /// When called from **outside** the callback, the closure is also freed immediately, releasing any captured values.
    /// When called from **inside** the callback (e.g. a one-shot animation that stops itself on the first frame),
    /// freeing the closure immediately would create a use-after-free of the still-executing closure body.
    /// Instead, `stop()` schedules a zero-delay `setTimeout`: by the time it fires the callback has fully returned and
    /// the captured values are promptly released regardless of when (or even if) the `AnimationLoop` handle is dropped.
    ///
    /// Calling `stop()` is idempotent.
    ///
    /// Repeated calls to `stop()` (either explicitly or via `Drop` when the handle is dropped inside the callback) are
    /// all safe and have no effect after the first call during a given dispatch.
    ///
    /// Normally, there is no need for you to call `stop()` explicitly since dropping the `AnimationLoop` calls it
    /// automatically via the `impl Drop for AnimationLoop` below.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{AnimationLoop, SvgRoot};
    /// use std::{cell::Cell, rc::Rc};
    ///
    /// let svg = SvgRoot::attach("vis").unwrap();
    /// let count = Rc::new(Cell::new(0u32));
    ///
    /// let count_cb = count.clone();
    /// let anim = AnimationLoop::start(move |_| {
    ///     count_cb.set(count_cb.get() + 1);
    /// }).unwrap();
    ///
    /// // Run for a while, then stop programmatically.
    /// // (In practice this would be triggered by a button click or a condition.)
    /// anim.stop();
    /// assert_eq!(count.get(), 0); // not yet run (this is a doc example — no real frames fire)
    /// ```
    pub fn stop(&self) {
        let _ = self.window.cancel_animation_frame(self.handle.get());
        self.handle.set(0);
        match self.state.get() {
            AnimLoopState::Dispatching => {
                // Called from inside the currently-executing RAF callback.  `StopPending` marks the intent to stop
                // without immediately freeing the closure, as freeing it now would create a use-after-free of the Rc
                // fields captured in the still-running closure body.
                //
                // The post-callback code in the RAF wrapper detects `StopPending`, transitions to the `Stopped` state,
                // and skips re-scheduling.  The deferred `setTimeout` then clears the slot.
                //
                // Any further `stop()` call during the same dispatch sees `StopPending` and takes the no-op branch.
                self.state.set(AnimLoopState::StopPending);
                let slot = self.closure.clone();
                let cb = Closure::once_into_js(move || {
                    *slot.borrow_mut() = None;
                });
                // If scheduling fails, `cb` is dropped without being called.  The post-callback code still transitions
                // state to `Stopped`, so a later Drop (if the handle outlives the callback) will enter the `Stopped`
                // branch below and clear the slot there.
                let _ = self
                    .window
                    .set_timeout_with_callback_and_timeout_and_arguments_0(cb.unchecked_ref(), 0);
            }
            AnimLoopState::StopPending => {
                // A second stop() arrived during the same dispatch.  The deferred timer is already scheduled, so this
                // becomes a no-op.
            }
            AnimLoopState::Stopped => {
                // Already stopped.  Also used as a recovery path: if the deferred timer never ran (e.g. setTimeout
                // scheduling failed), the closure slot may still be filled; clear it now since we are guaranteed to be
                // outside callback dispatch when the state is `Stopped`.
                *self.closure.borrow_mut() = None;
            }
            AnimLoopState::Idle => {
                // Not inside a callback: safe to immediately drop the closure and the values it has captured.
                self.state.set(AnimLoopState::Stopped);
                *self.closure.borrow_mut() = None;
            }
        }
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl Drop for AnimationLoop {
    fn drop(&mut self) {
        // stop() handles both the synchronous case (not dispatching — slot cleared immediately) and the deferred case
        // (dispatching — setTimeout(0) scheduled to clear the slot after the callback returns).
        self.stop();
    }
}
