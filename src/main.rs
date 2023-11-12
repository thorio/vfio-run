use cli::{Args, Configurations};
use context::ContextBuilder;
use log::*;
use nix::unistd::Uid;

mod cli;
mod context;
mod modprobe;
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

	let builder = get_builder(&cli);
	debug!("{:?}", builder);

	let context = builder.build();
	debug!("{:?}", context);

	if runner::run(context, cli.skip_attach).is_ok() {
		info!("exit successful")
	}
}

fn get_builder(cli: &Args) -> ContextBuilder {
	let mut builder = ContextBuilder::default()
		.with_cpu("host,topoext,kvm=off,hv_frequencies,hv_time,hv_relaxed,hv_vapic,hv_spinlocks=0x1fff,hv_vendor_id=thisisnotavm")
		.with_smp("sockets=1,cores=4,threads=2")
		.with_ram("8G")
		.with_ovmf_bios("/usr/share/edk2/x64/OVMF.fd")
		.with_vfio_disk("/dev/sdd")
		.with_pipewire("/run/user/1000")
		.with_vfio_user_networking()
		.with_looking_glass(1000, 1000)
		.with_spice();

	if cli.window {
		builder = builder.with_graphics();
	}

	match cli.configuration {
		Configurations::Foil => builder,
		Configurations::Thin => apply_light_config(builder),
		Configurations::Fat => apply_full_config(builder),
	}
}

// AMD iGPU
fn apply_light_config(context: ContextBuilder) -> ContextBuilder {
	context.with_pci_device("0000:10:00.0").with_pci_device("0000:10:00.1")
}

// NVIDIA GPU
fn apply_full_config(context: ContextBuilder) -> ContextBuilder {
	context
		.with_ram("24G")
		.with_smp("sockets=1,cores=6,threads=2")
		.with_cpu_affinity("0-5,8-13")
		.with_pci_device("0000:01:00.0")
		.with_pci_device("0000:01:00.1")
		.with_unloaded_drivers(vec!["nvidia_drm", "nvidia_uvm", "nvidia_modeset", "nvidia"])
}

fn init_logger(debug: bool) {
	stderrlog::new()
		.timestamp(stderrlog::Timestamp::Off)
		.verbosity(if debug { 3 } else { 2 })
		.init()
		.expect("logger already initialized");
}
