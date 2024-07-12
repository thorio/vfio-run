use super::util::{ArgWriter, EnvWriter, TmpFileWriter};
use super::*;
use nix::sys::stat::Mode;
use std::fmt::Write;
use std::path::Path;

pub fn add_defaults(args: &mut ArgWriter) {
	args.add_many(vec!["-nodefaults", "-enable-kvm"]);
}

pub fn add_monitor(args: &mut ArgWriter) {
	args.add_many(vec![
		"-mon",
		"chardev=char0,mode=readline",
		"-chardev",
		"stdio,id=char0,mux=on",
	]);
}

pub fn add_system(args: &mut ArgWriter, cpu: Option<String>, smp: Option<String>, ram: String) {
	if let Some(cpu) = cpu {
		args.add("-cpu").add(cpu);
	}

	if let Some(smp) = smp {
		args.add("-smp").add(smp);
	}

	args.add("-m").add(ram);
}

pub fn add_bios(args: &mut ArgWriter, bios: BiosType) {
	match bios {
		BiosType::Default => (),
		BiosType::Ovmf(path) => {
			let firmware_directory = path.parent().expect("bios file should be in a directory");

			args.add("-L")
				.add(firmware_directory.to_string_lossy())
				.add("-bios")
				.add(path.to_string_lossy());
		}
	}
}

pub fn add_smbios(args: &mut ArgWriter, smbios: SmBiosMap) {
	for (smbios_type, fields) in smbios {
		let mut buffer = format!("type={}", smbios_type as isize);

		for (key, value) in fields {
			write!(&mut buffer, ",{}={}", key, value.replace(',', ",,")).unwrap();
		}

		args.add("-smbios").add(buffer);
	}
}

pub fn add_vga(args: &mut ArgWriter, vga: Vga) {
	match vga {
		Vga::None => args.add_many(vec!["-vga", "none"]),
		Vga::Standard => args.add_many(vec!["-vga", "std"]),
		Vga::Virtio => args.add_many(vec!["-vga", "virtio"]),
		Vga::Qxl => args.add_many(vec!["-vga", "qxl"]),
	};
}

pub fn add_window(args: &mut ArgWriter, window: Window) {
	match window {
		Window::None => args.add_many(vec!["-display", "none"]),
		Window::Gtk => args.add_many(vec!["-display", "gtk"]),
	};
}

pub fn add_audio_backend(args: &mut ArgWriter, env: &mut EnvWriter, audio_backend: AudioBackend) {
	match audio_backend {
		AudioBackend::None => (),
		AudioBackend::Pipewire(runtime_dir) => {
			env.add("PIPEWIRE_RUNTIME_DIR", runtime_dir.to_string_lossy())
				.add("PIPEWIRE_LATENCY", "512/48000");

			args.add_many(vec!["-audiodev", "pipewire,id=snd"]);
		}
		AudioBackend::Spice => {
			args.add_many(vec!["-audiodev", "spice,id=snd"]);
		}
	}
}

pub fn add_audio_frontend(args: &mut ArgWriter, audio_backend: AudioFrontend) {
	match audio_backend {
		AudioFrontend::None => (),
		AudioFrontend::IntelHda(t) => intel_hda(args, "intel-hda", t),
		AudioFrontend::IntelHdaIch9(t) => intel_hda(args, "ich9-intel-hda", t),
		AudioFrontend::UsbAudio => {
			args.add_many(vec!["-device", "qemu-xhci", "-device", "usb-audio,audiodev=snd"]);
		}
	};

	fn intel_hda(args: &mut ArgWriter, hda_card: &str, hda_type: IntelHdaType) {
		let hda = format!("{},audiodev=snd,mixer=off", hda_type.device_name());
		args.add_many(vec!["-device", hda_card, "-device", &hda]);
	}
}

pub fn add_networking(args: &mut ArgWriter, networking: Networking) {
	match networking {
		Networking::None => {
			args.add_many(vec!["-nic", "none"]);
		}
		Networking::User => {
			args.add_many(vec!["-nic", "model=e1000"]);
		}
		Networking::VirtioUser => {
			args.add_many(vec!["-nic", "model=virtio-net-pci"]);
		}
	}
}

pub fn add_pci(args: &mut ArgWriter, devices: &[String]) {
	for address in devices {
		args.add("-device").add(format!("vfio-pci,host={address}"));
	}
}

pub fn add_disks(args: &mut ArgWriter, disks: Vec<Disk>) {
	for disk in &disks {
		_ = match disk {
			Disk::Raw(device) => args.add("-drive").add(raw_disk(device, "media=disk")),
			Disk::Virtio(device) => args.add("-drive").add(raw_disk(device, "if=virtio")),
		};
	}

	fn raw_disk(device: &Path, options: &str) -> String {
		let dev = device.to_string_lossy();
		format!("file={dev},format=raw,{options}")
	}
}

pub fn add_usb(args: &mut ArgWriter, devices: Vec<UsbDevice>) {
	if devices.is_empty() {
		return;
	}

	args.add("-usb");

	for device in devices.into_iter() {
		add_usb_device(args, device);
	}
}

pub fn add_usb_device(args: &mut ArgWriter, device: UsbDevice) {
	let device_config = match device {
		UsbDevice::HostVidPid { vendor, product } => format!("usb-host,vendorid=0x{vendor:x},productid=0x{product:x}"),
		UsbDevice::Device(device_config) => device_config,
	};

	args.add("-device").add(device_config);
}

pub fn add_looking_glass(args: &mut ArgWriter, tmp: &mut TmpFileWriter, config: LookingGlass) {
	let LookingGlass::Yes(uid, gid) = config else {
		return;
	};

	let mode = Mode::from_bits_truncate(0o644);
	tmp.add("/dev/shm/looking-glass", uid, gid, mode);

	args.add_many(vec![
		"-device",
		"ivshmem-plain,memdev=ivshmem,bus=pci.0",
		"-object",
		"memory-backend-file,id=ivshmem,share=on,mem-path=/dev/shm/looking-glass,size=32M",
	]);
}

pub fn add_spice(args: &mut ArgWriter, config: Spice) {
	match config {
		Spice::No => (),
		Spice::Yes => {
			args.add_many(vec![
				"-spice",
				"port=5900,disable-ticketing=on",
				"-device",
				"virtio-keyboard-pci",
				"-device",
				"virtio-mouse-pci",
			]);
		}
	};
}

pub fn add_spice_agent(args: &mut ArgWriter, config: SpiceAgent) {
	match config {
		SpiceAgent::No => (),
		SpiceAgent::Yes => {
			args.add_many(vec![
				"-device",
				"virtio-serial-pci",
				"-chardev",
				"spicevmc,id=vdagent,name=vdagent",
				"-device",
				"virtserialport,chardev=vdagent,name=com.redhat.spice.0",
			]);
		}
	};
}
