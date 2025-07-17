use super::util::run_command;
use anyhow::Result;
use std::{ffi::OsStr, process::Command};

pub fn load(drivers: &[impl AsRef<OsStr>]) -> Result<()> {
	run_cmd(None, drivers)
}

pub fn unload(drivers: &[impl AsRef<OsStr>]) -> Result<()> {
	run_cmd(Some("-r"), drivers)
}

fn run_cmd(arg: Option<&str>, drivers: &[impl AsRef<OsStr>]) -> Result<()> {
	let mut cmd = Command::new("modprobe");

	if let Some(arg) = arg {
		cmd.arg(arg);
	}

	cmd.args(drivers);
	run_command(&mut cmd)
}
