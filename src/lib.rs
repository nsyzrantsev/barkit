use clap::{command, ArgAction, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Tool for parsing barcodes from single-end or paired-end FASTQ files
    #[clap(arg_required_else_help = true)]
    Extract {
        /// Input forward FASTQ file
        #[arg(short = '1', long, value_name = "IN_FASTQ1", requires = "out_fq1")]
        fq1: String,

        /// Input reverse FASTQ file
        #[arg(short = '2', long, value_name = "IN_FASTQ2", requires_all = ["fq1", "out_fq2"])]
        fq2: Option<String>,

        /// Output forward FASTQ file
        #[arg(short = 'o', long, value_name = "OUT_FASTQ1")]
        out_fq1: String,

        /// Output reverse FASTQ file
        #[arg(short = 'O', long, value_name = "OUT_FASTQ2", requires = "out_fq1")]
        out_fq2: Option<String>,

        #[clap(flatten)]
        patterns: PatternsGroup,

        /// Max RAM usage in megabytes
        #[arg(short = 'm', long)]
        max_memory: Option<usize>,

        /// The approximate number of threads to use.
        #[arg(short = 't', long, default_value = "1")]
        threads: usize,

        /// Searches for both barcode pattern in reverse complement
        #[arg(short = 'r', long, action=ArgAction::SetTrue)]
        rc_barcodes: bool,

        /// Skip trimming the adapter sequence from the read
        #[arg(short = 's', long, action=ArgAction::SetTrue)]
        skip_trimming: bool,

        /// Max error (mismatch) between provided pattern and read sequence
        #[arg(short = 'e', long, default_value = "1")]
        max_error: usize,

        /// Compress outputs in gzip format
        #[arg(long, action = ArgAction::SetTrue, conflicts_with_all = ["bgz", "mgz", "lz4"])]
        gz: bool,

        /// Compress outputs in bgzf (bgzip) format
        #[arg(long, action = ArgAction::SetTrue, conflicts_with_all = ["gz", "mgz", "lz4"])]
        bgz: bool,

        /// Compress outputs in mgzip format
        #[arg(long, action = ArgAction::SetTrue, conflicts_with_all = ["gz", "bgz", "lz4"])]
        mgz: bool,

        /// Compress outputs in lz4 format
        #[arg(long, action = ArgAction::SetTrue, conflicts_with_all = ["gz", "bgz", "mgz"])]
        lz4: bool,

        /// Be quiet and do not show extra information
        #[arg(short = 'q', long, action = ArgAction::SetTrue)]
        quiet: bool,

        /// Overwrite output files
        #[arg(short = 'f', long, action = ArgAction::SetTrue)]
        force: bool,
    },
}

#[derive(Debug, clap::Args)]
#[group(required = true, multiple = true)]
pub struct PatternsGroup {
    /// Barcode pattern of forward reads
    #[arg(short = 'p', long, requires = "fq1")]
    pub pattern1: Option<String>,

    /// Barcode pattern of reverse reads
    #[arg(short = 'P', long, requires = "fq2")]
    pub pattern2: Option<String>,
}
