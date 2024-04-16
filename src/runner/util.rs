use anyhow::{bail, Result};
use std::process::Command;

pub fn run_command(cmd: &mut Command) -> Result<()> {
	let output = cmd.output()?;

	if output.status.success() {
		return Ok(());
	}

	let stderr = std::str::from_utf8(&output.stderr)
		.unwrap_or("<conversion errror: stderr was not valid utf8>")
		.trim();

	let cmd_name = cmd.get_program().to_string_lossy();
	bail!("{cmd_name} invocation failure: {stderr}");
}
