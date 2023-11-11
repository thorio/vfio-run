use std::{
	collections::HashMap,
	path::{Path, PathBuf},
};

use crate::util::{ArgWriter, EnvWriter};

#[derive(Debug)]
struct UsbAddress {
	vendor_id: u16,
	product_id: u16,
}

#[derive(Debug)]
enum Graphics {
	None,
	Virtio,
}

#[derive(Debug)]
enum BiosType {
	Default,
	Ovmf(PathBuf),
}

#[derive(Debug)]
enum Disk {
	Raw(PathBuf),
	Virtio(PathBuf),
}

#[derive(Debug)]
enum Audio {
	None,
	Pipewire(PathBuf),
}

#[derive(Debug)]
enum Networking {
	Default,
}

#[derive(Debug)]
pub struct Context {
	pub env: HashMap<String, String>,
	pub pci: Vec<String>,
	pub args: Vec<String>,
	pub cpu_affinity: Option<String>,
}

#[derive(Debug)]
pub struct ContextBuilder {
	cpu: Option<String>,
	smp: Option<String>,
	ram: String,
	bios_type: BiosType,
	graphics: Graphics,
	audio: Audio,
	networking: Networking,
	disks: Vec<Disk>,
	pci: Vec<String>,
	usb: Vec<UsbAddress>,
	cpu_affinity: Option<String>,
}

impl Default for ContextBuilder {
	fn default() -> Self {
		Self {
			cpu: None,
			smp: None,
			ram: String::from("4G"),
			bios_type: BiosType::Default,
			graphics: Graphics::None,
			audio: Audio::None,
			networking: Networking::Default,
			disks: vec![],
			pci: vec![],
			usb: vec![],
			cpu_affinity: None,
		}
	}
}

impl ContextBuilder {
	pub fn with_cpu(mut self, options: impl Into<String>) -> Self {
		self.cpu = Some(options.into());
		self
	}

	pub fn with_cpu_affinity(mut self, affinity: impl Into<String>) -> Self {
		self.cpu_affinity = Some(affinity.into());
		self
	}

	pub fn with_smp(mut self, layout: impl Into<String>) -> Self {
		self.smp = Some(layout.into());
		self
	}

	pub fn with_ram(mut self, size: impl Into<String>) -> Self {
		self.ram = size.into();
		self
	}

	pub fn with_ovmf_bios(mut self, path: impl Into<PathBuf>) -> Self {
		self.bios_type = BiosType::Ovmf(path.into());
		self
	}

	pub fn with_vfio_disk(mut self, path: impl Into<PathBuf>) -> Self {
		self.disks.push(Disk::Virtio(path.into()));
		self
	}

	#[allow(unused)]
	pub fn with_raw_disk(mut self, path: impl Into<PathBuf>) -> Self {
		self.disks.push(Disk::Raw(path.into()));
		self
	}

	pub fn with_pipewire(mut self, runtime_dir: impl Into<PathBuf>) -> Self {
		self.audio = Audio::Pipewire(runtime_dir.into());
		self
	}

	#[allow(unused)]
	pub fn with_usb_device(mut self, vendor_id: u16, product_id: u16) -> Self {
		self.usb.push(UsbAddress { vendor_id, product_id });
		self
	}

	pub fn with_pci_device(mut self, address: impl Into<String>) -> Self {
		self.pci.push(address.into());
		self
	}

	pub fn with_graphics(mut self) -> Self {
		self.graphics = Graphics::Virtio;
		self
	}

	pub fn build(self) -> Context {
		let mut arg_writer = ArgWriter::default();
		let mut env_writer = EnvWriter::default();

		add_defaults(&mut arg_writer);
		add_system(&mut arg_writer, self.cpu, self.smp, self.ram);
		add_bios(&mut arg_writer, self.bios_type);
		add_graphics(&mut arg_writer, self.graphics);
		add_audio(&mut arg_writer, &mut env_writer, self.audio);
		add_pci(&mut arg_writer, &self.pci);
		add_disks(&mut arg_writer, self.disks);
		add_usb(&mut arg_writer, self.usb);

		Context {
			env: env_writer.get_envs(),
			pci: self.pci,
			args: arg_writer.get_args(),
			cpu_affinity: self.cpu_affinity,
		}
	}
}

fn add_defaults(writer: &mut ArgWriter) {
	writer.push_many(vec![
		"-enable-kvm",
		"-serial",
		"none",
		"-mon",
		"chardev=char0,mode=readline",
		"-chardev",
		"stdio,id=char0,mux=on",
	]);
}

fn add_system(writer: &mut ArgWriter, cpu: Option<String>, smp: Option<String>, ram: String) {
	if let Some(cpu) = cpu {
		writer.push("-cpu").push(cpu);
	}

	if let Some(smp) = smp {
		writer.push("-smp").push(smp);
	}

	writer.push("-m").push(ram);
}

fn add_bios(writer: &mut ArgWriter, bios: BiosType) {
	match bios {
		BiosType::Default => (),
		BiosType::Ovmf(path) => {
			let firmware_directory = path.parent().expect("bios file should be in a directory");

			writer
				.push("-L")
				.push(firmware_directory.to_string_lossy())
				.push("-bios")
				.push(path.to_string_lossy());
		}
	}
}

fn add_graphics(writer: &mut ArgWriter, graphics: Graphics) {
	_ = match graphics {
		Graphics::None => writer.push_many(vec!["-nographic", "-vga", "none"]),
		Graphics::Virtio => writer.push_many(vec!["-vga", "virtio"]),
	}
}

fn add_audio(writer: &mut ArgWriter, env: &mut EnvWriter, audio: Audio) {
	match audio {
		Audio::None => (),
		Audio::Pipewire(runtime_dir) => {
			env.add("PIPEWIRE_RUNTIME_DIR", runtime_dir.to_string_lossy())
				.add("PIPEWIRE_LATENCY", "128/48000");

			writer.push_many(vec![
				"-audiodev",
				"pipewire,id=pw",
				"-device",
				"intel-hda",
				"-device",
				"hda-output,audiodev=pw,mixer=off",
			]);
		}
	}
}

fn add_pci(writer: &mut ArgWriter, devices: &[String]) {
	for address in devices.iter() {
		writer.push("-device").push(format!("vfio-pci,host={address}"));
	}
}

fn add_disks(writer: &mut ArgWriter, disks: Vec<Disk>) {
	for disk in disks.iter() {
		_ = match disk {
			Disk::Raw(device) => writer.push("-drive").push(raw_disk(device, "media=disk")),
			Disk::Virtio(device) => writer.push("-drive").push(raw_disk(device, "if=virtio")),
		};
	}

	fn raw_disk(device: &Path, options: &str) -> String {
		let dev = device.to_string_lossy();
		format!("file={dev},format=raw,{options}")
	}
}

fn add_usb(writer: &mut ArgWriter, devices: Vec<UsbAddress>) {
	if devices.is_empty() {
		return;
	}

	writer.push("-usb");

	for address in devices.iter() {
		let fmt = format!(
			"usb-host,vendorid={:x},productid={:x}",
			address.vendor_id, address.product_id
		);
		writer.push("-device").push(fmt);
	}
}
