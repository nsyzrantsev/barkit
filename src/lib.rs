use clap::{command, ArgAction, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Tool for parsing UMI barcodes from single-end or paired-end FASTQ files
    #[clap(arg_required_else_help = true)]
    Extract {
        /// (gzipped) input forward FASTQ file
        #[arg(short = '1', long, requires = "out_read1")]
        read1: String,

        /// (gzipped) input reverse FASTQ file
        #[arg(short='2', long, requires_all = ["read1", "out_read2"])]
        read2: Option<String>,

        /// (gzipped) output forward FASTQ file
        #[arg(short = 'o', long)]
        out_read1: String,

        /// (gzipped) output reverse FASTQ file
        #[arg(short = 'O', long, requires = "out_read1")]
        out_read2: Option<String>,

        #[clap(flatten)]
        patterns: PatternsGroup,

        /// max memory (RAM) usage in megabytes (MB)
        #[arg(short = 'm', long)]
        max_memory: Option<usize>,

        /// the approximate number of threads to use.
        #[arg(short = 't', long, default_value = "1")]
        threads: usize,

        /// searches for both barcode pattern in reverse complement
        #[arg(short='r', long, action=ArgAction::SetTrue)]
        rc_barcodes: bool,

        /// skip trimming the adapter sequence from the read
        #[arg(short='s', long, action=ArgAction::SetTrue)]
        skip_trimming: bool,

        /// max error (mistmatch) between provided pattern and read sequence
        #[arg(short = 'e', long, default_value = "1")]
        max_error: usize,

        /// Compress outputs in gzip format
        #[arg(long, action = ArgAction::SetTrue, conflicts_with_all = ["bgz", "mgz", "lz4"])]
        gz: bool,

        /// compress outputs in bgzf (bgzip) format
        #[arg(long, action = ArgAction::SetTrue, conflicts_with_all = ["gz", "mgz", "lz4"])]
        bgz: bool,

        /// compress outputs in mgzip format
        #[arg(long, action = ArgAction::SetTrue, conflicts_with_all = ["gz", "bgz", "lz4"])]
        mgz: bool,

        /// compress outputs in lz4 format
        #[arg(long, action = ArgAction::SetTrue, conflicts_with_all = ["gz", "bgz", "mgz"])]
        lz4: bool,

        /// enable skipping all logs
        #[arg(short = 'q', long, action = ArgAction::SetTrue)]
        quite: bool,
    }
}

#[derive(Debug, clap::Args)]
#[group(required = true, multiple = false)]
pub struct PatternsGroup {
    /// barcode pattern of forward reads
    #[arg(short = 'p', long, requires = "read1")]
    pub pattern1: Option<String>,

    /// barcode pattern of reverse reads
    #[arg(short = 'P', long, requires = "read2")]
    pub pattern2: Option<String>,
}
