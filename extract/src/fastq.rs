use flate2::read::GzDecoder;
use std::path::Path;
use std::fs::File;
use std::io::{Read, BufRead, BufReader};


const GZIP_MAGIC_BYTES: [u8; 2] = [0x1f, 0x8b];

pub fn read_fastq(fastq_path: &str) -> Box<dyn BufRead> {
    let path = Path::new(fastq_path);
    let file = File::open(&path).expect("couldn't open file");
    let mut first_two_bytes = [0u8; 2];
    
    File::open(&path)
        .expect("couldn't open file")
        .read_exact(&mut first_two_bytes)
        .expect("couldn't read the first two bytes of file");

    match first_two_bytes { 
        GZIP_MAGIC_BYTES => Box::new(BufReader::with_capacity(128 * 1024, GzDecoder::new(file))),
        _ => Box::new(BufReader::with_capacity(128 * 1024, file))
    }
}