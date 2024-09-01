#![allow(clippy::result_large_err)]

use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use std::path::Path;
use std::rc::Rc;
use std::sync::Mutex;

use flate2::{read::MultiGzDecoder, write::GzEncoder, Compression};
use gzp::{
    deflate::Bgzf, deflate::Mgzip, par::compress::ParCompressBuilder,
    par::decompress::ParDecompressBuilder,
};
use lz4::{Decoder, EncoderBuilder};
use seq_io::fastq::{OwnedRecord, Reader};

use crate::error::{self, Error};

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
    /// Returns magic bytes for specified compression type
    /// 
    /// Example:
    /// 
    /// use barkit_extract::fastq::CompressionType;
    /// 
    /// assert_eq!(CompressionType::Gzip.magic_bytes(), &[0x1f, 0x8b]);
    fn magic_bytes(&self) -> &'static [u8] {
        match self {
            CompressionType::Bgzf => &[0x42, 0x43, 0x02, 0x00],
            CompressionType::Gzip => &[0x1f, 0x8b],
            CompressionType::Mgzip => &[0x1f, 0x8b],
            CompressionType::Lz4 => &[0x04, 0x22, 0x4d, 0x18],
            CompressionType::No => &[],
        }
    }

    /// Selects `CompressionType` by provided values
    /// 
    /// Example:
    /// 
    /// use barkit_extract::fastq::CompressionType;
    /// 
    /// assert_eq!(
    ///     CompressionType::select(
    ///         true,
    ///         false,
    ///         false,
    ///         false,
    ///     ),
    ///     &[0x1f, 0x8b]
    /// );
    pub fn select(gz: &bool, bgz: &bool, mgz: &bool, lz4: &bool) -> Self {
        match (gz, bgz, mgz, lz4) {
            (true, false, false, false) => Self::Gzip,
            (false, true, false, false) => Self::Mgzip,
            (false, false, true, false) => Self::Bgzf,
            (false, false, false, true) => Self::Lz4,
            _ => CompressionType::No,
        }
    }

    /// Detects the compression type (`CompressionType`) of the provided file
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

/// Counts reads in the FASTQ
pub fn count_reads(
    file: &str,
    threads_num: usize,
    buffer_size_in_megabytes: Option<usize>,
) -> usize {
    create_reader(file, threads_num, buffer_size_in_megabytes)
        .unwrap_or_else(|_| panic!("couldn't open file {}", file))
        .into_records()
        .count()
}

/// Creates FASTQ reader instance
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

/// Caclulates optimal buffer size based on FASTQ file size and max memory consumption
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

pub struct FastqWriter {
    writer: WriterType
}

impl FastqWriter {
    pub fn new(
        fq: &str,
        compression: &CompressionType,
        threads_num: usize,
        force: bool,
    ) -> Result<Self, error::Error> {
        let path = Path::new(fq);

        // Check if file exists and handle force logic
        if path.exists() && !force {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("File {} already exists. Please, provide --force flag to overwrite it.", fq),
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

        Ok(
            Self {
                writer: Rc::new(Mutex::new(BufWriter::with_capacity(WRITE_BUFFER_SIZE, writer)))
            }
        )
    }

    fn write(&mut self, read: &OwnedRecord) -> Result<(), io::Error> {
        let mut writer = self.writer.lock().expect("Failed to lock writer");
        seq_io::fastq::write_to(&mut *writer, &read.head, &read.seq, &read.qual)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    pub fn write_all(&mut self, result_reads: Vec<OwnedRecord>) -> Result<(), io::Error> {
        for read_record in result_reads {
            self.write(&read_record)?;
        }

        Ok(())
    }
}

pub struct FastqsWriter {
    writer1: FastqWriter,
    writer2: FastqWriter,
}

impl FastqsWriter {
    pub fn new(
        fq1: &str,
        fq2: &str,
        compression: &CompressionType,
        threads_num: usize,
        force: bool,
    ) -> Result<Self, Error> {
        Ok(Self {
            writer1: FastqWriter::new(fq1, compression, threads_num, force)?,
            writer2: FastqWriter::new(fq2, compression, threads_num, force)?
        })
    }

    pub fn write_all(&mut self, pe_reads: Vec<(OwnedRecord, OwnedRecord)>) -> Result<(), io::Error> {
        for (read1_record, read2_record) in pe_reads {
            self.writer1.write(&read1_record)?;
            self.writer2.write(&read2_record)?;
        }
        Ok(())
    }
}