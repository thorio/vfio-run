use anyhow::{anyhow, Result};
use std::process::Command;

pub fn unbind_pci(address: &str) -> Result<()> {
	run_cmd("nodedev-detach", address)
}

pub fn rebind_pci(address: &str) -> Result<()> {
	run_cmd("nodedev-reattach", address)
}

pub fn run_cmd(verb: &str, pci_address: &str) -> Result<()> {
	let pci_address = convert_pci_address(pci_address);

	let output = Command::new("virsh").arg(verb).arg(pci_address).output()?;

	if output.status.success() {
		return Ok(());
	}

	let stderr = std::str::from_utf8(&output.stderr).unwrap_or("<conversion errror: stderr was not valid utf8>");
	Err(anyhow!("virsh invocation failure: {}", stderr.trim()))
}

pub fn convert_pci_address(address: &str) -> String {
	format!("pci_{}", address.replace(['.', ':'], "_"))
}
