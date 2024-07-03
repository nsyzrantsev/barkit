use clap::Parser;
use extract;

fn main() {
    let args = barkit::Args::parse();

    match &args.command {
        barkit::Commands::Extract {
            read1,
            read2,
            pattern,
            max_mismatch
        } => {
            extract::run(read1.to_string(), read2.clone(), pattern.clone(), max_mismatch.clone());
        }
    }
}
