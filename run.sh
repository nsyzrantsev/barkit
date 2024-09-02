cargo run -r -- extract \
-1 ~/tests/ERR7425614/ERR7425614_1.fastq.gz \
-2 ~/tests/ERR7425614/ERR7425614_2.fastq.gz \
-o /tmp/out.R1.fastq.gz \
-O /tmp/out.R2.fastq.gz \
-p "^(?<UMI>[ATGCN]{20})" \
--force

# fastp -i ~/tests/ERR7425614/ERR7425614_1.fastq.gz \
#  -I ~/tests/ERR7425614/ERR7425614_2.fastq.gz \
#  --umi \
#  --umi_loc read1 \
#  --umi_len 20 \
#  -o /tmp/R1.fastq \
#  -O /tmp/R2.fastq

