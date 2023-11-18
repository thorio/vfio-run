use crate::cli::Profile;
use crate::context::{ContextBuilder, Vga};

pub fn get_builder(window: bool, profile: &Profile) -> ContextBuilder {
	// These options always apply
	let mut builder = ContextBuilder::default()
		.with_cpu("host,topoext,kvm=off,hv_frequencies,hv_time,hv_relaxed,hv_vapic,hv_spinlocks=0x1fff,hv_vendor_id=thisisnotavm")
		.with_ovmf_bios("/usr/share/edk2/x64/OVMF.fd")
		.with_virtio_disk("/dev/sdd")
		.with_pipewire("/run/user/1000")
		.with_vfio_user_networking()
		.with_looking_glass(1000, 1000)
		.with_spice();

	// This only applies when the --window flag is passed
	if window {
		builder = builder.with_window().with_vga(Vga::Qxl);
	}

	// These options only apply when the VM is started in the given profile
	match profile {
		Profile::Slim => builder
			.with_ram("8G")
			.with_smp("sockets=1,cores=2,threads=2")
			.with_cpu_affinity("0-1,8-9")
			.with_vga(Vga::Qxl),

		Profile::Full => builder
			.with_ram("24G")
			.with_smp("sockets=1,cores=6,threads=2")
			.with_cpu_affinity("0-5,8-13")
			.with_pci_device("0000:01:00.0")
			.with_pci_device("0000:01:00.1")
			.with_unloaded_drivers(vec!["nvidia_drm", "nvidia_uvm", "nvidia_modeset", "nvidia"]),
	}
}
