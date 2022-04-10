use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn test_simple() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("rs_bpt")?;
    cmd.arg("tests/fixtures/transactions.csv");

    cmd.assert().success();
    cmd.assert().stdout(predicate::str::contains(
        "client,available,held,total,locked",
    ));
    cmd.assert()
        .stdout(predicate::str::contains("2,-1.0000,0.0000,-1.0000,false"));
    cmd.assert()
        .stdout(predicate::str::contains("1,1.5000,0.0000,1.5000,false"));

    Ok(())
}

#[test]
fn it_works_without_errors() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("rs_bpt")?;
    cmd.arg("tests/fixtures/transactions.csv");
    cmd.arg("--debug");

    cmd.assert().success();
    cmd.assert().stdout(predicate::str::contains(
        "client,available,held,total,locked",
    ));
    cmd.assert()
        .stdout(predicate::str::contains("2,-1.0000,0.0000,-1.0000,false"));
    cmd.assert()
        .stdout(predicate::str::contains("1,1.5000,0.0000,1.5000,false"));

    cmd.assert().stderr(predicate::str::is_empty());

    Ok(())
}

#[test]
fn it_ignores_dupe_transaction_id_but_logs_error_if_debug_mode(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("rs_bpt")?;
    cmd.arg("tests/fixtures/transactions-with-dupes.csv");
    cmd.arg("--debug");

    cmd.assert().success();
    cmd.assert().stdout(predicate::str::contains(
        "client,available,held,total,locked",
    ));
    cmd.assert()
        .stdout(predicate::str::contains("1,1.0000,0.0000,1.0000,false"));
    cmd.assert()
        .stderr(predicate::str::contains("TransactionIDAlreadyExists"));
    cmd.assert()
        .stderr(predicate::str::contains("transaction_id: 1"));

    Ok(())
}

#[test]
fn test_complex() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("rs_bpt")?;
    cmd.arg("tests/fixtures/transactions-complex.csv");
    cmd.arg("--debug");

    // Because the order of the clients in the output does not matter
    // and because the clients are stored in a HashMap which doesn't preserve order,
    // I'll assert the output should be one or the other or the following.
    let expected_stdout_order1 = r#"client,available,held,total,locked
1,110.0000,0.0000,110.0000,false
2,1000.0000,0.0000,1000.0000,true
"#;
    let expected_stdout_order2 = r#"client,available,held,total,locked
2,1000.0000,0.0000,1000.0000,true
1,110.0000,0.0000,110.0000,false
"#;

    cmd.assert().success();
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout == expected_stdout_order1 || stdout == expected_stdout_order2);

    cmd.assert().stderr(predicate::str::is_empty());

    Ok(())
}
