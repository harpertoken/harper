// Copyright 2026 harpertoken
// Licensed under the Apache License, Version 2.0
// See top-level LICENSE files for details.

use assert_cmd::Command;
use harper_core::core::constants;

#[test]
fn harper_cli_reports_exact_version() -> Result<(), Box<dyn std::error::Error>> {
    let expected = format!("harper v{}", constants::VERSION);

    let assert = Command::cargo_bin("harper")?
        .arg("--version")
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone())?;
    assert_eq!(
        stdout.trim(),
        expected,
        "CLI returned unexpected version string"
    );

    Ok(())
}
