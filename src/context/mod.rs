use nix::sys::stat::Mode;
use nix::unistd::{Gid, Uid};
use std::{collections::HashMap, path::PathBuf};

mod build;
mod builder;
mod smbios;
mod util;

pub use builder::ContextBuilder;

#[derive(Debug)]
pub enum UsbDevice {
	HostVidPid { vendor: u16, product: u16 },
	Device(String),
}

#[derive(Debug)]
pub struct TmpFile {
	pub path: PathBuf,
	pub uid: Uid,
	pub gid: Gid,
	pub mode: Mode,
}

#[derive(Debug)]
#[allow(unused)]
pub enum Vga {
	None,
	/// Standard QEMU VGA device. Compatible out of the box.
	Standard,
	/// VirtIO VGA device. I don't think there's a windows driver for this.
	Virtio,
	/// QXL VGA device. Compatible with Windows, may need drivers.
	Qxl,
}

#[derive(Debug)]
pub enum Window {
	None,
	Gtk,
}

#[derive(Debug)]
pub enum BiosType {
	Default,
	Ovmf(PathBuf),
}

#[derive(Debug)]
pub enum Disk {
	Raw(PathBuf),
	Virtio(PathBuf),
}

#[derive(Debug)]
pub enum AudioBackend {
	None,
	Pipewire(PathBuf),
	Spice,
}

#[derive(Debug)]
pub enum AudioFrontend {
	None,
	IntelHda(IntelHdaType),
	IntelHdaIch9(IntelHdaType),
	UsbAudio,
}

#[derive(Debug)]
#[allow(unused)]
pub enum IntelHdaType {
	Output,
	Duplex,
	Micro,
}

impl IntelHdaType {
	pub fn device_name(&self) -> &'static str {
		match self {
			IntelHdaType::Output => "hda-output",
			IntelHdaType::Duplex => "hda-duplex",
			IntelHdaType::Micro => "hda-micro",
		}
	}
}

#[derive(Debug)]
pub enum Networking {
	None,
	User,
	VirtioUser,
}

#[derive(Debug)]
pub enum LookingGlass {
	No,
	Yes(Uid, Gid),
}

#[derive(PartialEq, Debug)]
pub enum Spice {
	No,
	Yes,
}

#[derive(PartialEq, Debug)]
pub enum SpiceAgent {
	No,
	Yes,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SmBiosType {
	BiosInformation = 0,
	SystemInformation = 1,
	BaseboardInformation = 2,
	EnclosureInformation = 3,
	ProcessorInformation = 4,
	OemStrings = 11,
	MemoryDevice = 17,
}

pub type SmBiosMap = HashMap<SmBiosType, HashMap<String, String>>;

pub trait SmBiosMapExt {
	fn add_field(&mut self, smbios_type: SmBiosType, key: impl Into<String>, value: impl Into<String>) {
		self.add_fields(smbios_type, vec![(key, value)])
	}

	fn add_fields(&mut self, smbios_type: SmBiosType, pairs: Vec<(impl Into<String>, impl Into<String>)>);
}

impl SmBiosMapExt for SmBiosMap {
	fn add_fields(&mut self, smbios_type: SmBiosType, pairs: Vec<(impl Into<String>, impl Into<String>)>) {
		let values = self.entry(smbios_type).or_default();

		for (key, value) in pairs {
			values.insert(key.into(), value.into());
		}
	}
}

#[derive(Debug)]
pub struct Context {
	pub env: HashMap<String, String>,
	pub args: Vec<String>,
	pub pci: Vec<String>,
	pub pat_dealloc: Vec<String>,
	pub tmp_files: Vec<TmpFile>,
	pub cpu_affinity: Option<String>,
	pub unload_drivers: Option<Vec<String>>,
}
