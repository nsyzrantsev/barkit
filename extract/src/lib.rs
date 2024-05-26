mod fastq;
mod bitvector;
mod genasm;
mod errors;

use std::str;
use regex::Regex;

use seq_io::fastq::{Reader,Record};


pub fn run(read1: String, read2: Option<String>, pattern: String) {
    
    let fastq_buf = fastq::read_fastq(&read1);

    let mut reader = Reader::new(fastq_buf);

    while let Some(record) = reader.next() {
        let record = record.expect("Error reading record");
        println!("{:?}", str::from_utf8(record.head()).unwrap());
        println!("{:?}", str::from_utf8(record.seq()).unwrap());
        println!("{:?}", str::from_utf8(record.qual()).unwrap());
        let caps = fastq::parse_barcode(&pattern, str::from_utf8(record.seq()).unwrap()).unwrap();
        println!("The UMI is: {}", &caps["UMI"]);
    }
}