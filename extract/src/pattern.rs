use std::mem::size_of;

use fancy_regex::Regex;

const ERROR_PER_TEN_NUCLEOTIDES: usize = 2;
const PATTERN_REGEX: &str = r"(?<!\[)\b[ATGCN]+\b(?!\])";


pub fn generate_fuzzy_patterns(string: &str, error_num: usize) -> Vec<String> {
    if string.len() == 1 {
        return vec![string.to_string()];
    }
    if string.is_empty() {
        return Vec::new();
    }

    let num_chars = string.chars().count();
    assert!(num_chars <= size_of::<usize>() * 8, "too many characters");

    // Instead of using `pow` (which is intuitive to humans!), let's just generate a mask of all 1s
    // and then shift it to match the count of 1s with the number of characters.
    let max_permutation_mask = usize::max_value()
        .checked_shr(size_of::<usize>() as u32 * 8 - num_chars as u32)
        .unwrap();
    let mut cases = Vec::with_capacity(max_permutation_mask);

    let (upper, lower) = string.chars().fold(
        (Vec::with_capacity(num_chars), Vec::with_capacity(num_chars)),
        |(mut upper, mut lower), c| {
            upper.push(c.to_ascii_uppercase());
            lower.push('.');
            (upper, lower)
        }
    );

    let len = string.len();
    for permutation_mask in 0..=max_permutation_mask {
        let mut s = String::with_capacity(len);
        for idx in 0..num_chars {
            if (permutation_mask & (1 << idx)) == 0 {
                s.push(lower[idx])
            } else {
                s.push(upper[idx])
            }
        }
        cases.push(s);
    }

    let mut new_cases = Vec::<String>::new();
    for i in cases {
        if i.chars().filter(|c| *c == '.').count() == error_num {
            new_cases.push(i);
        } 
    }

    new_cases
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