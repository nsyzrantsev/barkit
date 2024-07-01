use regex::Regex;

mod fastq;

use clap::{command, Parser};
use seq_io::fastq::{Reader, Record};

use tre_regex::{RegApproxParams, RegcompFlags, Regex as TreRegex, RegexecFlags};


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// (gzipped) read1 file
    #[arg(long, short='1')]
    read1: String,

    /// (gzipped) read2 file
    #[arg(long, short='2', requires = "read1")]
    read2: Option<String>,
}

fn main() {

    let args = Args::parse();
}

