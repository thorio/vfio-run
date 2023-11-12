use crate::context::Context;
use crate::{modprobe, virsh};
use anyhow::Result;
use log::*;
use std::process::Command;

const QEMU_CMD: &str = "qemu-system-x86_64";

pub fn run(context: Context, skip_attach: bool) -> Result<(), ()> {
	ignore_sigint();

	detach_devices(&context)?;

	info!("starting qemu");

	let result = get_command(&context)
		.args(context.args)
		.envs(context.env)
		.spawn()
		.and_then(|mut handle| handle.wait());

	if let Err(e) = result {
		error!("error running qemu: {}", e);
	}

	if !skip_attach {
		// errors at this stage don't really need to be handled anymore,
		// we just try to restore what we can and exit.
		rebind_pci(&context.pci).ok();
		reload_drivers(context.unload_drivers.as_ref()).ok();
	}

	Ok(())
}

fn ignore_sigint() {
	// We ignore SIGINT, instead passing it to the wrapped QEMU process
	// and then cleaning up after it exits
	if let Err(err) = ctrlc::set_handler(|| ()) {
		warn!("error setting SIGINT handler: {err}")
	}
}

fn detach_devices(context: &Context) -> Result<(), ()> {
	if unload_drivers(context.unload_drivers.as_ref()).is_err() {
		info!("attempting to reload drivers");
		reload_drivers(context.unload_drivers.as_ref()).ok();
		return Err(());
	}

	if let Err(unbound) = unbind_pci(&context.pci) {
		if !unbound.is_empty() {
			info!("attempting to rebind pci devices");
			rebind_pci(&unbound).ok();
			reload_drivers(context.unload_drivers.as_ref()).ok();
		}

		return Err(());
	}

	Ok(())
}

fn unload_drivers(drivers: Option<&Vec<String>>) -> Result<(), ()> {
	if let Some(drivers) = drivers {
		if let Err(msg) = modprobe::unload(drivers) {
			error!("unloading {}", msg);
			return Err(());
		}
	}

	Ok(())
}

fn reload_drivers(drivers: Option<&Vec<String>>) -> Result<(), ()> {
	if let Some(drivers) = drivers {
		if let Err(msg) = modprobe::load(drivers) {
			error!("loading {}", msg);
			return Err(());
		}
	}

	Ok(())
}

fn unbind_pci<T: AsRef<str>>(addressses: &[T]) -> Result<(), Vec<&'_ str>> {
	let mut unbound = vec![];

	if addressses.is_empty() {
		return Ok(());
	}

	info!("unbinding pci devices");

	for addr in addressses.iter().map(|a| a.as_ref()) {
		debug!("unbinding {addr}");
		let result = virsh::unbind_pci(addr);

		if result.is_err() {
			error!("pci unbind {}", result.unwrap_err());
			return Err(unbound);
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
		debug!("rebinding {addr}");
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
