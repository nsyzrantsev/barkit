use regex::{self, Regex};
use fuzzy_regex::fuzzy::{FuzzyRegex, Match};
use std::{collections::HashMap, str};
use seq_io::fastq::{Record, RefRecord};

use crate::errors::Error;

pub struct BarcodeMatcher {
    regex: FuzzyRegex,
    capture_groups: HashMap<String, usize>
}

impl BarcodeMatcher {
    pub fn new(pattern: &str, max_error: usize) -> Result<Self, Error> {
        let capture_groups = get_capture_group_indices(&pattern);
        let posix_pattern = remove_capture_groups(pattern);
        let regex = FuzzyRegex::new(
            &posix_pattern, 
            max_error,
            0,
            0
        ).expect("Regex::new");
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

    pub fn cut_from_read_seq(barcode_type: &str, matched_pattern: Match, read: &RefRecord) -> Result<(Vec::<u8>, Vec::<u8>, Vec::<u8>), Error> {
        let start = matched_pattern.start();
        let end = matched_pattern.end();

        let read_seq = read.seq();
        let read_qual = read.qual();

        let new_read_seq = [&read_seq[..start], &read_seq[end..]].concat();
        let new_read_qual = [&read_qual[..start], &read_qual[end..]].concat();
        let new_read_header = Self::move_to_the_header(barcode_type, read, start, end)?;
        
        Ok((new_read_seq, new_read_qual, new_read_header))
    }

    fn move_to_the_header(barcode_type: &str, read: &RefRecord, start: usize, end: usize) -> Result<Vec<u8>, Error> {
        // Convert head, seq, and qual to UTF-8 strings
        let read_header = read.head();
        let read_seq = read.seq();
        let read_qual = read.qual();
    
        // Extract barcode_seq and barcode_qual
        let barcode_seq = &read_seq[start..end];
        let barcode_qual = &read_qual[start..end];
    
        // Construct the result directly as bytes
        let mut result = Vec::with_capacity(read_header.len() + barcode_type.len() + barcode_seq.len() + barcode_qual.len() + 3);
        result.extend_from_slice(read_header);
        result.push(b' ');
        result.extend_from_slice(barcode_type.as_bytes());
        result.push(b':');
        result.extend_from_slice(barcode_seq);
        result.push(b':');
        result.extend_from_slice(barcode_qual);
    
        Ok(result)
    }
}

fn get_capture_group_indices(pattern: &str) -> HashMap<String, usize> {
    let re = Regex::new(pattern).unwrap();
    let mut group_indices = HashMap::new();
    
    for (i, name) in re.capture_names().enumerate() {
        if let Some(name) = name {
            group_indices.insert(name.to_string(), i);
        }
    }

    group_indices
}

fn remove_capture_groups(pattern: &str) -> String {
    let re = Regex::new(r"\?P<\w*>").unwrap();
    re.replace(pattern, "").to_string()
}