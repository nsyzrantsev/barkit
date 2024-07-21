use regex::bytes::{Captures, Match, Regex};
use crate::pattern;

use std::{collections::HashMap, str};
use seq_io::fastq::{OwnedRecord, Record, RefRecord};

use crate::errors::Error;

/// https://www.bioinformatics.org/sms/iupac.html
const TRANSLATION_TABLE: [u8; 256] = {
    let mut table = [b'A'; 256];

    table[b'A' as usize] = b'T';
    table[b'T' as usize] = b'A';
    table[b'G' as usize] = b'C';
    table[b'C' as usize] = b'G';
    table[b'R' as usize] = b'R';
    table[b'Y' as usize] = b'Y';
    table[b'S' as usize] = b'S';
    table[b'W' as usize] = b'W';
    table[b'K' as usize] = b'K';
    table[b'M' as usize] = b'M';
    table[b'B' as usize] = b'B';
    table[b'D' as usize] = b'D';
    table[b'H' as usize] = b'H';
    table[b'V' as usize] = b'V';
    table[b'N' as usize] = b'N';
    
    table
};

enum BarcodeType {
    UMI
}

impl BarcodeType {
    fn to_string(&self) -> String {
        match *self {
            BarcodeType::UMI => "UMI".to_string(),
        }
    }
}

pub struct BarcodeParser {
    regex: Regex,
    rc_barcodes: bool
}

impl BarcodeParser {
    pub fn new(pattern: &str, rc_barcodes: &Option<bool>, max_error: usize) -> Result<Self, Error> {
        let fuzzy_pattern = pattern::create_fuzzy(&pattern, &max_error);
        let regex = Regex::new(&fuzzy_pattern)?;
        Ok(Self {
            regex,
            rc_barcodes: rc_barcodes.unwrap_or(false)
        })
    }

    fn capture_barcodes<'a>(&'a self, read_seq: &'a [u8]) -> Result<Captures<'a>, Error> {
        let captures = self.regex.captures(read_seq);
    
        // if self.search_in_barcodes_in_rc && captures.is_none() {
        //     let read_seq_rc = get_reverse_complement(read_seq);
        //     let captures_rc = self.regex.captures(&read_seq_rc);   
        //     return captures_rc.ok_or(Error::PatternNotMatched);
        // }
    
        captures.ok_or(Error::PatternNotMatched)
    }

    pub fn search_in_single_read<'a>(&'a self, read: &'a RefRecord) -> Result<Match, Error> {
        let read_seq = read.seq();
        let captures = self.capture_barcodes(read_seq)?;
    
        captures.name(&BarcodeType::UMI.to_string()).ok_or(Error::UMIPatternNotFound)
    }

    pub fn cut_from_read_seq(barcode_type: &str, matched_pattern: Match, read: &RefRecord) -> Result<OwnedRecord, Error> {
        let start = matched_pattern.start();
        let end = matched_pattern.end();

        let read_seq = read.seq();
        let read_qual = read.qual();
        
        Ok(OwnedRecord {
            head: Self::move_to_the_header(barcode_type, read, start, end)?,
            seq: [&read_seq[..start], &read_seq[end..]].concat(),
            qual: [&read_qual[..start], &read_qual[end..]].concat(),
        })
    }

    fn move_to_the_header(barcode_type: &str, read: &RefRecord, start: usize, end: usize) -> Result<Vec<u8>, Error> {
        let read_header = read.head();
        let barcode_seq = &read.seq()[start..end];
        let barcode_qual = &read.qual()[start..end];
    
        let mut result = Vec::with_capacity(read_header.len() + barcode_type.len() + barcode_seq.len() + barcode_qual.len() + 3);
        result.extend_from_slice(read_header);
        result.extend_from_slice(format!(" {}:{}:", barcode_type, std::str::from_utf8(barcode_seq)?).as_bytes());
        result.extend_from_slice(barcode_qual);
    
        Ok(result)
    }
}


fn get_reverse_complement(sequence: &[u8]) -> Vec<u8> {
    sequence
        .iter()
        .rev()
        .map(|&base| TRANSLATION_TABLE[base as usize])
        .collect()
}

pub fn replace_reads<'a>(
    read1: &RefRecord<'a>, 
    read2: &RefRecord<'a>, 
    read1_match: &Option<&Match>,
    read2_match: &Option<&Match>
) -> Result<(RefRecord<'a>, RefRecord<'a>), Error> {        
    match (read1_match, read2_match) {
        (Some(_), _) => Ok((read1.clone(), read2.clone())),
        (None, Some(_)) => Ok((read2.clone(), read1.clone())),
        _ => Err(Error::BothReadsNotMatch),
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::extract::get_reverse_complement;

    #[rstest]
    #[case(b"", b"")]
    #[case(b"GGGCCCAAATTT", b"AAATTTGGGCCC")]
    #[case(b"ATGCN", b"NGCAT")]
    #[case(b"AAP", b"ATT")]
    #[case(b"CCX", b"AGG")]
    #[case(b"PPP", b"AAA")]
    fn test_get_reverse_complement(#[case] sequence: &[u8], #[case] rc_sequence: &[u8]) {
        assert_eq!(get_reverse_complement(sequence), rc_sequence);
    }
}