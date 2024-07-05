mod fastq;
mod errors;
mod barcode;

use std::io::Write;

use barcode::BarcodeMatcher;


pub fn run(
    read1: String, 
    read2: Option<String>, 
    pattern1: String,
    pattern2: Option<String>,
    out_read1: String, 
    out_read2: Option<String>,
    max_mismatch: usize,
    max_memory: Option<usize>
) {
    process_fastq(read1, pattern1, out_read1, max_mismatch, max_memory);

    let out_read2 = out_read2.unwrap_or_else(|| {
        eprintln!("{}", errors::Error::OutputFastqFileNotProvided);
        std::process::exit(1); // or handle the error appropriately
    });
}

fn process_fastq(
    read: String,
    pattern: String,
    out_read: String,
    max_mismatch: usize,
    max_memory: Option<usize>
) {
    let barcode = BarcodeMatcher::new(&pattern, max_mismatch).expect("REASON");
    
    let mut fastq_reader = fastq::get_fastq_reader(&read, max_memory);
    
    let mut fastq_writer = fastq::get_fastq_writer(&out_read);

    while let Some(record) = fastq_reader.next() {
        let record = record.expect("Error reading record");
        let caps = barcode.match_read(&record);

        if let Ok(capture) = caps {
            let (read_seq, read_qual, read_header) = BarcodeMatcher::cut_from_read_seq("UMI", capture, &record).unwrap();
            if let Err(e) = seq_io::fastq::write_to(&mut fastq_writer, &read_header, &read_seq, &read_qual) {
                eprintln!("Failed to write to output file: {}", e);
            }
        }
    }

    fastq_writer.flush().expect("Failed to flush buffer");
    fastq_writer.into_inner().expect("Failed to finish compression").finish().expect("Failed to finish compression");
}