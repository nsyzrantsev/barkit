mod fastq;
mod errors;
mod barcode;

use seq_io::fastq::{Reader, Record};
use barcode::Barcode;



pub fn run(read1: String, read2: Option<String>, pattern: String) {
    
    let fastq_buf = fastq::read_fastq(&read1);

    let mut reader = Reader::new(fastq_buf);

    let barcode = Barcode::new(&pattern).expect("REASON");

    while let Some(record) = reader.next() {
        let record = record.expect("Error reading record");
        let caps = barcode.match_read(&record);
       match caps {
            Ok(capture) => {
                let (read_seq, read_qual, read_header) = Barcode::cut_from_read_seq("UMI", capture, &record).unwrap();
                println!("{}\n{}\n+\n{}", read_header, read_seq, read_qual);
            },
            Err(_) => {
                println!("{}\n{}\n+\n{}", std::str::from_utf8(record.head()).unwrap(), std::str::from_utf8(record.seq()).unwrap(), std::str::from_utf8(record.qual()).unwrap())
            }
        };
    }

}