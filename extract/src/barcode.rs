use regex::{self, Regex, Captures};
use std::str;
use seq_io::fastq::{Record, RefRecord};

use crate::errors::Error;

pub struct Barcode {
    pattern_regex: Regex,
}

impl Barcode {
    pub fn new(pattern: &str) -> Result<Self, Error> {
        let pattern_regex = Regex::new(pattern).map_err(|_| Error::InvalidPattern(pattern.to_string()))?;
        Ok(Self { pattern_regex })
    }

    pub fn match_read<'a>(&self, read: &'a RefRecord) -> Result<Captures<'a>, Error> {
        let read_seq = str::from_utf8(read.seq())?;
        self.pattern_regex.captures(read_seq).ok_or_else(|| Error::PatternNotMatched(read_seq.to_string()))
    }

    pub fn cut_from_read_seq(barcode_type: &str, matched_pattern: Captures, read: &RefRecord) -> Result<(String, String, String), Error> {
        let start = matched_pattern.name(barcode_type).unwrap().start();
        let end = matched_pattern.name(barcode_type).unwrap().end();

        let read_seq = read.seq();
        let read_qual = read.qual();

        let new_read_seq = [&read_seq[..start], &read_seq[end..]].concat();
        let new_read_qual = [&read_qual[..start], &read_qual[end..]].concat();

        let new_read_header = Self::move_to_the_header(barcode_type, read, start, end)?;
        Ok((String::from_utf8(new_read_seq)?, String::from_utf8(new_read_qual)?, new_read_header))
    }

    fn move_to_the_header(barcode_type: &str, read: &RefRecord, start: usize, end: usize) -> Result<String, Error> {
        let read_header = str::from_utf8(read.head())?;
        let read_seq = read.seq();
        let read_qual = read.qual();

        let barcode_seq = str::from_utf8(&read_seq[start..end])?;
        let barcode_qual = str::from_utf8(&read_qual[start..end])?;

        Ok(format!("{} {}:{}:{}", read_header, barcode_type, barcode_seq, barcode_qual))
    }
}