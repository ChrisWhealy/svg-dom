use crate::{animate::anim_frame::AnimationFrame, error::Error};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{JsCast, prelude::*};

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
pub struct AnimationLoop {
    window: web_sys::Window,
    handle: Rc<Cell<i32>>,
    closure: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>>,
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
    /// use svg_dom::{AnimationLoop, SvgRoot};
    /// let svg = SvgRoot::attach("vis").unwrap();
    /// let path = svg.path("M 0 50 L 200 50").unwrap();
    ///
    /// let _loop = AnimationLoop::start(move |ts| {
    ///     // Animate the midpoint of the path upward and downward.
    ///     let y = 50.0 + 30.0 * (ts / 600.0).sin();
    ///     let _ = path.set_d(&format!("M 0 50 Q 100 {y} 200 50"));
    /// }).unwrap();
    ///
    /// std::mem::forget(_loop); // keep alive for the lifetime of the page
    /// ```
    pub fn start<F: FnMut(f64) + 'static>(callback: F) -> Result<Self, Error> {
        Self::start_inner(callback)
    }

    /// Starts a `requestAnimationFrame` loop and gives each callback a reusable [`AnimationFrame`] buffer.
    ///
    /// This is intended for hot animation paths that update attributes such as `x`, `y`, `transform`, `d`, or text every
    /// frame.  Instead of allocating a fresh `String` via `format!(...)` on each frame, write the formatted value into
    /// the provided buffer with methods such as [`AnimationFrame::set_attr_fmt`].
    pub fn start_with_frame<F: FnMut(f64, &mut AnimationFrame) + 'static>(
        mut callback: F,
    ) -> Result<Self, Error> {
        let mut frame = AnimationFrame::new();
        Self::start_inner(move |ts| callback(ts, &mut frame))
    }

    fn start_inner<F: FnMut(f64) + 'static>(mut callback: F) -> Result<Self, Error> {
        let window = web_sys::window().ok_or_else(|| Error::Dom("no window".into()))?;

        let handle: Rc<Cell<i32>> = Rc::new(Cell::new(0));
        let closure: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));

        // Clones moved into the closure so it can re-schedule itself.
        let handle_inner = handle.clone();
        let closure_inner = closure.clone();
        let window_inner = window.clone();

        // The closure holds an Rc to its own slot so it can re-register after each frame.
        *closure.borrow_mut() = Some(Closure::new(move |ts: f64| {
            callback(ts);

            if let Some(c) = closure_inner.borrow().as_ref() {
                match window_inner.request_animation_frame(c.as_ref().unchecked_ref()) {
                    Ok(h) => handle_inner.set(h),
                    Err(_) => {} // requestAnimationFrame failed — loop stops silently
                }
            }
        }));

        let h = window
            .request_animation_frame(closure.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .map_err(|e| Error::Dom(format!("{e:?}")))?;
        handle.set(h);

        Ok(AnimationLoop {
            window,
            handle,
            closure,
        })
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Cancels the pending animation frame and stops the loop.
    ///
    /// After `stop()` returns:
    /// - the callback will not be called again,
    /// - the pending `requestAnimationFrame` handle is cancelled,
    /// - the closure is dropped (freeing any captured values).
    ///
    /// Calling `stop()` is idempotent; therefore, attempting to stop an already-stopped loop is safe and has no effect.
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
        // Setting the slot to None drops the Closure, preventing the next re-schedule.
        *self.closure.borrow_mut() = None;
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl Drop for AnimationLoop {
    fn drop(&mut self) {
        self.stop();
    }
}
