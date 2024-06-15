use fuzzy_matcher::{create_regex, create_fuzzy_matcher};
use cxx::CxxString;

fn main() {
    let pattern = "^[ATGCN]*T?(P<UMI>[ATGCN]{12})CTCCGCTTAAGGGACT";
    let input = "TCCTCTTAAACTTCCGCATGGCGTCTCCGCTTAAGGGACT";

    let regex = create_regex(pattern);
    println!("{:?}", regex);

    let matcher = create_fuzzy_matcher(pattern, 5, input);
    // println!("{:?}", matcher);
}
