//! Tiny self-contained Rust tokenizer used by the showcase docs and
//! tutorial pages to render syntax-highlighted code. Pure Rust — no
//! `web_sys`, no DOM — so the tokenizer can be unit-tested on the host
//! without spinning up a WASM runtime.
//!
//! The grammar is intentionally small: it covers the cases that
//! actually show up in our demo snippets (keywords, strings/chars,
//! line + block comments, numbers, macro invocations, function-call
//! sites, type-like identifiers, attribute headers). Anything more
//! exotic falls through as plain text so a parser miss never produces
//! garbage output — at worst it under-colours.

const KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn",
    "else", "enum", "extern", "false", "fn", "for", "if", "impl", "in",
    "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
    "self", "Self", "static", "struct", "super", "trait", "true", "type",
    "unsafe", "use", "where", "while", "box", "do", "macro", "union",
    "yield",
];

const PRIMITIVE_TYPES: &[&str] = &[
    "bool", "char", "str", "String", "i8", "i16", "i32", "i64", "i128",
    "isize", "u8", "u16", "u32", "u64", "u128", "usize", "f32", "f64",
    "Option", "Result", "Vec", "Box", "Rc", "Arc", "Cell", "RefCell",
    "HashMap", "BTreeMap", "HashSet", "BTreeSet",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tok {
    Plain,
    Keyword,
    PrimType,
    UserType,
    Str,
    Char,
    Num,
    Comment,
    Macro,
    FnCall,
    Attr,
    Punct,
    LifeTime,
}

impl Tok {
    /// CSS class name to apply for this token kind (empty for plain text).
    pub fn css_class(self) -> &'static str {
        match self {
            Tok::Plain => "",
            Tok::Keyword => "hl-kw",
            Tok::PrimType => "hl-pty",
            Tok::UserType => "hl-ty",
            Tok::Str | Tok::Char => "hl-str",
            Tok::Num => "hl-num",
            Tok::Comment => "hl-com",
            Tok::Macro => "hl-mac",
            Tok::FnCall => "hl-fn",
            Tok::Attr => "hl-attr",
            Tok::Punct => "hl-punct",
            Tok::LifeTime => "hl-life",
        }
    }
}

