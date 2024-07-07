use fuzzy_regex::fuzzy::{FuzzyRegex, FuzzyMatch, Match};
use crate::pattern::Pattern;

use std::{collections::HashMap, str};
use seq_io::fastq::{Record, RefRecord, OwnedRecord};

use crate::errors::Error;

const TRANSLATION_TABLE: [u8; 256] = {
    let mut table = [b'A'; 256];

    table[b'A' as usize] = b'T';
    table[b'T' as usize] = b'A';
    table[b'G' as usize] = b'C';
    table[b'C' as usize] = b'G';
    table[b'R' as usize] = b'A';
    table[b'Y' as usize] = b'A';
    table[b'S' as usize] = b'A';
    table[b'W' as usize] = b'A';
    table[b'K' as usize] = b'A';
    table[b'M' as usize] = b'A';
    table[b'B' as usize] = b'A';
    table[b'D' as usize] = b'A';
    table[b'H' as usize] = b'A';
    table[b'V' as usize] = b'A';
    table[b'N' as usize] = b'A';
    
    table
};

pub struct BarcodeExtractor {
    regex: FuzzyRegex,
    capture_groups: HashMap<String, usize>,
    rc_barcodes: bool
}

impl BarcodeExtractor {
    pub fn new(pattern: &str, rc_barcodes: &Option<bool>) -> Result<Self, Error> {
        let escaped_pattern = Pattern::new(pattern)?;
        let capture_groups = escaped_pattern.get_group_indices()?;
        let posix_pattern = escaped_pattern.clear()?;
        let regex = FuzzyRegex::new(&posix_pattern)?;
        Ok(Self {
            regex,
            capture_groups,
            rc_barcodes: rc_barcodes.unwrap_or(false)
        })
    }

    fn capture_barcodes(&self, read_seq: &[u8]) -> Result<FuzzyMatch<Vec<u8>, Match>, Error> {
        let capture = self.regex.captures(&read_seq, 3);
        if self.rc_barcodes {
            return match capture {
                Ok(fuzzy_match) => Ok(fuzzy_match),
                Err(_) => {
                    let read_seq_rc = get_reverse_complement(&read_seq);
                    Ok(self.regex.captures(&read_seq_rc, 3)?)
                }
            }
        }
        Ok(capture?)
    }

    pub fn search_in_read(&self, read: &OwnedRecord) -> Result<Option<Match>, Error> {
        let read_seq = read.seq();
        let captures = self.capture_barcodes(read_seq)?;
    
        let matches = captures.get_matches();
        let capture_group_index = self.capture_groups["UMI"];
    
        let result = matches
            .get(capture_group_index)
            .map_or(Err(Error::CaptureGroupIndexError(capture_group_index)), Ok)?;
        
        Ok(result.clone())
    }

    pub fn search_in_reads(&self, read1: &OwnedRecord, read2: &OwnedRecord) {
        let mut matched_read1 = self.search_in_read(&read1);
        let mut matched_read2 = self.search_in_read(read2);
    }

    pub fn cut_from_read_seq(barcode_type: &str, matched_pattern: Match, read: &OwnedRecord) -> Result<OwnedRecord, Error> {
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

    fn move_to_the_header(barcode_type: &str, read: &OwnedRecord, start: usize, end: usize) -> Result<Vec<u8>, Error> {
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


fn get_reverse_complement(sequence: &[u8]) -> Vec<u8> {
    let sequence_rc: Vec<u8> = sequence
        .iter()
        .map(|&base| TRANSLATION_TABLE[base as usize])
        .rev()
        .collect();
    sequence_rc[..].to_vec()
}