# BarKit

BarKit (Barcodes Toolkit) is an extremely fast toolkit designed for the manipulation of FASTQ barcodes. It offers a range of functionalities, including:

* `UMI` Barcodes: The toolkit excels at handling Unique Molecular Identifiers (UMIs) by providing tools for:
    * **Extraction**: Efficiently extracting UMI sequences from FASTQ files.
    * **Clustering**: Grouping similar UMI sequences together to identify unique molecules.
    * **Consensus Sequence Creation**: Generating consensus sequences from clusters of UMI barcodes to ensure accuracy and reduce sequencing errors.

## Extract

The Extract command is designed to parse UMI barcode sequences from FASTQ reads using approximate regex matching based on a provided pattern. 

```bash
barkit extract -1 R1.fastq.gz -p "^[ATGCN]*T?(?P<UMI>[ATGCN]{12})CTCCGCTTAAGGGACT{ 1s<3 }" -o R1.extracted.fastq.gz
```