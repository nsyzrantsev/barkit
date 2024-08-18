use regex::bytes::{Captures, Match, Regex};
use crate::pattern;

use std::str;
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

pub enum BarcodeType {
    UMI
}

impl BarcodeType {
    pub fn to_string(&self) -> String {
        match *self {
            BarcodeType::UMI => "UMI".to_string()
        }
    }
}

pub struct BarcodeParser {
    regex: Regex
}

impl BarcodeParser {
    pub fn new(pattern: &str, max_error: usize) -> Result<Self, Error> {
        let fuzzy_pattern = pattern::create_fuzzy(&pattern, &max_error);
        let regex = Regex::new(&fuzzy_pattern)?;
        Ok(Self {
            regex,
        })
    }


    pub fn get_captures<'a>(&'a self, read_seq: &'a [u8]) -> Result<Captures, Error> {
        match self.regex.captures(read_seq) {
            Some(capture) => Ok(capture),
            None => Err(Error::PatternNotMatched)
        }
    }

    fn get_full_match_positions(captures: &Captures) -> Result<(usize, usize), Error> {
        let full_match = captures
            .get(0)
            .ok_or(Error::BarcodeCaptureGroupNotFound("0".to_owned()))?;

        Ok((full_match.start(), full_match.end()))
    }

    fn get_barcode_match_positions(barcode_name: &str, captures: &Captures) -> Result<(usize, usize), Error> {
        let full_match = captures
            .name(barcode_name)
            .ok_or(Error::BarcodeCaptureGroupNotFound("0".to_owned()))?;

        Ok((full_match.start(), full_match.end()))
    }

    pub fn get_new_read_with_adapter_trimming(barcode_type: &str, captures: Captures, read: &RefRecord) -> Result<OwnedRecord, Error> {
        let (start, end) = Self::get_full_match_positions(&captures)?;
        let (umi_start, umi_end) = Self::get_barcode_match_positions(&BarcodeType::UMI.to_string(), &captures)?;

        let read_seq = read.seq();
        let read_qual = read.qual();
        
        Ok(OwnedRecord {
            head: Self::add_to_the_header(barcode_type, read, umi_start, umi_end)?,
            seq: [&read_seq[..start], &read_seq[end..]].concat(),
            qual: [&read_qual[..start], &read_qual[end..]].concat(),
        })
    }

    pub fn get_new_read_without_adapter_trimming(barcode_name: &str, captures: Captures, record: &RefRecord) -> Result<OwnedRecord, Error> {
        let (umi_start, umi_end) = Self::get_barcode_match_positions(
            &BarcodeType::UMI.to_string(), 
            &captures
        )?;
        Ok(OwnedRecord {
            head: Self::add_to_the_header(
                &barcode_name,
                record,
                umi_start,
                umi_end
            )?,
            seq: record.seq().to_vec(),
            qual: record.qual().to_vec(),
        })
    }

    pub fn create_new_read(read_captures: Result<Option<Captures>, Error>, record: &RefRecord, skip_trimming: bool) -> Option<seq_io::fastq::OwnedRecord> {
        let umi_capture_group_name = BarcodeType::UMI.to_string();
        match (read_captures, skip_trimming) {
            (Ok(Some(captures)), true) => {
                Some(Self::get_new_read_with_adapter_trimming(
                    &umi_capture_group_name,
                    captures,
                    record).ok()?
                )
            },
            (Ok(Some(captures)), false) => {
                Some(Self::get_new_read_without_adapter_trimming(
                    &umi_capture_group_name,
                    captures,
                    record).ok()?
                )
            }
            (Ok(None), _) => Some(OwnedRecord {
                head: record.head().to_vec(),
                seq: record.seq().to_vec(),
                qual: record.qual().to_vec(),
            }),
            (Err(_), _) => None,
        }
    }

    fn add_to_the_header(barcode_type: &str, read: &RefRecord, start: usize, end: usize) -> Result<Vec<u8>, Error> {
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

pub fn get_reverse_complement(sequence: &[u8]) -> Vec<u8> {
    sequence
        .iter()
        .rev()
        .map(|&base| TRANSLATION_TABLE[base as usize])
        .collect()
}

pub fn replace_reads(
    read1: OwnedRecord, 
    read2: OwnedRecord, 
    read1_match: Result<Option<Match>, Error>,
    read2_match: Result<Option<Match>, Error>
) -> Option<(OwnedRecord, OwnedRecord)> {
    let replace_result = match (read1_match, read2_match) {
        (Ok(_), _) => Ok((read1, read2)),
        (Err(_), Ok(_)) => Ok((OwnedRecord {
            head: read1.head,
            seq: read2.seq,
            qual: read2.qual
        }, OwnedRecord {
            head: read2.head,
            seq: read1.seq,
            qual: read1.qual
        })),
        _ => Err(Error::BothReadsNotMatch),
    };
    match replace_result {
        Ok((record1, record2)) => Some((record1, record2)),
        Err(_) => None,
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