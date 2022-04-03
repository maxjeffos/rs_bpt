use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn trivial() {
    assert_eq!(1, 1);
}

#[test]
fn it_works() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("rs_batch_process_txns")?;
    cmd.arg("tests/fixtures/transactions.csv");

    cmd.assert().success();
    cmd.assert().stdout(predicate::str::contains(
        "client,available,held,total,locked",
    ));
    cmd.assert()
        .stdout(predicate::str::contains("2,-1.0,0.0,-1.0,false"));
    cmd.assert()
        .stdout(predicate::str::contains("1,1.5,0.0,1.5,false"));

    Ok(())
}
