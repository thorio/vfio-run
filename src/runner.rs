use crate::{context::Context, virsh};
use anyhow::Result;
use log::{error, info};
use std::process::Command;

const QEMU_CMD: &str = "qemu-system-x86_64";

pub fn run(context: Context) -> Result<(), ()> {
	unbind_pci(&context.pci)?;

	info!("starting qemu");

	let result = get_command(&context)
		.args(context.args)
		.envs(context.env)
		.spawn()
		.map(|mut handle| handle.wait());

	if let Err(e) = result {
		error!("error running qemu: {}", e);
		return Err(());
	}

	rebind_pci(&context.pci)
}

fn unbind_pci<T: AsRef<str>>(addressses: &[T]) -> Result<(), ()> {
	let mut unbound = vec![];

	if addressses.is_empty() {
		return Ok(());
	}

	info!("unbinding pci devices");

	for addr in addressses.iter().map(|a| a.as_ref()) {
		let result = virsh::unbind_pci(addr);

		if result.is_err() {
			error!("pci unbind {}", result.unwrap_err());

			if unbound.is_empty() {
				return Err(());
			}

			info!("attempting to rebind pci devices");

			rebind_pci(&unbound)?;
			return Err(());
		}

		unbound.push(addr)
	}

	Ok(())
}

fn rebind_pci<T: AsRef<str>>(addressses: &[T]) -> Result<(), ()> {
	if addressses.is_empty() {
		return Ok(());
	}

	info!("rebinding pci devices");

	let mut had_error = false;

	for addr in addressses.iter().map(|a| a.as_ref()) {
		let result = virsh::rebind_pci(addr);

		// do not cancel rebind over one error, attempt rebinding the rest as well!
		if result.is_err() {
			error!("pci rebind {}", result.unwrap_err());
			had_error = true;
		}
	}

	if had_error {
		Err(())
	} else {
		Ok(())
	}
}

fn get_command(context: &Context) -> Command {
	match &context.cpu_affinity {
		None => Command::new(QEMU_CMD),
		Some(affinity) => {
			let mut cmd = Command::new("taskset");
			cmd.arg("--cpu-list").arg(affinity);
			cmd.arg(QEMU_CMD);

			cmd
		}
	}
}
