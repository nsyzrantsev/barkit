mod bitvector;
mod genasm;
use regex::Regex;

fn main() {
    let re = Regex::new(r"^[ATGCN]*T(?P<UMI>[ATGCN]{12})[ATGCN]{3}CGCTTAAGGGACT").unwrap(); // ^[ATGCN]*T(?P<UMI>[ATGCN]{12})CTCCGCTTAAGGGACT
    let read = "NATGTCTTAAACTTCCGCATGGCGTAGAGTAAACGGGCTCCGCTTAAGGGACTTCCGCATGGCGTAGAGTAAACGGGCTCCGCTTAAGGGACT";

    let Some(caps) = re.captures(read) else {
        println!("no match!");
        return;
    };

    println!("The UMI is: {}", &caps["UMI"]);

    assert_eq!("AGAGTAAACGGG", &caps["UMI"]);
}
