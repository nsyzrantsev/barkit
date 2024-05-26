mod fastq;
mod bitvector;
mod genasm;
mod errors;
mod barcode;

use std::str;

use seq_io::fastq::{Reader,Record};
use barcode::Barcode;


pub fn run(read1: String, read2: Option<String>, pattern: String) {
    
    let fastq_buf = fastq::read_fastq(&read1);

    let mut reader = Reader::new(fastq_buf);

    let barcode = Barcode::new(&pattern);

    while let Some(record) = reader.next() {
        let record = record.expect("Error reading record");
        let caps = barcode.match_read(&record).unwrap();
        let (read_seq, read_qual) = Barcode::cut_from_read(String::from("UMI"), caps, &record).unwrap();
        println!("{:?} {:?}", String::from_utf8(read_seq), String::from_utf8(read_qual));
    }
}