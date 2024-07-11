use regex::{self, Regex};
use std::collections::HashMap;
use crate::errors;

pub struct Pattern {
    pattern: String,
}

impl Pattern {
    pub fn new(raw_pattern: &str) -> Result<Self, errors::Error> {
        let escaped_pattern = escape_braces(raw_pattern)?;
        Ok(Pattern {
            pattern: escaped_pattern,
        })
    }

    pub fn get_group_indices(&self) -> Result<HashMap<String, usize>, errors::Error> {
        let re = Regex::new(&self.pattern)?;
        Ok(re.capture_names()
            .enumerate()
            .filter_map(|(i, name)| name.map(|name| (name.to_string(), i)))
            .collect())
    }

    pub fn clear(&self) -> Result<String, errors::Error> {
        let re = Regex::new(r"\?P<\w*>")?;
        Ok(re.replace_all(&self.pattern, "").replace("\\", ""))
    }

    pub fn to_string(&self) -> String {
        self.pattern.clone()
    }
}

fn escape_braces(raw_pattern: &str) -> Result<String, errors::Error> {
    let re = Regex::new(r"\{[^{}]*[a-zA-Z<][^{}]*\}")?;
    Ok(re.replace_all(
        raw_pattern, 
        |caps: &regex::Captures| {
            caps[0].replace("{", "\\{").replace("}", "\\}")
        }
    ).to_string())
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use crate::pattern::Pattern;

    #[rstest]
    #[case(r#"^[ATGCN]*T(?P<UMI>[ATGCN]{12})(CTCCGCTTAAGGGACT){1,3}"#)]
    #[case(r#"^N{0:2}TGGTATCAACGCAGAGT(?P<UMI>:T[ATGCN]{3}T[ATGCN]{3}T[ATGCN]{3}T)"#)]
    #[case(r#"^?P<UMI>[ATGCN]{0:12}"#)]
    #[case(r#"^[ATGCN]{0:12}"#)]
    fn test_pattern_new(#[case] pattern: &str) {
        assert_eq!(Pattern::new(pattern).unwrap().to_string(), pattern.to_string());
    }
}