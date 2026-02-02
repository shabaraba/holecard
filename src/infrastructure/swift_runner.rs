use anyhow::{Context, Result};
use std::io::Write;
use std::process::Command;

pub struct SwiftOutput {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

pub fn run_swift(script: &str) -> Result<SwiftOutput> {
    let mut child = Command::new("swift")
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .context("Failed to spawn swift")?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(script.as_bytes())?;
    }

    let output = child.wait_with_output()?;
    Ok(SwiftOutput {
        success: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).trim().to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
    })
}
