use nix::sys::stat::Mode;
use nix::unistd::{Gid, Uid};
use std::{collections::HashMap, path::PathBuf};

mod build;
mod builder;
mod smbios;
mod util;

pub use builder::ContextBuilder;

#[derive(Clone, Debug)]
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

#[derive(Clone, Copy, Debug)]
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

#[derive(Clone, Copy, Debug)]
pub enum Window {
	None,
	Gtk,
}

#[derive(Clone, Debug)]
pub enum BiosType {
	Default,
	Ovmf(PathBuf),
}

#[derive(Clone, Debug)]
pub enum Disk {
	Raw(PathBuf),
	Virtio(PathBuf),
}

#[derive(Clone, Debug)]
pub enum AudioBackend {
	None,
	Pipewire(PathBuf),
	Spice,
}

#[derive(Clone, Debug)]
pub enum AudioFrontend {
	None,
	IntelHda(IntelHdaType),
	IntelHdaIch9(IntelHdaType),
	UsbAudio,
}

#[derive(Clone, Debug)]
#[allow(unused)]
pub enum IntelHdaType {
	/// HDA Audio Codec, output-only (line-out)
	Output,
	/// HDA Audio Codec, duplex (line-out, line-in)
	Duplex,
	/// HDA Audio Codec, duplex (speaker, microphone)
	Micro,
}

#[derive(Clone, Copy, Debug)]
pub enum Networking {
	None,
	User,
	VirtioUser,
}

#[derive(Clone, Copy, Debug)]
pub enum LookingGlass {
	No,
	Yes(Uid, Gid),
}

#[derive(Clone, Copy, Debug)]
pub enum Spice {
	No,
	Yes,
}

#[derive(Clone, Copy, Debug)]
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

#[derive(Debug)]
pub struct Context {
	pub env: HashMap<String, String>,
	pub args: Vec<String>,
	pub pci: Vec<String>,
	pub pat_dealloc: Vec<String>,
	pub unload_drivers: Option<Vec<String>>,
	pub tmp_files: Vec<TmpFile>,
	pub cpu_affinity: Option<String>,
	pub cpu_governor: Option<String>,
}
