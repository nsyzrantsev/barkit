use clap::{command, ArgAction, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,

    /// Max RAM usage in megabytes
    #[arg(short = 'm', long)]
    pub max_memory: Option<usize>,

    /// The approximate number of threads to use.
    #[arg(short = 't', long, default_value = "1", global = true)]
    pub threads: usize,

    /// Be quiet and do not show extra information
    #[arg(long, action = ArgAction::SetTrue, global = true)]
    pub quiet: bool,

    /// Overwrite output files
    #[arg(short = 'f', long, action = ArgAction::SetTrue, global = true)]
    pub force: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Extract barcode nucleotide sequence according to a specified regex pattern
    #[clap(arg_required_else_help = true)]
    Extract {
        #[clap(flatten)]
        input_fastqs: InputsGroup,

        #[clap(flatten)]
        output_fastqs: OutputsGroup,

        #[clap(flatten)]
        patterns: PatternsGroup,

        #[clap(flatten)]
        compression: CompressionGroup,

        #[clap(flatten)]
        additional_params: AdditionalParamsGroup,
    },
}

#[derive(Debug, clap::Args)]
pub struct InputsGroup {
    /// Input forward FASTQ file
    #[arg(short = '1', long, value_name = "IN_FASTQ1", requires = "out_fq1")]
    pub fq1: String,

    /// Input reverse FASTQ file
    #[arg(short = '2', long, value_name = "IN_FASTQ2", requires_all = ["fq1", "out_fq2"])]
    pub fq2: Option<String>,
}

#[derive(Debug, clap::Args)]
pub struct OutputsGroup {
    /// Output forward FASTQ file
    #[arg(short = 'o', long, value_name = "OUT_FASTQ1")]
    pub out_fq1: String,

    /// Output reverse FASTQ file
    #[arg(short = 'O', long, value_name = "OUT_FASTQ2", requires = "out_fq1")]
    pub out_fq2: Option<String>,
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

#[derive(Debug, clap::Args)]
pub struct CompressionGroup {
    /// Compress outputs in gzip format
    #[arg(long, action = ArgAction::SetTrue, conflicts_with_all = ["bgz", "mgz", "lz4"])]
    pub gz: bool,

    /// Compress outputs in bgzf (bgzip) format
    #[arg(long, action = ArgAction::SetTrue, conflicts_with_all = ["gz", "mgz", "lz4"])]
    pub bgz: bool,

    /// Compress outputs in mgzip format
    #[arg(long, action = ArgAction::SetTrue, conflicts_with_all = ["gz", "bgz", "lz4"])]
    pub mgz: bool,

    /// Compress outputs in lz4 format
    #[arg(long, action = ArgAction::SetTrue, conflicts_with_all = ["gz", "bgz", "mgz"])]
    pub lz4: bool,
}

#[derive(Debug, clap::Args)]
pub struct AdditionalParamsGroup {
    /// Searches for both barcode pattern in reverse complement
    #[arg(short = 'r', long, action=ArgAction::SetTrue)]
    pub rc_barcodes: bool,

    /// Skip trimming the adapter sequence from the read
    #[arg(short = 's', long, action=ArgAction::SetTrue)]
    pub skip_trimming: bool,

    /// Max error (mismatch) between provided pattern and read sequence
    #[arg(short = 'e', long, default_value = "1")]
    pub max_error: usize,
}
