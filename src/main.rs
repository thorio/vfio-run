use cli::Configurations;
use context::ContextBuilder;

mod cli;
mod context;
mod runner;
mod util;

fn main() {
	let cli = cli::parse();

	init_logger();

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

	let context = builder.build();
	runner::run(context);
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

fn init_logger() {
	stderrlog::new()
		.timestamp(stderrlog::Timestamp::Off)
		.init()
		.expect("logger already initialized");
}
