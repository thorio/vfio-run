use super::util::{ArgWriter, EnvWriter, TmpFileWriter};
use super::{smbios::SmBiosMapExt, *};
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
	spice_agent: SpiceAgent,
	disks: Vec<Disk>,
	pci: Vec<String>,
	pat_dealloc: Vec<String>,
	unload_drivers: Option<Vec<String>>,
	usb: Vec<UsbDevice>,
	cpu_affinity: Option<String>,
	cpu_governor: Option<String>,
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
			spice_agent: SpiceAgent::No,
			disks: Vec::default(),
			pci: Vec::default(),
			pat_dealloc: Vec::default(),
			unload_drivers: None,
			usb: Vec::default(),
			cpu_affinity: None,
			cpu_governor: None,
		}
	}
}

// Not all configs use all methods
#[allow(dead_code)]
impl ContextBuilder {
	/// CPU options for QEMU. See `qemu-system-x86_64 -cpu help`.
	pub fn cpu(&mut self, options: impl Into<String>) -> &mut Self {
		self.cpu = Some(options.into());
		self
	}

	/// Implements CPU pinning. See `--cpu-list` of [taskset(1)](https://man7.org/linux/man-pages/man1/taskset.1.html).
	pub fn cpu_affinity(&mut self, affinity: impl Into<String>) -> &mut Self {
		self.cpu_affinity = Some(affinity.into());
		self
	}

	/// Sets the CPU frequency governor. See [cpupower-frequency-set(1)](https://linux.die.net/man/1/cpupower-frequency-set).  
	/// The governor is NOT reset on exit.
	pub fn cpu_governor(&mut self, governor: impl Into<String>) -> &mut Self {
		self.cpu_governor = Some(governor.into());
		self
	}

	/// Specify the number and topology of CPU cores. See `qemu-system-x86_64 -smp help`.
	pub fn smp(&mut self, layout: impl Into<String>) -> &mut Self {
		self.smp = Some(layout.into());
		self
	}

	/// Specifies the amount of RAM reserved for the VM (e.g. 4G, 256M).  
	/// Ballooning does not work with PCI device passthrough and is therefore not supported.
	pub fn ram(&mut self, size: impl Into<String>) -> &mut Self {
		self.ram = size.into();
		self
	}

	/// Boot in UEFI mode. `path` is the location of `OVMF.fd`.
	pub fn ovmf_bios(&mut self, path: impl Into<PathBuf>) -> &mut Self {
		self.bios_type = BiosType::Ovmf(path.into());
		self
	}

	/// Fills in SMBIOS fields from the host, falling back to defaults when unavailable.  
	/// This *may* fool some Anticheat's VM detection.
	pub fn smbios_auto(&mut self) -> &mut Self {
		smbios::populate_auto(&mut self.smbios);
		self
	}

	/// Sets the specified SMBIOS fields. Use after [`ContextBuilder::smbios_auto`] to overwrite select fields.
	pub fn smbios(
		&mut self,
		smbios_type: SmBiosType,
		pairs: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
	) -> &mut Self {
		self.smbios.add_fields(smbios_type, pairs);
		self
	}

	/// Adds a physical disk. Compatible with Windows out-of-the box, but has slow performance.
	pub fn raw_disk(&mut self, path: impl Into<PathBuf>) -> &mut Self {
		self.disks.push(Disk::Raw(path.into()));
		self
	}

	/// Adds a physical disk with VirtIO. Faster, but requires driver installation on guest.
	pub fn virtio_disk(&mut self, path: impl Into<PathBuf>) -> &mut Self {
		self.disks.push(Disk::Virtio(path.into()));
		self
	}

	/// Adds a pipewire audio backend. `runtime_dir` is typically `/run/user/$UID`.  
	/// Incomplete on its own, requires audio frontend to be configured.
	pub fn pipewire(&mut self, runtime_dir: impl Into<PathBuf>) -> &mut Self {
		self.audio_backend = AudioBackend::Pipewire(runtime_dir.into());
		self
	}

	/// Adds a spice audio backend.  
	/// Incomplete on its own, requires audio frontend to be configured.
	pub fn spice_audio(&mut self) -> &mut Self {
		self.audio_backend = AudioBackend::Spice;
		self
	}

