use super::util::run_command;
use anyhow::Result;
use std::process::Command;

pub fn unbind_pci(address: &str) -> Result<()> {
	run_cmd("nodedev-detach", address)
}

pub fn rebind_pci(address: &str) -> Result<()> {
	run_cmd("nodedev-reattach", address)
}

fn run_cmd(verb: &str, pci_address: &str) -> Result<()> {
	let pci_address = convert_pci_address(pci_address);

	run_command(Command::new("virsh").arg(verb).arg(pci_address))
}

pub fn convert_pci_address(address: &str) -> String {
	format!("pci_{}", address.replace(['.', ':'], "_"))
}
