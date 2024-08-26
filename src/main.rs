use clap::Parser;

fn main() {
    let args = barkit::Args::parse();

    match &args.command {
        barkit::Commands::Extract {
            fq1,
            fq2,
            out_fq1,
            out_fq2,
            max_memory,
            threads,
            rc_barcodes,
            skip_trimming,
            max_error,
            patterns,
            gz,
            bgz,
            mgz,
            lz4,
            quiet,
            force
        } => {
            let output_compression =
                extract::fastq::CompressionType::get_output_compression_type(gz, bgz, mgz, lz4);
            extract::run::run(
                fq1.to_string(),
                fq2.clone(),
                patterns.pattern1.clone(),
                patterns.pattern2.clone(),
                out_fq1.to_string(),
                out_fq2.clone(),
                *max_memory,
                *threads,
                *rc_barcodes,
                *skip_trimming,
                *max_error,
                output_compression,
                *quiet,
                *force
            );
        }
    }
}
