use rayon::prelude::*;

use crate::fastq::{self, CompressionType};
use crate::logger;
use crate::parse;
use crate::pattern::BarcodeRegex;

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
    quiet: bool,
    force: bool,
) {
    match (fq2, out_fq2, pattern1, pattern2) {
        (Some(fq2), Some(out_fq2), pattern1, pattern2) => process_pair_end_fastq(
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
            quiet,
            force
        ),
        (None, None, Some(pattern1), None) => process_single_end_fastq(
            fq1,
            pattern1,
            out_fq1,
            max_memory,
            threads,
            rc_barcodes,
            skip_trimming,
            max_error,
            output_compression,
            quiet,
            force
        ),
        _ => eprintln!(
            "Invalid arguments provided. Please ensure that you have provided the correct combination of input files and patterns."
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn process_single_end_fastq(
    fq: String,
    pattern: String,
    out_fq: String,
    max_memory: Option<usize>,
    threads: usize,
    rc_barcodes: bool,
    skip_trimming: bool,
    max_error: usize,
    output_compression: CompressionType,
    quiet: bool,
    force: bool,
) {
    let mut logger = logger::Logger::new(3, quiet);
    logger.message("Estimating reads count...");

    let lines_number = fastq::FastqReader::count_reads(&fq, threads, max_memory);
    logger.set_progress_bar(lines_number);

    let mut reader = fastq::FastqReader::new(&fq, threads, max_memory)
        .expect("Failed to create reader");

    let mut writer = fastq::FastqWriter::new(&out_fq, &output_compression, threads, force)
        .expect("Failed to create writer");

    logger.message("Parsing barcode patterns...");

    let barcode_re = BarcodeRegex::new(&pattern, max_error)
        .expect("Failed to create barcode regex with the provided pattern and max error.");

    logger.message("Extracting barcodes from reads...");

    loop {
        let record_set = reader.read_record_set();

        if let Some(records) = record_set {
            // Flatten the record set into individual records
            let records = records.into_iter().collect::<Vec<_>>();
    
            // Parallel processing of individual records to extract parsed reads
            let result_reads: Vec<_> = records
                .par_iter()
                .filter_map(|record| {
                    // Create a new BarcodeParser with the appropriate configuration
                    let barcodes_parser = parse::BarcodeParser::new(
                        Some(barcode_re.clone()),
                        skip_trimming,
                        rc_barcodes,
                    );
    
                    // Parse the barcodes from the RefRecord
                    // `record` needs to be passed as a `&RefRecord`
                    barcodes_parser?.parse_barcodes(record)
                })
                .collect();
    
            // Write the processed reads to the writer
            writer
                .write_all(result_reads)
                .expect("Failed to write processed reads");
    
            // Increment the progress tracker based on the number of records processed
            logger.increment_progress(records.len());
        } else {
            break;
        }
    }

    logger.final_message();
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
    quiet: bool,
    force: bool,
) {
    let mut logger = logger::Logger::new(3, quiet);
    logger.message("Estimating reads count...");

    let lines_number = fastq::FastqReader::count_reads(&fq1, threads, max_memory);
    logger.set_progress_bar(lines_number);

    let mut reader = fastq::FastqsReader::new(&fq1, &fq2, threads, max_memory)
        .expect("Failed to read input forward reads");

    let mut writer = fastq::FastqsWriter::new(&out_fq1, &out_fq2, &output_compression, threads, force)
        .expect("Failed to create writer");

    logger.message("Parsing barcode patterns...");

    let barcode1 = pattern1.as_ref().map(|pat| {
        BarcodeRegex::new(pat, max_error).expect(
            "Failed to create barcode regex for pattern1 with the provided pattern and max error",
        )
    });

    let barcode2 = pattern2.as_ref().map(|pat| {
        BarcodeRegex::new(pat, max_error).expect(
            "Failed to create barcode regex for pattern2 with the provided pattern and max error",
        )
    });

    logger.message("Extracting barcodes from reads...");

    loop {
        let record_sets = reader.read_record_sets();

        if let Ok((Some(records1), Some(records2))) = record_sets {
            let records1 = records1.into_iter().collect::<Vec<_>>();
            let records2 = records2.into_iter().collect::<Vec<_>>();

            let result_read_pairs: Vec<_> = records1
                .par_iter()
                .zip(records2.par_iter())
                .filter_map(|(record1, record2)| {
                    let barcode1_parser =
                        parse::BarcodeParser::new(barcode1.clone(), skip_trimming, rc_barcodes);
                    let barcode2_parser =
                        parse::BarcodeParser::new(barcode2.clone(), skip_trimming, rc_barcodes);

                    let new_records = (
                        barcode1_parser
                            .as_ref()
                            .and_then(|parser| parser.parse_barcodes(record1)),
                        barcode2_parser
                            .as_ref()
                            .and_then(|parser| parser.parse_barcodes(record2)),
                    );

                    match new_records {
                        (Some(new_record1), Some(new_record2)) => Some((new_record1, new_record2)),
                        (None, Some(new_record2)) => Some((record1.to_owned_record(), new_record2)),
                        (Some(new_record1), None) => Some((new_record1, record2.to_owned_record())),
                        (None, None) => None,
                    }
                })
                .collect();

            writer.write_all(result_read_pairs).expect("Failed to write processed reads");

            logger.increment_progress(records1.len());
        } else {
            break;
        }
    }
    logger.final_message();
}
