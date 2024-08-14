use clap::{command, ArgAction, Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(version)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}


#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Tool to parse molecular (UMI) barcodes from FASTQ file(s)
    #[clap(arg_required_else_help = true)]
    Extract {
        /// (gzipped) input forward FASTQ file
        #[arg(short='1', long)]
        read1: String,

        /// (gzipped) input reverse FASTQ file
        #[arg(short='2', long, requires = "read1")]
        read2: Option<String>,

        /// (gzipped) output forward FASTQ file
        #[arg(short='o', long)]
        out_read1: String,

        /// (gzipped) output reverse FASTQ file
        #[arg(short='O', long, requires = "out_read1")]
        out_read2: Option<String>,

        /// barcode pattern of forward reads
        #[arg(short='p', long)]
        pattern1: Option<String>,

        /// barcode pattern of reverse reads
        #[arg(short='P', long, requires = "pattern1")]
        pattern2: Option<String>,

        /// max memory (RAM) usage in megabytes (MB)
        #[arg(short='m', long)]
        max_memory: Option<usize>,

        /// the approximate number of threads to use.
        #[arg(short='t', long, default_value = "1")]
        threads: Option<usize>,

        /// searches for both barcode pattern in reverse complement
        #[arg(short='r', long, action=ArgAction::SetTrue)]
        rc_barcodes: Option<bool>,

        /// max error (mistmatch) between provided pattern and read sequence
        #[arg(short='e', long, default_value = "1")]
        max_error: Option<usize>,

        /// compression format for output FASTQ files
        #[arg(short='c', long, default_value = "bgzf", value_parser = ["gzip", "bgzf"])]
        compression_format: String,
    },
}