use crate::common::*;
use svg_dom::{BlendMode, Channel, CompositeOperator, TurbulenceType};
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
/// attributes) — it says nothing about the rendered pixels. `wasm-bindgen-test`'s WebDriver-run tests have no
/// access to rendered output; see `displacement_map`'s own doc comment for the spec's interoperability caveat
/// around what those pixels actually look like.
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
