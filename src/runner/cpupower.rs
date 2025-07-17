use super::util::run_command;
use anyhow::Result;
use std::process::Command;

pub fn set_governor(governor: &str) -> Result<()> {
	run_command(Command::new("cpupower").args(vec!["frequency-set", "-g"]).arg(governor))
}
