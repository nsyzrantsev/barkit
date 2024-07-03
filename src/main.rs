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
            max_mismatch,
        } => {
            extract::run(
                read1.to_string(),
                read2.clone(), 
                pattern1.clone(),
                pattern2.clone(),
                out_read1.clone(),
                out_read2.clone(),
                max_mismatch.clone(),
            );
        }
    }
}
