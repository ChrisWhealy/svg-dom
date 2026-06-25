//! A small, Rust-only syntax highlighter for the demo source frames.
//!
//! The demo gallery only ever displays Rust, so this is deliberately a stripped-down lexer rather than a general-purpose
//! highlighter: it recognises comments, strings, char literals, lifetimes, numbers, keywords, type-ish identifiers,
//! macros and call-position identifiers, and passes everything else through verbatim. The output is an HTML string of
//! `<span class="…">` tokens (with `&`, `<`, `>` escaped) suitable for `Element::set_inner_html`; the colours live in
//! `demo/style.css`.
//!
//! It is not a full Rust parser and does not need to be — it only has to make the embedded demo functions readable. Raw
//! strings (`r"…"`) are not special-cased because the demo source does not use them.

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Converts a Rust source string into highlighted HTML (`<span class="…">` tokens, HTML-escaped).
pub fn rust_to_html(src: &str) -> String {
    let b = src.as_bytes();
    let n = b.len();
    let mut out = String::with_capacity(src.len() * 2);
    let mut i = 0;

    while i < n {
        let c = b[i];

        // Line comment: // … to end of line.
        if c == b'/' && i + 1 < n && b[i + 1] == b'/' {
            let start = i;
            while i < n && b[i] != b'\n' {
                i += 1;
            }
            span(&mut out, "com", &src[start..i]);
            continue;
        }

        // Block comment: /* … */ (not nested, which is enough for the demo source).
        if c == b'/' && i + 1 < n && b[i + 1] == b'*' {
            let start = i;
            i += 2;
            while i < n && !(b[i] == b'*' && i + 1 < n && b[i + 1] == b'/') {
                i += 1;
            }
            i = (i + 2).min(n);
            span(&mut out, "com", &src[start..i]);
            continue;
        }

        // String literal: "…" with backslash escapes.
        if c == b'"' {
            let start = i;
            i += 1;
            while i < n {
                match b[i] {
                    b'\\' => i += 2,
                    b'"' => {
                        i += 1;
                        break;
                    },
                    _ => i += 1,
                }
            }
            span(&mut out, "str", &src[start..i.min(n)]);
            continue;
        }

        // Single quote: either a char literal ('a', '\n') or a lifetime ('static, '_).
        if c == b'\'' {
            if let Some(len) = char_literal_len(b, i, n) {
                span(&mut out, "str", &src[i..i + len]);
                i += len;
            } else {
                let start = i;
                i += 1;
                while i < n && is_ident_continue(b[i]) {
                    i += 1;
                }
                span(&mut out, "lif", &src[start..i]);
            }
            continue;
        }

        // Number literal: digits, with an optional fractional part / suffix / separators. Stops before a `..` range.
        if c.is_ascii_digit() {
            let start = i;
            while i < n && (b[i].is_ascii_alphanumeric() || b[i] == b'_' || b[i] == b'.') {
                if b[i] == b'.' && i + 1 < n && b[i + 1] == b'.' {
                    break;
                }
                i += 1;
            }
            span(&mut out, "num", &src[start..i]);
            continue;
        }

        // Identifier-like run: keyword / type / macro / call / plain.
        if is_ident_start(c) {
            let start = i;
            i += 1;
            while i < n && is_ident_continue(b[i]) {
                i += 1;
            }
            let word = &src[start..i];

            if i < n && b[i] == b'!' {
                // `name!` — a macro invocation; pull the `!` into the same span.
                i += 1;
                span(&mut out, "mac", &src[start..i]);
            } else if is_keyword(word) {
                span(&mut out, "kw", word);
            } else if word.as_bytes()[0].is_ascii_uppercase() {
                span(&mut out, "typ", word);
            } else if i < n && b[i] == b'(' {
                span(&mut out, "fnc", word);
            } else {
                escape_into(&mut out, word);
            }
            continue;
        }

        // Anything else (whitespace, punctuation, operators, non-ASCII in code) — pass one UTF-8
        // character through, escaped. The boundary guard keeps the lexer panic-free even if an
        // earlier branch (e.g. a string escape) advanced `i` into the middle of a codepoint.
        if !src.is_char_boundary(i) {
            i += 1;
            continue;
        }
        match src[i..].chars().next() {
            Some(ch) => {
                let len = ch.len_utf8();
                escape_into(&mut out, &src[i..i + len]);
                i += len;
            },
            None => break,
        }
    }

    out
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Emits `<span class="{class}">{escaped text}</span>`.
fn span(out: &mut String, class: &str, text: &str) {
    out.push_str("<span class=\"");
    out.push_str(class);
    out.push_str("\">");
    escape_into(out, text);
    out.push_str("</span>");
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Appends `text` with the three HTML-significant characters escaped.
fn escape_into(out: &mut String, text: &str) {
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(c),
        }
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Returns the byte length of a char literal starting at `i` (where `b[i] == '\''`), or `None` if this quote begins a
/// lifetime instead. Distinguishing the two is what stops `'static` being mistaken for an unterminated string.
fn char_literal_len(b: &[u8], i: usize, n: usize) -> Option<usize> {
    if i + 1 >= n {
        return None;
    }
    if b[i + 1] == b'\\' {
        // Escaped char: '\n', '\'', '\\', … — scan to the closing quote within a short bound.
        let mut j = i + 2;
        while j < n && j < i + 8 && b[j] != b'\'' {
            j += 1;
        }
        return (j < n && b[j] == b'\'').then_some(j - i + 1);
    }
    // Unescaped: a single ASCII char closed by a quote, e.g. 'a'. A non-quote at i+2 means this was a lifetime.
    if b[i + 1] == b'\'' || b[i + 1] == b'\n' {
        return None;
    }
    (i + 2 < n && b[i + 2] == b'\'').then_some(3)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn is_ident_start(b: u8) -> bool {
    b == b'_' || b.is_ascii_alphabetic()
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn is_ident_continue(b: u8) -> bool {
    b == b'_' || b.is_ascii_alphanumeric()
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn is_keyword(word: &str) -> bool {
    matches!(
        word,
        "as" | "async"
            | "await"
            | "break"
            | "const"
            | "continue"
            | "crate"
            | "dyn"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
    )
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[cfg(test)]
mod unit_tests;
