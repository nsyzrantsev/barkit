use regex::{self, Regex, Captures};
use tre_regex::{RegApproxParams, RegcompFlags, Regex as TreRegex, RegexError, RegexecFlags};
use std::{borrow::Cow, str};
use seq_io::fastq::{Record, RefRecord};

use crate::errors::Error;

pub struct Barcode {
    compiled_reg: TreRegex,
}

impl Barcode {
    pub fn new(pattern: &str) -> Result<Self, Error> {
        let regcomp_flags = RegcompFlags::new()
            .add(RegcompFlags::EXTENDED)
            .add(RegcompFlags::ICASE);
        let compiled_reg = TreRegex::new_bytes(pattern.as_bytes(), regcomp_flags).expect("Regex::new");

        Ok(Self { compiled_reg })
    }

    pub fn match_read<'a>(&self, read: &'a RefRecord) -> Result<(Cow<'a, [u8]>, usize, usize), Error> {
        let read_seq = str::from_utf8(read.seq())?;
        
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
    
        let result = self.compiled_reg
            .regaexec_bytes(
                read_seq.as_bytes(),           // String to match against
                &regaexec_params,   // Matching parameters
                3,                  // Number of matches we want
                regaexec_flags,     // Flags
            ).unwrap();
        
        let matched = result.get_matches();
    
        Ok(matched[1].clone().unwrap())
    }

    pub fn cut_from_read_seq(barcode_type: &str, matched_pattern: (std::borrow::Cow<[u8]>, usize, usize), read: &RefRecord) -> Result<(String, String, String), Error> {
        let start = matched_pattern.1;
        let end = matched_pattern.2;

        let read_seq = read.seq();
        let read_qual = read.qual();

        let new_read_seq = [&read_seq[..start], &read_seq[end..]].concat();
        let new_read_qual = [&read_qual[..start], &read_qual[end..]].concat();

        let new_read_header = Self::move_to_the_header(barcode_type, read, start, end)?;
        Ok((String::from_utf8(new_read_seq)?, String::from_utf8(new_read_qual)?, new_read_header))
    }

    fn move_to_the_header(barcode_type: &str, read: &RefRecord, start: usize, end: usize) -> Result<String, Error> {
        let read_header = str::from_utf8(read.head())?;
        let read_seq = read.seq();
        let read_qual = read.qual();

        let barcode_seq = str::from_utf8(&read_seq[start..end])?;
        let barcode_qual = str::from_utf8(&read_qual[start..end])?;

        Ok(format!("{} {}:{}:{}", read_header, barcode_type, barcode_seq, barcode_qual))
    }
}