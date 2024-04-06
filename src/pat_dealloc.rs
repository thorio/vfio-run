use anyhow::{bail, Result};
use std::{ffi::OsStr, process::Command};

pub fn clear_pat(pci_address: impl AsRef<OsStr>) -> Result<()> {
	let output = Command::new("pat-dealloc")
		.args(vec!["pci", "--load", "--address"])
		.arg(pci_address)
		.output()?;

	if output.status.success() {
		return Ok(());
	}

	let stderr = std::str::from_utf8(&output.stderr).unwrap_or("<conversion errror: stderr was not valid utf8>");
	bail!("pat-dealloc invocation failure: {}", stderr.trim());
}
