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
        /// (gzipped) read1 file
        #[arg(short='1', long)]
        read1: String,

        /// (gzipped) read2 file
        #[arg(short='2', long, requires = "read1")]
        read2: Option<String>,
    },
}