use std::{fmt, mem::size_of};

use fancy_regex::Regex as FancyRegex;
use regex::bytes::{Captures, Regex};

use crate::error::Error;

const FUZZY_CHARACTER: &str = ".";
const ADAPTER_PATTERN_REGEX: &str = r"(?<!\[)\b[atgcryswkmbdhvn]+\b(?!\])";

pub struct BarcodePattern {
    adapter_pattern: FancyRegex,
    barcode_pattern: String,
    max_error: usize
}

impl BarcodePattern {
    pub fn new(pattern: &str, max_error: &usize) -> Result<Self, Error> {
        Ok(Self {
            adapter_pattern: FancyRegex::new(ADAPTER_PATTERN_REGEX)?,
            barcode_pattern: pattern.to_owned(),
            max_error: *max_error
        })
    }

    /// Generates sequences with errors that may occur during amplification.
    ///
    /// # Example
    ///
    /// ```
    /// use barkit_extract::pattern::BarcodePattern;
    /// 
    /// let barcode_pattern = BarcodePattern::new("^atgc(?<UMI>[ATGCN]{12})", &1).expect("REASON");
    /// 
    /// let sequences_with_errors = barcode_pattern.get_sequence_with_errors("ATGC");
    /// assert_eq!(vec!["ATG.", "AT.C", "A.GC", ".TGC"], sequences_with_errors);
    /// ```
    pub fn get_sequence_with_errors(&self, sequence: &str) -> Vec<String> {
        if self.max_error == 0 {
            return vec![sequence.to_string().to_ascii_uppercase()];
        }
    
        if sequence.is_empty() {
            return Vec::new();
        }
    
        if self.max_error >= sequence.len() {
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
            if permutation_mask.count_ones() as usize != num_chars - self.max_error {
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
    /// use barkit_extract::pattern::BarcodePattern;
    /// 
    /// let barcode_pattern = BarcodePattern::new("^atgc(?<UMI>[ATGCN]{12})", &1).expect("REASON");
    /// 
    /// let pattern_with_pcr_errors = barcode_pattern.get_pattern_with_errors().expect("REASON");
    /// assert_eq!("^(ATG.|AT.C|A.GC|.TGC)(?<UMI>[ATGCN]{12})", pattern_with_pcr_errors);
    /// ```
    pub fn get_pattern_with_errors(&self) -> Result<String, Error> {
        let mut result = String::new();
        let mut last_end = 0;

        for mat in self.adapter_pattern.find_iter(&self.barcode_pattern) {
            let mat = mat.unwrap();
            result.push_str(&self.barcode_pattern[last_end..mat.start()]);

            let fuzzy_patterns = self.get_sequence_with_errors(mat.as_str());
            result.push_str(&format!("({})", fuzzy_patterns.join("|")));

            last_end = mat.end();
        }

        result.push_str(&self.barcode_pattern[last_end..]);
        Ok(result)
    }
}


#[derive(Clone)]
pub enum BarcodeType {
    Umi,
    Sample,
    Cell,
}

impl fmt::Display for BarcodeType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let barcode_str = match self {
            BarcodeType::Umi => "UMI",
            BarcodeType::Sample => "SB",
            BarcodeType::Cell => "CB",
        };
        write!(f, "{}", barcode_str)
    }
}

impl BarcodeType {
    fn get_barcode_type(name: &str) -> Result<Self, Error> {
        match name {
            "UMI" => Ok(BarcodeType::Umi),
            "SB" => Ok(BarcodeType::Sample),
            "CB" => Ok(BarcodeType::Cell),
            _ => Err(Error::UnexpectedCaptureGroupName(name.to_owned())),
        }
    }
}

#[derive(Clone)]
pub struct BarcodeRegex {
    regex: Regex,
    barcode_types: Vec<BarcodeType>,
}

impl BarcodeRegex {
    pub fn new(pattern: &str, max_error: usize) -> Result<Self, Error> {
        let barcode_pattern = BarcodePattern::new(pattern, &max_error)?;
        let fuzzy_pattern = barcode_pattern.get_pattern_with_errors()?;
        let regex = Regex::new(&fuzzy_pattern)?;
        let barcode_types = Self::parse_capture_groups(&regex)?;
        Ok(Self {
            regex,
            barcode_types,
        })
    }

    pub fn get_captures<'a>(&'a self, read_seq: &'a [u8]) -> Result<Captures, Error> {
        match self.regex.captures(read_seq) {
            Some(capture) => Ok(capture),
            None => Err(Error::PatternNotMatched),
        }
    }

    fn parse_capture_groups(regex: &Regex) -> Result<Vec<BarcodeType>, Error> {
        let mut capture_groups = Vec::<BarcodeType>::new();
        for capture_group in regex
            .capture_names()
            .collect::<Vec<_>>()
            .into_iter()
            .flatten()
        {
            capture_groups.push(BarcodeType::get_barcode_type(capture_group)?)
        }
        if capture_groups.is_empty() {
            return Err(Error::BarcodeCaptureGroupNotFound(regex.to_string()));
        }
        Ok(capture_groups)
    }

    pub fn get_barcode_types(&self) -> Vec<BarcodeType> {
        self.barcode_types.clone()
    }
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
        #[case] max_error: usize,
    ) {
        let barcode_pattern = pattern::BarcodePattern::new("", &max_error).unwrap();
        assert_eq!(
            expected,
            barcode_pattern.get_sequence_with_errors(text)
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
        let barcode_pattern = pattern::BarcodePattern::new(pattern, &max_error).unwrap();
        assert_eq!(expected, barcode_pattern.get_pattern_with_errors().unwrap())
    }
}