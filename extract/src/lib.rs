mod io;
mod errors;
mod extract;
mod pattern;

use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};

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
    let max_error = max_error.unwrap_or(1);
    let threads = threads.unwrap_or(1);

    let barcode = BarcodeParser::new(&pattern, &rc_barcodes, max_error).expect("REASON");
    
    let reader = Reader::new(BufReader::with_capacity(128 * 1024 * 1024, File::open(&Path::new(&read)).expect("couldn't open file")));
    // let reader: Arc<Mutex<Reader<Box<dyn BufRead>>>> = Arc::new(Mutex::new(io::create_reader(&read, max_memory).expect("Failed to create reader")));
    let writer = io::create_writer(&out_read).expect("Failed to create writer");

    parallel_fastq(reader, threads as u32, 10,
        |record, found| {
            *found = barcode.search_in_single_read(&record).is_ok();
        },
        |record, found| {
            if *found {
                let mut writer = writer.lock().unwrap();
                record.write(&mut *writer).expect("Failed to write record");
            }
            None::<()>
    }).expect("Failed to process fastq in parallel");
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