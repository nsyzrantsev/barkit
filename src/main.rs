use clap::Parser;
use extract;

fn main() {
    let args = barkit::Args::parse();

    match &args.command {
        barkit::Commands::Extract {
            read1,
            read2
        } => {
            extract::run(read1.to_string(), read2.clone());
        }
    }
}
