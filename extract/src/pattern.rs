use regex::{self, Regex};
use std::collections::HashMap;

pub struct Pattern {
    pattern: String,
}

impl Pattern {
    pub fn new(raw_pattern: &str) -> Self {
        let escaped_pattern = escape_braces(raw_pattern);
        Pattern {
            pattern: escaped_pattern,
        }
    }

    pub fn get_indices(&self) -> HashMap<String, usize> {
        let re = Regex::new(&self.pattern).unwrap();
        re.capture_names()
            .enumerate()
            .filter_map(|(i, name)| name.map(|name| (name.to_string(), i)))
            .collect()
    }

    pub fn clear(&self) -> String {
        let re = Regex::new(r"\?P<\w*>").unwrap();
        re.replace_all(&self.pattern, "").replace("/", "")
    }

    pub fn to_string(&self) -> String {
        self.pattern.clone()
    }
}

fn escape_braces(raw_pattern: &str) -> String {
    let re = Regex::new(r"\{[^{}]*[a-zA-Z<][^{}]*\}").unwrap();
    re.replace_all(raw_pattern, |caps: &regex::Captures| {
        caps[0].replace("{", "\\{").replace("}", "\\}")
    }).to_string()
}

#[test]
fn test_pattern_new() {
    let patterns = [
        r#"^[ATGCN]*T(?P<UMI>[ATGCN]{12})(CTCCGCTTAAGGGACT){1,3}"#,
        r#"^N{0:2}TGGTATCAACGCAGAGT(?P<UMI>:T[ATGCN]{3}T[ATGCN]{3}T[ATGCN]{3}T)"#,
        r#"^?P<UMI>[ATGCN]{0:12}"#,
        r#"^[ATGCN]{0:12}"#,
    ];

    for &pattern in &patterns {
        assert_eq!(Pattern::new(pattern).to_string(), pattern.to_string());
    }
}
