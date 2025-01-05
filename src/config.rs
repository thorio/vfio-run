use crate::cli::Profile;
use crate::context::{ContextBuilder, IntelHdaType, Vga};

const PCI_GPU: &str = "0000:01:00.0";
const PCI_GPU_AUDIO: &str = "0000:01:00.1";
const PCI_USB: &str = "0000:05:00.0";

// Look at the readme for setup instructions. The builder functions also have doc comments.

pub fn get_builder(window: bool, profile: &Profile) -> ContextBuilder {
	// These options always apply
	let mut builder = ContextBuilder::default()
		.cpu("host,topoext,kvm=off,hv_frequencies,hv_time,hv_relaxed,hv_vapic,hv_spinlocks=0x1fff,hv_vendor_id=thisisnotavm")
		.cpu_governor("performance")
		.ovmf_bios("/usr/share/edk2/x64/OVMF.4m.fd")
		.smbios_auto()
		.pipewire("/run/user/1000")
		.intel_hda(IntelHdaType::Output)
		.vfio_user_networking()
		.looking_glass(1000, 1000)
		.spice_kvm()
		.spice_agent();

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
			.virtio_disk("/dev/sdd")
			.pci_device(PCI_USB)
			.vga(Vga::Qxl),

		Profile::Full => builder
			.ram("24G")
			.smp("sockets=1,cores=6,threads=2")
			.cpu_affinity("0-5,8-13")
			.virtio_disk("/dev/sdd")
			.pci_device(PCI_GPU)
			.pci_device(PCI_GPU_AUDIO)
			.pci_device(PCI_USB)
			.pat_dealloc(PCI_GPU)
			.unloaded_drivers(vec!["nvidia_drm", "nvidia_uvm", "nvidia_modeset", "nvidia"]),

		Profile::Work => builder
			.ram("20G")
			.smp("sockets=1,cores=6,threads=2")
			.cpu_affinity("0-5,8-13")
			.virtio_disk("/dev/sdc")
			.virtio_disk("/dev/sdb")
			.intel_hda(IntelHdaType::Duplex)
			.pci_device(PCI_GPU)
			.pci_device(PCI_GPU_AUDIO)
			.pat_dealloc(PCI_GPU)
			.unloaded_drivers(vec!["nvidia_drm", "nvidia_uvm", "nvidia_modeset", "nvidia"]),
	}
}
