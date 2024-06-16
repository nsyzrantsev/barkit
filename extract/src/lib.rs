mod fastq;
mod errors;
mod barcode;

use seq_io::fastq::Reader;
use barcode::Barcode;

use regex_syntax::{hir::{Hir, HirKind, Literal}, Parser};
use std::str;

fn generate_patterns(literal: &[u8]) -> Vec<String> {
    let mut patterns = Vec::new();
    for i in 0..literal.len() {
        let mut pattern = literal.to_vec();
        pattern[i] = b'.';
        patterns.push(String::from_utf8(pattern).unwrap());
    }
    patterns
}

fn print_hir(hir: &Hir) {
    match hir.kind() {
        HirKind::Empty => println!("Empty {:?}", hir),
        HirKind::Literal(Literal(bytes)) => {
            let bytes_ref: &[u8] = &bytes;
            let s = std::str::from_utf8(bytes_ref).unwrap();
            println!("Literal: {:?}", s);
            let patterns = generate_patterns(bytes_ref);
            let new_pattern = patterns.join("|");
            println!("Updated Pattern: {:?}", new_pattern);
        },
        HirKind::Class(class) => println!("Class: {:?}", class),
        HirKind::Concat(hirs) => {
            println!("Concat: {:?}", hirs);
            for hir in hirs {
                print_hir(hir);
            }
        },
        HirKind::Alternation(hirs) => {
            println!("Alternation: {:?}", hirs);
            for hir in hirs {
                print_hir(hir);
            }
        },
        HirKind::Look(hirs) => println!("{:?}", hirs),
        HirKind::Repetition(hirs) => {
            println!("Repetition: {:?}", hirs);
            print_hir(&hirs.sub);

        },
        HirKind::Capture(hirs) => {
            println!("Capture: {:?}", hirs);
            print_hir(&hirs.sub);
        },
    }
}


pub fn run(read1: String, read2: Option<String>, pattern: String) {
    
    let fastq_buf = fastq::read_fastq(&read1);

    let mut reader = Reader::new(fastq_buf);

    let barcode = Barcode::new(&pattern).expect("REASON");

    // while let Some(record) = reader.next() {
    //     let record = record.expect("Error reading record");
    //     let caps = barcode.match_read(&record).unwrap();
    //     let (read_seq, read_qual, read_header) = Barcode::cut_from_read_seq("UMI", caps, &record).unwrap();
    //     println!("{}\n{}\n+\n{}", read_header, read_seq, read_qual);
    // }

    let mut parser = Parser::new();
    match parser.parse(&pattern) {
        Ok(hir) => {
            println!("Parsed HIR: {:?}", hir);
            print_hir(&hir);
        }
        Err(err) => println!("Error parsing regex: {}", err),
    }

}