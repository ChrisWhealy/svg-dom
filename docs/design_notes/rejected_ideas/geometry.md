# Geometry and viewport

[← Back to rejected ideas](README.md)

See [Geometry read-back](../geometry.md) for the design notes on the actual `bounding_box()`/`bounding_client_rect()` coordinate-space distinction this rejection foreshadowed.

## Provide a rendered-size fallback (`getBoundingClientRect`) when seeding the cached viewport

`SvgRoot::attach` reads only the `width` and `height` attributes to seed the cached viewport.
An `<svg>` sized purely with CSS will therefore have cached dimensions of `0 × 0`.
One proposal was a `read_viewport` fallback.
It would return the rendered measurement, such as `getBoundingClientRect()` or the client dimensions, when these attributes are absent.
We tightened the documentation instead.
`attach` now states that only the two attributes are read.
It also points CSS-sized callers at `set_viewport`.

* **It would mix two incompatible coordinate spaces and break the write-elision it feeds.**<br>
  The cached viewport is authoritative for `width()`/`height()` and for `set_viewport`.
  `set_viewport` skips redundant DOM writes by comparing the requested size against the cache.
  It then writes the `width`/`height` **attributes**, in user units.
  `getBoundingClientRect()` returns rendered **CSS pixels**.
  CSS pixels differ from the attribute units whenever a `viewBox` or CSS scaling is in play.
  Seeding the cache from rendered pixels, then eliding attribute writes against it, would compare values in two different units.
  The comparison would not account for that mismatch.
  This would turn a correctness-neutral optimisation into a latent bug.

* **The fallback becomes unreliable exactly when it is needed most.**<br>
  `attach` is frequently called during module `init`, before first layout/paint, or on a `display:none`/not-yet-attached element.
  All of these cases return a measurement of `0`.
  So the fallback would not reliably fix the motivating case.
  It would only mask part of the problem.
  It would also make the failure mode harder to reason about, since the value returned would sometimes be `0` and sometimes a stale pre-layout value.

* **Rendered CSS-pixel geometry is available through `SvgNode::bounding_client_rect()`, but it remains unsuitable as a fallback for `SvgRoot`'s user-unit viewport cache.**<br>
  At the time of this rejection, `docs/gaps.md` listed `getBoundingClientRect()` among the deliberately out-of-scope DOM-geometry features.
  That blocker no longer applies.
  `bounding_client_rect()` has now been implemented as a general-purpose read-only geometry method (see [Geometry read-back](../geometry.md)).

  This does not reopen that rejection.
  `bounding_client_rect()` is not used to seed any cache.
  The coordinate-space-mixing problem from the first bullet above is unaffected by this method.
  Feeding a CSS-pixel measurement into an attribute-unit cache is exactly as wrong today as it was before this method was wrapped.

  What changed is where a caller gets the rendered size from.
  A caller who wants it no longer needs a method from outside the crate.
  `bounding_client_rect()` is the supported way to obtain it explicitly.
  The caller then passes the result to `set_viewport`, which keeps the cache coherent with what the crate actually writes.
  `SvgRoot::attach`, `width()`, and `height()` still never call it implicitly.
