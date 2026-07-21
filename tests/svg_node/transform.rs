use crate::{common, helpers::make_svg};
use svg_dom::root::utils::Matrix2D;
use wasm_bindgen_test::*;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// transform helpers (set_translate / set_rotate / set_scale / ...)
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `set_translate` writes a `translate(x, y)` transform formatted to one decimal place.
#[wasm_bindgen_test]
fn should_write_translate_transform() -> Result<(), String> {
    let node = make_svg("node-set-translate").group().map_err(|e| e.to_string())?;
    let mut buf = String::new();
    node.set_translate(&mut buf, 100.0, 50.0).map_err(|e| e.to_string())?;
    common::check_eq(node.attr("transform"), Some("translate(100.0, 50.0)".into()))
}

/// The same scratch buffer can be reused across calls and the latest value wins.
#[wasm_bindgen_test]
fn should_reuse_scratch_buffer_across_translate_calls() -> Result<(), String> {
    let node = make_svg("node-translate-reuse").group().map_err(|e| e.to_string())?;
    let mut buf = String::new();
    node.set_translate(&mut buf, 1.0, 2.0).map_err(|e| e.to_string())?;
    node.set_translate(&mut buf, 33.0, 44.0).map_err(|e| e.to_string())?;
    common::check_eq(node.attr("transform"), Some("translate(33.0, 44.0)".into()))
}

/// `set_rotate` writes a single-argument `rotate(angle)` transform.
#[wasm_bindgen_test]
fn should_write_rotate_transform() -> Result<(), String> {
    let node = make_svg("node-set-rotate").group().map_err(|e| e.to_string())?;
    let mut buf = String::new();
    node.set_rotate(&mut buf, 45.0).map_err(|e| e.to_string())?;
    common::check_eq(node.attr("transform"), Some("rotate(45.0)".into()))
}

/// `set_rotate_about` writes a `rotate(angle, cx, cy)` transform.
#[wasm_bindgen_test]
fn should_write_rotate_about_transform() -> Result<(), String> {
    let node = make_svg("node-set-rotate-about").group().map_err(|e| e.to_string())?;
    let mut buf = String::new();
    node.set_rotate_about(&mut buf, 90.0, 10.0, 20.0).map_err(|e| e.to_string())?;
    common::check_eq(node.attr("transform"), Some("rotate(90.0, 10.0, 20.0)".into()))
}

/// `set_scale` writes a uniform `scale(s)` transform formatted to three decimal places.
#[wasm_bindgen_test]
fn should_write_uniform_scale_transform() -> Result<(), String> {
    let node = make_svg("node-set-scale").group().map_err(|e| e.to_string())?;
    let mut buf = String::new();
    node.set_scale(&mut buf, 1.5).map_err(|e| e.to_string())?;
    common::check_eq(node.attr("transform"), Some("scale(1.500)".into()))
}

/// `set_scale_xy` writes a non-uniform `scale(x, y)` transform.
#[wasm_bindgen_test]
fn should_write_non_uniform_scale_transform() -> Result<(), String> {
    let node = make_svg("node-set-scale-xy").group().map_err(|e| e.to_string())?;
    let mut buf = String::new();
    node.set_scale_xy(&mut buf, 2.0, 0.5).map_err(|e| e.to_string())?;
    common::check_eq(node.attr("transform"), Some("scale(2.000, 0.500)".into()))
}

/// `set_translate_scale` writes the combined `translate(...) scale(...)` shape used by pan/zoom code.
#[wasm_bindgen_test]
fn should_write_translate_scale_transform() -> Result<(), String> {
    let node = make_svg("node-set-translate-scale").group().map_err(|e| e.to_string())?;
    let mut buf = String::new();
    node.set_translate_scale(&mut buf, 12.0, 34.0, 2.0).map_err(|e| e.to_string())?;
    common::check_eq(node.attr("transform"), Some("translate(12.0, 34.0) scale(2.000)".into()))
}

/// `set_matrix` writes a `matrix(a, b, c, d, e, f)` transform from `Matrix2D`'s named fields, mapped to the SVG
/// function's own `a, b, c, d, e, f` order (`h_scale`→`a`, `v_skew`→`b`, `h_skew`→`c`, `v_scale`→`d`, `h_trans`→`e`,
/// `v_trans`→`f`), with the linear part at three decimal places and the translation part at one, matching
/// `set_scale`'s and `set_translate`'s precision respectively.
///
/// All six fields are given distinct values (including a negative one) specifically so that any two fields being
/// swapped — `h_scale`/`v_scale`, or `h_skew`/`v_skew`, the two pairs most likely to be transposed by mistake since
/// each pair shares a magnitude in the common case of a plain rotation — would produce a different, wrong output
/// rather than silently passing. A fixture using `1.0`/`0.0`-heavy values (as an earlier version of this test did)
/// cannot tell such a swap apart from a correct mapping.
#[wasm_bindgen_test]
fn should_write_matrix_transform() -> Result<(), String> {
    let node = make_svg("node-set-matrix").group().map_err(|e| e.to_string())?;
    let mut buf = String::new();
    node.set_matrix(
        &mut buf,
        Matrix2D {
            h_scale: 1.1,
            v_scale: 2.2,
            h_skew: 3.3,
            v_skew: 4.4,
            h_trans: 5.5,
            v_trans: -6.6,
        },
    )
    .map_err(|e| e.to_string())?;
    common::check_eq(
        node.attr("transform"),
        Some("matrix(1.100, 4.400, 3.300, 2.200, 5.5, -6.6)".into()),
    )
}

