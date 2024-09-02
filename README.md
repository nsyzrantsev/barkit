# BarKit

BarKit (**Bar**codes Tool**Kit**) is a toolkit designed for manipulating FASTQ barcodes.

## Installation

### From crates.io

Barkit can be installed from [`crates.io`](https://crates.io/crates/barkit) using `cargo`. This can be done with the following command:

```bash
cargo install barkit
```

### Build from source

1. Clone the repository:

```bash
git clone https://github.com/nsyzrantsev/barkit
cd barkit/
```

2. Build:

```bash
cargo build --release && sudo mv target/release/barkit /usr/local/bin/
```

## Extract subcommand

The extract subcommand is designed to parse barcode sequences from FASTQ reads using approximate regex matching based on a provided pattern.

All parsed barcode sequences are moved to the read header with base quality, separated by colons:

```
@SEQ_ID UMI:ATGC:???? CB:ATGC:???? SB:ATGC:????
```

* **UMI**: Unique Molecular Identifier (Molecular Barcode)
* **CB**: Cell Barcode
* **SB**: Sample Barcode


### Examples

Parse the first twelve nucleotides as a UMI from each forward read:

```bash
barkit extract -1 <IN_FASTQ1> -2 <IN_FASTQ2> -p "^(?P<UMI>[ATGCN]{12})" -o <OUT_FASTQ1> -O <OUT_FASTQ2>
```

Parse the first sixteen nucleotides as a cell barcode from each reverse read before the `atgccat` adapter sequence:

```bash
barkit extract -1 <IN_FASTQ1> -2 <IN_FASTQ2> -P "^(?P<CB>[ATGCN]{16})atgccat" -o <OUT_FASTQ1> -O <OUT_FASTQ2>
```

> [!NOTE]
> Use lowercase letters for fuzzy match patterns.