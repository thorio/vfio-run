use super::build;
use super::util::*;
use super::*;
use nix::unistd::{Gid, Uid};
use std::path::PathBuf;

#[derive(Debug)]
pub struct ContextBuilder {
	cpu: Option<String>,
	smp: Option<String>,
	ram: String,
	bios_type: BiosType,
	smbios: SmBiosMap,
	vga: Vga,
	window: Window,
	audio_backend: AudioBackend,
	audio_frontend: AudioFrontend,
	networking: Networking,
	looking_glass: LookingGlass,
	spice: Spice,
	disks: Vec<Disk>,
	pci: Vec<String>,
	pat_dealloc: Vec<String>,
	unload_drivers: Option<Vec<String>>,
	usb: Vec<UsbDevice>,
	cpu_affinity: Option<String>,
}

impl Default for ContextBuilder {
	fn default() -> Self {
		Self {
			cpu: None,
			smp: None,
			ram: String::from("4G"),
			bios_type: BiosType::Default,
			smbios: SmBiosMap::default(),
			vga: Vga::None,
			window: Window::None,
			audio_backend: AudioBackend::None,
			audio_frontend: AudioFrontend::None,
			networking: Networking::None,
			looking_glass: LookingGlass::No,
			spice: Spice::No,
			disks: Vec::default(),
			pci: Vec::default(),
			pat_dealloc: Vec::default(),
			usb: Vec::default(),
			unload_drivers: None,
			cpu_affinity: None,
		}
	}
}

// Not all configs use all methods
#[allow(dead_code)]
impl ContextBuilder {
	/// CPU options for QEMU
	pub fn cpu(mut self, options: impl Into<String>) -> Self {
		self.cpu = Some(options.into());
		self
	}

	/// Implements CPU pinning. See [taskset(1)](https://man7.org/linux/man-pages/man1/taskset.1.html)
	pub fn cpu_affinity(mut self, affinity: impl Into<String>) -> Self {
		self.cpu_affinity = Some(affinity.into());
		self
	}

	/// Specify the number and topology of CPU cores.
	pub fn smp(mut self, layout: impl Into<String>) -> Self {
		self.smp = Some(layout.into());
		self
	}

	/// Specifies the amount of RAM reserved for the VM.
	pub fn ram(mut self, size: impl Into<String>) -> Self {
		self.ram = size.into();
		self
	}

	/// Boot in UEFI mode. Argument is the path to OVMF.fd.  
	/// On Arch, install `edk2-ovmf`.
	pub fn ovmf_bios(mut self, path: impl Into<PathBuf>) -> Self {
		self.bios_type = BiosType::Ovmf(path.into());
		self
	}

	/// Fills in SMBIOS fields from real hardware, falling back to defaults when unavailable.  
	/// This may allow you to fool some Anticheats into thinking you're running on bare metal.
	pub fn smbios_auto(mut self) -> Self {
		smbios::populate_auto(&mut self.smbios);
		self
	}

	/// Allows you to manually fill in SMBIOS fields. Repeated keys replace previous values.
	pub fn smbios(mut self, smbios_type: SmBiosType, pairs: Vec<(impl Into<String>, impl Into<String>)>) -> Self {
		self.smbios.add_fields(smbios_type, pairs);
		self
	}

	/// Adds a physical disk. Compatible with Windows out-of-the box, but slow performance.
	pub fn raw_disk(mut self, path: impl Into<PathBuf>) -> Self {
		self.disks.push(Disk::Raw(path.into()));
		self
	}

	/// Adds a physical disk with VirtIO. Faster, but requires driver installation on guest.
	pub fn virtio_disk(mut self, path: impl Into<PathBuf>) -> Self {
		self.disks.push(Disk::Virtio(path.into()));
		self
	}

	/// Adds pipewire audio backend. The argument is the run dir, typically `/run/user/$UID`.  
	/// Incomplete on its own, requires audio frontend to be configured.
	pub fn pipewire(mut self, runtime_dir: impl Into<PathBuf>) -> Self {
		self.audio_backend = AudioBackend::Pipewire(runtime_dir.into());
		self
	}

	/// Adds spice audio backend.  
	/// Incomplete on its own, requires audio frontend to be configured.
	pub fn spice_audio(mut self) -> Self {
		self.audio_backend = AudioBackend::Spice;
		self
	}

	/// Adds and emulated intel ich6 soundcard.  
	/// Requires an audio backend to be configured.
	pub fn intel_hda(mut self, hda_type: IntelHdaType) -> Self {
		self.audio_frontend = AudioFrontend::IntelHda(hda_type);
		self
	}

