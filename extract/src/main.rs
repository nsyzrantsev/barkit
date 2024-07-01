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

    // let args = Args::parse();
    
    // let fastq_buf = fastq::read_fastq(&args.read1);

    // println!("{:?}", fastq::print_fastq(fastq_buf));
    
    let regcomp_flags = RegcompFlags::new()
        .add(RegcompFlags::EXTENDED)
        .add(RegcompFlags::ICASE);
    let regaexec_flags = RegexecFlags::new().add(RegexecFlags::NONE);
    let regaexec_params = RegApproxParams::new()
        .cost_ins(0)
        .cost_del(0)
        .cost_subst(1)
        .max_cost(2)
        .max_del(0)
        .max_ins(0)
        .max_subst(2)
        .max_err(2);

    let compiled_reg = TreRegex::new_bytes(b"^[ATGCN]*T([ATGCN]{12})CTCCGCTTAAGGGACT", regcomp_flags).expect("Regex::new");
    let result = compiled_reg
        .regaexec_bytes(
            b"NATGTCTTAAACTTCCGCATGGCGTAGAGTAAACGGGCTCCGCTTAAGGGACT",   // String to match against
            &regaexec_params, // Matching parameters
            3,                // Number of matches we want
            regaexec_flags,   // Flags
        )
        .expect("regaexec");

    let matched = result.get_matches();

    let matched_0 = matched[0].as_ref();
    assert!(matched_0.is_some());
    assert_eq!(matched_0.unwrap().0.as_ref(), b"NATGTCTTAAACTTCCGCATGGCGTAGAGTAAACGGGCTCCGCTTAAGGGACT");

    let matched_1 = matched[1].as_ref();
    assert!(matched_1.is_some());
    assert_eq!(matched_1.unwrap().0.as_ref(), b"AGAGTAAACGGG");
    
    let start = matched_1.unwrap().1;
    let end = matched_1.unwrap().2;
    
    let read = std::str::from_utf8(matched_0.unwrap().0.as_ref()).unwrap();
    assert_eq!(&read[start..end], "AGAGTAAACGGG");
}

