use rs_batch_process_txns::process_transactions_file;

fn cli(
    args: &[String],
    debug_logger: &mut Option<impl std::io::Write>,
) -> Result<(), Box<dyn std::error::Error>> {
    let input_file = args.get(0);

    if let Some(input_file) = input_file {
        process_transactions_file(input_file.to_string(), debug_logger)
    } else {
        Err(
            "Expected exactly one intput argument - the transactions file you want to process"
                .into(),
        )
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let debug_logger = &mut Some(std::io::stderr());
    let args: Vec<String> = std::env::args().collect();

    cli(&args[1..], debug_logger)
}