	/// Adds and emulated intel ich9 soundcard.  
	/// Requires an audio backend to be configured.
	pub fn intel_hda_ich9(mut self, hda_type: IntelHdaType) -> Self {
		self.audio_frontend = AudioFrontend::IntelHdaIch9(hda_type);
		self
	}

	/// Adds and emulated usb audio device.  
	/// Requires an audio backend to be configured.
	pub fn usb_audio(mut self) -> Self {
		self.audio_frontend = AudioFrontend::UsbAudio;
		self
	}

	/// Adds basic Networking. Compatible with Windows out-of-the box, but high CPU overhead and wonky performance.
	pub fn user_networking(mut self) -> Self {
		self.networking = Networking::User;
		self
	}

	/// Adds VirtIO networking. Less overhead, more stable, but requires driver installation on guest.
	pub fn vfio_user_networking(mut self) -> Self {
		self.networking = Networking::VirtioUser;
		self
	}

	/// Fully passes a USB device through to the VM. Useful for single-GPU passthrough.
	pub fn usb_device(mut self, vendor: u16, product: u16) -> Self {
		self.usb.push(UsbDevice::HostVidPid { vendor, product });
		self
	}

	/// Unbinds and Rebinds the specified PCI devices before starting and after stopping the VM.  
	/// Then passes the devices through to the VM.
	pub fn pci_device(mut self, address: impl Into<String>) -> Self {
		self.pci.push(address.into());
		self
	}

	/// Clears the PAT entries of the specified PCI devices' memory regions
	/// after unbinding and before rebinding to work around the "Failed to mmap ... BAR" issue.
	///
	/// Requires [pat-dealloc](https://github.com/thorio/pat-dealloc) to be installed.
	pub fn pat_dealloc(mut self, address: impl Into<String>) -> Self {
		self.pat_dealloc.push(address.into());
		self
	}

	/// Unloads and Reloads the specified drivers before starting and after stopping the VM. e.g. nvidia drivers.
	pub fn unloaded_drivers<T: Into<String>>(mut self, drivers: Vec<T>) -> Self {
		let drivers = drivers.into_iter().map(|d| d.into()).collect::<Vec<_>>();
		self.unload_drivers = Some(drivers);
		self
	}

	/// Adds a virtual graphics device.
	pub fn vga(mut self, vga: Vga) -> Self {
		self.vga = vga;
		self
	}

	pub fn usb_tablet(mut self) -> Self {
		self.usb.push(UsbDevice::Device(String::from("usb-tablet")));
		self
	}

	/// Adds a GTK window for debugging purposes. You don't want to use this for long, it's not very performant.  
	/// Pointless without a virtual VGA device, see [`vga`]
	pub fn window(mut self) -> Self {
		self.window = Window::Gtk;
		self
	}

	/// Adds the [Looking Glass](https://looking-glass.io/) IVSHMEM device and sets the specified owner and group.
	pub fn looking_glass(mut self, owner: impl Into<Uid>, group: impl Into<Gid>) -> Self {
		self.looking_glass = LookingGlass::Yes(owner.into(), group.into());
		self
	}

	/// Adds Spice display, mouse and keyboard. Useful for Looking Glass as well.
	pub fn spice_kvm(mut self) -> Self {
		self.spice = Spice::Yes;
		self
	}

	pub fn build(self) -> Context {
		let mut arg_writer = ArgWriter::default();
		let mut env_writer = EnvWriter::default();
		let mut tmp_file_writer = TmpFileWriter::default();

		build::add_defaults(&mut arg_writer);
		build::add_monitor(&mut arg_writer);
		build::add_system(&mut arg_writer, self.cpu, self.smp, self.ram);
		build::add_bios(&mut arg_writer, self.bios_type);
		build::add_smbios(&mut arg_writer, self.smbios);
		build::add_vga(&mut arg_writer, self.vga);
		build::add_window(&mut arg_writer, self.window);
		build::add_audio_backend(&mut arg_writer, &mut env_writer, self.audio_backend);
		build::add_audio_frontend(&mut arg_writer, self.audio_frontend);
		build::add_networking(&mut arg_writer, self.networking);
		build::add_pci(&mut arg_writer, &self.pci);
		build::add_disks(&mut arg_writer, self.disks);
		build::add_usb(&mut arg_writer, self.usb);
		build::add_looking_glass(&mut arg_writer, &mut tmp_file_writer, self.looking_glass);
		build::add_spice(&mut arg_writer, self.spice);

		Context {
			env: env_writer.get_envs(),
			args: arg_writer.get_args(),
			pci: self.pci,
			pat_dealloc: self.pat_dealloc,
			cpu_affinity: self.cpu_affinity,
			unload_drivers: self.unload_drivers,
			tmp_files: tmp_file_writer.get_tmp_files(),
		}
	}
}
