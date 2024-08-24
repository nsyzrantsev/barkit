use std::time::Instant;

use indicatif::HumanDuration;
use rayon::prelude::*;
use seq_io::fastq::{Record, RecordSet};
use console::style;

use crate::error;
use crate::logger;
use crate::barcode::{self, BarcodeParser};
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
    let progress_bar = match quite {
        false => {
            println!("{} Estimating reads count...", style("[1/3]").bold().dim());
            Some(logger::create_progress_bar(&read, threads, max_memory).expect("Failed to create progress bar"))
        },
        true => None
    };

    let barcode = BarcodeParser::new(&pattern, max_error).expect("REASON");

    if !quite {
        println!("{} Parsing barcode patterns...", style("[2/3]").bold().dim());
    }

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
                    let read_captures: Result<regex::bytes::Captures, error::Error> =
                        barcode.get_captures(record.seq());
                    let read_seq_rc: Vec<u8>;
                    let read_captures = if read_captures.is_err() && rc_barcodes {
                        read_seq_rc = barcode::get_reverse_complement(record.seq());
                        barcode.get_captures(&read_seq_rc)
                    } else {
                        read_captures
                    };
                    barcode::create_new_read(read_captures.map(Some), record, skip_trimming)
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
    let started = Instant::now();
    let progress_bar = match quite {
        false => {
            println!("{} Estimating reads count...", style("[1/3]").bold().dim());
            Some(logger::create_progress_bar(&fq1, threads, max_memory).expect("Failed to create progress bar"))
        },
        true => None
    };

    if !quite {
        println!("{} Parsing barcode patterns...", style("[2/3]").bold().dim());
    }

    let barcode1 = pattern1.as_ref().map(|pat| {
        BarcodeParser::new(pat, max_error).expect("Failed to create barcode parser for pattern1")
    });

    let barcode2 = pattern2.as_ref().map(|pat| {
        BarcodeParser::new(pat, max_error).expect("Failed to create barcode parser for pattern2")
    });

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
                    let read1_captures = barcode1
                        .as_ref()
                        .map_or(Ok(None), |b| b.get_captures(record1.seq()).map(Some));
                    let read2_captures = barcode2
                        .as_ref()
                        .map_or(Ok(None), |b| b.get_captures(record2.seq()).map(Some));

                    let read1_seq_rc;
                    let read1_captures = if read1_captures.is_err() && rc_barcodes {
                        read1_seq_rc = barcode::get_reverse_complement(record1.seq());
                        barcode1
                            .as_ref()
                            .map_or(Ok(None), |b| b.get_captures(&read1_seq_rc).map(Some))
                    } else {
                        read1_captures
                    };

                    let read2_seq_rc;
                    let read2_captures = if read2_captures.is_err() && rc_barcodes {
                        read2_seq_rc = barcode::get_reverse_complement(record2.seq());
                        barcode2
                            .as_ref()
                            .map_or(Ok(None), |b| b.get_captures(&read2_seq_rc).map(Some))
                    } else {
                        read2_captures
                    };

                    let new_record1 =
                        barcode::create_new_read(read1_captures, record1, skip_trimming);
                    let new_record2 =
                        barcode::create_new_read(read2_captures, record2, skip_trimming);

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
