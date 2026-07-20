# Text

[← Back to supported elements](README.md)

**Contents**

- [`<text>` presentation attributes](#text-presentation-attributes)
- [`<textPath>`](#textpath)
- [`<tspan>`](#tspan)

## `<text>` presentation attributes

The `<text>` factory (`SvgRoot::text`, `SvgBatch::text`) returns a plain `SvgNode`.

Four typed helpers are available on any `SvgNode` for styling text:

| Method | Attribute | Type |
|---|---|---|
| `set_font_family(family)` | `font-family` | Any CSS font-family string |
| `set_font_size(size)` | `font-size` | `f64` in user units |
| `set_text_anchor(TextAnchor)` | `text-anchor` | `TextAnchor::{Start, Middle, End}` |
| `set_dominant_baseline(DominantBaseline)` | `dominant-baseline` | `DominantBaseline::{Auto, Alphabetic, Middle, …}` |

**`TextAnchor`** controls which part of the string aligns with the `x` coordinate.

* `Start` (the default) places the beginning of the text at `x`
* `Middle` centres it
* `End` places the end

**`DominantBaseline`** controls which font baseline aligns with the `y` coordinate:

* `Auto` or `Alphabetic` (the default) places the alphabetic baseline on `y`, so ascenders rise above it.
* `Middle` or `Central` vertically centres text on a coordinate.
* `Hanging` is for scripts such as Devanagari or Tibetan, etc. whose bodies hang from the top of the line box.

---

## `<textPath>`

`<textPath>` glues a `<text>` string to the outline of a `<path>` (or, as per the SVG2 specification, a basic shape).
In other words, the baseline of the letters follow the outline defined by the path instead of a straight line.

Obtain a handle by calling `text_path(href, content)` on any `SvgNode` that wraps a `<text>` element (or another `<tspan>`/`<textPath>`).

| Method | Effect |
|---|---|
| `node.text_path(href, content)` | Appends a `<textPath>` with `content`, following the path referenced by `href`. |
| `node.set_start_offset(offset)` | Sets `startOffset` — the distance in user units along the path where the text begins. |
| `node.set_text_path_method(TextPathMethod)` | Sets `method` — `Align` (default) rotates whole glyphs onto the path; `Stretch` distorts glyph outlines to match its curvature. |
| `node.set_text_path_spacing(TextPathSpacing)` | Sets `spacing` — `Auto` (default) compensates spacing for curvature; `Exact` uses the font's natural advance widths. |
| `node.set_text_path_side(TextPathSide)` | Sets the SVG2 `side` attribute — `Left` (default) or `Right` of the path. |

- `href` is a local fragment reference such as `"#wave"` (the `id` attribute of the target `<path>`).
- The referenced path is typically defined inside `<defs>`, or given no fill/stroke, so only the text is visible rather than the guide geometry.
- All text styling helpers (`set_fill`, `set_font_size`, `set_font_family`) work on the returned `SvgNode` exactly as they do for `<tspan>`.
- To offset by a percentage of the path length instead of an absolute distance, call `set_attr("startOffset", "50%")` directly.

**Browser support:** `side` is an SVG2 addition; verify it renders as expected on every browser you target before relying on `TextPathSide::Right` in production.

---

## `<tspan>`

`<tspan>` is an inline text span that lives inside a `<text>` element (or another `<tspan>`).

Each span can override any text presentation attribute inherited from its parent, making it the standard mechanism for multi-line text and mixed-style inline text in SVG.

Obtain a span by calling `tspan` or `tspan_dy` on any `SvgNode` that wraps a `<text>` or `<tspan>` element:

| Method | Effect |
|---|---|
| `node.tspan(content)` | Appends a `<tspan>` with `content`; inherits position from the parent. |
| `node.tspan_dy(dy, content)` | Same but also sets `dy` — advances the text position `dy` user units downward before rendering. |
| `node.set_dy(dy)` | Sets the `dy` attribute on an existing node. |
| `node.set_dx(dx)` | Sets the `dx` attribute on an existing node (horizontal offset). |

All text styling helpers (`set_fill`, `set_font_size`, `set_font_family`, `set_text_anchor`, `set_dominant_baseline`) work on the returned `SvgNode` and override the inherited value for that span only.

**Multi-line text:**<br>
Create a `<text>` with an empty content string (`""`), add the first line as a `tspan`, then add subsequent lines with `tspan_dy` and a consistent `dy` value equal to the desired line height.

**Mixed-style inline text:**<br>
Create a `<text>`, then add each word or phrase as a `tspan`, setting fill/size per span.
When any `<tspan>` children are present the `<text>` element's own text content should be empty.
