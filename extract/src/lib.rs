mod io;
mod errors;
mod extract;
mod pattern;

use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter};
use std::path::Path;

use seq_io::fastq::Reader;

use extract::BarcodeParser;
use seq_io::fastq::Record;

use seq_io::parallel::parallel_fastq;


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
    match (pattern1, pattern2) {
        (Some(pattern1), Some(pattern2)) => process_pe_fastq(read1, read2.unwrap(), pattern1, pattern2, out_read1, out_read2.unwrap(), max_memory, threads.unwrap(), rc_barcodes, max_error),
        (Some(pattern1), None) => process_se_fastq(read1, pattern1, out_read1, max_memory, threads, rc_barcodes, max_error),
        (None, _) => todo!()
    }
}

fn process_se_fastq(
    read: String,
    pattern: String,
    out_read: String,
    max_memory: Option<usize>,
    threads: Option<usize>,
    rc_barcodes: Option<bool>,
    max_error: Option<usize>
) {
    let barcode = BarcodeParser::new(&pattern, &rc_barcodes, max_error.unwrap()).expect("REASON");
    
    // let mut reader = io::create_reader(&read, max_memory).unwrap();
    // let mut writer = io::create_writer(&out_read);

    let reader = Reader::new(BufReader::with_capacity(128 * 1024 * 1024, File::open(&Path::new(&read)).expect("couldn't open file")));

    let mut writer = BufWriter::new(File::create("filtered.fastq").unwrap());

    parallel_fastq(reader, threads.unwrap() as u32, 10,
        |record, found| {
            *found = barcode.search_in_single_read(&record).is_ok();
        },
        |record, found| { // runs in main thread
            if *found {
                record.write(&mut writer).unwrap();
            }
            None::<()>
    }).unwrap();
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
    max_error: Option<usize>
) {}