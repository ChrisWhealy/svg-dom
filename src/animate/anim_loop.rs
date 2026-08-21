use crate::{animate::anim_frame::AnimationFrame, dom_err, error::Error};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{JsCast, prelude::*};

/// The per-frame closure registered with `requestAnimationFrame`.
type FrameClosure = Closure<dyn FnMut(f64)>;
/// Shared, self-referencing slot used by the closure to re-register itself each frame. Cleared on `stop`.
///
/// This slot is the single source of truth for whether the loop is still running: `stop()` clears it, and the RAF
/// wrapper re-borrows it after `callback(ts)` returns to tell whether `stop()` ran during that call.
type SharedClosure = Rc<RefCell<Option<FrameClosure>>>;
/// Shared cell holding the pending `requestAnimationFrame` id, so it can be cancelled.
///
/// `Some(id)` means a request is currently pending; `None` means it is not.
/// `0` is deliberately not used as a "no pending request" sentinel: it is a valid id `requestAnimationFrame` can
/// return (MDN warns against using `0` this way, since ids are generally an incrementing counter that browsers are
/// not required to handle consistently on overflow).
type RafHandle = Rc<Cell<Option<i32>>>;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// # A running `window.requestAnimationFrame` loop.
///
/// `requestAnimationFrame` is the browser API that schedules a callback immediately before the browser paints the next
/// frame, typically 60 times per second on a 60 Hz display.
/// Your callback receives the frame timestamp as a `DOMHighResTimeStamp`, in the same high-resolution timing domain as
/// `performance.now()`; however, these two values will, most likely, not be the same!
/// Use it to drive time-based animations that stay frame-rate-independent.
///
/// The loop continues until [`stop`](Self::stop) is called or this value is dropped.  Dropping an `AnimationLoop` is
/// always safe since the `Drop` impl calls `stop()` automatically, thus cancelling any pending frame and releasing the
/// closure.
///
/// ## Keeping the loop alive
///
/// The `AnimationLoop` value **must** be kept alive for the loop to continue running.
/// If you drop it immediately (e.g. by assigning it to `_`), `stop()` cancels the pending first frame before it ever
/// fires, and the callback may never run at all.
///
/// The `AnimationLoop` can be kept alive by storing it in a `static`, a `Closure` captured variable, or some other
/// location whose lifespan outlives your animation.
#[must_use = "dropping the AnimationLoop stops the requestAnimationFrame loop — store the handle for as long as the animation should run"]
pub struct AnimationLoop {
    window: web_sys::Window,
    handle: RafHandle,
    closure: SharedClosure,
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl AnimationLoop {
    /// Starts a `requestAnimationFrame` loop, calling `callback(timestamp_ms)` each frame.
    ///
    /// The first frame is scheduled immediately.  Subsequent frames are re-scheduled from inside the closure, using
    /// the self-referencing `Rc<RefCell<Option<Closure>>>` pattern.
    /// The closure captures a reference-counted clone of its own slot, and re-fills it each time it runs.
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
    /// // One page-lifetime slot to hold the running loop (this DOM-facing code runs on the page's main thread).
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
    /// // Keep the loop alive for the page's lifetime. Dropping it would stop it via `Drop`.
    /// ANIM.with(|slot| *slot.borrow_mut() = Some(anim));
    /// ```
    pub fn start<F: FnMut(f64) + 'static>(callback: F) -> Result<Self, Error> {
        Self::start_inner(callback)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Starts a `requestAnimationFrame` loop and gives each callback a reusable [`AnimationFrame`] buffer.
    ///
    /// This is intended for hot animation paths that update attributes such as `x`, `y`, `transform`, `d`, or text every
    /// frame.  Instead of allocating a fresh `String` via `format!(...)` on each frame, write the formatted value into
    /// the provided buffer with methods such as [`AnimationFrame::set_attr_fmt`].
    pub fn start_with_frame<F: FnMut(f64, &mut AnimationFrame) + 'static>(mut callback: F) -> Result<Self, Error> {
        let mut frame = AnimationFrame::new();
        Self::start_inner(move |ts| callback(ts, &mut frame))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    fn start_inner<F: FnMut(f64) + 'static>(mut callback: F) -> Result<Self, Error> {
        let window = web_sys::window().ok_or_else(|| Error::Dom("no window".into()))?;

        let handle: RafHandle = Rc::new(Cell::new(None));
        let closure: SharedClosure = Rc::new(RefCell::new(None));

        // Clones moved into the closure so it can re-schedule itself.
        let handle_inner = handle.clone();
        let closure_inner = closure.clone();
        let window_inner = window.clone();

        // The closure holds an Rc to its own slot so it can re-register after each frame.
        let raf_closure: FrameClosure = Closure::new(move |ts: f64| {
            // The request this callback is currently running for has already fired, so it is no longer pending —
            // clear it before invoking the user callback, so `handle_inner` never names an already-fired id.
            handle_inner.set(None);
            callback(ts);

            // Borrow, extract the RAF result, then release the borrow before potentially mutating the slot — avoids a
            // BorrowMutError on the failure path. If `stop()` ran from inside `callback(ts)` (directly, or via `Drop`
            // firing because the handle was dropped inside the callback), the slot is already `None` here, and
            // `raf_result` is `None` — skip re-scheduling.
            let raf_result = {
                let borrow = closure_inner.borrow();
                borrow
                    .as_ref()
                    .map(|c| window_inner.request_animation_frame(c.as_ref().unchecked_ref()))
            };
            match raf_result {
                Some(Ok(h)) => handle_inner.set(Some(h)),
                Some(Err(_)) => {
                    // requestAnimationFrame failed. The loop cannot continue.
                    // Clear the slot now rather than holding it until the AnimationLoop is dropped. This runs from
                    // inside the still-executing closure, so just like stop(), the actual release follows once this
                    // callback invocation returns.
                    *closure_inner.borrow_mut() = None;
                },
                None => {}, // stop() already cleared the slot during this callback — nothing to do
            }
        });

        // Schedule the first frame from the local binding, then hand the closure to the shared slot. This avoids
        // re-borrowing the slot we just filled, and the `unwrap` that would have required. On failure, `?` drops the
        // closure before anything was scheduled.
        let h = window
            .request_animation_frame(raf_closure.as_ref().unchecked_ref())
            .map_err(dom_err)?;
        handle.set(Some(h));

        *closure.borrow_mut() = Some(raf_closure);

        Ok(AnimationLoop { window, handle, closure })
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Cancels the pending animation frame and stops the loop.
    ///
    /// After `stop()` returns, the callback will not be called again, and the pending `requestAnimationFrame` handle is
    /// cancelled.
    /// `stop()` also immediately clears the stored `Closure` handle.
    /// If the closure is not currently executing, that releases it — and everything it captured — at that point.
    /// If `stop()` is called from **inside** the running callback itself (e.g. a one-shot animation that stops itself
    /// on the first frame), `wasm_bindgen::Closure` keeps the executing closure's data alive until that invocation
    /// returns, then frees it: the captures are never retained merely because the `AnimationLoop` handle itself stays
    /// alive, but they do outlive the `stop()` call by the remainder of the current callback.
    /// `tests/animation_loop.rs`'s `closure_can_drop_itself_from_within_its_own_invocation` verifies this directly, in
    /// a real browser.
    /// See `docs/design_notes/animation.md` for more detail.
    ///
    /// Calling `stop()` is idempotent.
    ///
    /// Repeated calls to `stop()` — either explicitly, or via `Drop` when the handle is dropped inside the callback —
    /// are all safe: a second call finds no pending RAF id to cancel and re-clears an already-empty closure slot.
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
        if let Some(handle) = self.handle.take() {
            let _ = self.window.cancel_animation_frame(handle);
        }
        // Safe even when called from inside the currently-executing RAF callback (`self.closure` is the same slot the
        // closure holds a clone of): dropping an owned `Closure` from within its own invocation keeps its data alive
        // for the rest of that call, only freeing it once the call returns. See this method's doc comment.
        *self.closure.borrow_mut() = None;
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl Drop for AnimationLoop {
    fn drop(&mut self) {
        self.stop();
    }
}
