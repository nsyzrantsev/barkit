use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
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

const WRITE_BUFFER_SIZE: usize = 128 * 1024 * 1024; // 128 KB buffer size, you can adjust this size as needed

pub enum CompressionType {
    /// BGZF (BGZIP) compression format
    Bgzf,

    /// Gzip compression format
    Gzip,

    /// Mgzip compression format
    Mgzip,

    /// LZ4 compression format
    Lz4,

    /// Without compression
    No,
}

impl CompressionType {
    fn magic_bytes(&self) -> &'static [u8] {
        match self {
            CompressionType::Bgzf => &[0x42, 0x43, 0x02, 0x00],
            CompressionType::Gzip => &[0x1f, 0x8b],
            CompressionType::Mgzip => &[0x1f, 0x8b],
            CompressionType::Lz4 => &[0x04, 0x22, 0x4d, 0x18],
            CompressionType::No => &[],
        }
    }

    pub fn select(gz: &bool, bgz: &bool, mgz: &bool, lz4: &bool) -> Self {
        match (gz, bgz, mgz, lz4) {
            (true, false, false, false) => Self::Gzip,
            (false, true, false, false) => Self::Mgzip,
            (false, false, true, false) => Self::Bgzf,
            (false, false, false, true) => Self::Lz4,
            _ => CompressionType::No,
        }
    }

    fn detect(path: &Path) -> CompressionType {
        let mut buffer = [0u8; 16];

        File::open(path)
            .expect("couldn't open file")
            .read_exact(&mut buffer)
            .expect("couldn't read the first two bytes of file");

        if &buffer[..2] == CompressionType::Gzip.magic_bytes() {
            CompressionType::Gzip
        } else if &buffer[..4] == CompressionType::Lz4.magic_bytes() {
            CompressionType::Lz4
        } else if &buffer[12..16] == CompressionType::Bgzf.magic_bytes() {
            CompressionType::Bgzf
        } else {
            CompressionType::No
        }
    }
}

pub fn get_reads_count(
    file: &str,
    threads_num: usize,
    buffer_size_in_megabytes: Option<usize>,
) -> usize {
    create_reader(file, threads_num, buffer_size_in_megabytes)
        .unwrap_or_else(|_| panic!("couldn't open file {}", file))
        .into_records()
        .count()
}

pub fn create_reader(
    fastq_path: &str,
    threads_num: usize,
    buffer_size_in_megabytes: Option<usize>,
) -> Result<seq_io::fastq::Reader<Box<dyn BufRead>>, error::Error> {
    let path = Path::new(fastq_path);
    let file = File::open(path).unwrap_or_else(|_| panic!("couldn't open file {}", fastq_path));

    let buffer_size_in_bytes = get_reader_buffer_size(&file, buffer_size_in_megabytes)?;

    let decoder: Box<dyn Read> = match CompressionType::detect(path) {
        CompressionType::Gzip | CompressionType::Mgzip => Box::new(MultiGzDecoder::new(file)),
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
    compression: &CompressionType,
    threads_num: usize,
    force: bool,
) -> Result<WriterType, error::Error> {
    let path = Path::new(file);

    // Check if file exists and handle force logic
    if path.exists() && !force {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("File {} already exists and force is set to false", file),
        )
        .into());
    }

    let file = if force {
        File::create(path)?
    } else {
        OpenOptions::new().write(true).create_new(true).open(path)?
    };

    let writer: Box<dyn Write> = match compression {
        CompressionType::Gzip => Box::new(GzEncoder::new(file, Compression::default())),
        CompressionType::Bgzf => Box::new(
            ParCompressBuilder::<Bgzf>::new()
                .num_threads(threads_num)
                .expect("Provided unexpected number of threads")
                .from_writer(file),
        ),
        CompressionType::Mgzip => Box::new(
            ParCompressBuilder::<Mgzip>::new()
                .num_threads(threads_num)
                .expect("Provided unexpected number of threads")
                .from_writer(file),
        ),
        CompressionType::Lz4 => Box::new(EncoderBuilder::new().build(file)?),
        _ => Box::new(file),
    };

    Ok(Rc::new(Mutex::new(BufWriter::with_capacity(
        WRITE_BUFFER_SIZE,
        writer,
    ))))
}

fn write_read_to_file(read: &OwnedRecord, writer: &mut MutexGuard<BufWriter<Box<dyn Write>>>) {
    let _ = seq_io::fastq::write_to(&mut **writer, &read.head, &read.seq, &read.qual);
}

pub fn save_pair_end_reads_to_file(
    result_read_pairs: Vec<(OwnedRecord, OwnedRecord)>,
    mut writer1: MutexGuard<BufWriter<Box<dyn Write>>>,
    mut writer2: MutexGuard<BufWriter<Box<dyn Write>>>,
) {
    for (read1_record, read2_record) in result_read_pairs {
        write_read_to_file(&read1_record, &mut writer1);
        write_read_to_file(&read2_record, &mut writer2);
    }
}

pub fn save_single_end_reads_to_file(
    result_reads: Vec<OwnedRecord>,
    mut writer: MutexGuard<BufWriter<Box<dyn Write>>>,
) {
    for read_record in result_reads {
        write_read_to_file(&read_record, &mut writer);
    }
}
