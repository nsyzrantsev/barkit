use clap::Parser;

fn main() {
    let args = barkit::Args::parse();

    match &args.command {
        barkit::Commands::Extract {
            input_fastqs,
            output_fastqs,
            resources,
            additional_params,
            patterns,
            compression,
            quiet,
            force,
        } => {
            let output_compression =
                barkit_extract::fastq::CompressionType::select(
                    &compression.gz, &compression.bgz, &compression.mgz, &compression.lz4,
                );
            barkit_extract::run::run(
                input_fastqs.fq1.to_string(),
                input_fastqs.fq2.clone(),
                patterns.pattern1.clone(),
                patterns.pattern2.clone(),
                output_fastqs.out_fq1.to_string(),
                output_fastqs.out_fq2.clone(),
                resources.max_memory,
                resources.threads,
                additional_params.rc_barcodes,
                additional_params.skip_trimming,
                additional_params.max_error,
                output_compression,
                *quiet,
                *force,
            );
        }
    }
}
