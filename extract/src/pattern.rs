use std::mem::size_of;

use fancy_regex::Regex;

const ERROR_PER_TEN_NUCLEOTIDES: usize = 1;
const PATTERN_REGEX: &str = r"(?<!\[)\b[ATGCN]+\b(?!\])";


pub fn generate_fuzzy_patterns(string: &str, error_num: usize) -> Vec<String> {
    if string.len() == 1 || error_num == 0 {
        return vec![string.to_string()];
    }
    if string.is_empty() {
        return Vec::new();
    }

    let num_chars = string.chars().count();
    assert!(num_chars <= std::mem::size_of::<usize>() * 8, "too many characters");

    let max_permutation_mask = usize::max_value()
        .checked_shr(size_of::<usize>() as u32 * 8 - num_chars as u32)
        .unwrap();

    let mut cases = Vec::new();
    
    let upper: Vec<char> = string.chars().map(|c| c.to_ascii_uppercase()).collect();
    
    for permutation_mask in 0..=max_permutation_mask {
        if (permutation_mask as u32).count_ones() as usize != num_chars - error_num {
            continue;
        }
        
        let mut s = String::with_capacity(num_chars);
        for idx in 0..num_chars {
            if (permutation_mask & (1 << idx)) == 0 {
                s.push('.');
            } else {
                s.push(upper[idx]);
            }
        }
        cases.push(s);
    }

    cases
}



pub fn update_pattern(pattern: &str) -> String {

    let regex_pattern = Regex::new(PATTERN_REGEX).unwrap();

    let mut result = String::new();
    let mut last_end = 0;

    for mat in regex_pattern.find_iter(pattern) {
        let mat = mat.unwrap();
        result.push_str(&pattern[last_end..mat.start()]);

        let error_num = ((mat.as_str().len() * ERROR_PER_TEN_NUCLEOTIDES) as f32 / 10.0).ceil() as usize;

        let fuzzy_patterns = generate_fuzzy_patterns(mat.as_str(), error_num);
        result.push_str(&fuzzy_patterns.join("|"));

        last_end = mat.end();
    }

    result.push_str(&pattern[last_end..]);
    result
}