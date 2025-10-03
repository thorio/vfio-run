> [!IMPORTANT]
This project is aimed at power users, you're expected to bring some technical knowledge and compile your own config.  
> For more digestible, beginner friendly methods see BlandManStudios on YouTube: [Single GPU][single-gpu], [Dual GPU][multi-gpu]

[single-gpu]: https://www.youtube.com/watch?v=eTWf5D092VY
[multi-gpu]: https://www.youtube.com/watch?v=m8xj2Py8KPc

# vfio-run

![GitHub License](https://img.shields.io/github/license/thorio/vfio-run?style=flat-square)
![GitHub last commit](https://img.shields.io/github/last-commit/thorio/vfio-run?style=flat-square)

This is a helper for running a qemu VM with VFIO GPU-passthrough in multiple configurations.  
It allows you to configure your VM in a simple manner, skipping the endless pages of XML.  
You can also pick and choose configs, devices and features in multiple profiles.

Configuration is done in code, with the gory details abstracted away.  
See `src/config.rs`, then just `cargo build` when you're done. If you don't have rust set up on your machine, you can use the included devcontainer.  
Rust statically links most dependencies, so you can then run the resulting binary on your host system.

> [!TIP]
> You can add more CLI options to the `Options` struct in `src/cli.rs` if you need them.

# Setup
This is a very concise guide and probably missing some stuff. If something doesn't work or you get stuck, here's some supplementary reading: [Complete Single GPU Passthrough][single-gpu-passthrough], [Looking Glass Documentation][looking-glass].

**1**. Setup IOMMU and determine the PCI address(es) of your GPU. Refer to the [Arch wiki][iommu].

**2**. Install dependencies on the host:
- **Arch:** `qemu-full libvirt edk2-ovmf cpupower`
- **Debian:** `qemu-system libvirt-daemon-system ovmf linux-cpupower`
- **Ubuntu:** `qemu-system libvirt-daemon-system ovmf cpupower-gui`

**3**. Install Windows normally on bare metal. Doing this allows you to dual-boot in addition to running in a VM.
You should probably unplug your network cable and other drives to protect them from any funny business on windows' part.

**4**. Start with a minimal config to get going:
```rust
config
	.smp("sockets=1,cores=4,threads=2")
	.ram("8G")
	.raw_disk("/dev/disk/by-id/wwn-0x7666696f2d72756e") // the disk you installed Windows on
	.user_networking()
	.window()
	.vga(Vga::Standard);
```

**5**. Start the VM with `vfio-run run full`.

**6**. Install [Spice guest utils][spice-guest-utils] and [VirtIO drivers][virtio-win] on the guest.

**7**. Add OVMF bios, VirtIO networking, VirtIO disk, audio, then check if it works:
```rust
.ovmf_bios("/usr/share/edk2/x64/OVMF.fd") // path may need to be adjusted
.vfio_user_networking()
.virtio_disk("/dev/disk/by-id/wwn-0x7666696f2d72756e")
.pipewire("/run/user/1000") // your UID
.intel_hda(IntelHdaType::Output)
```

> [!NOTE]
> Windows will likely refuse to boot from VirtIO at first, this requires [some fiddling][virtio-dummy-disk].  
> You can also use another physical disk for this.

> [!NOTE]
> If using VirtIO disks backed by physical SSDs, Windows may want to defrag them. Disable defragging to avoid unnecessary wear.

**8**. Add your GPU and, optionally, the drivers that need to be unloaded. NVIDIA Example:
```rust
.pci_device("0000:01:00.0") // PCI address(es) determined in the first step
.pci_device("0000:01:00.1")
.unloaded_drivers(["nvidia_drm", "nvidia_uvm", "nvidia_modeset", "nvidia"])
```

> [!IMPORTANT]
> If you try to run this from your graphical session, it will probably fail due to your GPU being in use.  
> Stop your graphical session and switch to a TTY, then run it.  

> [!CAUTION]
> If you only have one GPU, this **will steal your screen(s) until the VM shuts down**, then try its best to put it back.  
> Especially on NVIDIA cards, you might not get your TTY back on the screen after the VM stops. Starting your Xorg or Wayland server again should work, as long as you can do it blind.

**9**. If you're doing Single-GPU passthrough, you also want to add your keyboard and mouse:
```rust
.usb_device(0x046d, 0xc08b) // get these IDs from lsusb
.usb_device(0x75fa, 0x0088)
```

**10**. Boot the VM. You should see Windows start on the monitor(s) attached to the GPU you passed.

**11**. If you have a second GPU, add looking glass and spice, then try connecting with the looking glass client.
```rust
.looking_glass(1000, 1000) // your UID and GID
.spice_kvm()
.spice_agent()
```

[single-gpu-passthrough]: https://github.com/QaidVoid/Complete-Single-GPU-Passthrough
[looking-glass]: https://looking-glass.io/docs/B6/install/
[iommu]: https://wiki.archlinux.org/title/PCI_passthrough_via_OVMF
[spice-guest-utils]: https://www.spice-space.org/download/windows/spice-guest-tools/spice-guest-tools-latest.exe
[virtio-win]: https://fedorapeople.org/groups/virt/virtio-win/direct-downloads/stable-virtio/virtio-win.iso
[virtio-dummy-disk]: https://forum.proxmox.com/threads/vm-wont-start-after-disk-set-to-virtio.94646/

# Performance tuning

For best performance, you should use these cpu options:
```rust
.cpu("host,topoext,kvm=off,hv_frequencies,hv_time,hv_relaxed,hv_vapic,hv_spinlocks=0x1fff,hv_vendor_id=thisisnotavm")
.cpu_governor("performance")
.cpu_affinity("0-5,8-13") // depends on your CPU
```

The options for `cpu_affinity` will vary based on your CPU and alotted cores, see [taskset(1)][taskset] and [lstopo(1)][lstopo].  
The example is valid for 6 cores with corresponding hyperthreading pairs on Ryzen 5800X and 7800X3D.

[taskset]: https://man7.org/linux/man-pages/man1/taskset.1.html
[lstopo]: https://linux.die.net/man/1/lstopo

# Known issues

### Application doesn't want to run in VM

Some applications or anticheats will refuse to run in a VM. In some cases, they can be fooled by configuring SMBIOS. Use `.smbios_auto()` to automatically read relevant values from the host system and build a credible config. Tested with VRChat EAC, others may or may not work.

### QEMU warnings "Failed to mmap 0000:01:00.0 BAR 1. Performance may be slow"
See [this issue](https://github.com/thorio/vfio-run/issues/1).
