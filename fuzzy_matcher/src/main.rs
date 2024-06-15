use fuzzy_matcher::{create_regex, create_fuzzy_matcher, matches, edits, distance};
use cxx::CxxString;

fn main() {
    let pattern = "^[ATGCN]*T?(P<UMI>[ATGCN]{12})CTCCGCTTAAGGGACT"; // "^[ATGCN]*T?(P<UMI>[ATGCN]{12})CTCCGCTTAAGGGACT";
    let input = "TCCTCTTAAACTTCCGCATGGCGTCTCCGCTTAAGGGACT"; // "TCCTCTTAAACTTCCGCATGGCGTCTCCGCTTAAGGGACT";

    let regex = create_regex(pattern);
    println!("{:?}", regex);

    let matcher = create_fuzzy_matcher(regex, 100, input);

    println!("{}", matches(matcher));
    // println!("{}", edits(matcher));
    // println!("{}", distance(matcher));
}
