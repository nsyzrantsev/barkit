use clap::Parser;
use extract;

fn main() {
    let args = barkit::Args::parse();

    match &args.command {
        barkit::Commands::Extract {
            read1,
            read2,
            out_read1,
            out_read2,
            pattern1,
            pattern2,
            max_memory,
            threads,
            rc_barcodes,
            max_error
        } => {
            extract::run(
                read1.to_string(),
                read2.clone(), 
                pattern1.clone(),
                pattern2.clone(),
                out_read1.to_string(),
                out_read2.clone(),
                max_memory.clone(),
                threads.clone(),
                rc_barcodes.clone(),
                max_error.clone()
            );
        }
    }
}
