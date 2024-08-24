use clap::Parser;
use extract::io::CompressionType;

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
            patterns,
            gz,
            bgz,
            mgz,
            lz4
        } => {
            let output_compression = extract::io::CompressionType::get_output_compression_type(gz, bgz, mgz, lz4);
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
                output_compression
            );
        }
    }
}
