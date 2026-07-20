# Structural and reusable elements

[← Back to supported elements](README.md)

**Contents**

- [`<defs>`](#defs)
- [`<marker>`](#marker)
- [`<image>`](#image)
- [`<symbol>`](#symbol)
- [`<use>`](#use)

## `<defs>`

`<defs>` is the standard SVG container for reusable assets and can be obtained from `SvgRoot::defs()`.
All shape factory methods are available on `SvgDefs` for building inner content.

---

## `<marker>`

`<marker>` defines a reusable graphic (e.g. an arrowhead or a dot etc) rendered at the start, mid-point, or end of a stroked path and can be obtained from `SvgDefs::marker(id)`.

Apply it to any stroked element — `<line>`, `<path>`, `<polyline>`, `<polygon>` — via `SvgNode::set_marker_start`, `set_marker_mid`, or `set_marker_end`.
The `MarkerUnits` enum controls whether `markerWidth`/`markerHeight` are relative to `strokeWidth` (default) or user coordinates.

`set_view_box(x, y, width, height)` establishes an internal coordinate system for the marker's content, mapped onto the `markerWidth`/`markerHeight` viewport — the same `viewBox` relationship `<symbol>`/`<use>` has, validated the same way (`Error::InvalidViewBox` on a non-finite component or a negative `width`/`height`). `preserveAspectRatio` has no dedicated setter for `<marker>`; use `set_attr("preserveAspectRatio", value)`.

---

## `<image>`

`<image>` embeds a raster image (PNG, JPEG, WebP etc) or another SVG into the current document.
Obtain a handle via `SvgRoot::image(href, top_left, size)` or `SvgBatch::image(href, top_left, size)`.

- `href` accepts any URL the browser can fetch: a relative path, an absolute URL, or a `data:` URI.
  When using `data:image/svg+xml`, use base64 encoding to avoid percent-encoding `<`, `>`, and `#`.
- `top_left` and `size` define the display rectangle.

  `svg-dom`'s `image` constructor requires a `Size` and therefore always writes both `width` and `height`; a zero value for either dimension prevents rendering.
  This constraint is applied only by thois convenience constructor, it is not actually part of SVG 2 itself.
  SVG 2 permits automatic sizing from the referenced resource's own intrinsic dimensions when `width`/`height` are omitted.

- Control aspect-ratio handling with `set_attr("preserveAspectRatio", value)`:
  - `"xMidYMid meet"` — fit the whole image inside the box, adding letterbox bars if needed (default).
  - `"none"` — stretch to fill the box exactly, ignoring the source aspect ratio.
  - `"xMidYMid slice"` — scale up to fill the box and clip any overflow.
- To swap the image source after creation, call `SvgNode::set_href`.

---

## `<symbol>`

A `<symbol>` defines a reusable viewport.
Unlike a plain `<g>` in `<defs>`, it can carry its own `viewBox` and `preserveAspectRatio`.
The browser scales the symbol's content to fit the `<use>` element's `width` and `height`, exactly as it would an embedded `<svg>` &mdash; so the same definition renders correctly at any size with no manual rescaling.

### <symbol> API

Obtain a handle via `SvgDefs::symbol(id)` or the transactional `SvgDefs::build_symbol(id, closure)`:

| Method | Description |
|---|---|
| `set_view_box(x, y, w, h)` | Establishes the symbol's internal coordinate space |
| `set_preserve_aspect_ratio(value)` | Controls alignment / clipping when the `<use>` dimensions differ from the `viewBox` aspect ratio |
| `set_id(&mut self, id)` | Renames the symbol (updates both the DOM and the cached id) |
| `set_attr(name, value)` | Generic setter for unlisted attributes (`class`, `style`, `overflow` …) |

All shape factory methods (`rect`, `circle`, `ellipse`, `line`, `path`, `polyline`, `polygon`, `text`, `group`) are available on `SvgSymbol`.

### Stamping Copies

Pass the symbol's id (prefixed with `#`) to `SvgRoot::use_node` to stamp a copy of the symbol at a given position:

```rust,no_run
defs.build_symbol("badge", |s| {
    s.set_view_box(0.0, 0.0, 40.0, 40.0)?;
    s.circle(Point::new(20.0, 20.0), 18.0)?.set_fill("steelblue")?;
    Ok(())
})?;

// Each <use> can have its own width/height; the viewBox scales the content automatically.
svg.use_node("#badge", Point::new(10.0, 10.0))?.set_attr("width", "40")?;
svg.use_node("#badge", Point::new(60.0, 10.0))?.set_attr("width", "80")?;
```

### `id` Rules

Symbol ids follow the same allow-pattern as markers and gradients: `[A-Za-z_][A-Za-z0-9_-]*`.
A non-conforming id causes `Error::InvalidSymbolId` to be raised before any DOM call is made.

Always use `SvgSymbol::set_id` to rename a symbol after construction; `set_attr("id", ...)` will be rejected with `Error::ReservedAttribute` to protect the cached value.

---

## `<use>`

`<use>` stamps a copy of any element (typically one defined inside `<defs>`) into the rendered tree without duplicating the DOM node.

Obtain a handle via `SvgRoot::use_node(href, at)` or `SvgBatch::use_node(href, at)`.

- `href` is a local fragment reference such as `"#my-shape"` (the `id` attribute of the target element).
- `at` is an `(x, y)` offset in the parent coordinate system; pass `Point::origin()` to control positioning entirely through `transform`.
- Each returned `SvgNode` is independent: attributes set on one copy never affect the original or any other copy, but what an attribute actually does depends on its kind:
  - `transform` is a geometric attribute and is not inherited.
    `opacity` is applied once to the generated instance, like a group opacity.
    Both of these attributes always take effect independently per copy.
  - Presentation properties such as `fill` or `stroke` provide inherited values to the referenced instance only.
    They do **not** override an explicit `fill` or `stroke` already set on the referenced element or one of its descendants.
  - A `<use>` on a `<symbol>` that hard-codes its own colours will not be recoloured by `set_fill` on the `<use>` instance.
    This is the single most common surprise when styling `<use>` copies.
- To change the referenced element after creation, call `SvgNode::set_href("#other-shape")`.

Any change to the original definition is immediately visible in all copies.
A `<use>` element can reference any element by id, including a `<symbol>` (see the [`<symbol>`](#symbol) section above).
