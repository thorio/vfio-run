use cli::Configurations;
use context::ContextBuilder;
use log::*;
use nix::unistd::Uid;

mod cli;
mod context;
mod runner;
mod util;
mod virsh;

fn main() {
	let cli = cli::parse();
	init_logger(cli.debug);
	debug!("{:?}", cli);

	if !Uid::effective().is_root() {
		warn!("running as non-root, here be dragons");
	}

	let mut builder = ContextBuilder::default()
		.with_cpu("host,topoext,kvm=off,hv_frequencies,hv_time,hv_relaxed,hv_vapic,hv_spinlocks=0x1fff,hv_vendor_id=thisisnotavm")
		.with_smp("sockets=1,cores=6,threads=2")
		.with_cpu_affinity("0-5,8-13")
		.with_ovmf_bios("/usr/share/edk2/x64/OVMF.fd")
		.with_vfio_disk("/dev/sdd")
		.with_pipewire("/run/user/1000");

	if cli.window {
		builder = builder.with_graphics();
	}

	let builder = match cli.configuration {
		Configurations::Foil => builder.with_ram("8G"),
		Configurations::Thin => apply_light_config(builder),
		Configurations::Fat => apply_full_config(builder),
	};

	debug!("{:?}", builder);

	let context = builder.build();
	debug!("{:?}", context);

	runner::run(context).ok();
}

fn apply_light_config(context: ContextBuilder) -> ContextBuilder {
	context
		.with_ram("8G")
		.with_pci_device("0000:10:00.0")
		.with_pci_device("0000:10:00.1")
}

fn apply_full_config(context: ContextBuilder) -> ContextBuilder {
	context
		.with_ram("24G")
		.with_pci_device("0000:01:00.0")
		.with_pci_device("0000:01:00.1")
}

fn init_logger(debug: bool) {
	stderrlog::new()
		.timestamp(stderrlog::Timestamp::Off)
		.verbosity(if debug { 3 } else { 2 })
		.init()
		.expect("logger already initialized");
}
