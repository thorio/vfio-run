use super::util::run_command;
use anyhow::Result;
use std::{ffi::OsStr, process::Command};

pub fn load<T: AsRef<OsStr>>(drivers: &[T]) -> Result<()> {
	run_cmd(None, drivers)
}

pub fn unload<T: AsRef<OsStr>>(drivers: &[T]) -> Result<()> {
	run_cmd(Some("-r"), drivers)
}

fn run_cmd<T: AsRef<OsStr>>(arg: Option<&str>, drivers: &[T]) -> Result<()> {
	let mut cmd = Command::new("modprobe");

	if let Some(arg) = arg {
		cmd.arg(arg);
	}

	cmd.args(drivers);
	run_command(&mut cmd)
}
