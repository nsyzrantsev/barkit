use regex::{self, Regex};
use tre_regex::{fuzzy::FuzzyRegexParams, flags::{RegcompFlags, RegexecFlags}, TreRegex, Match};
use std::{collections::HashMap, str};
use seq_io::fastq::{Record, RefRecord};

use crate::errors::Error;

pub struct Barcode {
    compiled_regex: TreRegex,
    caputure_groups: HashMap<String, usize>,
    regaexec_flags: RegexecFlags,
    regaexec_params: FuzzyRegexParams
}

impl Barcode {
    pub fn new(pattern: &str, max_mismatch: usize) -> Result<Self, Error> {
        let caputure_groups = get_capture_group_indices(&pattern);
        let posix_pattern = remove_capture_groups(pattern);
        let regcomp_flags = RegcompFlags::new()
            .add(RegcompFlags::EXTENDED)
            .add(RegcompFlags::ICASE);
        let compiled_regex = TreRegex::new_bytes(posix_pattern.as_bytes(), regcomp_flags).expect("Regex::new");
        let regaexec_flags = RegexecFlags::new().add(RegexecFlags::NONE);
        let regaexec_params = FuzzyRegexParams::new()
            .cost_insertion(0)
            .cost_deletion(0)
            .cost_substitution(1)
            .max_cost(2)
            .max_deletion(0)
            .max_insertion(0)
            .max_substitution(max_mismatch as i32)
            .max_error(2);

        Ok(Self {
            compiled_regex,
            caputure_groups,
            regaexec_flags,
            regaexec_params
        })
    }

    pub fn match_read<'a>(&self, read: &'a RefRecord) -> Result<Match<'a>, Error> {
        let read_seq = str::from_utf8(read.seq())?;
    
        let result = self.compiled_regex
            .regaexec_bytes(
                read_seq.as_bytes(),
                &self.regaexec_params,
                3,
                self.regaexec_flags,
            )?;
        
        let matched = result.get_matches();

        let capture_group_index = self.caputure_groups["UMI"];

        Ok(matched[capture_group_index].clone().unwrap())
    }

    pub fn cut_from_read_seq(barcode_type: &str, matched_pattern: Match, read: &RefRecord) -> Result<(Vec::<u8>, Vec::<u8>, Vec::<u8>), Error> {
        let start = matched_pattern.start();
        let end = matched_pattern.end();

        let read_seq = read.seq();
        let read_qual = read.qual();

        let mut new_read_seq = [&read_seq[..start], &read_seq[end..]].concat();
        let mut new_read_qual = [&read_qual[..start], &read_qual[end..]].concat();

        let mut new_read_header = Self::move_to_the_header(barcode_type, read, start, end)?;

        new_read_header.push(b'\n');
        new_read_seq.push(b'\n');
        new_read_qual.push(b'\n');
        
        Ok((new_read_seq, new_read_qual, new_read_header))
    }

    fn move_to_the_header(barcode_type: &str, read: &RefRecord, start: usize, end: usize) -> Result<Vec::<u8>, Error> {
        let read_header = str::from_utf8(read.head())?;
        let read_seq = read.seq();
        let read_qual = read.qual();

        let barcode_seq = str::from_utf8(&read_seq[start..end])?;
        let barcode_qual = str::from_utf8(&read_qual[start..end])?;

        Ok(format!("{} {}:{}:{}", read_header, barcode_type, barcode_seq, barcode_qual).as_bytes().to_vec())
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