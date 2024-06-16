use regex_syntax::hir::{Hir, HirKind, Literal, Class};

fn generate_patterns(literal: &[u8]) -> Vec<String> {
    let mut patterns = Vec::new();
    for i in 0..literal.len() {
        let mut pattern = literal.to_vec();
        pattern[i] = b'.';
        patterns.push(String::from_utf8(pattern).unwrap());
    }
    patterns
}

fn class_to_string(class: &Class) -> String {
    match class {
        Class::Bytes(class_bytes) => {
            let mut class_str = String::from("[");
            for range in class_bytes.ranges() {
                if range.start() == range.end() {
                    class_str.push(range.start() as char);
                } else {
                    class_str.push_str(&format!("{}-{}", range.start() as char, range.end() as char));
                }
            }
            class_str.push(']');
            class_str
        }
        Class::Unicode(class_unicode) => {
            let mut class_str = String::from("[");
            for range in class_unicode.ranges() {
                if range.start() == range.end() {
                    if let Some(c) = std::char::from_u32(range.start() as u32) {
                        class_str.push(c);
                    }
                } else {
                    if let (Some(start), Some(end)) = (
                        std::char::from_u32(range.start() as u32),
                        std::char::from_u32(range.end() as u32),
                    ) {
                        class_str.push_str(&format!("{}-{}", start, end));
                    }
                }
            }
            class_str.push(']');
            class_str
        }
    }
}

pub fn update_hir_pattern(hir: &Hir) -> String {
    match hir.kind() {
        HirKind::Empty => String::new(),
        HirKind::Literal(Literal(bytes)) => {
            let bytes_ref: &[u8] = &bytes;
            let patterns = generate_patterns(bytes_ref);
            patterns.join("|")
        },
        HirKind::Class(class) => class_to_string(class),
        HirKind::Concat(hirs) => {
            let updated_patterns: Vec<String> = hirs.iter().map(|hir| update_hir_pattern(hir)).collect();
            updated_patterns.join("")
        },
        HirKind::Alternation(hirs) => {
            let updated_patterns: Vec<String> = hirs.iter().map(|hir| update_hir_pattern(hir)).collect();
            updated_patterns.join("|")
        },
        HirKind::Look(_) => String::new(),
        HirKind::Repetition(hirs) => {
            let sub_pattern = update_hir_pattern(&hirs.sub);
            format!("({})*", sub_pattern)
        },
        HirKind::Capture(hirs) => {
            let sub_pattern = update_hir_pattern(&hirs.sub);
            format!("(?P<{}>{})", hirs.name.as_deref().unwrap_or(""), sub_pattern)
        },
    }
}