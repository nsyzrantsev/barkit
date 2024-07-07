mod fastq;
mod errors;
mod extract;
mod pattern;

use std::io::Write;

use rayon::prelude::*;
use std::sync::{Arc, Mutex};

use extract::BarcodeExtractor;
use seq_io::fastq::Record;


pub fn run(
    read1: String, 
    read2: Option<String>, 
    pattern1: String,
    pattern2: Option<String>,
    out_read1: String, 
    out_read2: Option<String>,
    max_memory: Option<usize>
) {
    process_se_fastq(read1, pattern1, out_read1, max_memory);

    // let out_read2 = out_read2.unwrap_or_else(|| {
    //     eprintln!("{}", errors::Error::OutputFastqFileNotProvided);
    //     std::process::exit(1); // or handle the error appropriately
    // });
}

fn process_se_fastq(
    read: String,
    pattern: String,
    out_read: String,
    max_memory: Option<usize>
) {
    let barcode = BarcodeExtractor::new(&pattern).expect("REASON");
    
    let mut fastq_reader = fastq::create_reader(&read, max_memory).unwrap();
    
    let fastq_writer = Arc::new(Mutex::new(fastq::create_writer(&out_read)));

    let records: Vec<_> = fastq_reader.records().collect();
    records.into_par_iter().for_each(|record| {
        let record = record.expect("Error reading record");
        let caps = barcode.match_read(&record);

        if let Ok(capture) = caps {
            let new_read = BarcodeExtractor::cut_from_read_seq("UMI", capture.unwrap(), &record).unwrap();
            let mut writer = fastq_writer.lock().unwrap();
            if let Err(e) = seq_io::fastq::write_to(
                &mut *writer, 
                new_read.head(), 
                new_read.seq(), 
                new_read.qual()
            ) {
                eprintln!("Failed to write to output file: {}", e);
            }
        }
    });

    let mut writer = Arc::try_unwrap(fastq_writer).expect("Lock still has multiple owners").into_inner().expect("Mutex cannot be locked");
    writer.flush().expect("Failed to flush buffer");
    writer.into_inner().expect("Failed to finish compression").finish().expect("Failed to finish compression");
}