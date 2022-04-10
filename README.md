# rs_bpt

An experiment in batch processing transactions in Rust.


## Usage

Simple usage, displaying output to stdout:

```
cargo run -- <transactions-file.csv>
```

To save output to a file:

```
cargo run -- <transactions-file.csv> > <output-file.csv>
```

To include debug logging to stderr which shows errors such as invalid transactions, include either `--debug` or `-d`. For example:

```
cargo run -- --debug tests/fixtures/transactions-with-dupes.csv > accounts.csv
```

In this case, `accounts.csv` will contain the well-formed account information and stderr will be displayed in the terminal. You can also pipe it to a seperate file, for example:

```
cargo run -- --debug tests/fixtures/transactions-with-dupes.csv > accounts.csv 2> errors.log
```

## Tests

To run tests:

```
cargo test
```
