use regex::Regex;

mod fastq;

use clap::{command, Parser};
use seq_io::fastq::{Reader, Record};

use pyo3::prelude::*;
use pyo3::types::PyTuple;


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

fn main() -> PyResult<()> {

    // let args = Args::parse();
    
    // let fastq_buf = fastq::read_fastq(&args.read1);

    // println!("{:?}", fastq::print_fastq(fastq_buf));
    

    pyo3::prepare_freethreaded_python();

    let pattern = "^[ATGCN]*T(?P<UMI>[ATGCN]{12})[ATGCN]{3}CGCTTAAGGGACT";
    let text = "NATGTCTTAAACTTCCGCATGGCGTAGAGTAAACGGGCTCCGCTTAAGGGACTTCCGCATGGCGTAGAGTAAACGGGCTCCGCTTAAGGGACT";

    Python::with_gil(|py| {
        let module_code = r#"
import regex

def match_barcode(pattern, text):
    match = regex.search(pattern, text, regex.BESTMATCH)
    if match:
        barcode_seq = match.group("UMI").replace("N", "A")
        barcode_start, barcode_end = match.span("UMI")
        pattern_match_start, pattern_match_end = match.span()
        return barcode_start, barcode_end, pattern_match_start, pattern_match_end
    return None, None, None, None
"#;

        let module = PyModule::from_code(py, module_code, "", "")?;
        let fun = module.getattr("match_barcode")?;

        let args = (pattern, text);
        let result = fun.call1(args)?;
        
        // Convert result to Rust tuple
        let result_tuple: (Option<i64>, Option<i64>, Option<i64>, Option<i64>) = result.extract()?;

        println!("UMI barcode start: {}", result_tuple.0.unwrap());
        println!("UMI barcode end: {}", result_tuple.1.unwrap());
        println!("Pattern match start: {}", result_tuple.2.unwrap());
        println!("Pattern mathc end: {}", result_tuple.3.unwrap());

        Ok(())
    })
}

