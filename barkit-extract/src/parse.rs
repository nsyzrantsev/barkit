#![allow(clippy::result_large_err)]

use crate::pattern::BarcodeRegex;
use regex::bytes::Captures;

use seq_io::fastq::{OwnedRecord, Record, RefRecord};
use std::str;

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

pub struct BarcodeParser {
    /// Prepared regex pattern to parse barcodes
    barcode_regex: BarcodeRegex,

    /// If `true`, all captured patterns will not be trimmed
    skip_trimming: bool,

    /// If `true`, the barcode pattern will also be matched in the reverse complement sequence.
    rc_barcodes: bool,
}

impl BarcodeParser {
    pub fn new(
        barcode_regex: Option<BarcodeRegex>,
        skip_trimming: bool,
        rc_barcodes: bool,
    ) -> Option<Self> {
        barcode_regex.map(|regex| BarcodeParser {
            barcode_regex: regex,
            skip_trimming,
            rc_barcodes,
        })
    }

    pub fn extract_barcodes(&self, record: &RefRecord) -> Option<OwnedRecord> {
        let read_captures = self.barcode_regex.get_captures(record.seq());
        let read_seq_rc: Vec<u8>;
        let read_captures = if read_captures.is_err() && self.rc_barcodes {
            read_seq_rc = get_reverse_complement(record.seq());
            self.barcode_regex.get_captures(&read_seq_rc)
        } else {
            read_captures
        };
        self.create_new_read(read_captures.map(Some), record)
    }

    fn create_new_read(
        &self,
        read_captures: Result<Option<Captures>, Error>,
        record: &RefRecord,
    ) -> Option<seq_io::fastq::OwnedRecord> {
        match (read_captures, self.skip_trimming) {
            (Ok(Some(captures)), true) => {
                Some(self.get_read_with_new_header(&captures, record).ok()?)
            }
            (Ok(Some(captures)), false) => {
                let new_read = self.get_read_with_new_header(&captures, record).ok()?;
                Some(trim_adapters(captures, &new_read).ok()?)
            }
            (Ok(None), _) => Some(OwnedRecord {
                head: record.head().to_vec(),
                seq: record.seq().to_vec(),
                qual: record.qual().to_vec(),
            }),
            (Err(_), _) => None,
        }
    }

    fn get_read_with_new_header(
        &self,
        captures: &Captures,
        record: &RefRecord,
    ) -> Result<OwnedRecord, Error> {
        let mut head = record.head().to_vec();
        let seq = record.seq().to_vec();
        let qual = record.qual().to_vec();

        for barcode in &self.barcode_regex.get_barcode_types() {
            let barcode_name = barcode.to_string();
            let (barcode_start, barcode_end) =
                get_barcode_match_positions(&barcode_name, captures)?;
            head = add_to_the_header(
                &barcode_name,
                &head,
                &seq,
                &qual,
                barcode_start,
                barcode_end,
            )?;
        }

        Ok(OwnedRecord { head, seq, qual })
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

fn trim_adapters(captures: Captures, record: &OwnedRecord) -> Result<OwnedRecord, Error> {
    let (start, end) = get_full_match_positions(&captures)?;
    let seq = [&record.seq()[..start], &record.seq()[end..]].concat();
    let qual = [&record.qual()[..start], &record.qual()[end..]].concat();

    Ok(OwnedRecord {
        head: record.head().to_vec(),
        seq,
        qual,
    })
}

fn add_to_the_header(
    barcode_type: &str,
    head: &[u8],
    seq: &[u8],
    qual: &[u8],
    start: usize,
    end: usize,
) -> Result<Vec<u8>, Error> {
    let barcode_seq = &seq[start..end];
    let barcode_qual = &qual[start..end];

    let mut result = Vec::with_capacity(
        head.len() + barcode_type.len() + barcode_seq.len() + barcode_qual.len() + 3,
    );
    result.extend_from_slice(head);
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

    use crate::parse::get_reverse_complement;

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
