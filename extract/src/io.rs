use std::path::Path;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::sync::{Arc, Mutex};

use lz4::Decoder;
use seq_io::fastq::{Reader, RecordSet, RefRecord};
use flate2::{read::MultiGzDecoder, write::GzEncoder, Compression};

use rayon::prelude::*;

use crate::errors;

const GZIP_MAGIC_BYTES: [u8; 2] = [0x1f, 0x8b]; // a magic number in the header of GZIP files
const LZ4_MAGIC_BYTES: [u8; 4] = [0x04, 0x22, 0x4d, 0x18]; // a magic number in the header of LZ4 files
const WRITE_BUFFER_SIZE: usize = 512 * 1024 * 1024; // 64 KB buffer size, you can adjust this size as needed

enum CompressionType {
    GZIP,
    LZ4,
    NO
}

fn get_fastq_compression_type(path: &Path) -> CompressionType {
    let mut first_four_bytes = [0u8; 4];

    File::open(&path)
        .expect("couldn't open file")
        .read_exact(&mut first_four_bytes)
        .expect("couldn't read the first two bytes of file");

    if first_four_bytes[..2] == GZIP_MAGIC_BYTES {
        return CompressionType::GZIP
    } else if first_four_bytes[..4] == LZ4_MAGIC_BYTES {
        return CompressionType::LZ4
    } else {
        CompressionType::NO
    }
}

pub fn create_reader(fastq_path: &str, buffer_size_in_megabytes: Option<usize>) -> Result<seq_io::fastq::Reader<Box<dyn BufRead>>, errors::Error> {
    let path = Path::new(&fastq_path);
    let file = File::open(&path).expect("couldn't open file");

    let buffer_size_in_bytes = get_reader_buffer_size(&file, buffer_size_in_megabytes)?;

    let decoder: Box<dyn Read> = match get_fastq_compression_type(path) {
        CompressionType::GZIP => Box::new(MultiGzDecoder::new(file)),
        CompressionType::LZ4 => Box::new(Decoder::new(file)?),
        CompressionType::NO => Box::new(file),
    };

    Ok(Reader::new(Box::new(BufReader::with_capacity(buffer_size_in_bytes, decoder))))
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


struct FastqReaderPair {
}


impl FastqReaderPair {
    
}


pub fn create_writer(file: &str) -> Result<BufWriter<Box<dyn Write>>, errors::Error> {
    let path = Path::new(file);
    let file = File::create(path)?;
    let writer: Box<dyn std::io::Write> = match path.extension().and_then(|ext| ext.to_str()) {
        Some("gz") => Box::new(GzEncoder::new(file, Compression::default())),
        _ => Box::new(file),
    };
    Ok(BufWriter::with_capacity(WRITE_BUFFER_SIZE, writer))
}