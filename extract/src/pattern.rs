use regex::{self, Regex};
use std::{collections::HashMap, str};

pub struct Pattern {
    pattern: String
}

impl Pattern {
    pub fn new(raw_pattern: &str) -> Self {
        let re = Regex::new(r"\{[^{}]*[a-zA-Z<][^{}]*\}").unwrap();
    
        let result = re.replace_all(raw_pattern, |caps: &regex::Captures| {
            let original = &caps[0];
            let escaped = original.replace("{", "\\{").replace("}", "\\}");
            escaped
        });
        Pattern {
            pattern: result.to_string()
        }
    }

    pub fn get_indices(&self) -> HashMap<String, usize> {
        let re = Regex::new(&self.pattern).unwrap();
        let mut capture_group_indices = HashMap::new();
        
        for (i, name) in re.capture_names().enumerate() {
            if let Some(name) = name {
                capture_group_indices.insert(name.to_string(), i);
            }
        }
    
        capture_group_indices
    }

    pub fn clear(&self) -> String {
        let re = Regex::new(r"\?P<\w*>").unwrap();
        re.replace(&self.pattern, "").replace("/", "")
    }
}