use crate::common::*;
use svg_dom::{BlendMode, Channel, ColorMatrixType, CompositeOperator, EdgeMode, MorphologyOperator, TurbulenceType};
use wasm_bindgen_test::*;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// blend + flood + composite — tint chain
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `blend` composes with `flood` and a final `composite(In)` into a working, alpha-preserving tint filter: flood a
/// colour, blend it over the source graphic, then clip the result back to the source's own alpha coverage — the
/// example from `SvgFilter::blend`'s doc comment. The final `composite` step is not cosmetic: without it, `flood`'s
/// opaque colour would leak through wherever the source graphic was itself transparent, since `blend` alone only
/// changes colour, not alpha.
#[wasm_bindgen_test]
fn should_build_tint_filter_chain() -> Result<(), String> {
    let svg = make_svg("filter-blend-tint-chain");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs
        .build_filter("tint", |f| {
            f.flood("steelblue", 1.0)?.set_attr("result", "colour")?;
            f.blend("colour", BlendMode::Multiply)?
                .set_attrs([("in", "SourceGraphic"), ("result", "tinted")])?;
            f.composite("SourceGraphic", CompositeOperator::In)?.set_attr("in", "tinted")?;
            Ok(())
        })
        .map_err(|e| e.to_string())?;
    check_eq(filter.as_element().child_element_count(), 3)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// A full blur + offset + merge drop-shadow chain
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// The three primitives compose into a working drop-shadow filter: blur the source alpha, offset it, then merge
/// it underneath the original graphic.
#[wasm_bindgen_test]
fn should_build_drop_shadow_filter_chain() -> Result<(), String> {
    let svg = make_svg("filter-drop-shadow");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs
        .build_filter("shadow", |f| {
            f.gaussian_blur(4.0)?.set_attrs([("in", "SourceAlpha"), ("result", "blur")])?;
            f.offset(4.0, 4.0)?.set_attrs([("in", "blur"), ("result", "offset-blur")])?;
            f.merge(&["offset-blur", "SourceGraphic"])?;
            Ok(())
        })
        .map_err(|e| e.to_string())?;
    check_eq(filter.as_element().child_element_count(), 3)
}

/// Adding `flood` + `composite` composes into a working *tinted* drop-shadow filter: blur the source alpha,
/// composite a flood colour into the blurred mask, offset it, then merge it underneath the original graphic.
#[wasm_bindgen_test]
fn should_build_tinted_drop_shadow_filter_chain() -> Result<(), String> {
    let svg = make_svg("filter-tinted-drop-shadow");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs
        .build_filter("tinted-shadow", |f| {
            f.gaussian_blur(4.0)?.set_attrs([("in", "SourceAlpha"), ("result", "blur")])?;
            f.flood("black", 0.5)?.set_attr("result", "colour")?;
            f.composite("blur", CompositeOperator::In)?
                .set_attrs([("in", "colour"), ("result", "tinted")])?;
            f.offset(4.0, 4.0)?.set_attrs([("in", "tinted"), ("result", "offset-shadow")])?;
            f.merge(&["offset-shadow", "SourceGraphic"])?;
            Ok(())
        })
        .map_err(|e| e.to_string())?;
    check_eq(filter.as_element().child_element_count(), 5)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// turbulence + displacement_map — organic-edge distortion chain
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `turbulence` composes with `displacement_map` into a well-formed noise-displacement filter: generate fractal
/// noise, name it via `result`, then warp `SourceGraphic` using that noise as the displacement field — the standard
/// `feTurbulence` + `feDisplacementMap` pairing, and the example from `SvgFilter::displacement_map`'s doc comment.
/// Uses `Channel::Red`/`Channel::Green` (rather than `Alpha`/`Alpha`) so the displacement is free to point in any
/// direction: passing the same channel for both selectors would compute `dx`/`dy` from the identical value at
/// every pixel, confining every displacement vector to the `y = x` diagonal.
///
/// This only proves the DOM is assembled correctly (the right elements, wired together with the right
/// attributes) — it says nothing about the rendered pixels. These tests do not capture or inspect rendered
/// output; see `displacement_map`'s own doc comment for the spec's interoperability caveat around what those
/// pixels actually look like.
#[wasm_bindgen_test]
fn should_build_turbulence_displacement_chain() -> Result<(), String> {
    let svg = make_svg("filter-turbulence-displacement-chain");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs
        .build_filter("organic-edge", |f| {
            f.turbulence(0.02, 3, 5.0, TurbulenceType::FractalNoise)?
                .set_attr("result", "noise")?;
            f.displacement_map("noise", 24.0, Channel::Red, Channel::Green)?
                .set_attr("in", "SourceGraphic")?;
            Ok(())
        })
        .map_err(|e| e.to_string())?;
    let el = filter.as_element();
    check_eq(el.child_element_count(), 2)?;
    let turb = el.first_element_child().ok_or("expected a <feTurbulence> child")?;
    let disp = turb.next_element_sibling().ok_or("expected a <feDisplacementMap> sibling")?;
    check_eq(turb.tag_name(), "feTurbulence".to_owned())?;
    check_eq(turb.get_attribute("result"), Some("noise".into()))?;
    check_eq(disp.tag_name(), "feDisplacementMap".to_owned())?;
    check_eq(disp.get_attribute("in"), Some("SourceGraphic".into()))?;
    check_eq(disp.get_attribute("in2"), Some("noise".into()))?;
    check_eq(disp.get_attribute("xChannelSelector"), Some("R".into()))?;
    check_eq(disp.get_attribute("yChannelSelector"), Some("G".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// morphology + merge — bold-outline chain
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `morphology` composes with `merge` into a well-formed bold-outline filter: dilate the source alpha, then merge
/// it underneath the original graphic so only the grown-outward fringe shows — the example from
/// `SvgFilter::morphology`'s doc comment. As with the turbulence+displacement_map chain above, this only proves
/// the DOM is assembled correctly; it does not capture or inspect rendered output.
#[wasm_bindgen_test]
fn should_build_bold_outline_filter_chain() -> Result<(), String> {
    let svg = make_svg("filter-morphology-outline-chain");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs
        .build_filter("bold-outline", |f| {
            f.morphology(2.5, MorphologyOperator::Dilate)?
                .set_attrs([("in", "SourceAlpha"), ("result", "thickened")])?;
            f.merge(&["thickened", "SourceGraphic"])?;
            Ok(())
        })
        .map_err(|e| e.to_string())?;
    let el = filter.as_element();
    check_eq(el.child_element_count(), 2)?;
    let morph = el.first_element_child().ok_or("expected a <feMorphology> child")?;
    let merge = morph.next_element_sibling().ok_or("expected a <feMerge> sibling")?;
    check_eq(morph.tag_name(), "feMorphology".to_owned())?;
    check_eq(morph.get_attribute("in"), Some("SourceAlpha".into()))?;
    check_eq(morph.get_attribute("result"), Some("thickened".into()))?;
    check_eq(merge.tag_name(), "feMerge".to_owned())?;
    check_eq(merge.child_element_count(), 2)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// image + color_matrix — filtered-image chain
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `image` composes with `color_matrix` into a well-formed filtered-image chain: import external image content
/// via `href`, then greyscale it — the example from `SvgFilter::image`'s doc comment. Since `image` reads no `in`
/// at all, `color_matrix`'s implicit input (being the filter's second primitive) is `image`'s own output, not
/// `SourceGraphic`. A filtered plain `<image>` element could be greyscaled the same way (it becomes `SourceGraphic`
/// on its own); what this chain actually demonstrates is only that `image`'s output composes with a later primitive
/// like any other primitive's output does.
#[wasm_bindgen_test]
fn should_build_filtered_image_chain() -> Result<(), String> {
    let svg = make_svg("filter-image-color-matrix-chain");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs
        .build_filter("greyscale-image", |f| {
            f.image("photo.jpg")?;
            f.color_matrix(ColorMatrixType::Saturate(0.0))?;
            Ok(())
        })
        .map_err(|e| e.to_string())?;
    let el = filter.as_element();
    check_eq(el.child_element_count(), 2)?;
    let img = el.first_element_child().ok_or("expected a <feImage> child")?;
    let cm = img.next_element_sibling().ok_or("expected a <feColorMatrix> sibling")?;
    check_eq(img.tag_name(), "feImage".to_owned())?;
    check_eq(img.get_attribute("href"), Some("photo.jpg".into()))?;
    check_eq(cm.tag_name(), "feColorMatrix".to_owned())?;
    check_eq(cm.get_attribute("type"), Some("saturate".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// image + composite + blend — texture-on-object chain
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `image` composes with `composite` and `blend` into the chain that actually distinguishes `<feImage>` from
/// filtering a plain `<image>` element: `composite(SourceAlpha, In)` clips the imported texture to the filtered
/// element's own silhouette, then `blend(SourceGraphic, Multiply)` composes it back over the element's own fill.
/// Unlike `should_build_filtered_image_chain` above, this reads the filtered element's own `SourceGraphic`/
/// `SourceAlpha` — something a filtered plain `<image>` cannot do, since it has no second, independent source to
/// combine with.
#[wasm_bindgen_test]
fn should_build_textured_object_chain() -> Result<(), String> {
    let svg = make_svg("filter-image-composite-blend-chain");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs
        .build_filter("textured-object", |f| {
            f.image("texture.jpg")?.set_attr("result", "texture")?;
            f.composite("SourceAlpha", CompositeOperator::In)?
                .set_attrs([("in", "texture"), ("result", "clipped-texture")])?;
            f.blend("clipped-texture", BlendMode::Multiply)?
                .set_attr("in", "SourceGraphic")?;
            Ok(())
        })
        .map_err(|e| e.to_string())?;
    let el = filter.as_element();
    check_eq(el.child_element_count(), 3)?;
    let img = el.first_element_child().ok_or("expected a <feImage> child")?;
    let comp = img.next_element_sibling().ok_or("expected a <feComposite> sibling")?;
    let blend = comp.next_element_sibling().ok_or("expected a <feBlend> sibling")?;
    check_eq(img.tag_name(), "feImage".to_owned())?;
    check_eq(img.get_attribute("result"), Some("texture".into()))?;
    check_eq(comp.tag_name(), "feComposite".to_owned())?;
    check_eq(comp.get_attribute("in"), Some("texture".into()))?;
    check_eq(comp.get_attribute("in2"), Some("SourceAlpha".into()))?;
    check_eq(comp.get_attribute("operator"), Some("in".into()))?;
    check_eq(comp.get_attribute("result"), Some("clipped-texture".into()))?;
    check_eq(blend.tag_name(), "feBlend".to_owned())?;
    check_eq(blend.get_attribute("in"), Some("SourceGraphic".into()))?;
    check_eq(blend.get_attribute("in2"), Some("clipped-texture".into()))?;
    check_eq(blend.get_attribute("mode"), Some("multiply".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// turbulence + tile — tiled-noise chain
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `turbulence` composes with `tile` into a well-formed tiled-noise filter: generate noise, narrow it to a small
/// subregion, then repeat that subregion across the whole filter region — the example from `SvgFilter::tile`'s doc
/// comment. Narrowing the turbulence primitive's own `x`/`y`/`width`/`height` (rather than leaving them at the
/// default full-filter-region subregion) is not cosmetic: without it, `tile` would have nothing smaller than the
/// full region to repeat, and its output would be indistinguishable from the noise passed through unchanged.
#[wasm_bindgen_test]
fn should_build_tiled_noise_chain() -> Result<(), String> {
    let svg = make_svg("filter-turbulence-tile-chain");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs
        .build_filter("tiled-noise", |f| {
            f.turbulence(0.2, 2, 4.0, TurbulenceType::FractalNoise)?.set_attrs([
                ("x", "0"),
                ("y", "0"),
                ("width", "20"),
                ("height", "20"),
            ])?;
            f.tile()?;
            Ok(())
        })
        .map_err(|e| e.to_string())?;
    let el = filter.as_element();
    check_eq(el.child_element_count(), 2)?;
    let turb = el.first_element_child().ok_or("expected a <feTurbulence> child")?;
    let tile = turb.next_element_sibling().ok_or("expected a <feTile> sibling")?;
    check_eq(turb.tag_name(), "feTurbulence".to_owned())?;
    check_eq(turb.get_attribute("x"), Some("0".into()))?;
    check_eq(turb.get_attribute("y"), Some("0".into()))?;
    check_eq(turb.get_attribute("width"), Some("20".into()))?;
    check_eq(turb.get_attribute("height"), Some("20".into()))?;
    check_eq(tile.tag_name(), "feTile".to_owned())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// convolve_matrix + color_matrix — desaturated emboss chain
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `convolve_matrix` composes with `color_matrix` into a well-formed classic-emboss filter: convolve with a
/// zero-sum emboss kernel (`bias: 0.5`, set via the generic escape hatch, shifts flat regions to mid-grey instead of
/// clamping to black), then desaturate the result — the standard finishing touch for an embossed look, since the
/// raw convolution output still carries the source's own hue. As with the other chains in this file, this only
/// proves the DOM is assembled correctly; it does not capture or inspect rendered output.
#[wasm_bindgen_test]
fn should_build_desaturated_emboss_chain() -> Result<(), String> {
    let svg = make_svg("filter-convolve-matrix-emboss-chain");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    #[rustfmt::skip]
    let kernel = [
        -2.0, -1.0, 0.0,
        -1.0,  1.0, 1.0,
         0.0,  1.0, 2.0,
    ];
    let filter = defs
        .build_filter("emboss", |f| {
            f.convolve_matrix(3, &kernel, 1.0, EdgeMode::Duplicate, true)?
                .set_attrs([("bias", "0.5"), ("result", "embossed")])?;
            f.color_matrix(ColorMatrixType::Saturate(0.0))?.set_attr("in", "embossed")?;
            Ok(())
        })
        .map_err(|e| e.to_string())?;
    let el = filter.as_element();
    check_eq(el.child_element_count(), 2)?;
    let conv = el.first_element_child().ok_or("expected a <feConvolveMatrix> child")?;
    let matrix = conv.next_element_sibling().ok_or("expected a <feColorMatrix> sibling")?;
    check_eq(conv.tag_name(), "feConvolveMatrix".to_owned())?;
    check_eq(conv.get_attribute("bias"), Some("0.5".into()))?;
    check_eq(conv.get_attribute("result"), Some("embossed".into()))?;
    check_eq(matrix.tag_name(), "feColorMatrix".to_owned())?;
    check_eq(matrix.get_attribute("in"), Some("embossed".into()))?;
    check_eq(matrix.get_attribute("type"), Some("saturate".into()))
}
