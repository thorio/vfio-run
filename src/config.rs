use crate::cli::Profile;
use crate::context::{AudioDirection, ContextBuilder, Vga};

pub fn get_builder(window: bool, profile: &Profile) -> ContextBuilder {
	// These options always apply
	let mut builder = ContextBuilder::default()
		.cpu("host,topoext,kvm=off,hv_frequencies,hv_time,hv_relaxed,hv_vapic,hv_spinlocks=0x1fff,hv_vendor_id=thisisnotavm")
		.ovmf_bios("/usr/share/edk2/x64/OVMF.fd")
		.virtio_disk("/dev/sdd")
		.pipewire("/run/user/1000", AudioDirection::Output)
		.vfio_user_networking()
		.looking_glass(1000, 1000)
		.spice();

	// This only applies when the --window flag is passed
	if window {
		builder = builder.window().vga(Vga::Qxl).usb_tablet();
	}

	// These options only apply when the VM is started in the given profile
	match profile {
		Profile::Slim => builder
			.ram("8G")
			.smp("sockets=1,cores=2,threads=2")
			.cpu_affinity("0-1,8-9")
			.vga(Vga::Qxl),

		Profile::Full => builder
			.ram("24G")
			.smp("sockets=1,cores=6,threads=2")
			.cpu_affinity("0-5,8-13")
			.pci_device("0000:01:00.0")
			.pci_device("0000:01:00.1")
			.unloaded_drivers(vec!["nvidia_drm", "nvidia_uvm", "nvidia_modeset", "nvidia"]),
	}
}
