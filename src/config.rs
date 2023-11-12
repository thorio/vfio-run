use crate::{cli::Configuration, context::ContextBuilder};

pub fn get_builder(window: bool, configuration: &Configuration) -> ContextBuilder {
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

	if window {
		builder = builder.with_graphics();
	}

	match configuration {
		Configuration::Foil => builder,
		Configuration::Thin => apply_thin_config(builder),
		Configuration::Fat => apply_fat_config(builder),
	}
}

// AMD iGPU
fn apply_thin_config(context: ContextBuilder) -> ContextBuilder {
	context.with_pci_device("0000:10:00.0").with_pci_device("0000:10:00.1")
}

// NVIDIA GPU
fn apply_fat_config(context: ContextBuilder) -> ContextBuilder {
	context
		.with_ram("24G")
		.with_smp("sockets=1,cores=6,threads=2")
		.with_cpu_affinity("0-5,8-13")
		.with_pci_device("0000:01:00.0")
		.with_pci_device("0000:01:00.1")
		.with_unloaded_drivers(vec!["nvidia_drm", "nvidia_uvm", "nvidia_modeset", "nvidia"])
}
