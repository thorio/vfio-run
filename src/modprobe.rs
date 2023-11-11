use anyhow::{anyhow, Result};
use std::{ffi::OsStr, process::Command};

//<T: AsRef<str>>(addressses: &[T])
pub fn load<T: AsRef<OsStr>>(drivers: &[T]) -> Result<()> {
	run_cmd(None, drivers)
}

pub fn unload<T: AsRef<OsStr>>(drivers: &[T]) -> Result<()> {
	run_cmd(Some("-r"), drivers)
}

pub fn run_cmd<T: AsRef<OsStr>>(arg: Option<&str>, drivers: &[T]) -> Result<()> {
	let mut cmd = Command::new("modprobe");

	if let Some(arg) = arg {
		cmd.arg(arg);
	}

	cmd.args(drivers);
	let output = cmd.output()?;

	if output.status.success() {
		return Ok(());
	}

	let stderr = std::str::from_utf8(&output.stderr).unwrap_or("<conversion errror: stderr was not valid utf8>");
	Err(anyhow!("modprobe invocation failure: {}", stderr.trim()))
}
