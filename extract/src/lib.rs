mod io;
mod errors;
mod extract;
mod pattern;

use rayon::prelude::*;
use seq_io::fastq::{Record, RecordSet};

use extract::{BarcodeParser, BarcodeType};

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
    max_error: Option<usize>,
    compression_format: String
) {
    let max_error = max_error.unwrap_or(1);
    let threads = threads.unwrap_or(1);
    let rc_barcodes = rc_barcodes.unwrap_or(false);
    match (pattern1, pattern2) {
        (Some(pattern1), Some(pattern2)) => process_pe_fastq(
            read1, 
            read2.unwrap(),
            pattern1,
            pattern2,
            out_read1,
            out_read2.unwrap(),
            max_memory,
            threads,
            rc_barcodes,
            max_error,
            compression_format
        ),
        (Some(pattern1), None) => process_se_fastq(
            read1,
            pattern1,
            out_read1,
            max_memory,
            threads,
            rc_barcodes,
            max_error,
            compression_format
        ),
        (None, _) => todo!()
    }
}

fn process_se_fastq(
    read: String,
    pattern: String,
    out_read: String,
    max_memory: Option<usize>,
    threads: usize,
    rc_barcodes: bool,
    max_error: usize,
    compression_format: String
) {
    let barcode = BarcodeParser::new(&pattern, &rc_barcodes, max_error).expect("REASON");

    let mut reader = io::create_reader(&read, threads, max_memory).expect("Failed to create reader");
    let writer = io::create_writer(&out_read, &compression_format).expect("Failed to create writer");

    loop {
        let mut record_set = RecordSet::default();

        let filled_set = reader.read_record_set(&mut record_set);

        if filled_set.is_none() {
            break;
        } else {
            let result_reads: Vec<_> = record_set
            .into_iter()
            .collect::<Vec<_>>()
            .par_iter()
            .filter_map(|record| {
                let matched_read = barcode.search_in_single_read(&record.seq());
                let read_seq_rc: Vec<u8>;
                let matched_read = if matched_read.is_err() && rc_barcodes {
                    read_seq_rc = extract::get_reverse_complement(record.seq());
                    barcode.search_in_single_read(&read_seq_rc)
                } else {
                    matched_read
                };
                match matched_read {
                    Ok(match_val) => {
                        Some(BarcodeParser::cut_from_read_seq(
                            &BarcodeType::UMI.to_string(),
                            match_val,
                            record).unwrap())
                    },
                    Err(_) => None
                }
            }).collect();
            
            let mut writer = writer.lock().unwrap();
            for read_record in result_reads {
                let _ = seq_io::fastq::write_to(
                    &mut *writer,
                    &read_record.head(),
                    &read_record.seq(),
                    &read_record.qual()
                );
            }
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
    rc_barcodes: bool,
    max_error: usize,
    compression_format: String
) {
    let barcode1 = BarcodeParser::new(&pattern1, &rc_barcodes, max_error).expect("REASON");
    let barcode2 = BarcodeParser::new(&pattern2, &rc_barcodes, max_error).expect("REASON");


    let mut reader1 = io::create_reader(&read1, threads, max_memory).expect("Failed to create reader");
    let mut reader2 = io::create_reader(&read2, threads, max_memory).expect("Failed to create reader");

    let writer1 = io::create_writer(&out_read1, &compression_format).expect("Failed to create writer");
    let writer2 = io::create_writer(&out_read2, &compression_format).expect("Failed to create writer");


    loop {
        let mut record_set1 = RecordSet::default();
        let mut record_set2 = RecordSet::default();

        let filled_set1 = reader1.read_record_set(&mut record_set1);
        let filled_set2 = reader2.read_record_set(&mut record_set2);

        if filled_set1.is_none() | filled_set2.is_none() {
            break;
        } else {
            let records1: Vec<seq_io::fastq::RefRecord> = record_set1.into_iter().collect::<Vec<_>>();
            let records2: Vec<seq_io::fastq::RefRecord> = record_set2.into_iter().collect::<Vec<_>>();

            let result_read_pairs: Vec<_> = records1
            .par_iter()
            .zip(records2.par_iter())
            .filter_map(|(record1, record2)| {
                let read1_match = barcode1.search_in_single_read(&record1.seq());
                let read2_match = barcode2.search_in_single_read(&record2.seq());

                // Handle read1_match
                let read1_seq_rc;
                let read1_match = match read1_match {
                    Ok(matched_record) => Ok(matched_record),
                    Err(_) => {
                        read1_seq_rc = extract::get_reverse_complement(record1.seq());
                        barcode1.search_in_single_read(&read1_seq_rc)
                    }
                };

                // Handle read2_match
                let read2_seq_rc;
                let read2_match = match read2_match {
                    Ok(matched_record) => Ok(matched_record),
                    Err(_) => {
                        read2_seq_rc = extract::get_reverse_complement(record2.seq());
                        barcode1.search_in_single_read(&read2_seq_rc)
                    }
                };

                // Replace reads and check the result
                match extract::replace_reads(record1, record2, read1_match, read2_match) {
                    Ok((record1, record2)) => Some((record1, record2)),
                    Err(_) => None,
                }
            })
            .collect();

            let mut writer1 = writer1.lock().unwrap();
            let mut writer2 = writer2.lock().unwrap();
            for (read1_record, read2_record) in result_read_pairs {
                let _ = seq_io::fastq::write_to(
                    &mut *writer1,
                    &read1_record.head(),
                    &read1_record.seq(),
                    &read1_record.qual()    
                );

                let _ = seq_io::fastq::write_to(
                    &mut *writer2,
                    &read2_record.head(),
                    &read2_record.seq(),
                    &read2_record.qual()    
                );
            }
        }
    }
}