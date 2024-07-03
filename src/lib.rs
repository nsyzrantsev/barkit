use clap::{command, Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(version)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}


#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Preprocess molecular, cell and sample barcodes
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
        out_read1: Option<String>,

        /// (gzipped) output reverse FASTQ file
        #[arg(short='O', long, requires = "out_read1")]
        out_read2: Option<String>,

        /// barcode pattern of read1
        #[arg(short='p', long)]
        pattern: String,

        /// max mismatch with pattern
        #[arg(short='m', long, default_value = "2")]
        max_mismatch: usize,
    },
}