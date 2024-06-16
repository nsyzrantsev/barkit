use fancy_regex::Regex;

const MIN_FUZZY_LEN: usize = 10;
const FUZZY_LITERAL_REGEX: &str = r"(?<!\[)\b[ATGCN]+\b(?!\])";


fn generate_fuzzy_patterns(literal: &[u8]) -> Vec<String> {
    if literal.len() < MIN_FUZZY_LEN {
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

pub fn update_pattern(pattern: &str) -> String {

    let regex_pattern = Regex::new(FUZZY_LITERAL_REGEX).unwrap();

    let mut result = String::new();
    let mut last_end = 0;

    for mat in regex_pattern.find_iter(pattern) {
        let mat = mat.unwrap();
        result.push_str(&pattern[last_end..mat.start()]);

        let fuzzy_patterns = generate_fuzzy_patterns(mat.as_str().as_bytes());
        result.push_str(&fuzzy_patterns.join("|"));

        last_end = mat.end();
    }

    result.push_str(&pattern[last_end..]);
    result
}