/// `set_matrix` on a genuine rotation matrix — `h_scale`/`v_scale` both `cos(θ)`, `v_skew` = `+sin(θ)`, `h_skew` =
/// `-sin(θ)` — checks the mapping [`should_write_matrix_transform`] cannot: swapping `v_skew` and `h_skew` (or
/// dropping either sign) is invisible to a fixture built from six arbitrary distinct numbers, since nothing there
/// constrains two fields to be exact negatives of one another. A real rotation does, and is also the shape every
/// realistic caller of `set_matrix` actually produces.
#[wasm_bindgen_test]
fn should_write_matrix_transform_for_a_rotation() -> Result<(), String> {
    let node = make_svg("node-set-matrix-rotation").group().map_err(|e| e.to_string())?;
    let mut buf = String::new();
    let angle: f64 = 30.0_f64.to_radians();
    node.set_matrix(
        &mut buf,
        Matrix2D {
            h_scale: angle.cos(),
            v_skew: angle.sin(),
            h_skew: -angle.sin(),
            v_scale: angle.cos(),
            h_trans: 10.0,
            v_trans: 20.0,
        },
    )
    .map_err(|e| e.to_string())?;
    common::check_eq(
        node.attr("transform"),
        Some("matrix(0.866, 0.500, -0.500, 0.866, 10.0, 20.0)".into()),
    )
}

/// `set_matrix_precise` writes a `matrix(a, b, c, d, e, f)` transform using shortest round-trip `Display`
/// formatting for every field, rather than `set_matrix`'s fixed three/one decimal places.
#[wasm_bindgen_test]
fn should_write_matrix_precise_transform() -> Result<(), String> {
    let node = make_svg("node-set-matrix-precise").group().map_err(|e| e.to_string())?;
    let mut buf = String::new();
    node.set_matrix_precise(
        &mut buf,
        Matrix2D {
            h_scale: 1.23456,
            v_scale: 2.0,
            h_skew: 0.1,
            v_skew: 0.2,
            h_trans: 12.345,
            v_trans: 6.0,
        },
    )
    .map_err(|e| e.to_string())?;
    common::check_eq(node.attr("transform"), Some("matrix(1.23456, 0.2, 0.1, 2, 12.345, 6)".into()))
}

/// `set_matrix`'s fixed three-decimal-place precision rounds a small rotation's sine term to zero, losing the
/// rotation entirely; `set_matrix_precise` preserves it. This is the specific failure mode both methods' doc
/// comments warn about.
#[wasm_bindgen_test]
fn should_preserve_tiny_rotation_only_via_matrix_precise() -> Result<(), String> {
    let node = make_svg("node-set-matrix-tiny-rotation").group().map_err(|e| e.to_string())?;
    let mut buf = String::new();

    // The coefficients of a 0.01-degree rotation matrix (cos(0.01deg), sin(0.01deg)), given as literal constants
    // rather than computed via f64::sin/f64::cos here. Rust documents those functions' precision as
    // non-deterministic (it can vary by platform, libm, or Rust version), so calling them in the test itself would
    // make a mathematically insignificant one-ULP difference capable of changing the shortest-round-trip string
    // set_matrix_precise produces and failing the test despite set_matrix_precise remaining entirely correct. Fixed
    // literals make this a test of serialisation only, not of the platform's trig implementation.
    let m = Matrix2D {
        h_scale: 0.999_999_984_769_129_1,
        v_skew: 0.000_174_532_924_313_336_8,
        h_skew: -0.000_174_532_924_313_336_8,
        v_scale: 0.999_999_984_769_129_1,
        h_trans: 0.0,
        v_trans: 0.0,
    };

    node.set_matrix(&mut buf, m).map_err(|e| e.to_string())?;
    // The rotation has vanished: this is indistinguishable from the identity matrix.
    common::check_eq(
        node.attr("transform"),
        Some("matrix(1.000, 0.000, -0.000, 1.000, 0.0, 0.0)".into()),
    )?;

    node.set_matrix_precise(&mut buf, m).map_err(|e| e.to_string())?;
    // The same rotation, preserved: the sine terms are visibly non-zero.
    common::check_eq(
        node.attr("transform"),
        Some(
            "matrix(0.9999999847691291, 0.0001745329243133368, -0.0001745329243133368, \
             0.9999999847691291, 0, 0)"
                .into(),
        ),
    )
}

/// `set_transform_fmt` writes an arbitrary transform built from `format_args!`.
#[wasm_bindgen_test]
fn should_write_arbitrary_transform_via_fmt() -> Result<(), String> {
    let node = make_svg("node-set-transform-fmt").group().map_err(|e| e.to_string())?;
    let mut buf = String::new();
    let (x, y, angle) = (10.0, 20.0, 45.0);
    node.set_transform_fmt(&mut buf, format_args!("translate({x:.1}, {y:.1}) rotate({angle:.1})"))
        .map_err(|e| e.to_string())?;
    common::check_eq(node.attr("transform"), Some("translate(10.0, 20.0) rotate(45.0)".into()))
}
