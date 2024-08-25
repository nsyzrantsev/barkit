# BarKit

> [!WARNING]  
> This tool is under development. Please use the first release version when it becomes available.

BarKit (Barcodes Toolkit) is a toolkit designed for the manipulation of FASTQ barcodes.

## Building from source

```bash
cargo build --release
sudo mv barkit /usr/local/bin/
```


## Extract command

The `extract` command is designed to parse any barcode sequence from FASTQ reads using approximate regex matching based on a provided pattern.

All parsed barcode sequences are moved to the read header with base quality separated colon:

```
@SEQ_ID UMI:ATGC:???? CB:ATGC:???? SB:ATGC:????
```

* `UMI` means UMI (molecular barcode)
* `CB` = cell barcode
* `SB` = sample barcode

> [!NOTE]
> `barkit extract` supports both reads: **single-end** and **paired-end**!

> [!WARNING]
> barcode pattern should be in [Unicode](https://github.com/rust-lang/regex/blob/master/UNICODE.md) format.

### Examples


Parse first twelve nucleotides as an UMI from each read:
```
barkit extract -1 <IN_FASTQ1> -2 <IN_FASTQ2> -p "^(?P<UMI>[ATGCN]{12})" -o <OUT_FASTQ1> -O <OUT_FASTQ2>
```


Parse first sixteen nucleotides as an single cell barcode from each read before `atgccat` sequence:
```
barkit extract -1 <IN_FASTQ1> -2 <IN_FASTQ2> -p "^(?P<CB>[ATGCN]{16})atgccat" -o <OUT_FASTQ1> -O <OUT_FASTQ2>
```

> [!NOTE]
> Use lower case letters for fuzzy match patterns

