mod fastq;
mod errors;
mod barcode;

use std::io::BufWriter;
use std::{fs::File, io::Write};

use flate2::write::GzEncoder;
use flate2::Compression;

use seq_io::fastq::Reader;
use barcode::BarcodeMatcher;

use tre_regex::fuzzy::Match;

pub fn run(
    read1: String, 
    read2: Option<String>, 
    pattern1: String,
    pattern2: Option<String>,
    out_read1: Option<String>, 
    out_read2: Option<String>,
    max_mismatch: Option<usize>,
) {
    
    let fastq_buf = fastq::read_fastq(&read1);

    let mut reader = Reader::new(fastq_buf);

    let barcode = BarcodeMatcher::new(&pattern1, max_mismatch.unwrap()).expect("REASON");

    let file = File::create(out_read1.clone().unwrap()).expect("Unable to create file");
    let encoder = GzEncoder::new(file, Compression::default());
    let buffer_size = 50 * 1024 * 1024; // 50 MB buffer size, you can adjust this size as needed
    let mut f = BufWriter::with_capacity(buffer_size, encoder);

    while let Some(record) = reader.next() {
        let record = record.expect("Error reading record");
        let caps: Result<Match, errors::Error> = barcode.match_read(&record);
        match caps {
            Ok(capture) => {
                let (read_seq, read_qual, read_header) = BarcodeMatcher::cut_from_read_seq("UMI", capture, &record).unwrap();
                match out_read1 {
                    Some(_) => {
                        f.write_all(&read_header).expect("Unable to write read header");
                        f.write_all(b"+\n").expect("Unable to write read separator");
                        f.write_all(&read_seq).expect("Unable to write read sequence");
                        f.write_all(&read_qual).expect("Unable to write read quality");
                    },
                    None => eprintln!("Failed to write to output file"),
                }
            },
            Err(_) => {}
        };
    }

    f.flush().expect("Failed to flush buffer");
    f.into_inner().expect("Failed to finish compression").finish().expect("Failed to finish compression");
}