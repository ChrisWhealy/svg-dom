# Structural and reusable elements

[← Back to supported elements](README.md)

**Contents**

- [`<defs>`](#defs)
- [`<marker>`](#marker)
- [`<image>`](#image)
- [`<symbol>`](#symbol)
- [`<use>`](#use)
- [`<a>`](#a)
- [`<switch>`](#switch)
- [`<view>`](#view)

## `<defs>`

`<defs>` is the standard SVG container for reusable assets and can be obtained from `SvgRoot::defs()`.
All shape factory methods are available on `SvgDefs` for building inner content.

---

## `<marker>`

`<marker>` defines a reusable graphic (e.g. an arrowhead or a dot etc) rendered at the start, mid-point, or end of a shape and can be obtained from `SvgDefs::marker(id)`.

Apply it to a shape using `set_marker_start`, `set_marker_mid`, or `set_marker_end`.
The commonly used targets are `<path>`, `<line>`, `<polyline>`, and `<polygon>` (which have been markable elements since SVG 1.1).

SVG 2 also defines marker positions for `<rect>`, `<circle>`, and `<ellipse>` (derived from each shape's equivalent path).

***Caveat***: Verify browser support for those SVG 2 additions on the engines you target.

Marker rendering is not conditional on the shape having visible stroke since markers are a separate painting operation.
`marker-start`/`-mid`/`-end` apply regardless of `stroke`.

The `MarkerUnits` enum controls whether `markerWidth`/`markerHeight` are relative to `strokeWidth` (which is the default, used only as a scale factor for the marker's own size) or user coordinates.

`stroke="none"` does not itself make the marker properties inapplicable.

`set_view_box(x, y, width, height)` establishes an internal coordinate system for the marker's content, mapped onto the `markerWidth`/`markerHeight` viewport — the same `viewBox` relationship `<symbol>`/`<use>` has, validated the same way (`Error::InvalidViewBox` on a non-finite component or a negative `width`/`height`). `preserveAspectRatio` has no dedicated setter for `<marker>`; use `set_attr("preserveAspectRatio", value)`.

---

## `<image>`

`<image>` embeds a raster image (PNG, JPEG, WebP etc) or another SVG into the current document.
Obtain a handle via `SvgRoot::image(href, top_left, size)` or `SvgBatch::image(href, top_left, size)`.

- `href` accepts any URL the browser can fetch: a relative path, an absolute URL, or a `data:` URI.
  When using `data:image/svg+xml`, use base64 encoding to avoid percent-encoding `<`, `>`, and `#`.
- `top_left` and `size` define the display rectangle.

  `svg-dom`'s `image` constructor requires a `Size` and therefore always writes both `width` and `height`; a zero value for either dimension prevents rendering.
  This constraint is applied only by this convenience constructor, it is not actually part of SVG 2 itself.
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

- `href` is normally a local fragment reference such as `"#my-shape"` (the `id` attribute of the target element).
  The crate writes the value unchanged, so same-origin external SVG references such as `"icons.svg#my-shape"` are also representable (a URL containing no fragment references the whole external document), subject to browser security and resource-loading restrictions.
- SVG 2 prohibits cross-origin external `<use>` references.
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

---

## `<a>`

`<a>` is a `<g>`-like wrapper: it has no visual appearance of its own, but turns every child appended to it into a hyperlink.

Obtain a handle via `SvgRoot::anchor(href)` or `SvgBatch::anchor(href)`, then add children with `SvgNode::append`, exactly as with `group`.

- `href` accepts anything a browser can navigate to: a relative path, an absolute URL, or a same-document fragment (`"#section"`).
- `target` (`"_blank"`, `"_self"`, `"_parent"`, `"_top"`, or a named frame — the same vocabulary as HTML `<a target>`) is not wrapped by a named parameter; set it via `set_attr("target", value)`, alongside any other attribute (`download`, `rel`, ...) not covered here.
- `href` is written verbatim; do not pass a `javascript:` URL or other attacker-controlled string without validation.

```rust,no_run
let link = svg.anchor("https://example.com")?;
link.set_attr("target", "_blank")?;

let icon = svg.circle(Point::new(30.0, 30.0), 18.0)?;
let label = svg.text(Point::new(56.0, 35.0), "Learn more")?;
link.append(&icon)?;
link.append(&label)?;
```

***⚠️ Links cannot be nested*** — an `<a>` appended somewhere inside another `<a>` has its own `href` ignored and is inactive, the same as in HTML. `SvgNode::append` does not check for this, so avoid appending the result of one `anchor` call inside another.

***⚠️ The clickable region is each child's own hit region, not the wrapper's bounding box*** — unlike wrapping children in a `<g>` purely for a shared transform, `<a>` does not make the whole rectangular area spanning its children clickable.
Only points within each rendered child's `pointer-events`-defined hit region are clickable — not necessarily identical to its visibly painted pixels, since `fill`, `stroke`, `visibility`, and `pointer-events` itself all influence what that region actually covers; empty space between or around the children does not automatically become part of the link.

---

## `<switch>`

`<switch>` renders at most one of its direct children: the first one, in document order, whose conditional  processing attributes all evaluate to true, rather than rendering every child as `<g>` would.
As per the SVG 2 specification, if none match, it renders **nothing**.
A child with none of those attributes set always passes, so by appending an attribute-free element last (in document order), you create a fallback that guarantees something renders even when every other conditional child fails.

Obtain a handle via `SvgRoot::switch()` or `SvgBatch::switch()`, then add children with `SvgNode::append`, exactly as with `group`.

The conditional attributes themselves — `systemLanguage`, `requiredExtensions` (`requiredFeatures` existed in earlier SVG versions but was removed from SVG 2 because it proved unreliable as a feature-support test) — are not wrapped by named parameters; set them directly on each child via `set_attr`/`set_attrs`.
This crate performs no validation or selection of its own: the browser evaluates each child's test attributes and picks the first match at render time.

```rust,no_run
let switch = svg.switch()?;

let french = svg.text(Point::new(10.0, 30.0), "Bonjour")?;
french.set_attr("systemLanguage", "fr")?;
let german = svg.text(Point::new(10.0, 30.0), "Hallo")?;
german.set_attr("systemLanguage", "de")?;
let fallback = svg.text(Point::new(10.0, 30.0), "Hello")?;

switch.append(&french)?;
switch.append(&german)?;
switch.append(&fallback)?;
```

---

## `<view>`

`<view>` names a `viewBox`/`preserveAspectRatio` combination, activated via a `#viewId` URL fragment. Unlike `<symbol>`, it has no rendered graphical content of its own.

### Fragment navigation: three cases, only two of which apply to this crate

SVG 2 activates `#viewId` fragment navigation only when the SVG resource named by `viewId` is itself *the document being navigated*, not merely any document that happens to contain a matching id. Concretely:

- **A standalone SVG document, navigated directly** (opened in its own tab, or as the top-level document a same-document `#viewId` link/hash change targets): the browser substitutes its effective `viewBox`/`preserveAspectRatio` with this view's own, with no JavaScript needed.
- **An external reference into an exported SVG file** — `<img src="diagram.svg#viewId">`, an SVG `<image>` with that `href`, or a plain hyperlink to `diagram.svg#viewId` — activates the same substitution for that resource.

  [`SvgNode::set_href`](crate::SvgNode::set_href) can re-trigger it on an already-loaded reference by changing the
  fragment.
- **An inline `<svg>` embedded in an HTML page — the case every [`SvgRoot::attach`](crate::SvgRoot::attach)/
  [`SvgRoot::create_in`](crate::SvgRoot::create_in) call in this crate deals with — does *not* qualify.**

  The embedded SVG is not itself the navigated document, so this behaviour never activates for it; a same-page `<a href="#viewId">` click — whether an SVG-native [`SvgRoot::anchor`](crate::SvgRoot::anchor) inside it or a plain HTML link outside it — only updates `location.hash`, with no visible effect.
  Use [`SvgRoot::set_view_box`](crate::SvgRoot::set_view_box)/[`SvgRoot::set_viewport`](crate::SvgRoot::set_viewport) directly instead — the caller already has a live handle, so there is no need to go through a URL fragment.

`<view>` is therefore useful primarily when an SVG document is exported, or embedded/navigated independently of any running WASM code — not for switching the viewport of the very SVG a running WASM instance is attached to.

Obtain a handle via `SvgDefs::view(id)` or the transactional `SvgDefs::build_view(id, closure)`.

| Method | Description |
|---|---|
| `set_view_box(x, y, w, h)` | The `viewBox` this view switches the document to |
| `set_preserve_aspect_ratio(value)` | Alignment/clipping when the navigated-to viewport's aspect ratio differs |
| `set_id(&mut self, id)` | Renames the view (updates both the DOM and the cached id) |
| `set_attr`/`set_attrs` | Generic setter(s) for unlisted attributes |

```rust,no_run
let defs = svg.defs()?;
let detail = defs.view("detail")?;
detail.set_view_box(0.0, 0.0, 50.0, 50.0)?;

// Fragment navigation only activates for an externally referenced (or standalone) copy of this document — here,
// an exported "diagram.svg" loaded through an <image>. Re-setting `href` re-navigates it.
let preview = svg.image("diagram.svg", Point::origin(), Size::new(50.0, 50.0))?;
preview.set_href("diagram.svg#detail")?;
```

`<view>` is therefore useful primarily when an SVG document is exported, or embedded/navigated independently of any running WASM code — not for switching the viewport of the very SVG a running WASM instance is attached to.

View ids follow the same crate-imposed allow-pattern as symbols and markers: `[A-Za-z_][A-Za-z0-9_-]*`. This is narrower than SVG/XML's own id grammar; it is a restriction this crate chooses, not a claim about what SVG itself permits, in exchange for every accepted id being unambiguously safe to embed in a `#id` fragment reference. A non-conforming id causes `Error::InvalidViewId` before any DOM call is made.

Always use `SvgView::set_id` to rename a view after construction; `set_attr("id", ...)` will be rejected with `Error::ReservedAttribute` to protect the cached value.
