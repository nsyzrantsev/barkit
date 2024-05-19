use regex::Regex;

mod fastq;
mod bitvector;
mod genasm;

use clap::{command, Parser, Subcommand};
use seq_io::fastq::{Reader, Record};


pub fn run(read1: String, read2: Option<String>) {
    
    let fastq_buf = fastq::read_fastq(&read1);

    println!("{:?}", fastq::print_fastq(fastq_buf));
}