use std::mem::size_of;

use fancy_regex::Regex;

const FUZZY_CHARACTER: &str = ".";
const ADAPTER_PATTERN_REGEX: &str = r"(?i)(?<!\[)\b[atgcn]+\b(?!\])";

pub fn generate_sequences_with_pcr_errors(string: &str, errors_num: &usize) -> Vec<String> {
    
    if *errors_num == 0 {
        return vec![string.to_string().to_ascii_uppercase()];
    }
    
    if string.is_empty() {
        return Vec::new();
    }
    
    if *errors_num >= string.len() {
        return vec![FUZZY_CHARACTER.repeat(string.len())];
    }

    let num_chars = string.chars().count();
    assert!(num_chars <= size_of::<usize>() * 8, "too many characters");

    let max_permutation_mask = usize::max_value()
        .checked_shr(size_of::<usize>() as u32 * 8 - num_chars as u32)
        .unwrap();

    let mut cases = Vec::new();

    let upper: Vec<char> = string.chars().map(|c| c.to_ascii_uppercase()).collect();

    for permutation_mask in 0..=max_permutation_mask {
        if permutation_mask.count_ones() as usize != num_chars - errors_num {
            continue;
        }
        let mut s = String::new();
        for idx in 0..num_chars {
            if (permutation_mask & (1 << idx)) == 0 {
                s.push_str(FUZZY_CHARACTER)
            } else {
                s.push(upper[idx])
            }
        }
        cases.push(s);
    }

    cases
}



pub fn create_fuzzy(pattern: &str, max_error: &usize) -> String {

    let regex_pattern = Regex::new(ADAPTER_PATTERN_REGEX).unwrap();

    let mut result = String::new();
    let mut last_end = 0;

    for mat in regex_pattern.find_iter(pattern) {
        let mat = mat.unwrap();
        result.push_str(&pattern[last_end..mat.start()]);

        let fuzzy_patterns = generate_sequences_with_pcr_errors(mat.as_str(), max_error);
        result.push_str(&fuzzy_patterns.join("|"));

        last_end = mat.end();
    }

    result.push_str(&pattern[last_end..]);
    result
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::pattern::generate_sequences_with_pcr_errors;

    #[rstest]
    #[case(vec!["."], "a", 1)]
    #[case(vec!["A"], "a", 0)]
    #[case(vec![], "", 1)]
    #[case(vec!["AAA.", "AA.A", "A.AA", ".AAA"], "AAAA", 1)]
    #[case(vec!["..."], "AAA", 3)]
    #[case(vec!["..."], "AAA", 4)]
    fn test_generate_sequences_with_pcr_errors(#[case] expected: Vec<&str>, #[case] pattern: &str, #[case] errors_num: usize) {
        assert_eq!(expected, generate_sequences_with_pcr_errors(pattern, &errors_num));
    }
}