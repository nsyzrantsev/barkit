use clap::Parser;
use extract;

fn main() {
    let args = barkit::Args::parse();

    match &args.command {
        barkit::Commands::Extract {
            read1,
            read2,
            pattern,
            max_mismatch,
            out_read1,
            out_read2
        } => {
            extract::run(
                read1.to_string(),
                read2.clone(), 
                pattern.clone(), 
                max_mismatch.clone(),
                out_read1.clone(),
                out_read2.clone()
            );
        }
    }
}