	/// Adds an emulated intel ich6 soundcard.  
	/// Requires an audio backend to be configured.
	pub fn intel_hda(&mut self, hda_type: IntelHdaType) -> &mut Self {
		self.audio_frontend = AudioFrontend::IntelHda(hda_type);
		self
	}

	/// Adds an emulated intel ich9 soundcard.  
	/// Requires an audio backend to be configured.
	pub fn intel_hda_ich9(&mut self, hda_type: IntelHdaType) -> &mut Self {
		self.audio_frontend = AudioFrontend::IntelHdaIch9(hda_type);
		self
	}

	/// Adds an emulated usb audio device.  
	/// Requires an audio backend to be configured.
	pub fn usb_audio(&mut self) -> &mut Self {
		self.audio_frontend = AudioFrontend::UsbAudio;
		self
	}

	/// Adds basic networking. Compatible with Windows out-of-the box, but incurs high CPU overhead and wonky performance.
	pub fn user_networking(&mut self) -> &mut Self {
		self.networking = Networking::User;
		self
	}

	/// Adds VirtIO networking. Less overhead, more stable, but requires driver installation on guest.
	pub fn vfio_user_networking(&mut self) -> &mut Self {
		self.networking = Networking::VirtioUser;
		self
	}

	/// Fully passes a USB device through to the VM. Useful for single-GPU passthrough.
	pub fn usb_device(&mut self, vendor: u16, product: u16) -> &mut Self {
		self.usb.push(UsbDevice::HostVidPid { vendor, product });
		self
	}

	/// Passes the specified PCI devices through to the VM; automatically un- and rebinds devices.
	pub fn pci_device(&mut self, address: impl Into<String>) -> &mut Self {
		self.pci.push(address.into());
		self
	}

	/// Clears the PAT entries of the specified PCI devices' memory regions
	/// after unbinding and before rebinding to work around the "Failed to mmap ... BAR" issue.
	///
	/// Requires [pat-dealloc](https://github.com/thorio/pat-dealloc) to be installed.
	pub fn pat_dealloc(&mut self, address: impl Into<String>) -> &mut Self {
		self.pat_dealloc.push(address.into());
		self
	}

	/// Unloads and Reloads the specified drivers before starting and after stopping the VM. e.g. nvidia drivers.
	pub fn unloaded_drivers(&mut self, drivers: impl IntoIterator<Item = impl AsRef<str>>) -> &mut Self {
		let drivers = drivers.into_iter().map(|a| a.as_ref().to_owned()).collect::<Vec<_>>();
		self.unload_drivers = Some(drivers);
		self
	}

	/// Adds a virtual graphics device.
	pub fn vga(&mut self, vga: Vga) -> &mut Self {
		self.vga = vga;
		self
	}

	pub fn usb_tablet(&mut self) -> &mut Self {
		self.usb.push(UsbDevice::Device(String::from("usb-tablet")));
		self
	}

	/// Adds a GTK window for debugging purposes. You don't want to use this for long, it's not very performant.  
	/// Pointless without a virtual VGA device, see [`vga`].
	pub fn window(&mut self) -> &mut Self {
		self.window = Window::Gtk;
		self
	}

	/// Adds the [Looking Glass](https://looking-glass.io/) IVSHMEM device
	/// and creates the file with the specified owner and group.
	pub fn looking_glass(&mut self, owner: impl Into<Uid>, group: impl Into<Gid>) -> &mut Self {
		self.looking_glass = LookingGlass::Yes(owner.into(), group.into());
		self
	}

	/// Adds Spice display, mouse and keyboard. Useful for Looking Glass as well.
	pub fn spice_kvm(&mut self) -> &mut Self {
		self.spice = Spice::Yes;
		self
	}

	/// Adds Spice vdagent for clipboard synchronisation. Useful for Looking Glass as well.
	pub fn spice_agent(&mut self) -> &mut Self {
		self.spice_agent = SpiceAgent::Yes;
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
		build::add_spice_agent(&mut arg_writer, self.spice_agent);

		Context {
			env: env_writer.get_envs(),
			args: arg_writer.get_args(),
			pci: self.pci,
			pat_dealloc: self.pat_dealloc,
			unload_drivers: self.unload_drivers,
			tmp_files: tmp_file_writer.get_tmp_files(),
			cpu_affinity: self.cpu_affinity,
			cpu_governor: self.cpu_governor,
		}
	}
}
