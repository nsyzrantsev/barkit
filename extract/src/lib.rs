mod fastq;
mod errors;
mod barcode;

use std::{fs::File, io::Write};

use seq_io::fastq::{Reader, Record};
use barcode::Barcode;



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

    let barcode = Barcode::new(&pattern1, max_mismatch.unwrap()).expect("REASON");

    let mut processed_reads: Vec<Vec<u8>> = Vec::new();

    while let Some(record) = reader.next() {
        let record = record.expect("Error reading record");
        let caps: Result<(std::borrow::Cow<[u8]>, usize, usize), errors::Error> = barcode.match_read(&record);
        match caps {
            Ok(capture) => {
                let (read_seq, read_qual, read_header) = Barcode::cut_from_read_seq("UMI", capture, &record).unwrap();
                match out_read1 {
                    Some(_) => {
                        let read = format!("{}\n{}\n+\n{}\n", read_header, read_seq, read_qual).into_bytes();
                        processed_reads.push(read);
                    },
                    None => println!("{}\n{}\n+\n{}", read_header, read_seq, read_qual),
                }
            },
            Err(_) => {}
        };
    }

    match out_read1 {
        Some(out_file_path) => {
            save_reads_into_file(processed_reads, &out_file_path);
        },
        None => {}
    };
}


fn save_reads_into_file(reads: Vec<Vec<u8>>, out_file: &str) {
    let mut f = File::create(out_file).expect("Unable to create file");
    f.write_all(&reads.concat()).expect("Unable to write data");
}