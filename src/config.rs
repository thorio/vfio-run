use crate::cli::*;
use crate::context::*;

const PCI_GPU: &str = "0000:01:00.0";
const PCI_GPU_AUDIO: &str = "0000:01:00.1";
const PCI_USB: &str = "0000:05:00.0";

pub fn configure(config: &mut ContextBuilder, cli: &Options) {
	config
		.cpu("host,topoext,kvm=off,hv_frequencies,hv_time,hv_relaxed,hv_vapic,hv_spinlocks=0x1fff,hv_vendor_id=thisisnotavm")
		.cpu_governor("performance")
		.ovmf_bios("/usr/share/edk2/x64/OVMF.4m.fd")
		.smbios_auto()
		.spice_audio()
		.vfio_user_networking()
		.looking_glass(1000, 1000)
		.spice_kvm()
		.spice_agent();

	if cli.window {
		config.window().usb_tablet();
	}

	match cli.cpu {
		Cpu::Full => config
			.ram("20G")
			.smp("sockets=1,cores=6,threads=2")
			.cpu_affinity("0-5,8-13"),
		Cpu::Slim => config
			.ram("8G")
			.smp("sockets=1,cores=2,threads=2")
			.cpu_affinity("0-1,8-9"),
	};

	match cli.graphics {
		Graphics::Virtual => config.vga(Vga::Qxl),
		Graphics::Passthrough => config
			.pci_device(PCI_GPU)
			.pci_device(PCI_GPU_AUDIO)
			.unloaded_drivers(vec!["nvidia_drm", "nvidia_uvm", "nvidia_modeset", "nvidia"]),
	};

	match cli.profile {
		Profile::Game => config
			.virtio_disk("/dev/disk/by-id/wwn-0x5002538d411f8d4e")
			.intel_hda(IntelHdaType::Output)
			.pci_device(PCI_USB),
		Profile::Work => config
			.virtio_disk("/dev/disk/by-id/wwn-0x5002538e4114386e")
			.intel_hda(IntelHdaType::Duplex),
	};
}
