use regex_syntax::hir::{Hir, HirKind, Literal, Class, Look, Repetition, Capture};

fn generate_patterns(literal: &[u8]) -> Vec<String> {
    if literal.len() < 10 {
        return vec![String::from_utf8(literal.to_vec()).unwrap()];
    }
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
            let patterns = generate_patterns(bytes);
            patterns.join("|")
        },
        HirKind::Class(class) => class_to_string(class),
        HirKind::Concat(hirs) => {
            let mut patterns = Vec::new();
            for hir in hirs {
                patterns.push(update_hir_pattern(hir));
            }
            patterns.join("")
        },
        HirKind::Alternation(hirs) => {
            let mut patterns = Vec::new();
            for hir in hirs {
                patterns.push(update_hir_pattern(hir));
            }
            patterns.join("|")
        },
        HirKind::Look(_) => String::new(),
        HirKind::Repetition(repetition) => {
            let sub_pattern = update_hir_pattern(&repetition.sub);
            let quantifier = match repetition.max {
                Some(max) if max == repetition.min => format!("{{{}}}", max),
                Some(max) => format!("{{{},{}}}", repetition.min, max),
                None => format!("{{{},{}}}", repetition.min, ""),
            };
            format!("{}{}", sub_pattern, quantifier)
        },
        HirKind::Capture(capture) => {
            let sub_pattern = update_hir_pattern(&capture.sub);
            format!("(?P<{}>{})", capture.name.as_ref().unwrap(), sub_pattern)
        },
    }
}

// fn main() {
//     let pattern = "^[ATGCN]*T(?P<UMI>[ATGCN]{12})CTCCGCTTAAGGGACT";
//     let hir_structure = regex_syntax::ParserBuilder::new().build().parse(pattern).unwrap();
//     let result = update_hir_pattern(&hir_structure);
//     println!("{}", result);
// }