/// Tokenize Rust source into `(text, kind)` runs. Lossless: the
/// concatenation of every run's text equals the input.
pub fn tokenize(src: &str) -> Vec<(String, Tok)> {
    let bytes = src.as_bytes();
    let mut out: Vec<(String, Tok)> = Vec::new();
    let mut i = 0;

    fn push(out: &mut Vec<(String, Tok)>, s: &str, t: Tok) {
        if s.is_empty() {
            return;
        }
        if let Some(last) = out.last_mut() {
            if last.1 == t {
                last.0.push_str(s);
                return;
            }
        }
        out.push((s.to_string(), t));
    }

    while i < bytes.len() {
        let c = bytes[i] as char;

        // Line comment.
        if c == '/' && bytes.get(i + 1).map(|b| *b as char) == Some('/') {
            let start = i;
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            push(&mut out, &src[start..i], Tok::Comment);
            continue;
        }

        // Block comment (non-nested — uncommon in displayed snippets).
        if c == '/' && bytes.get(i + 1).map(|b| *b as char) == Some('*') {
            let start = i;
            i += 2;
            while i + 1 < bytes.len()
                && !(bytes[i] == b'*' && bytes[i + 1] == b'/')
            {
                i += 1;
            }
            i = (i + 2).min(bytes.len());
            push(&mut out, &src[start..i], Tok::Comment);
            continue;
        }

        // String literal — regular + raw r"..." / r#"..."#.
        if c == '"'
            || (c == 'r'
                && matches!(bytes.get(i + 1), Some(&b'"') | Some(&b'#')))
        {
            let start = i;
            let mut hashes = 0usize;
            if c == 'r' {
                i += 1;
                while bytes.get(i) == Some(&b'#') {
                    hashes += 1;
                    i += 1;
                }
            }
            if bytes.get(i) == Some(&b'"') {
                i += 1;
                loop {
                    if i >= bytes.len() {
                        break;
                    }
                    if bytes[i] == b'\\' && hashes == 0 {
                        i = (i + 2).min(bytes.len());
                        continue;
                    }
                    if bytes[i] == b'"' {
                        i += 1;
                        let mut closing = 0usize;
                        while closing < hashes && bytes.get(i) == Some(&b'#') {
                            closing += 1;
                            i += 1;
                        }
                        if closing == hashes {
                            break;
                        }
                        continue;
                    }
                    i += 1;
                }
            }
            push(&mut out, &src[start..i], Tok::Str);
            continue;
        }

        // Char literal — also doubles as the source of confusion for
        // lifetimes (`'a`, `'static`).
        if c == '\'' {
            let start = i;
            i += 1;
            if i < bytes.len() && bytes[i] == b'\\' {
                i = (i + 2).min(bytes.len());
                while i < bytes.len() && bytes[i] != b'\'' && bytes[i] != b'\n' {
                    i += 1;
                }
                if i < bytes.len() && bytes[i] == b'\'' {
                    i += 1;
                }
                push(&mut out, &src[start..i], Tok::Char);
                continue;
            }
            let mut j = i;
            while j < bytes.len()
                && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_')
            {
                j += 1;
            }
            if bytes.get(j) == Some(&b'\'') && j > i {
                i = j + 1;
                push(&mut out, &src[start..i], Tok::Char);
            } else if j > i {
                i = j;
                push(&mut out, &src[start..i], Tok::LifeTime);
            } else {
                push(&mut out, &src[start..i], Tok::Punct);
            }
            continue;
        }

        // Attribute / inner-attribute: #[...] / #![...]
        if c == '#'
            && matches!(bytes.get(i + 1), Some(&b'[') | Some(&b'!'))
        {
            let start = i;
            let mut depth = 0i32;
            let mut seen_bracket = false;
            while i < bytes.len() {
                match bytes[i] {
                    b'[' => {
                        depth += 1;
                        seen_bracket = true;
                    }
                    b']' => {
                        depth -= 1;
                        if seen_bracket && depth == 0 {
                            i += 1;
                            break;
                        }
                    }
                    _ => {}
                }
                i += 1;
            }
            push(&mut out, &src[start..i], Tok::Attr);
            continue;
        }

        // Number literal.
        if c.is_ascii_digit() {
            let start = i;
            i += 1;
            while i < bytes.len() {
                let b = bytes[i];
                if b.is_ascii_alphanumeric() || b == b'_' || b == b'.' {
                    i += 1;
                } else {
                    break;
                }
            }
            push(&mut out, &src[start..i], Tok::Num);
            continue;
        }

        // Identifier / keyword / type / macro / function-call.
        if c.is_ascii_alphabetic() || c == '_' {
            let start = i;
            i += 1;
            while i < bytes.len() {
                let b = bytes[i];
                if b.is_ascii_alphanumeric() || b == b'_' {
                    i += 1;
                } else {
                    break;
                }
            }
            let word = &src[start..i];
            // Look ahead past spaces/tabs to disambiguate.
            let mut peek = i;
            while peek < bytes.len() && (bytes[peek] == b' ' || bytes[peek] == b'\t') {
                peek += 1;
            }
            let next = bytes.get(peek).copied();
            let tok = if KEYWORDS.contains(&word) {
                Tok::Keyword
            } else if next == Some(b'!') {
                push(&mut out, word, Tok::Macro);
                let bang_end = peek + 1;
                push(&mut out, &src[i..bang_end], Tok::Macro);
                i = bang_end;
                continue;
            } else if PRIMITIVE_TYPES.contains(&word) {
                Tok::PrimType
            } else if word
                .chars()
                .next()
                .map(|ch| ch.is_ascii_uppercase())
                .unwrap_or(false)
            {
                Tok::UserType
            } else if next == Some(b'(') {
                Tok::FnCall
            } else {
                Tok::Plain
            };
            push(&mut out, word, tok);
            continue;
        }

        // Punctuation — emit per-codepoint so an identifier sweeping into
        // a punct span can't happen.
        if !c.is_whitespace() {
            push(&mut out, &src[i..i + c.len_utf8()], Tok::Punct);
            i += c.len_utf8();
            continue;
        }

        // Whitespace.
        let start = i;
        while i < bytes.len() && (bytes[i] as char).is_whitespace() {
            i += 1;
        }
        push(&mut out, &src[start..i], Tok::Plain);
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat(src: &str) -> String {
        tokenize(src)
            .into_iter()
            .map(|(s, _)| s)
            .collect::<String>()
    }

    fn kinds(src: &str) -> Vec<Tok> {
        tokenize(src).into_iter().map(|(_, k)| k).collect()
    }

    #[test]
    fn tokenization_is_lossless() {
        let src = "let x = 42;\nfn add(a: i32, b: i32) -> i32 { a + b }\n// comment here\nlet s = \"hello\\nworld\";\nlet v = vec![1, 2, 3];\n#[derive(Debug)]\nstruct Point { x: f64 }\nlet life: &'static str = r#\"raw \"inside\" raw\"#;\n";
        assert_eq!(flat(src), src);
    }

    #[test]
    fn keywords_are_recognised() {
        let toks = tokenize("let mut x = if true { 1 } else { 2 };");
        let kws: Vec<&str> = toks
            .iter()
            .filter(|(_, k)| *k == Tok::Keyword)
            .map(|(s, _)| s.as_str())
            .collect();
        assert!(kws.contains(&"let"));
        assert!(kws.contains(&"mut"));
        assert!(kws.contains(&"if"));
        assert!(kws.contains(&"true"));
        assert!(kws.contains(&"else"));
    }

    #[test]
    fn macros_are_distinct_from_function_calls() {
        let toks = tokenize("vec![1, 2]; println!(\"hi\"); add(1, 2);");
        let macs: String = toks
            .iter()
            .filter(|(_, k)| *k == Tok::Macro)
            .map(|(s, _)| s.as_str())
            .collect();
        let fns: Vec<&str> = toks
            .iter()
            .filter(|(_, k)| *k == Tok::FnCall)
            .map(|(s, _)| s.as_str())
            .collect();
        assert!(macs.contains("vec!"));
        assert!(macs.contains("println!"));
        assert!(fns.contains(&"add"));
    }

    #[test]
    fn string_with_escape_does_not_swallow_following_code() {
        let src = "let s = \"a\\\"b\"; let n = 5;";
        let toks = tokenize(src);
        let after_str: String = toks
            .iter()
            .skip_while(|(_, k)| *k != Tok::Str)
            .skip(1)
            .map(|(s, _)| s.as_str())
            .collect();
        assert!(after_str.contains("let"));
        assert!(after_str.contains('5'));
    }

    #[test]
    fn raw_string_with_hashes_is_handled() {
        let src = r##"let s = r#"contains "quotes""#;"##;
        assert_eq!(flat(src), src);
        assert!(kinds(src).contains(&Tok::Str));
    }

    #[test]
    fn lifetimes_vs_chars() {
        let toks = tokenize("fn f<'a>(s: &'a str) -> char { 'x' }");
        let has_life = toks.iter().any(|(_, k)| *k == Tok::LifeTime);
        let has_char = toks.iter().any(|(_, k)| *k == Tok::Char);
        assert!(has_life, "should recognise a lifetime");
        assert!(has_char, "should recognise a char literal");
    }

    #[test]
    fn attribute_block_is_one_run() {
        let toks = tokenize("#[derive(Debug, Clone)]\nstruct A;");
        let attr_count = toks.iter().filter(|(_, k)| *k == Tok::Attr).count();
        assert_eq!(
            attr_count, 1,
            "the whole #[…] block should be a single attribute run"
        );
    }

    #[test]
    fn unterminated_inputs_do_not_panic() {
        let _ = tokenize("let s = \"unterminated");
        let _ = tokenize("let c = '");
        let _ = tokenize("/* block /* never closed");
    }

    #[test]
    fn css_class_is_stable_for_each_kind() {
        assert_eq!(Tok::Keyword.css_class(), "hl-kw");
        assert_eq!(Tok::Str.css_class(), "hl-str");
        assert_eq!(Tok::Char.css_class(), "hl-str");
        assert_eq!(Tok::Num.css_class(), "hl-num");
        assert_eq!(Tok::Comment.css_class(), "hl-com");
        assert_eq!(Tok::Macro.css_class(), "hl-mac");
        assert_eq!(Tok::FnCall.css_class(), "hl-fn");
        assert_eq!(Tok::UserType.css_class(), "hl-ty");
        assert_eq!(Tok::PrimType.css_class(), "hl-pty");
        assert_eq!(Tok::Attr.css_class(), "hl-attr");
        assert_eq!(Tok::Punct.css_class(), "hl-punct");
        assert_eq!(Tok::LifeTime.css_class(), "hl-life");
        assert_eq!(Tok::Plain.css_class(), "");
    }
}
