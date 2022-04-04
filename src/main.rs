use rs_batch_process_txns::process_transactions_file;

fn cli(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let input_file = args.get(0);

    if let Some(input_file) = input_file {
        process_transactions_file(input_file.to_string())
    } else {
        Err(
            "Expected exactly one intput argument - the transactions file you want to process"
                .into(),
        )
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    cli(&args[1..])
}

#[cfg(test)]
mod tests {
    use super::*;
}
