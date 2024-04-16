use anyhow::Result;
use std::{ffi::OsStr, process::Command};

use super::util::run_command;

pub fn set_governor(governor: impl AsRef<OsStr>) -> Result<()> {
	run_command(Command::new("cpupower").args(vec!["frequency-set", "-g"]).arg(governor))
}
