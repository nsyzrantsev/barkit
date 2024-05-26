# BarKit

BarKit (barcodes toolkit) is an ultrafast toolkit for manipulating FASTQ barcodes.

## extract tool

```bash
barkit extract -1 data/test.fastq.gz -p "^[ATGCN]*T(?P<UMI>[ATGCN]{12})CTCCGCTTAAGGGACT"
```