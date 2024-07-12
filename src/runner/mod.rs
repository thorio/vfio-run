use crate::context::{Context, TmpFile};
use anyhow::Result;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::os::fd::AsRawFd;

mod cpupower;
mod modprobe;
mod pat_dealloc;
mod qemu;
mod util;
mod virsh;

pub fn run(context: Context, skip_attach: bool) -> Result<(), ()> {
	set_governor(context.cpu_governor.as_ref())?;
	create_tmp_files(&context.tmp_files)?;

	ignore_sigint();
	detach_devices(&context)?;

	log::info!("starting qemu");

	let result = qemu::run_qemu(&context);

	if let Err(e) = result {
		log::error!("error running qemu: {}", e);
	}

	if !skip_attach {
		// errors at this stage don't really need to be handled anymore,
		// we just try to restore what we can and exit.
		reattach_devices(&context)?;
	}

	Ok(())
}

fn set_governor(governor: Option<impl AsRef<OsStr>>) -> Result<(), ()> {
	let Some(governor) = governor else {
		return Ok(());
	};

	log::info!("setting cpu frequency governor");

	if let Err(err) = cpupower::set_governor(governor) {
		log::error!("{err}");
		return Err(());
	}

	Ok(())
}

fn ignore_sigint() {
	// We ignore SIGINT, instead passing it to the wrapped QEMU process
	// and then cleaning up after it exits
	if let Err(err) = ctrlc::set_handler(|| ()) {
		log::warn!("error setting SIGINT handler: {err}");
	}
}

fn create_tmp_files(files: &[TmpFile]) -> Result<(), ()> {
	for file in files {
		if let Err(err) = create_tmp_file(file) {
			log::error!("error creating file {file:?} {err}");
			return Err(());
		}
	}

	Ok(())
}

fn create_tmp_file(tmp_file: &TmpFile) -> Result<()> {
	fs::remove_file(&tmp_file.path).ok();
	let file = File::create(&tmp_file.path)?;
	nix::unistd::fchown(file.as_raw_fd(), Some(tmp_file.uid), Some(tmp_file.gid))?;
	nix::sys::stat::fchmod(file.as_raw_fd(), tmp_file.mode)?;

	Ok(())
}

pub fn reattach_devices(context: &Context) -> Result<(), ()> {
	pat_dealloc(&context.pat_dealloc);
	rebind_pci(&context.pci)?;
	reload_drivers(context.unload_drivers.as_ref())
}

pub fn detach_devices(context: &Context) -> Result<(), ()> {
	if unload_drivers(context.unload_drivers.as_ref()).is_err() {
		log::info!("attempting to reload drivers");
		reload_drivers(context.unload_drivers.as_ref()).ok();
		return Err(());
	}

	if let Err(unbound) = unbind_pci(&context.pci) {
		if !unbound.is_empty() {
			log::info!("attempting to rebind pci devices");
			rebind_pci(&unbound).ok();
			reload_drivers(context.unload_drivers.as_ref()).ok();
		}

		return Err(());
	}

	pat_dealloc(&context.pat_dealloc);

	Ok(())
}

pub fn pat_dealloc(addresses: &[String]) {
	if addresses.is_empty() {
		return;
	}

	log::info!("clearing PAT entries");

	for address in addresses {
		if let Err(e) = pat_dealloc::clear_pat(address) {
			log::error!("erroring clearing PAT for {address}: {e}");
		}
	}
}

fn unload_drivers(drivers: Option<&Vec<String>>) -> Result<(), ()> {
	if let Some(drivers) = drivers {
		log::info!("unloading drivers");
		log::debug!("unloading {drivers:?}");
		if let Err(msg) = modprobe::unload(drivers) {
			log::error!("unloading {}", msg);
			return Err(());
		}
	}

	Ok(())
}

fn reload_drivers(drivers: Option<&Vec<String>>) -> Result<(), ()> {
	if let Some(drivers) = drivers {
		log::info!("loading drivers");
		log::debug!("loading {drivers:?}");
		if let Err(msg) = modprobe::load(drivers) {
			log::error!("loading {}", msg);
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

	log::info!("unbinding pci devices");

	for addr in addressses.iter().map(AsRef::as_ref) {
		log::debug!("unbinding {addr}");
		let result = virsh::unbind_pci(addr);

		if let Err(e) = result {
			log::error!("pci unbind {}", e);
			return Err(unbound);
		}

		unbound.push(addr);
	}

	Ok(())
}

fn rebind_pci<T: AsRef<str>>(addressses: &[T]) -> Result<(), ()> {
	if addressses.is_empty() {
		return Ok(());
	}

	log::info!("rebinding pci devices");

	let mut had_error = false;

	for addr in addressses.iter().map(AsRef::as_ref) {
		log::debug!("rebinding {addr}");
		let result = virsh::rebind_pci(addr);

		// do not cancel rebind over one error, attempt rebinding the rest as well!
		if let Err(e) = result {
			log::error!("pci rebind {}", e);
			had_error = true;
		}
	}

	if had_error {
		Err(())
	} else {
		Ok(())
	}
}
