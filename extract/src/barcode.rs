use crate::pattern;
use regex::bytes::{Captures, Regex};

use seq_io::fastq::{OwnedRecord, Record, RefRecord};
use std::{fmt, str};

use crate::error::Error;

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

pub enum BarcodeType {
    Umi,
    Sample,
}

impl fmt::Display for BarcodeType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BarcodeType::Umi => write!(f, "UMI"),
            BarcodeType::Sample => write!(f, "SB"),
        }
    }
}

pub struct BarcodeParser {
    regex: Regex,
}

impl BarcodeParser {
    pub fn new(pattern: &str, max_error: usize) -> Result<Self, Error> {
        let fuzzy_pattern = pattern::get_with_errors(pattern, &max_error);
        let regex = Regex::new(&fuzzy_pattern)?;
        Ok(Self { regex })
    }

    pub fn get_captures<'a>(&'a self, read_seq: &'a [u8]) -> Result<Captures, Error> {
        match self.regex.captures(read_seq) {
            Some(capture) => Ok(capture),
            None => Err(Error::PatternNotMatched),
        }
    }
}

fn get_full_match_positions(captures: &Captures) -> Result<(usize, usize), Error> {
    let full_match = captures
        .get(0)
        .ok_or(Error::BarcodeCaptureGroupNotFound("0".to_owned()))?;

    Ok((full_match.start(), full_match.end()))
}

fn get_barcode_match_positions(
    barcode_name: &str,
    captures: &Captures,
) -> Result<(usize, usize), Error> {
    let full_match = captures
        .name(barcode_name)
        .ok_or(Error::BarcodeCaptureGroupNotFound(barcode_name.to_string()))?;

    Ok((full_match.start(), full_match.end()))
}

fn create_owned_record(head: Vec<u8>, seq: Vec<u8>, qual: Vec<u8>) -> Result<OwnedRecord, Error> {
    Ok(OwnedRecord { head, seq, qual })
}

fn get_umi_positions(captures: &Captures) -> Result<(usize, usize), Error> {
    get_barcode_match_positions(&BarcodeType::Umi.to_string(), captures)
}

fn get_sample_barcode_positions(captures: &Captures) -> Result<(usize, usize), Error> {
    get_barcode_match_positions(&BarcodeType::Sample.to_string(), captures)
}

fn trim_adapters(
    captures: Captures,
    read: &OwnedRecord,
) -> Result<OwnedRecord, Error> {
    let (start, end) = get_full_match_positions(&captures)?;
    let seq = [&read.seq()[..start], &read.seq()[end..]].concat();
    let qual = [&read.qual()[..start], &read.qual()[end..]].concat();

    create_owned_record(read.head().to_vec(), seq, qual)
}

fn get_read_with_new_header(
    barcode_name: &str,
    captures: &Captures,
    record: &RefRecord,
) -> Result<OwnedRecord, Error> {
    let (umi_start, umi_end) = get_umi_positions(&captures)?;
    // let (sample_barcode_start, sample_barcode_end) = get_sample_barcode_positions(&captures)?;

    let head = add_to_the_header(barcode_name, record, umi_start, umi_end)?;
    let seq = record.seq().to_vec();
    let qual = record.qual().to_vec();

    create_owned_record(head, seq, qual)
}

pub fn create_new_read(
    read_captures: Result<Option<Captures>, Error>,
    record: &RefRecord,
    skip_trimming: bool,
) -> Option<seq_io::fastq::OwnedRecord> {
    let umi_capture_group_name = BarcodeType::Umi.to_string();
    match (read_captures, skip_trimming) {
        (Ok(Some(captures)), true) => Some(get_read_with_new_header(
            &umi_capture_group_name, &captures, record
        ).ok()?),
        (Ok(Some(captures)), false) => {
            let new_read = get_read_with_new_header(
                &umi_capture_group_name, &captures, record
            ).ok()?;
            Some(trim_adapters(captures, &new_read).ok()?)
        },
        (Ok(None), _) => Some(OwnedRecord {
            head: record.head().to_vec(),
            seq: record.seq().to_vec(),
            qual: record.qual().to_vec(),
        }),
        (Err(_), _) => None,
    }
}

fn add_to_the_header(
    barcode_type: &str,
    read: &RefRecord,
    start: usize,
    end: usize,
) -> Result<Vec<u8>, Error> {
    let read_header = read.head();
    let barcode_seq = &read.seq()[start..end];
    let barcode_qual = &read.qual()[start..end];

    let mut result = Vec::with_capacity(
        read_header.len() + barcode_type.len() + barcode_seq.len() + barcode_qual.len() + 3,
    );
    result.extend_from_slice(read_header);
    result.extend_from_slice(
        format!(" {}:{}:", barcode_type, std::str::from_utf8(barcode_seq)?).as_bytes(),
    );
    result.extend_from_slice(barcode_qual);

    Ok(result)
}

pub fn get_reverse_complement(sequence: &[u8]) -> Vec<u8> {
    sequence
        .iter()
        .rev()
        .map(|&base| TRANSLATION_TABLE[base as usize])
        .collect()
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::barcode::get_reverse_complement;

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
