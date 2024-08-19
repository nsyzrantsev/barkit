# BarKit

> [!WARNING]  
> This tool is under development. Please use the first release version when it becomes available.

BarKit (Barcodes Toolkit) is a toolkit designed for the manipulation of FASTQ barcodes.

## Building from source

```bash
cargo build --release
sudo mv barkit /usr/local/bin/
```


## Extract

The `extract` command is designed to parse UMI barcode sequences from FASTQ reads using approximate regex matching based on a provided pattern.

Example of usage:
```bash
barkit extract -1 in.R1.fastq.gz -2 in.R2.fastq.gz -p "^(?P<UMI>[ATGCN]{12})" -o out.R1.fastq.gz -O out.R2.fastq.gz
```