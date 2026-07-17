# Typesafe Path Data Builder

[← Back to design notes](README.md)

**Contents**

- [Two enums, not one, wrapped in a third](#two-enums-not-one-wrapped-in-a-third)
- [Measuring the nested-enum layout cost, rather than assuming it](#measuring-the-nested-enum-layout-cost-rather-than-assuming-it)
- [`HorizontalLineTo` / `VerticalLineTo` take `f64`, not `Point`](#horizontallineto--verticallineto-take-f64-not-point)
- [`EllipticalArc` is a named-field struct, not a five-element tuple](#ellipticalarc-is-a-named-field-struct-not-a-five-element-tuple)
- [Formatting matches the existing `write_points` convention](#formatting-matches-the-existing-write_points-convention)
- [Path `d` strings omit whitespace](#path-d-strings-omit-whitespace)
- [Two allocation tiers, mirroring `points` / `set_attr_display`](#two-allocation-tiers-mirroring-points--set_attr_display)
- [`build_d` / `build_d_fixed` pre-size their `String`; `write_d` / `write_d_fixed` deliberately do not](#build_d--build_d_fixed-pre-size-their-string-write_d--write_d_fixed-deliberately-do-not)
- [`dps` is clamped once per `write_d_fixed` call, not once per command — but splitting the serializer into `write_default`/`write_fixed` was measured and rejected](#dps-is-clamped-once-per-write_d_fixed-call-not-once-per-command--but-splitting-the-serializer-into-write_defaultwrite_fixed-was-measured-and-rejected)
- [`build_d_fixed`'s capacity estimate scales with `dps`](#build_d_fixeds-capacity-estimate-scales-with-dps)
- [What "prevents malformed path data" actually covers](#what-prevents-malformed-path-data-actually-covers)
- [`create_path_from_defs` validates once, not twice](#create_path_from_defs-validates-once-not-twice)

`SvgRoot::path(d: &str)` (and its siblings on `SvgBatch`, `SvgDefs`, `SvgClipPath`, `SvgMarker`, `SvgPattern`, `SvgSymbol`) writes a `d` path verbatim.
A hand-written `d` string is free text, so there are no safeguards against it being malformed such as a wrong command letter, a missing argument or a transposed flag.
The SVG parser does not reject a malformed `d` string outright; it simply stops rendering at the first token it cannot parse, so the failure is silent and possibly quite difficult to debug

`PathDef` (in `root::path::path_def`, re-exported at the crate root) removes that failure mode by definition.
A `<path>`'s `d` attribute is built from an ordered `&[PathDef]` slice instead of a string; `build_d` / `write_d` do the formatting.

Since a `PathDef` can only ever represent one well-formed SVG command, there is no possibility of creating a malformed `d` string.

## Two enums, not one, wrapped in a third

`PathDefAbsolute` and `PathDefRelative` mirror each other variant-for-variant (`MoveTo`, `LineTo`, `EllipticalArcTo` etc.), differing only in whether the emitted command is upper- or lower-case.

Real SVG path data routinely mixes both within a single path: an initial absolute move command (`M`) followed by a run of relative line (`l`) or curve (`c`) commands is the idiomatic, compact way to define path data by hand.
It is commonplace for callers to mix both absolute and relative path definitions within the same `d` string.

`PathDef::{Abs, Rel}` is the thinnest possible wrapper that permits this: a single `Vec<PathDef>` (or array/slice literal) can freely interleave absolute and relative segments, exactly as hand-written path data would, while each individual segment stays unambiguous about which coordinate space it uses.

## Measuring the nested-enum layout cost, rather than assuming it

Rust does not guarantee enum layout, so whether wrapping `PathDefAbsolute`/`PathDefRelative` in `PathDef` actually costs anything beyond a single flattened enum is a question best answered by using `size_of`/`align_of`, not intuition.

The `pathdef_size_diagnostics` unit test (`src/root/path/unit_tests.rs`) measures it directly and prints the numbers on every run (`cargo nextest run --lib pathdef_size_diagnostics --no-capture`), rather than asserting a fixed byte count that could legitimately change across targets or compiler versions.

Measured on both the host target (x86_64/aarch64, `usize` = 8 bytes) and `wasm32-unknown-unknown` (`usize` = 4 bytes) — the numbers were identical on both, because every field in these types is an `f64`, `Point`, or a small fieldless enum, so alignment is driven entirely by `f64`'s 8-byte alignment, not by pointer width:

| Type | `size_of` | `align_of` |
|---|---|---|
| `Point` | 16 | 8 |
| `EllipticalArc` | 48 | 8 |
| `PathDefAbsolute` | 56 | 8 |
| `PathDefRelative` | 56 | 8 |
| `PathDef` | 64 | 8 |

`PathDef` is 8 bytes larger than either inner enum alone — a real, measured cost, not a hypothetical one.
`ArcSize`/`ArcSweep` are two-variant fieldless enums, which do have a spare-bit-pattern niche a wrapping enum's discriminant could in principle occupy, but rustc's current layout algorithm does not thread that niche out through `EllipticalArc` and then through `PathDefAbsolute`/`PathDefRelative` to `PathDef`; instead the outer discriminant gets its own padded slot, sized to the type's 8-byte alignment.
That slot is exactly one alignment unit, not an unbounded amount — `pathdef_size_diagnostics` asserts `size_of::<PathDef>() <= size_of::<PathDefAbsolute>() + align_of::<PathDefAbsolute>()` (and the `PathDefRelative` equivalent) as a structural regression guard, so a future accidental size regression (e.g. an added field, or a future rustc layout change that stops finding even this bound) fails the test rather than going unnoticed.

For a `Vec<PathDef>` holding many commands, that is a genuine ~14% (8/56) memory overhead per command versus a single flattened ~20-variant enum, which, because its own largest variant is no bigger than `PathDefAbsolute`'s, would likely pay the same one-alignment-unit discriminant cost but only once, not twice.

This difference is real and worth knowing about, but on its own, this is not a reason to flatten: it only matters if a program builds and retains large `Vec<PathDef>` arrays long-term (most callers build a `d` string once via `build_d`/`write_d` and then discard or reuse the `defs` slice), and flattening would double the variant count and duplicate the absolute/relative distinction across every command name — the API cost [`Two enums, not one, wrapped in a third`](#two-enums-not-one-wrapped-in-a-third) above already weighed against.

Revisit only if profiling (not this measurement alone) shows stored `PathDef` arrays materially affecting memory footprint or serializer dispatch time in a real caller.

## `HorizontalLineTo` / `VerticalLineTo` take `f64`, not `Point`

The SVG `H`/`h` and `V`/`v` commands each take a single coordinate.
`H` takes a bare `x` and `V` takes a bare `y`, not a full `(x, y)` coordinate pair.

## `EllipticalArc` is a named-field struct, not a five-element tuple

The SVG arc commands (`A`/`a`) take two boolean flags (`large-arc-flag`, `sweep-flag`) to select between the (up to) four geometric solutions for an arc between two points at a given radius.

As adjacent positional `bool`s in a tuple variant, they are easy to transpose — `(true, false)` vs `(false, true)` looks the same at a glance and the compiler cannot catch the swap.
`ArcSize` (`Small`/`Large`) and `ArcSweep` (`CounterClockwise`/`Clockwise`) turn each flag into a self-documenting enum, and bundling all five arc parameters into one named-field `EllipticalArc` struct (rather than a five-argument tuple variant) means every field is labelled at the construction site instead of positional.

`EllipticalArc::write` takes a `cmd: char` so the one method can serve both the `A` and `a` forms without duplicating its formatting body, but a bare `char` parameter accepts anything — nothing about the argument type stops a caller passing some nonsense value such as `'X'` and producing a command letter no SVG parser recognises.
The two real call sites (`path_def.rs`, passing the literal `'A'`/`'a'`) are the only ones that need to exist, so `write` is `pub(super)`, not `pub`, even though `EllipticalArc` itself is public: the struct's fields must stay public for callers to construct `PathDefAbsolute::EllipticalArcTo(EllipticalArc { .. })` literals, but the serialization method is purely internal machinery, and leaving it `pub` would have let a caller bypass `PathDef`'s well-formed-command guarantee through this one method while every other route into a `d` string stayed safe.

## Formatting matches the existing `write_points` convention

Coordinates are written with plain `{}` (`Display`) formatting (Rust's shortest round-trip representation) rather than a fixed decimal count, mirroring `write_points`'s default-precision path in `root::utils`.
This keeps whole-number demo coordinates compact (`"70"`, not `"70.0"`).

`write_d_fixed` / `build_d_fixed` (and the `d_from_defs_fixed` methods layered on top, mirroring `points_fixed`) do add a fixed-precision mode — but the "n decimal places for everything" knob only ever reaches the genuinely continuous arguments: coordinates, lengths, and the arc's `x_axis_rotation`.
It deliberately never reaches the two Boolean flags belonging to [`EllipticalArc`].
`large-arc-flag` and `sweep-flag` are written via `ArcSize`/`ArcSweep`'s `u8` `Display` regardless of `dps`, because the SVG `flag` grammar require Boolean `true` and `false` to be represented as `"0"` or `"1"`.

This is the concrete version of the general caution about path data mixing several different argument shapes: a uniform `dps` is safe for every numeric field *except* the numeric representation of a Boolean value.
So those two fields are simply carved out of the fixed-precision path entirely rather than trusting a caller to remember not to round them.

## Path `d` strings omit whitespace

The Backus-Naur Form (BNF) of the SVG path-data allows every command to have zero or more whitespace characters (`wsp*`) between the command letter and the first argument, not one or more (`wsp+`).
Since a command letter can never appear inside a number, that command letter unambiguously terminates whichever number preceded it, meaning the separator between a command's last argument and the next command's letter is grammatically unnecessary.
`write_d` and every per-command `write` method rely on both of these facts: thus we can write `"M{} {}"` instead of `"M {} {}"` for the command/first-argument boundary, and within the `write_d` loop, there is no need to add whitespace between commands.

`"M10 10L100 50L10 90Z"` and `"M 10 10 L 100 50 L 10 90 Z"` parse to the identical path in every conforming SVG implementation — this is a standard, lossless minification technique (the same one tools like SVGO apply), not an approximation, so there is no loss of precision or correctness trade-off.

For a path of `N` commands, of which `K` of them take arguments, this removes exactly `(N - 1) + K` bytes.
So for a long, procedurally-generated path (e.g. a fine-grained curve sampled as many `LineTo` segments), the saving is proportional to the number of commands, which is exactly the case where a smaller `d` string matters most: less data serialized means less data has to cross the WASM/JS boundary resulting in a shorter DOM attribute.

Separator elision between arguments *within* a command is deliberately not attempted (e.g. relying on a leading `-` or `.` to glue two numbers together without a space).
That trick is real per the SVG grammar too, but it depends on the sign and shape of each emitted number and requires per-value inspection to stay unambiguous
In reality, thus buys us far less than the always-safe, context-free whitespace removal described above.

**NOTE**:Eliding a repeated command letter (`"M0 0L10 10 20 20 30 30"` instead of `"M0 0L10 10L20 20L30 30"`) is also permitted by the grammar but has not been implemented as this requires stateful serialization (tracking the previous command's letter in both the absolute and relative forms across multiple iterations).
This in turn introduces a real correctness hazard specific to the move (`M`/`m`) command: a repeated move command's extra coordinate pairs are reinterpreted by the parser as implicit `L`/`l` commands, so naively eliding a repeated `M` changes the path's meaning, not just its byte count.

That complexity is only worth taking on for paths long enough that the extra savings are measurable; until then, the always-safe whitespace removal above is the better cost/benefit trade-off.

## Two allocation tiers, mirroring `points` / `set_attr_display`

An earlier version of this feature had `path_from_defs` and `SvgNode::set_d_from_defs` both call `build_d`, which allocates a fresh `String` on every call.
That included the shared `SvgFactory::create_path_from_defs` default method used by every `path_from_defs` factory sibling — nothing in the shipped API actually called `write_d` outside of `build_d`'s own body, contradicting `write_d`'s own documentation, which describes it as the buffer-reusing path for hot call sites.

The fix follows the crate's existing two-tier split for `points`, verbatim:

- **Node *creation*** (`path_from_defs` on `SvgRoot` and its factory siblings) now writes `d` through the factory's own retained `SvgAttrs` buffer — the same `self.attrs().borrow_mut()` pattern `create_rect` and friends already use — so repeated calls on one factory allocate at most once (for buffer growth), not once per call.

- **Node *updates* on a live `SvgNode`** still have two tiers, exactly as `set_font_size` (allocating) and `set_attr_display` (caller-owned buffer) do for other attributes: `SvgNode::set_d_from_defs` remains a convenience that allocates a short-lived `String` per call (which is fine for an occasional update) while `SvgAttrs::d_from_defs` / `AttrWriter::d_from_defs` and `AnimationFrame::set_d_from_defs` reuse a caller-owned buffer for a path that is morphed on every `pointermove` event or every animation frame.

`SvgNode` has no buffer of its own to reuse (it is a lightweight `Rc` handle, not a factory), which is exactly why the crate's hot-path attribute setters — `set_attr_display`, the transform setters, `AnimationFrame` — all take the scratch buffer as a parameter rather than owning one (see [Performance patterns](performance.md)).
`d_from_defs` follows that same shape rather than inventing a new one.

## `build_d` / `build_d_fixed` pre-size their `String`; `write_d` / `write_d_fixed` deliberately do not

`build_d` and `build_d_fixed` are the one guaranteed-fresh-allocation case in the whole path API: every other entry point writes into a buffer the caller already owns and is expected to reuse.
Starting that fresh `String` from `String::new()` means hitting the usual doubling-reallocation pattern, as a path will grow from nothing.
This then incurs the cost `write_points` already avoids for a point lists by reserving a rough capacity upfront.

Both functions now reserve `defs.len() * APPROX_BYTES_PER_COMMAND` before writing.
`APPROX_BYTES_PER_COMMAND` is set to 24, the same per-entry "best guess" used by `write_points` for its default-precision path.

24 bytes is a rough, deliberately non-variant-aware estimate: `ClosePath` needs one byte, but a six-argument `CubicBezierTo` with large float coordinates needs several times that, so no single flat constant is exactly right for every path shape.
Computing a precise per-variant estimate would mean a second pass over `defs`, matching every command to sum its exact argument count and typical width — more work than the reallocations it would save, for a number that is already only ever a lower-bound guess (a `String` that undershoots just grows normally; it never produces wrong output).

`write_d` / `write_d_fixed` do not reserve anything themselves, unlike `write_points`, which calls `out.reserve(..)` on every invocation regardless of whether the caller is reusing the buffer.
The two functions serve different callers: `write_points` has no one-shot sibling to shoulder the sizing concern, so it has to do double duty.
`write_d` does have one (`build_d`), so the buffer-reusing function stays lean — clear, then append — and relies on the caller-owned buffer's capacity already being retained from a previous call (or, for a caller who cares about even the first call, constructing the buffer via `SvgAttrs::with_capacity` upfront rather than `SvgAttrs::new()`).

## `dps` is clamped once per `write_d_fixed` call, not once per command — but splitting the serializer into `write_default`/`write_fixed` was measured and rejected

Every per-command `write` originally took `dps: Option<usize>` and re-derived `n.min(MAX_DPS)` inside its own `Some(n)` arm — for `SmoothQuadraticBezierTo`/`EllipticalArcTo` specifically, more than once per arm, since each numeric argument's `{:.*}` format spec repeated the `.min(MAX_DPS)` call.

Since `dps` does not vary across a single `write_d_fixed` call, clamping is now done exactly once, before the loop and the already-clamped value is threaded down unchanged.
This part is a pure win with no downside: it is strictly less source, strictly fewer redundant comparisons, and provably produces byte-identical output (every existing fixed-precision test, including the one asserting `usize::MAX` and `MAX_DPS` clamp to the same result, passed unchanged).

A further step — splitting each `write` into separate `write_default(&self, out)` / `write_fixed(&self, out, dps: usize)` methods, so the per-command code no longer branches on `Option<usize>` at all.
This idea was also tried, but discarded after measurement rather than adopted on the strength of the argument alone.

The two versions were built and compared: full `cargo build --release --target wasm32-unknown-unknown`, then `wasm-opt -O3`, for the crate with only the clamp-hoist applied versus the crate with the full `write_default`/`write_fixed` split on top.
The resulting `.wasm` files were **byte-for-byte identical** (same MD5, both before and after `wasm-opt`) in both cases.
Rustc/LLVM already specializes `write_d`'s and `write_d_fixed`'s respective inlined call sites against the constant `None`/`Some(..)` they each always pass, so hand-writing that specialization as two separate methods produced no binary difference of any kind — no size change, and (since the generated code is identical) no possible runtime difference either.

Given that outcome, the split was reverted: it would have doubled the match-arm source for every current and future `PathDef` variant (a real, ongoing risk of the two copies drifting apart) in exchange for a measured benefit of exactly zero.
This is the concrete version of the reasoning that already kept this crate from making a dependency to `ryu`/`itoa` for numeric formatting — an optimization is only worth its complexity cost if it can provide a measurable benefit, not merely because it looks like it should.

## `build_d_fixed`'s capacity estimate scales with `dps`

`BASE_BYTES_PER_COMMAND` (24) was tuned for the *default*, shortest-round-trip format, and both `build_d` and `build_d_fixed` originally reserved `defs.len() * 24` regardless of precision.
This is fine until we encounter a high `dps` value applied to, say, a six-argument `CubicBezierTo`.
Here, for `dps = 20`, the six-argument `CubicBezierTo` formats to roughly 138 bytes (`"C0.00000000000000000000 0.00000000000000000000 ..."`), against a 24-byte reservation for the whole command, which is nearly a 6-fold shortfall, guaranteeing at least one (but usually several), reallocate-and-copy doublings for that command alone.

`build_d_fixed` now reserves `defs.len() * (BASE_BYTES_PER_COMMAND + APPROX_VALUES_PER_COMMAND * dps.min(MAX_DPS))`, with `APPROX_VALUES_PER_COMMAND = 3`.
`build_d` (no `dps`) is unaffected and keeps the flat 24-byte guess.

Setting `APPROX_VALUES_PER_COMMAND` to `3` is a deliberate *average*, not a per-command worst-case bound: real commands range from zero numeric arguments (`ClosePath`) to six (`CubicBezierTo`).

A test proved this directly (`build_d_fixed_capacity_formula_improves_on_flat_guess_for_high_precision_cubic_bezier`): for the six-argument case above, the new formula reserves 84 bytes against a real 138 — still short, but the shortfall drops from 114 bytes to 54, roughly halved, and the reservation is closer to exact for the far more common shorter commands (`MoveTo`, `LineTo`, `HorizontalLineTo`) that a real path is mostly made of.

The first version of this fix asserted the new formula fully covered the worst case; that test failed immediately (84 << 138), so the assertion was corrected to match what three-as-an-average actually promises: a measurable improvement, not a guarantee.

A second, more accurate option exists: sum each command's actual numeric-argument count via a `PathDef::numeric_arg_count` helper, in a dedicated pass over `defs` before allocating.

This has deliberately not been implemented.

It is exactly the "second pass over `defs` matching every variant" this module's capacity estimates already decline to perform elsewhere, for the same reason: the win only matters for `build_d_fixed`'s one guaranteed-fresh-allocation case (a direct call, a first use of a fresh buffer, or a workload that keeps constructing new paths rather than updating one in place — `write_d_fixed` on a retained buffer is unaffected either way), and no benchmark has shown that case to be a real bottleneck worth a second traversal by which further reallocations can be avoided.
If one ever does, the variant-aware pass is the documented next step, not a redesign.

## What "prevents malformed path data" actually covers

Early documentation for `PathDef` claimed the resulting `d` string "can never contain a mistyped command letter, a missing argument, or *any other* malformed path data."
The last clause overstated the guarantee: `PathDef` prevents malformed *commands* — spelling, argument arity, arc-flag validity — but was silent about two ways a *sequence* of individually well-formed commands can still fail to be a valid path.

**SVG requires a non-empty path to start with a moveto.**

`[PathDef::Abs(PathDefAbsolute::LineTo(..))]` formats into perfectly well-formed path *syntax* — `"L1 1"` — that is nonetheless not valid path *data*: the SVG grammar requires a non-empty path to begin with an `M`/`m`.
Not only will a conforming user agent silently render nothing for a path that starts with anything else, it will also not report an error.
This is cheap to catch (an O(1) look at `defs.first()`), so `path_from_defs`, `SvgNode::set_d_from_defs`, `SvgAttrs::d_from_defs` / `d_from_defs_fixed`, and `AnimationFrame::set_d_from_defs` / `set_d_from_defs_fixed` all call `validate_starts_with_moveto` and return `Error::InvalidPathData` if it fails — including the per-frame `SvgAttrs`/`AnimationFrame` methods, since the check costs nothing beyond that single comparison regardless of call frequency.

A leading relative moveto (`m`) is accepted because no current point yet exists to which a relative point can refer, so the SVG spec always treats a path's very first moveto command as absolute, irrespective of whether `m` or `M` is used.

`build_d` / `write_d` (and their `_fixed` siblings) deliberately do **not** call this check.
They are the lowest-level formatters in the module and may legitimately be asked to build a path-data *fragment* that isn't meant to stand alone (e.g. a caller is assembling several `PathDef` slices before concatenating them) so enforcing "must start with a moveto" at this location would reject legitimate uses.
The check exists only at the boundary where a sequence is committed to an element's actual `d` attribute.

**Coordinates are unconstrained `f64` values.**

Nothing with the definition of a `Point` field stops it from holding values such as `f64::NAN` or `f64::INFINITY`.

The SVG number grammar has no token for either, so Rust's `Display` output for them (`"NaN"`, `"inf"`, `"-inf"`) is not valid path syntax, and unlike the moveto check, catching this is *not* cheap: it means visiting every numeric argument of every command, an O(total arguments) traversal rather than an O(1) look at one element.
That cost would land squarely on `write_d`/`write_d_fixed`, the functions this whole feature exists to keep cheap for a per-frame caller, so this crate does not check for it anywhere in the path API.

⚠️ Caveat ⚠️

A caller whose coordinates come from a calculation that could produce a non-finite value (division, trigonometry) is expected to validate with `f64::is_finite()` before constructing the `PathDef` — the same "caller's responsibility at the boundary" shape as the `set_attr` security caveat elsewhere in this crate.

## `create_path_from_defs` validates once, not twice

Wiring `validate_starts_with_moveto` into both `create_path_from_defs` (the shared `SvgFactory` default method behind every `path_from_defs` factory) and `SvgAttrs::d_from_defs` (the natural place to put each check in isolation) meant `path_from_defs` ran the same check twice: once before `make_node("path")`, and again inside `d_from_defs` when the factory wrote the freshly-created node's `d` attribute.

The factory's own check has to stay: it is the one that matters, since it rejects a bad `defs` slice *before* a detached `<path>` element is ever created, rather than after.
Removing it and relying solely on `SvgAttrs::d_from_defs`'s check would mean a bad path first allocates a DOM node, then discards it — wasted work on the failure path and a small window where a detached, doomed element exists for no reason.

`SvgAttrs::d_from_defs` is therefore split into two: the public method still validates (a caller reaching it directly — via `AttrWriter` or by hand — has had no earlier chance to check), then delegates to `pub(crate) fn d_from_validated_defs`, the unchecked core that just writes.
`create_path_from_defs` calls `d_from_validated_defs` directly, skipping the redundant second pass over the same three-or-so-byte slice prefix it already inspected moments earlier.
The saving is not the point since an O(1) check is not worth optimising away for its own sake, but leaving it in obscured which validation call in the sequence was the one actually performing the protection, which on clarity grounds alone, is worth fixing.
