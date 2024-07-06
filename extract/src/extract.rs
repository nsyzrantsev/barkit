use fuzzy_regex::fuzzy::{FuzzyRegex, Match};
use crate::pattern::Pattern;

use std::{collections::HashMap, str};
use seq_io::fastq::{Record, RefRecord, OwnedRecord};

use crate::errors::Error;

pub struct BarcodeExtractor {
    regex: FuzzyRegex,
    capture_groups: HashMap<String, usize>
}

impl BarcodeExtractor {
    pub fn new(pattern: &str) -> Result<Self, Error> {
        let pattern_escaped = Pattern::new(pattern);
        let capture_groups = pattern_escaped.get_indices();
        let posix_pattern = pattern_escaped.clear();
        let regex = FuzzyRegex::new(&posix_pattern).expect("FuzzyRegex::new");
        Ok(Self {
            regex,
            capture_groups,
        })
    }

    pub fn match_read<'a>(&self, read: &'a RefRecord) -> Result<Match<'a>, Error> {
        let read_seq = read.seq();
        let result = self.regex.captures(read_seq,3)?;
        let matched = result.get_matches();
        let capture_group_index = self.capture_groups["UMI"];
        Ok(matched[capture_group_index].clone().unwrap())
    }

    pub fn cut_from_read_seq(barcode_type: &str, matched_pattern: Match, read: &RefRecord) -> Result<OwnedRecord, Error> {
        let start = matched_pattern.start();
        let end = matched_pattern.end();

        let read_seq = read.seq();
        let read_qual = read.qual();
        
        Ok(OwnedRecord {
            head: [&read_seq[..start], &read_seq[end..]].concat(),
            seq: [&read_qual[..start], &read_qual[end..]].concat(),
            qual: Self::move_to_the_header(barcode_type, read, start, end)?
        })
    }

    fn move_to_the_header(barcode_type: &str, read: &RefRecord, start: usize, end: usize) -> Result<Vec<u8>, Error> {
        let read_header = read.head();
        let barcode_seq = &read.seq()[start..end];
        let barcode_qual = &read.qual()[start..end];
    
        let mut result = Vec::with_capacity(read_header.len() + barcode_type.len() + barcode_seq.len() + barcode_qual.len() + 3);
        result.extend_from_slice(read_header);
        result.extend_from_slice(format!(" {}:{}", barcode_type, std::str::from_utf8(barcode_seq)?).as_bytes());
        result.extend_from_slice(barcode_qual);
    
        Ok(result)
    }
}