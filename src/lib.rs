use clap::{command, ArgAction, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
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
        #[arg(short='1', long, requires = "out_read1")]
        read1: String,

        /// (gzipped) input reverse FASTQ file
        #[arg(short='2', long, requires_all = ["read1", "out_read2"])]
        read2: Option<String>,

        /// (gzipped) output forward FASTQ file
        #[arg(short='o', long)]
        out_read1: String,

        /// (gzipped) output reverse FASTQ file
        #[arg(short='O', long, requires = "out_read1")]
        out_read2: Option<String>,

        #[clap(flatten)]
        patterns: PatternsGroup,

        /// max memory (RAM) usage in megabytes (MB)
        #[arg(short='m', long)]
        max_memory: Option<usize>,

        /// the approximate number of threads to use.
        #[arg(short='t', long, default_value = "1")]
        threads: usize,

        /// searches for both barcode pattern in reverse complement
        #[arg(short='r', long, action=ArgAction::SetTrue)]
        rc_barcodes: bool,

        /// skip trimming adapter sequence from the read
        #[arg(short='s', long, action=ArgAction::SetTrue)]
        skip_trimming: bool,

        /// max error (mistmatch) between provided pattern and read sequence
        #[arg(short='e', long, default_value = "1")]
        max_error: usize,

        /// compression format for output FASTQ files
        #[arg(short='c', long, default_value = "bgzf", value_parser = ["gzip", "bgzf", "no"])]
        compression_format: String,
    },
}

#[derive(Debug, clap::Args)]
#[group(required = true, multiple = false)]
pub struct PatternsGroup {
    /// barcode pattern of forward reads
    #[arg(short='p', long, requires = "read1")]
    pub pattern1: Option<String>,

    /// barcode pattern of reverse reads
    #[arg(short='P', long, requires = "read2")]
    pub pattern2: Option<String>,
}