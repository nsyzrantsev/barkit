use regex::Regex;

mod fastq;
mod bitvector;
mod genasm;

use clap::{command, Parser};
use seq_io::fastq::{Reader, Record};


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
    
    let fastq_buf = fastq::read_fastq(&args.read1);

    println!("{:?}", fastq::print_fastq(fastq_buf));

    // let re = Regex::new(r"^[ATGCN]*T(?P<UMI>[ATGCN]{12})[ATGCN]{3}CGCTTAAGGGACT").unwrap(); // ^[ATGCN]*T(?P<UMI>[ATGCN]{12})CTCCGCTTAAGGGACT
    // let read = "NATGTCTTAAACTTCCGCATGGCGTAGAGTAAACGGGCTCCGCTTAAGGGACTTCCGCATGGCGTAGAGTAAACGGGCTCCGCTTAAGGGACT";

    // let Some(caps) = re.captures(read) else {
    //     println!("no match!");
    //     return;
    // };

    // println!("The UMI is: {}", &caps["UMI"]);

    // assert_eq!("AGAGTAAACGGG", &caps["UMI"]);
}