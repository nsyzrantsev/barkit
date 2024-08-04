mod io;
mod errors;
mod extract;
mod pattern;

use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use seq_io::fastq::{OwnedRecord, Reader, RecordSet, RefRecord};

use extract::BarcodeParser;
use seq_io::fastq::Record;


pub fn run(
    read1: String, 
    read2: Option<String>, 
    pattern1: Option<String>,
    pattern2: Option<String>,
    out_read1: String, 
    out_read2: Option<String>,
    max_memory: Option<usize>,
    threads: Option<usize>,
    rc_barcodes: Option<bool>,
    max_error: Option<usize>
) {
    let max_error = max_error.unwrap_or(1);
    let threads = threads.unwrap_or(1);
    match (pattern1, pattern2) {
        (Some(pattern1), Some(pattern2)) => process_pe_fastq(read1, read2.unwrap(), pattern1, pattern2, out_read1, out_read2.unwrap(), max_memory, threads, rc_barcodes, max_error),
        (Some(pattern1), None) => process_se_fastq(read1, pattern1, out_read1, max_memory, threads, rc_barcodes, max_error),
        (None, _) => todo!()
    }
}

fn process_se_fastq(
    read: String,
    pattern: String,
    out_read: String,
    max_memory: Option<usize>,
    threads: usize,
    rc_barcodes: Option<bool>,
    max_error: usize
) {
    let barcode = BarcodeParser::new(&pattern, &rc_barcodes, max_error).expect("REASON");

    let mut reader = io::create_reader(&read, max_memory).expect("Failed to create reader");
    let writer = io::create_writer(&out_read).expect("Failed to create writer");

    // let mut found = barcode.search_in_single_read(&record.seq()).is_ok();
    // let read_seq_rc = extract::get_reverse_complement(record.seq());
    // let found = barcode.search_in_single_read(&read_seq_rc).is_ok();

    let mut records_number = 500_000;
    let mut records: Vec<OwnedRecord> = vec![];
    while let Some(result) = reader.next() {
        match result {
            Ok(record) => {
                records.push(record.to_owned_record());
            },
            Err(e) => eprintln!("Error reading record: {}", e),
        }
        records_number -= 1;
        if records_number <= 0 {
            records.par_iter().for_each(|record| {
                let found = barcode.search_in_single_read(&record.seq()).is_ok();
            });
            records.clear();
        }
    }
}


fn process_pe_fastq(
    read1: String,
    read2: String,
    pattern1: String,
    pattern2: String,
    out_read1: String,
    out_read2: String,
    max_memory: Option<usize>,
    threads: usize,
    rc_barcodes: Option<bool>,
    max_error: usize
) {}