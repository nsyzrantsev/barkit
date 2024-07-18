use std::path::Path;
use std::fs::File;
use std::io::{Read, BufRead, BufReader, BufWriter};

use seq_io::fastq::{Reader, RecordSet, RefRecord};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};

use rayon::prelude::*;

use crate::errors;

const GZIP_MAGIC_BYTES: [u8; 2] = [0x1f, 0x8b]; // a magic number (1f 8b) in the header of gzipped files
const WRITE_BUFFER_SIZE: usize = 512 * 1024 * 1024; // 64 KB buffer size, you can adjust this size as needed


pub fn create_reader(fastq_path: &str, buffer_size_in_megabytes: Option<usize>) -> Result<Reader<Box<dyn BufRead>>, errors::Error> {
    let path = Path::new(&fastq_path);
    let file = File::open(&path).expect("couldn't open file");

    let buffer_size_in_bytes = get_reader_buffer_size(&file, buffer_size_in_megabytes)?;

    let mut first_two_bytes = [0u8; 2];

    File::open(&path)
        .expect("couldn't open file")
        .read_exact(&mut first_two_bytes)
        .expect("couldn't read the first two bytes of file");

    let reader: Box<dyn BufRead> = match first_two_bytes {
        GZIP_MAGIC_BYTES => Box::new(BufReader::with_capacity(buffer_size_in_bytes, GzDecoder::new(file))),
        _ => Box::new(BufReader::with_capacity(buffer_size_in_bytes, file)),
    };

    Ok(Reader::new(reader))
}

fn get_reader_buffer_size(fastq_file: &File, max_memory: Option<usize>) -> Result<usize, errors::Error> {
    let fastq_file_size_bytes = fastq_file.metadata()?.len() as usize;
    match max_memory {
        Some(buffer_size) => {
            let buffer_size_bytes = buffer_size * 1024 * 1024;
            if buffer_size_bytes > fastq_file_size_bytes {
                Ok(fastq_file_size_bytes)
            } else {
                Ok(buffer_size_bytes)
            }
        },
        None => Ok(fastq_file_size_bytes)
    }
}


pub fn create_writer(file: &str) -> BufWriter<GzEncoder<File>> {
    let file = File::create(file).expect("Unable to create file");
    let encoder = GzEncoder::new(file, Compression::default());
    BufWriter::with_capacity(WRITE_BUFFER_SIZE, encoder)
}