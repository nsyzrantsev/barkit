use std::path::Path;
use std::fs::File;
use std::io::{Read, BufRead, BufReader, BufWriter};

use seq_io::fastq::Reader;
use flate2::{read::GzDecoder, write::GzEncoder, Compression};

const GZIP_MAGIC_BYTES: [u8; 2] = [0x1f, 0x8b];
const BUFFER_SIZE: usize = 50 * 1024 * 1024; // 500 MB buffer size, you can adjust this size as needed

pub fn get_fastq_reader(fastq_path: &str) -> Reader<Box<dyn BufRead>> {
    let path = Path::new(fastq_path);
    let file = File::open(&path).expect("couldn't open file");
    let mut first_two_bytes = [0u8; 2];
    
    File::open(&path)
        .expect("couldn't open file")
        .read_exact(&mut first_two_bytes)
        .expect("couldn't read the first two bytes of file");

    match first_two_bytes { 
        GZIP_MAGIC_BYTES => Reader::new(
            Box::new(
                BufReader::with_capacity(
                    BUFFER_SIZE, 
                    GzDecoder::new(file)
                )
            )
        ),
        _ => Reader::new(
            Box::new(
                BufReader::with_capacity(
                    BUFFER_SIZE, 
                    file
                )
            )
        )
    }
}

pub fn get_fastq_writer(file: &str) -> BufWriter<flate2::write::GzEncoder<File>>{
    let file = File::create(file).expect("Unable to create file");
    let encoder = GzEncoder::new(file, Compression::default());
    BufWriter::with_capacity(BUFFER_SIZE, encoder)
}