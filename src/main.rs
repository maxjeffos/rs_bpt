use std::path::PathBuf;
use structopt::StructOpt;

use rs_bpt::cli;

#[derive(StructOpt, Debug)]
#[structopt(name = "rs_bpt", about = "Batch process transactions")]
struct Opt {
    /// debug mode
    #[structopt(short, long)]
    debug: bool,

    /// Input file
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();
    let transactions_file = opt.input;
    let debug = opt.debug;

    let mut debug_logger: Box<dyn std::io::Write> = if debug {
        Box::new(std::io::stderr())
    } else {
        Box::new(std::io::sink())
    };

    cli(transactions_file, &mut debug_logger)
}
