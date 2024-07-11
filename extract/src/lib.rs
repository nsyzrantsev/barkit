mod reader;
mod errors;
mod extract;
mod pattern;

use std::io::Write;
use seq_io::fastq::RecordSet;

use rayon::prelude::*;
use std::sync::{Arc, Mutex};

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
    rc_barcodes: Option<bool>
) {
    match (pattern1, pattern2) {
        (Some(pattern1), Some(pattern2)) => process_pe_fastq(read1, read2.unwrap(), pattern1, pattern2, out_read1, out_read2.unwrap(), max_memory, rc_barcodes),
        (Some(pattern1), None) => process_se_fastq(read1, pattern1, out_read1, max_memory, rc_barcodes),
        (None, Some(pattern2)) => process_se_fastq(read2.unwrap(), pattern2, out_read2.unwrap(), max_memory, rc_barcodes),
        (None, None) => todo!(),
    }
}

fn process_se_fastq(
    read: String,
    pattern: String,
    out_read: String,
    max_memory: Option<usize>,
    rc_barcodes: Option<bool>
) {
    let barcode = BarcodeParser::new(&pattern, &rc_barcodes).expect("REASON");
    
    let mut reader = reader::FastqBufReader::new(&read, max_memory).unwrap();
    
    let fastq_writer = Arc::new(Mutex::new(reader::create_writer(&out_read)));

    while let Ok(records) = reader.read_record_set_exact(&mut RecordSet::default()) {
        records.into_par_iter().for_each(|record| {
            if let Ok(Some(capture)) = barcode.search_in_single_read(&record) {
                if let Ok(new_read) = BarcodeParser::cut_from_read_seq("UMI", capture, &record) {
                    let mut writer = fastq_writer.lock().unwrap();
                    if let Err(e) = seq_io::fastq::write_to(&mut *writer, new_read.head(), new_read.seq(), new_read.qual()) {
                        eprintln!("Failed to write to output file: {}", e);
                    }
                }
            }
        });
    }

    let mut writer = Arc::try_unwrap(fastq_writer).expect("Lock still has multiple owners").into_inner().expect("Mutex cannot be locked");
    writer.flush().expect("Failed to flush buffer");
    writer.into_inner().expect("Failed to finish compression").finish().expect("Failed to finish compression");
}


fn process_pe_fastq(
    read1: String, 
    read2: String, 
    pattern1: String,
    pattern2: String,
    out_read1: String, 
    out_read2: String,
    max_memory: Option<usize>,
    rc_barcodes: Option<bool>
) {
    let barcode1 = BarcodeParser::new(&pattern1, &rc_barcodes).expect("REASON");
    let barcode2 = BarcodeParser::new(&pattern2, &rc_barcodes).expect("REASON");
    
    let mut fastq1_reader = reader::FastqBufReader::new(&read1, max_memory).unwrap();
    let fastq1_writer = Arc::new(Mutex::new(reader::create_writer(&out_read1)));

    let mut fastq2_reader = reader::FastqBufReader::new(&read2, max_memory).unwrap();
    let fastq2_writer = Arc::new(Mutex::new(reader::create_writer(&out_read2)));

    while let Ok(records1) = fastq1_reader.read_record_set_exact(&mut RecordSet::default()) {
        if let Ok(records2) = fastq2_reader.read_record_set_exact(&mut RecordSet::default()) {
            records1.into_par_iter().zip(records2.into_par_iter()).for_each(|(record1, record2)| {

                let caps1 = barcode1.search_in_single_read(&record1);
                let caps2 = barcode2.search_in_single_read(&record2);
    
                let (new_record1, new_record2) = extract::replace_reads(&record1, &record2, &caps1.clone().unwrap().as_ref(), &caps2.clone().unwrap().as_ref()).unwrap();

                if let Ok(capture1) = caps1 {
                    let new_read1 = BarcodeParser::cut_from_read_seq("UMI", capture1.unwrap(), &new_record1).unwrap();
                    let mut writer1 = fastq1_writer.lock().unwrap();
                    if let Err(e) = seq_io::fastq::write_to(&mut *writer1, new_read1.head(), new_read1.seq(), new_read1.qual()) {
                        eprintln!("Failed to write to output file: {}", e);
                    }
                }
                
                if let Ok(capture2) = caps2 {
                    let new_read2 = BarcodeParser::cut_from_read_seq("UMI", capture2.unwrap(), &new_record2).unwrap();
                    let mut writer2 = fastq2_writer.lock().unwrap();
                    if let Err(e) = seq_io::fastq::write_to(&mut *writer2, new_read2.head(), new_read2.seq(), new_read2.qual()) {
                        eprintln!("Failed to write to output file: {}", e);
                    }
                }
            });
        } else {
            break;
        }
    }
    let mut writer1 = Arc::try_unwrap(fastq1_writer).expect("Lock still has multiple owners").into_inner().expect("Mutex cannot be locked");
    writer1.flush().expect("Failed to flush buffer");
    writer1.into_inner().expect("Failed to finish compression").finish().expect("Failed to finish compression");

    let mut writer2 = Arc::try_unwrap(fastq2_writer).expect("Lock still has multiple owners").into_inner().expect("Mutex cannot be locked");
    writer2.flush().expect("Failed to flush buffer");
    writer2.into_inner().expect("Failed to finish compression").finish().expect("Failed to finish compression");
}