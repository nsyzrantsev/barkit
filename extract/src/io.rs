use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::Path;
use std::rc::Rc;
use std::sync::{Mutex, MutexGuard};

use flate2::{read::MultiGzDecoder, write::GzEncoder, Compression};
use gzp::{
    deflate::Bgzf, deflate::Mgzip, par::compress::ParCompressBuilder,
    par::decompress::ParDecompressBuilder,
};
use lz4::{Decoder, EncoderBuilder};
use seq_io::fastq::{OwnedRecord, Reader};

use crate::error;

// magic numbers in the header of each file type
const GZIP_MAGIC_BYTES: [u8; 2] = [0x1f, 0x8b];
const LZ4_MAGIC_BYTES: [u8; 4] = [0x04, 0x22, 0x4d, 0x18];
const BGZIP_MAGIC_BYTES: [u8; 4] = [0x42, 0x43, 0x02, 0x00];

const WRITE_BUFFER_SIZE: usize = 128 * 1024 * 1024; // 128 KB buffer size, you can adjust this size as needed

enum CompressionType {
    Bgzf,
    Gzip,
    Lz4,
    No,
}

fn get_fastq_compression_type(path: &Path) -> CompressionType {
    let mut buffer = [0u8; 16];

    File::open(path)
        .expect("couldn't open file")
        .read_exact(&mut buffer)
        .expect("couldn't read the first two bytes of file");

    if buffer[..2] == GZIP_MAGIC_BYTES {
        CompressionType::Gzip
    } else if buffer[..4] == LZ4_MAGIC_BYTES {
        CompressionType::Lz4
    } else if buffer[12..16] == BGZIP_MAGIC_BYTES {
        CompressionType::Bgzf
    } else {
        CompressionType::No
    }
}

pub fn create_reader(
    fastq_path: &str,
    threads_num: usize,
    buffer_size_in_megabytes: Option<usize>,
) -> Result<seq_io::fastq::Reader<Box<dyn BufRead>>, error::Error> {
    let path = Path::new(fastq_path);
    let file = File::open(path).expect("couldn't open file");

    let buffer_size_in_bytes = get_reader_buffer_size(&file, buffer_size_in_megabytes)?;

    let decoder: Box<dyn Read> = match get_fastq_compression_type(path) {
        CompressionType::Gzip => Box::new(MultiGzDecoder::new(file)),
        CompressionType::Lz4 => Box::new(Decoder::new(file)?),
        CompressionType::Bgzf => Box::new(
            ParDecompressBuilder::<Bgzf>::new()
                .num_threads(threads_num)
                .expect("Provided unexpected number of threads")
                .from_reader(BufReader::with_capacity(buffer_size_in_bytes, file)),
        ),
        CompressionType::No => Box::new(file),
    };

    Ok(Reader::new(Box::new(BufReader::with_capacity(
        buffer_size_in_bytes,
        decoder,
    ))))
}

fn get_reader_buffer_size(
    fastq_file: &File,
    max_memory: Option<usize>,
) -> Result<usize, error::Error> {
    let fastq_file_size_bytes = fastq_file.metadata()?.len() as usize;
    match max_memory {
        Some(buffer_size) => {
            let buffer_size_bytes = buffer_size * 1024 * 1024;
            if buffer_size_bytes > fastq_file_size_bytes {
                Ok(fastq_file_size_bytes)
            } else {
                Ok(buffer_size_bytes)
            }
        }
        None => Ok(fastq_file_size_bytes),
    }
}

type WriterType = Rc<Mutex<BufWriter<Box<dyn std::io::Write>>>>;

pub fn create_writer(
    file: &str,
    compression_format: &str,
    threads_num: usize,
) -> Result<WriterType, error::Error> {
    let path = Path::new(file);
    let file = File::create(path)?;
    let writer: Box<dyn std::io::Write> = match compression_format {
        "gzip" => Box::new(GzEncoder::new(file, Compression::default())),
        "bgzf" => Box::new(
            ParCompressBuilder::<Bgzf>::new()
                .num_threads(threads_num)
                .expect("Provided unexpected number of threads")
                .from_writer(file),
        ),
        "mgzip" => Box::new(
            ParCompressBuilder::<Mgzip>::new()
                .num_threads(threads_num)
                .expect("Provided unexpected number of threads")
                .from_writer(file),
        ),
        "lz4" => Box::new(
            EncoderBuilder::new()
            .build(file)?
        ),
        _ => Box::new(file),
    };
    Ok(Rc::new(Mutex::new(BufWriter::with_capacity(
        WRITE_BUFFER_SIZE,
        writer,
    ))))
}

pub fn save_pair_end_reads_to_file(
    result_read_pairs: Vec<(OwnedRecord, OwnedRecord)>,
    mut writer1: MutexGuard<BufWriter<Box<dyn Write>>>,
    mut writer2: MutexGuard<BufWriter<Box<dyn Write>>>,
) {
    for (read1_record, read2_record) in result_read_pairs {
        let _ = seq_io::fastq::write_to(
            &mut *writer1,
            &read1_record.head,
            &read1_record.seq,
            &read1_record.qual,
        );

        let _ = seq_io::fastq::write_to(
            &mut *writer2,
            &read2_record.head,
            &read2_record.seq,
            &read2_record.qual,
        );
    }
}

pub fn save_single_end_reads_to_file(
    result_reads: Vec<OwnedRecord>,
    mut writer: MutexGuard<BufWriter<Box<dyn Write>>>,
) {
    for read_record in result_reads {
        let _ = seq_io::fastq::write_to(
            &mut *writer,
            &read_record.head,
            &read_record.seq,
            &read_record.qual,
        );
    }
}
