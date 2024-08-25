use std::time::Instant;

use indicatif::HumanDuration;
use rayon::prelude::*;
use seq_io::fastq::{Record, RecordSet};
use console::style;

use crate::error;
use crate::logger;
use crate::barcode::{self, BarcodeRegex};
use crate::io::{self, CompressionType};

#[allow(clippy::too_many_arguments)]
pub fn run(
    fq1: String,
    fq2: Option<String>,
    pattern1: Option<String>,
    pattern2: Option<String>,
    out_fq1: String,
    out_fq2: Option<String>,
    max_memory: Option<usize>,
    threads: usize,
    rc_barcodes: bool,
    skip_trimming: bool,
    max_error: usize,
    output_compression: CompressionType,
    quite: bool
) {
    match (fq2, out_fq2, pattern1, pattern2) {
        (Some(fq2), Some(out_fq2), pattern1, pattern2) => {
            process_pair_end_fastq(
            fq1,
            fq2,
            pattern1,
            pattern2,
            out_fq1,
            out_fq2,
            max_memory,
            threads,
            rc_barcodes,
            skip_trimming,
            max_error,
            output_compression,
            quite
        )},
        (None, None, Some(pattern1), None) => {
            process_single_end_fastq(
            fq1,
            pattern1,
            out_fq1,
            max_memory,
            threads,
            rc_barcodes,
            skip_trimming,
            max_error,
            output_compression,
            quite
        )},
        _ => println!("{}", "Something unexpected happend... Please, check provided arguments."),
    }
}

#[allow(clippy::too_many_arguments)]
fn process_single_end_fastq(
    read: String,
    pattern: String,
    out_read: String,
    max_memory: Option<usize>,
    threads: usize,
    rc_barcodes: bool,
    skip_trimming: bool,
    max_error: usize,
    output_compression: CompressionType,
    quite: bool
) {
    if !quite {
        println!("{} Parsing barcode patterns...", style("[1/3]").bold().dim());
    }
    
    let barcode_re = BarcodeRegex::new(&pattern, max_error).expect("REASON");
    
    let progress_bar = match quite {
        false => {
            println!("{} Estimating reads count...", style("[2/3]").bold().dim());
            Some(logger::create_progress_bar(&read, threads, max_memory).expect("Failed to create progress bar"))
        },
        true => None
    };

    let mut reader =
        io::create_reader(&read, threads, max_memory).expect("Failed to create reader");
    let writer = io::create_writer(&out_read, &output_compression, threads)
        .expect("Failed to create writer");

    if !quite {
        println!("{} Extracting barcodes from reads...", style("[3/3]").bold().dim());
    }

    loop {
        let mut record_set = RecordSet::default();

        let filled_set = reader.read_record_set(&mut record_set);

        if filled_set.is_none() {
            break;
        } else {
            let records = record_set
                .into_iter()
                .collect::<Vec<_>>();
            let result_reads: Vec<_> = records
                .par_iter()
                .filter_map(|record| {
                    let barcodes_parser = barcode::BarcodeParser::new(
                        Some(barcode_re.clone()),
                        skip_trimming,
                        rc_barcodes
                    );
                    barcodes_parser?.extract_barcodes(record)
                })
                .collect();

            let writer = writer.lock().unwrap();
            io::save_single_end_reads_to_file(result_reads, writer);

            match progress_bar {
                Some(ref pb) => pb.inc(records.len() as u64),
                None => ()
            }
        }
    }

    match progress_bar {
        Some(pb) => pb.finish_with_message("all reads successfully processed"),
        None => ()
    }
}

#[allow(clippy::too_many_arguments)]
fn process_pair_end_fastq(
    fq1: String,
    fq2: String,
    pattern1: Option<String>,
    pattern2: Option<String>,
    out_fq1: String,
    out_fq2: String,
    max_memory: Option<usize>,
    threads: usize,
    rc_barcodes: bool,
    skip_trimming: bool,
    max_error: usize,
    output_compression: CompressionType,
    quite: bool
) {
    if !quite {
        println!("{} Parsing barcode patterns...", style("[1/3]").bold().dim());
    }
    
    let barcode1 = pattern1.as_ref().map(|pat| {
        BarcodeRegex::new(pat, max_error).expect("Failed to create barcode parser for pattern1")
    });

    let barcode2 = pattern2.as_ref().map(|pat| {
        BarcodeRegex::new(pat, max_error).expect("Failed to create barcode parser for pattern2")
    });

    let started = Instant::now();
    let progress_bar = match quite {
        false => {
            println!("{} Estimating reads count...", style("[2/3]").bold().dim());
            Some(logger::create_progress_bar(&fq1, threads, max_memory).expect("Failed to create progress bar"))
        },
        true => None
    };

    let mut reader1 =
        io::create_reader(&fq1, threads, max_memory).expect("Failed to read input forward reads");
    let mut reader2 =
        io::create_reader(&fq2, threads, max_memory).expect("Failed to read input reverse reads");

    let writer1 = io::create_writer(&out_fq1, &output_compression, threads)
        .expect("Failed to write output forward reads");
    let writer2 = io::create_writer(&out_fq2, &output_compression, threads)
        .expect("Failed to write output reverse reads");

    if !quite {
        println!("{} Extracting barcodes from reads...", style("[3/3]").bold().dim());
    }

    loop {
        let mut record_set1 = RecordSet::default();
        let mut record_set2 = RecordSet::default();

        let filled_set1 = reader1.read_record_set(&mut record_set1);
        let filled_set2 = reader2.read_record_set(&mut record_set2);

        if filled_set1.is_none() || filled_set2.is_none() {
            break;
        } else {
            let records1: Vec<seq_io::fastq::RefRecord> =
                record_set1.into_iter().collect::<Vec<_>>();
            let records2: Vec<seq_io::fastq::RefRecord> =
                record_set2.into_iter().collect::<Vec<_>>();

            let result_read_pairs: Vec<_> = records1
                .par_iter()
                .zip(records2.par_iter())
                .filter_map(|(record1, record2)| {
                    let barcode1_parser = barcode::BarcodeParser::new(barcode1.clone(), skip_trimming, rc_barcodes);
                    let barcode2_parser = barcode::BarcodeParser::new(barcode2.clone(), skip_trimming, rc_barcodes);

                    let new_records = (
                        barcode1_parser.as_ref().and_then(|parser| parser.extract_barcodes(record1)),
                        barcode2_parser.as_ref().and_then(|parser| parser.extract_barcodes(record2)),
                    );
                    
                    match new_records {
                        (Some(new_record1), Some(new_record2)) => Some((new_record1, new_record2)),
                        (None, Some(new_record2)) => Some((record1.to_owned_record(), new_record2)),
                        (Some(new_record1), None) => Some((new_record1, record2.to_owned_record())),
                        (None, None) => None,
                    }
                })
                .collect();

            let writer1 = writer1.lock().unwrap();
            let writer2 = writer2.lock().unwrap();
            io::save_pair_end_reads_to_file(result_read_pairs, writer1, writer2);
            
            match progress_bar {
                Some(ref pb) => pb.inc(records1.len() as u64),
                None => ()
            }
        }
    }
    match progress_bar {
        Some(_) => println!("{} Done in {}", logger::SPARKLE, HumanDuration(started.elapsed())),
        None => ()
    }
}
