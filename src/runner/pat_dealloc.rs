use anyhow::Result;
use std::{ffi::OsStr, process::Command};

use super::util::run_command;

pub fn clear_pat(pci_address: impl AsRef<OsStr>) -> Result<()> {
	run_command(
		Command::new("pat-dealloc")
			.args(vec!["pci", "--load", "--address"])
			.arg(pci_address),
	)
}
