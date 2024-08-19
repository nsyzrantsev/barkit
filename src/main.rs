use clap::Parser;

fn main() {
    let args = barkit::Args::parse();

    match &args.command {
        barkit::Commands::Extract {
            read1,
            read2,
            out_read1,
            out_read2,
            max_memory,
            threads,
            rc_barcodes,
            skip_trimming,
            max_error,
            compression_format,
            patterns,
        } => {
            extract::run::run(
                read1.to_string(),
                read2.clone(),
                patterns.pattern1.clone(),
                patterns.pattern2.clone(),
                out_read1.to_string(),
                out_read2.clone(),
                *max_memory,
                *threads,
                *rc_barcodes,
                *skip_trimming,
                *max_error,
                compression_format.clone(),
            );
        }
    }
}
