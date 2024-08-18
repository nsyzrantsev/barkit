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
    threads: usize,
    rc_barcodes: bool,
    max_error: usize,
    compression_format: String
) {
    match (read2, out_read2, pattern1, pattern2) {
        (Some(read2), Some(out_read2), pattern1, pattern2) => process_pair_end_fastq(
            read1, 
            read2,
            pattern1,
            pattern2,
            out_read1,
            out_read2,
            max_memory,
            threads,
            rc_barcodes,
            max_error,
            compression_format
        ),
        (None, None, Some(pattern1), None) => process_single_end_fastq(
            read1,
            pattern1,
            out_read1,
            max_memory,
            threads,
            rc_barcodes,
            max_error,
            compression_format
        ),
        _ => todo!(),
    }
}

fn process_single_end_fastq(
    read: String,
    pattern: String,
    out_read: String,
    max_memory: Option<usize>,
    threads: usize,
    rc_barcodes: bool,
    max_error: usize,
    compression_format: String
) {
    let barcode = BarcodeParser::new(&pattern, max_error).expect("REASON");

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
                let matched_read = barcode.get_umi_match(&record.seq());
                let read_seq_rc: Vec<u8>;
                let matched_read = if matched_read.is_err() && rc_barcodes {
                    read_seq_rc = extract::get_reverse_complement(record.seq());
                    barcode.get_umi_match(&read_seq_rc)
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


fn process_pair_end_fastq(
    read1: String,
    read2: String,
    pattern1: Option<String>,
    pattern2: Option<String>,
    out_read1: String,
    out_read2: String,
    max_memory: Option<usize>,
    threads: usize,
    rc_barcodes: bool,
    max_error: usize,
    compression_format: String
) {
    let barcode1 = pattern1
        .as_ref()
        .map(|pat| BarcodeParser::new(pat, max_error).expect("Failed to create barcode parser for pattern1"));
    
    let barcode2 = pattern2
        .as_ref()
        .map(|pat| BarcodeParser::new(pat, max_error).expect("Failed to create barcode parser for pattern2"));

    let mut reader1 = io::create_reader(&read1, threads, max_memory)
        .expect("Failed to read input forward reads");
    let mut reader2 = io::create_reader(&read2, threads, max_memory)
        .expect("Failed to read input reverse reads");

    let writer1 = io::create_writer(&out_read1, &compression_format)
        .expect("Failed to write output forward reads");
    let writer2 = io::create_writer(&out_read2, &compression_format)
        .expect("Failed to write output reverse reads");

    loop {
        let mut record_set1 = RecordSet::default();
        let mut record_set2 = RecordSet::default();

        let filled_set1 = reader1.read_record_set(&mut record_set1);
        let filled_set2 = reader2.read_record_set(&mut record_set2);

        if filled_set1.is_none() || filled_set2.is_none() {
            break;
        } else {
            let records1: Vec<seq_io::fastq::RefRecord> = record_set1.into_iter().collect::<Vec<_>>();
            let records2: Vec<seq_io::fastq::RefRecord> = record_set2.into_iter().collect::<Vec<_>>();

            let result_read_pairs: Vec<_> = records1
            .par_iter()
            .zip(records2.par_iter())
            .filter_map(|(record1, record2)| {
                let read1_match = barcode1.as_ref().map_or(Ok(None), |b| b.get_umi_match(&record1.seq()).map(Some));
                let read2_match = barcode2.as_ref().map_or(Ok(None), |b| b.get_umi_match(&record2.seq()).map(Some));

                let read1_seq_rc;
                let read1_match = if read1_match.is_err() && rc_barcodes {
                    read1_seq_rc = extract::get_reverse_complement(record1.seq());
                    barcode1.as_ref().map_or(Ok(None), |b| b.get_umi_match(&read1_seq_rc).map(Some))
                } else {
                    read1_match
                };

                let read2_seq_rc;
                let read2_match = if read2_match.is_err() && rc_barcodes {
                    read2_seq_rc = extract::get_reverse_complement(record2.seq());
                    barcode2.as_ref().map_or(Ok(None), |b| b.get_umi_match(&read2_seq_rc).map(Some))
                } else {
                    read2_match
                };

                let new_record1 = BarcodeParser::create_new_read(read1_match.clone(), record1);
                let new_record2 = BarcodeParser::create_new_read(read2_match.clone(), record2);

                match (new_record1, new_record2) {
                    (Some(new_record1), Some(new_record2)) => Some((new_record1, new_record2)),
                    (None, Some(new_record1)) => Some((record1.to_owned_record(), new_record1)),
                    (Some(new_record1), None) => Some((new_record1, record2.to_owned_record())),
                    (None, None) => None,
                }
            })
            .collect();

            let writer1 = writer1.lock().unwrap();
            let writer2 = writer2.lock().unwrap();
            io::save_pair_end_reads_to_file(result_read_pairs, writer1, writer2);
        }
    }
}