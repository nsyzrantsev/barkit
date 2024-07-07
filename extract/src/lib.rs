mod fastq;
mod errors;
mod extract;
mod pattern;

use std::io::Write;

use rayon::prelude::*;
use std::sync::{Arc, Mutex};

use extract::BarcodeParser;
use seq_io::fastq::Record;


pub fn run(
    read1: String, 
    read2: Option<String>, 
    pattern1: String,
    pattern2: Option<String>,
    out_read1: String, 
    out_read2: Option<String>,
    max_memory: Option<usize>,
    rc_barcodes: Option<bool>
) {
    // process_se_fastq(read1, pattern1, out_read1, max_memory, rc_barcodes);

    process_pe_fastq(read1, read2, pattern1, pattern2, out_read1, out_read2, max_memory, rc_barcodes)

    // let out_read2 = out_read2.unwrap_or_else(|| {
    //     eprintln!("{}", errors::Error::OutputFastqFileNotProvided);
    //     std::process::exit(1); // or handle the error appropriately
    // });
}

fn process_se_fastq(
    read: String,
    pattern: String,
    out_read: String,
    max_memory: Option<usize>,
    rc_barcodes: Option<bool>
) {
    let barcode = BarcodeParser::new(&pattern, &rc_barcodes).expect("REASON");
    
    let mut fastq_reader = fastq::create_reader(&read, max_memory).unwrap();
    
    let fastq_writer = Arc::new(Mutex::new(fastq::create_writer(&out_read)));

    let records: Vec<_> = fastq_reader.records().collect();
    records.into_par_iter().for_each(|record| {
        let record = record.expect("Error reading record");
        let caps = barcode.search_in_single_read(&record);

        if let Ok(capture) = caps {
            let new_read = BarcodeParser::cut_from_read_seq("UMI", capture.unwrap(), &record).unwrap();
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


fn process_pe_fastq(
    read1: String, 
    read2: Option<String>, 
    pattern1: String,
    pattern2: Option<String>,
    out_read1: String, 
    out_read2: Option<String>,
    max_memory: Option<usize>,
    rc_barcodes: Option<bool>
) {
    let barcode1 = BarcodeParser::new(&pattern1, &rc_barcodes).expect("REASON");
    let barcode2 = BarcodeParser::new(&pattern2.unwrap(), &rc_barcodes).expect("REASON");
    
    let mut fastq1_reader = fastq::create_reader(&read1, max_memory).unwrap();
    let fastq1_writer = Arc::new(Mutex::new(fastq::create_writer(&out_read1)));

    let mut fastq2_reader = fastq::create_reader(&read2.unwrap(), max_memory).unwrap();
    let fastq2_writer = Arc::new(Mutex::new(fastq::create_writer(&out_read2.unwrap())));

    let records1: Vec<_> = fastq1_reader.records().collect();
    let records2: Vec<_> = fastq2_reader.records().collect();

    records1.into_par_iter().zip(records2.into_par_iter()).for_each(|(record1, record2)| {
        let record1 = record1.expect("Error reading record");
        let record2 = record2.expect("Error reading record");

        let caps1: Result<Option<fuzzy_regex::fuzzy::Match>, errors::Error> = barcode1.search_in_single_read(&record1);
        let caps2 = barcode2.search_in_single_read(&record2);

        let (new_record1, new_record2) = extract::replace_reads(&record1, &record2, &caps1.clone().unwrap().as_ref(), &caps2.clone().unwrap().as_ref()).unwrap();

        if let Ok(capture1) = caps1 {
            let new_read1 = BarcodeParser::cut_from_read_seq("UMI", capture1.unwrap(), &new_record1).unwrap();
            let mut writer1 = fastq1_writer.lock().unwrap();
            if let Err(e) = seq_io::fastq::write_to(
                &mut *writer1,
                new_read1.head(), 
                new_read1.seq(), 
                new_read1.qual()
            ) {
                eprintln!("Failed to write to output file: {}", e);
            }
        }

        if let Ok(capture2) = caps2 {
            let new_read2 = BarcodeParser::cut_from_read_seq("UMI", capture2.unwrap(), &new_record2).unwrap();
            let mut writer2 = fastq2_writer.lock().unwrap();
            if let Err(e) = seq_io::fastq::write_to(
                &mut *writer2, 
                new_read2.head(), 
                new_read2.seq(), 
                new_read2.qual()
            ) {
                eprintln!("Failed to write to output file: {}", e);
            }
        }
    });

    let mut writer1 = Arc::try_unwrap(fastq1_writer).expect("Lock still has multiple owners").into_inner().expect("Mutex cannot be locked");
    writer1.flush().expect("Failed to flush buffer");
    writer1.into_inner().expect("Failed to finish compression").finish().expect("Failed to finish compression");

    let mut writer2 = Arc::try_unwrap(fastq2_writer).expect("Lock still has multiple owners").into_inner().expect("Mutex cannot be locked");
    writer2.flush().expect("Failed to flush buffer");
    writer2.into_inner().expect("Failed to finish compression").finish().expect("Failed to finish compression");
}