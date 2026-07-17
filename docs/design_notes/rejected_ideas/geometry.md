# Geometry and viewport

[← Back to rejected ideas](README.md)

See [Geometry read-back](../geometry.md) for the design notes on the actual `bounding_box()`/`bounding_client_rect()`
coordinate-space distinction this rejection foreshadowed.

## Provide a rendered-size fallback (`getBoundingClientRect`) when seeding the cached viewport

`SvgRoot::attach` reads only the `width` and `height` attributes to seed the cached viewport, so an `<svg>` sized purely with CSS will have cached dimensions of `0 × 0`.
It was suggested that it is necessary to provide a `read_viewport` fall back that returns the rendered measurement such as `getBoundingClientRect()` or the client dimensions when these attributes are absent.
We tightened the documentation instead (`attach` now states that only the two attributes are read, and points CSS-sized callers at `set_viewport`).

* **It would mix two incompatible coordinate spaces and break the write-elision it feeds.**<br>
  The cached viewport is authoritative for `width()`/`height()` *and* for `set_viewport`, which skips redundant DOM writes by comparing the requested size against the cache, then writing `width`/`height` **attributes** (in user units).
  `getBoundingClientRect()` returns rendered **CSS pixels**, which will differ from the attribute units whenever a `viewBox` or CSS scaling is in play.
  Seeding the cache from rendered pixels and then removing attribute writes against it would end up comparing raw values without considering that the units of measure may have become incompatible.
  This would turn a correctness-neutral optimisation into a latent bug.

* **The fallback becomes unreliable exactly when it is needed most.**<br>
  `attach` is frequently called either during module `init`, before first layout/paint, or on a `display:none`/not-yet-attached element — all of which will return a measurement of `0`.
  So the fallback would not even fix the motivating case dependably; it would only paper over some of it, while making the failure mode harder to reason about (sometimes `0`, sometimes a stale pre-layout value).

* **Rendered measurement was, at the time, a documented non-goal — and even now that it is wrapped, the specific proposed *use* remains wrong.**<br>
  At the time of this rejection, `docs/gaps.md` listed `getBoundingClientRect()` among the deliberately out-of-scope DOM-geometry features.
  `SvgNode::bounding_client_rect()` has since been implemented as a general-purpose read-only geometry method (see [Geometry read-back](../geometry.md)), so that specific blocker no longer applies — but this does not reopen idea 7.
  `bounding_client_rect()` is not used to seed any cache, and the coordinate-space-mixing problem in the first bullet above is unaffected by the method now existing: feeding a CSS-pixel measurement into an attribute-unit cache is exactly as wrong today as it was before the method was wrapped.
  The crate's contract remains that `width()`/`height()` report the *attribute* values read once at attach time; a caller who needs the rendered size can call `bounding_client_rect()` themselves and pass the result to `set_viewport`, which keeps the cache coherent with what the crate actually writes.
