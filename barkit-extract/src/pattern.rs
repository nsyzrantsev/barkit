use std::mem::size_of;

use fancy_regex::Regex;

const FUZZY_CHARACTER: &str = ".";
const ADAPTER_PATTERN_REGEX: &str = r"(?<!\[)\b[atgcryswkmbdhvn]+\b(?!\])";

/// Generates sequences with errors that may occur during amplification.
///
/// # Example
///
/// ```
/// use barkit_extract::pattern::generate_sequences_with_pcr_errors;
/// let sequence = "ATGC";
/// let errors_number = 1;
/// let sequences_with_errors = generate_sequences_with_pcr_errors(sequence, &errors_number);
/// assert_eq!(vec!["ATG.", "AT.C", "A.GC", ".TGC"], sequences_with_errors);
/// ```
pub fn generate_sequences_with_pcr_errors(sequence: &str, errors_num: &usize) -> Vec<String> {
    if *errors_num == 0 {
        return vec![sequence.to_string().to_ascii_uppercase()];
    }

    if sequence.is_empty() {
        return Vec::new();
    }

    if *errors_num >= sequence.len() {
        return vec![FUZZY_CHARACTER.repeat(sequence.len())];
    }

    let num_chars = sequence.chars().count();
    assert!(num_chars <= usize::BITS as usize * 8, "too many characters");

    let max_permutation_mask = usize::MAX
        .checked_shr(size_of::<usize>() as u32 * 8 - num_chars as u32)
        .unwrap();

    let mut cases = Vec::new();

    let upper: Vec<char> = sequence.chars().map(|c| c.to_ascii_uppercase()).collect();

    for permutation_mask in 0..=max_permutation_mask {
        if permutation_mask.count_ones() as usize != num_chars - errors_num {
            continue;
        }
        let mut s = String::new();
        for (idx, _) in upper.iter().enumerate().take(num_chars) {
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

/// Returns regex pattern with PCR errors.
///
/// # Example
///
/// ```
/// use barkit_extract::pattern::get_with_errors;
/// let sequence = "^atgc(?<UMI>[ATGCN]{12})";
/// let max_error = 1;
/// let pattern_with_pcr_errors = get_with_errors(sequence, &max_error);
/// assert_eq!("^(ATG.|AT.C|A.GC|.TGC)(?<UMI>[ATGCN]{12})", pattern_with_pcr_errors);
/// ```
pub fn get_with_errors(pattern: &str, max_error: &usize) -> String {
    let regex_pattern = Regex::new(ADAPTER_PATTERN_REGEX).unwrap();

    let mut result = String::new();
    let mut last_end = 0;

    for mat in regex_pattern.find_iter(pattern) {
        let mat = mat.unwrap();
        result.push_str(&pattern[last_end..mat.start()]);

        let fuzzy_patterns = generate_sequences_with_pcr_errors(mat.as_str(), max_error);
        result.push_str(&format!("({})", fuzzy_patterns.join("|")));

        last_end = mat.end();
    }

    result.push_str(&pattern[last_end..]);
    result
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::pattern;

    #[rstest]
    #[case(vec!["."], "a", 1)]
    #[case(vec!["A"], "a", 0)]
    #[case(vec![], "", 1)]
    #[case(vec!["AAA.", "AA.A", "A.AA", ".AAA"], "AAAA", 1)]
    #[case(vec!["..."], "AAA", 3)]
    #[case(vec!["..."], "AAA", 4)]
    fn test_generate_sequences_with_pcr_errors(
        #[case] expected: Vec<&str>,
        #[case] text: &str,
        #[case] errors_num: usize,
    ) {
        assert_eq!(
            expected,
            pattern::generate_sequences_with_pcr_errors(text, &errors_num)
        );
    }

    #[rstest]
    #[case("^(AA.|A.A|.AA)(?P<UMI>[ATGCN]{3})", "^aaa(?P<UMI>[ATGCN]{3})", 1)]
    #[case("^(...)(?P<UMI>[ATGCN]{3})", "^aaa(?P<UMI>[ATGCN]{3})", 3)]
    #[case("^(...)(?P<UMI>[ATGCN]{3})", "^aaa(?P<UMI>[ATGCN]{3})", 4)]
    #[case("^((...))(?P<UMI>[ATGCN]{3})", "^(aaa)(?P<UMI>[ATGCN]{3})", 4)]
    #[case(
        "^(AA.|A.A|.AA)(?P<UMI>[ATGCN]{3})CCC",
        "^aaa(?P<UMI>[ATGCN]{3})CCC",
        1
    )]
    #[case("^(?P<UMI>[ATGCN]{3})", "^(?P<UMI>[ATGCN]{3})", 1)]
    fn test_create_fuzzy(#[case] expected: &str, #[case] pattern: &str, #[case] max_error: usize) {
        assert_eq!(expected, pattern::get_with_errors(pattern, &max_error))
    }
}
