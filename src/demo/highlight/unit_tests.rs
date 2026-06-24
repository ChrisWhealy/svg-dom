use super::rust_to_html;

/// Succeeds when `out` contains `needle`, otherwise returns a descriptive error.
fn ensure_contains(out: &str, needle: &str) -> Result<(), String> {
    if out.contains(needle) {
        Ok(())
    } else {
        Err(format!("expected to find {needle:?} in:\n{out}"))
    }
}

/// Succeeds when `out` does **not** contain `needle`, otherwise returns a descriptive error.
fn ensure_absent(out: &str, needle: &str) -> Result<(), String> {
    if out.contains(needle) {
        Err(format!("expected not to find {needle:?} in:\n{out}"))
    } else {
        Ok(())
    }
}

#[test]
fn should_highlight_keyword_and_call() -> Result<(), String> {
    let out = rust_to_html("let x = foo();");
    ensure_contains(&out, r#"<span class="kw">let</span>"#)?;
    ensure_contains(&out, r#"<span class="fnc">foo</span>"#)
}

#[test]
fn should_keep_format_string_braces_in_one_token() -> Result<(), String> {
    // The braces inside the format string must not leak out as separate tokens.
    let out = rust_to_html(r#"format!("box: {nx:.0}")"#);
    ensure_contains(&out, r#"<span class="mac">format!</span>"#)?;
    ensure_contains(&out, r#"<span class="str">"box: {nx:.0}"</span>"#)
}

#[test]
fn should_not_treat_lifetime_as_string() -> Result<(), String> {
    // The leading quote of `'static` must be treated as a lifetime, not an unterminated string literal.
    let out = rust_to_html("impl Fn(E) + 'static {}");
    ensure_contains(&out, r#"<span class="kw">impl</span>"#)?;
    ensure_contains(&out, r#"<span class="lif">'static</span>"#)?;
    ensure_absent(&out, r#"class="str""#)
}

#[test]
fn should_highlight_char_literal_as_string() -> Result<(), String> {
    let out = rust_to_html("let c = 'a';");
    ensure_contains(&out, r#"<span class="str">'a'</span>"#)
}

#[test]
fn should_highlight_types_numbers_and_comments() -> Result<(), String> {
    let out = rust_to_html("let p = Point::new(10.0, 2); // make a point");
    ensure_contains(&out, r#"<span class="typ">Point</span>"#)?;
    ensure_contains(&out, r#"<span class="num">10.0</span>"#)?;
    ensure_contains(&out, r#"<span class="num">2</span>"#)?;
    ensure_contains(&out, r#"<span class="com">// make a point</span>"#)
}

#[test]
fn should_escape_html_special_chars() -> Result<(), String> {
    let out = rust_to_html("fn f() -> Result<(), Error> { a & b }");
    ensure_contains(&out, "Result")?;
    ensure_contains(&out, "&lt;")?;
    ensure_contains(&out, "&gt;")?;
    ensure_contains(&out, "&amp;")?;
    // The raw angle brackets must never appear unescaped in the output.
    ensure_absent(&out, "<(), ")
}

#[test]
fn should_not_swallow_range_into_number() -> Result<(), String> {
    let out = rust_to_html("for i in 0..5 {}");
    ensure_contains(&out, r#"<span class="num">0</span>"#)?;
    ensure_contains(&out, r#"<span class="num">5</span>"#)?;
    // The `..` must remain between the two numbers, not be eaten as part of `0`.
    ensure_contains(&out, "</span>..<span")
}
