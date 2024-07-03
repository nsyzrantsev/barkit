use regex::{self, Regex, Captures};
use tre_regex::{RegApproxParams, RegcompFlags, Regex as TreRegex, RegexError, RegexecFlags};
use std::{borrow::Cow, collections::HashMap, str};
use seq_io::fastq::{Record, RefRecord};

use crate::errors::Error;

pub struct Barcode {
    compiled_reg: TreRegex,
    caputure_groups: HashMap<String, usize>
}

impl Barcode {
    pub fn new(pattern: &str) -> Result<Self, Error> {
        let caputure_groups = get_capture_group_indices(&pattern);
        let modified_pattern = remove_capture_groups(pattern);
        let regcomp_flags = RegcompFlags::new()
            .add(RegcompFlags::EXTENDED)
            .add(RegcompFlags::ICASE);
        let compiled_reg = TreRegex::new_bytes(modified_pattern.as_bytes(), regcomp_flags).expect("Regex::new");

        Ok(Self { compiled_reg, caputure_groups })
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
                read_seq.as_bytes(),
                &regaexec_params,
                3,
                regaexec_flags,
            )?;
        
        let matched = result.get_matches();

        let capture_group_index = self.caputure_groups["UMI"];

        Ok(matched[capture_group_index].clone().unwrap())
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

fn get_capture_group_indices(pattern: &str) -> HashMap<String, usize> {
    let re = Regex::new(pattern).unwrap();
    let mut group_indices = HashMap::new();
    
    for (i, name) in re.capture_names().enumerate() {
        if let Some(name) = name {
            group_indices.insert(name.to_string(), i);
        }
    }

    group_indices
}

fn remove_capture_groups(pattern: &str) -> String {
    let re = Regex::new(r"\?P<\w*>").unwrap();
    re.replace(pattern, "").to_string()
}