use regex::{self, Regex, Captures};

use seq_io::fastq::{Record, RefRecord};

use crate::errors;

pub struct Barcode {
    pattern_regex: Regex
}

impl Barcode {
    pub fn new(pattern: &str) -> Self {
        Self {
            pattern_regex: Regex::new(pattern).map_err(|_| errors::Error::InvalidPattern(pattern.to_string())).unwrap()
        }
    }

    pub fn match_read<'a>(&self, read: &'a RefRecord) -> Result<regex::Captures<'a>, errors::Error> {

        let read_seq = std::str::from_utf8(read.seq()).unwrap();
    
        match self.pattern_regex.captures(read_seq) {
            Some(value) => Ok(value),
            None => Err(errors::Error::PatternNotMatched(read_seq.to_string()))
        }
    }

    pub fn cut_from_read(barcode_type: String, matched_pattern: Captures, read: &RefRecord) -> Result<(Vec<u8>, Vec<u8>), errors::Error> {
        let start: usize = matched_pattern.name(&barcode_type).unwrap().start();
        let end = matched_pattern.name(&barcode_type).unwrap().end();
        let read_seq = read.seq();
        let read_qual = read.qual();
        let new_read_seq = [&read_seq[..start], &read_seq[end..]].concat();
        let new_read_qual = [&read_qual[..start], &read_qual[end..]].concat();
        Ok((new_read_seq, new_read_qual))
    }

    fn move_to_the_header() {}
}