### Disclaimer: This is intended for power users, you're expected to write a little rust and bring some technical knowledge for this to work.

For more digestible, beginner friendly methods see BlandManStudios on YouTube: [Single GPU][single-gpu], [Dual GPU][multi-gpu]

[single-gpu]: https://www.youtube.com/watch?v=eTWf5D092VY
[multi-gpu]: https://www.youtube.com/watch?v=m8xj2Py8KPc

# vfio-run

This is a helper for running a qemu VM with VFIO GPU-passthrough in multiple configurations.  
It allows you to configure your VM in a simple manner, skipping the endless pages of XML.  
You can also pick and choose configs, devices and features in multiple profiles.

This isn't a complete project, it's more of a template for your own Helper.

Configuration is done in code, with the gory details abstracted away.  
See `src/config.rs`, then just `cargo build` when you're done. If you don't have rust set up on your machine, you can use the included devcontainer.  
Rust statically links most dependencies, so you can then run the resulting binary on your host system.

# Setup
This is a very concice guide and probably missing some stuff. If something doesn't work or you get stuck, here's some supplementary reading: [Complete Single GPU Passthrough][single-gpu-passthrough], [Looking Glass Documentation][looking-glass].

- Setup IOMMU and determine the PCI address(es) of your GPU. Refer to the [Arch wiki][iommu]

- Install dependencies on the host
  - **Arch:** `qemu-full libvirt edk2-ovmf`

- Install Windows normally on bare metal. Doing this allows you to dual-boot in addition to running in a VM.
  Probably unplug your other drives to protect them from any funny business on windows' part

- Configure a minimal config to get going:
  ```rust
  let builder = ContextBuilder::default()
  	.smp("sockets=1,cores=4,threads=2")
  	.ram("8G")
  	.raw_disk("/dev/sdd") // the block device you installed Windows on
  	.user_networking()
  	.window()
  	.vga(Vga::Standard);
  ```

- Start the VM with `vfio-run run full`

- Install [Spice guest utils][spice-guest-utils] and [VirtIO drivers][virtio-win] on the guest

- Add OVMF bios, VirtIO networking, VirtIO disk, audio, then check if it works
  ```rust
  .ovmf_bios("/usr/share/edk2/x64/OVMF.fd")
  .vfio_user_networking()
  .virtio_disk("/dev/sdd")
  .pipewire("/run/user/1000") // your UID
  .intel_hda(IntelHdaType::Output)
  ```

  > Windows will likely refuse to boot from VirtIO at first, this requires [some fiddling][virtio-dummy-disk].  
  > You can also use another physical disk for this.

- If using VirtIO disks backed by physical SSDs, disable disk defragging in Windows

- Add your GPU and, optionally, the drivers that need to be unloaded. NVIDIA Example:
  ```rust
  .pci_device("0000:01:00.0") // PCI address(es) determined in the first step
  .pci_device("0000:01:00.1")
  .unloaded_drivers(vec!["nvidia_drm", "nvidia_uvm", "nvidia_modeset", "nvidia"])
  ```

  > Important: If you try to run this from your graphical session, it will probably fail due to your GPU being in use.  
  > Stop your graphical session and switch to a TTY, then run it.  
  > If you only have one GPU, this **will steal your screen(s) until the VM shuts down**. then try its best to put it back.  
  > Especially on NVIDIA cards, you might not get your TTY back on the screen after the VM stops. Starting your Xorg or Wayland server again should work, as long as you can do it blind.

- If you're doing Single-GPU passthrough, you also want to add your keyboard and mouse:
  ```rust
  .usb_device(0x046d, 0xc08b) // get these IDs from lsusb
  .usb_device(0x75fa, 0x0088)
  ```

- Boot the VM. You should see Windows start on the monitor(s) attached to the GPU you passed.
  > You will likely have to drop to a TTY and stop your display server to successfully detach the GPU. How this is done may differ depending on your distro and setup, please check the relevant documentation.

- If you have a second GPU, add looking glass and spice, then try connecting with the looking glass client.
  ```rust
  .looking_glass(1000, 1000) // your UID and GID
  .spice_kvm()
  ```

[single-gpu-passthrough]: https://github.com/QaidVoid/Complete-Single-GPU-Passthrough
[looking-glass]: https://looking-glass.io/docs/B6/install/
[iommu]: https://wiki.archlinux.org/title/PCI_passthrough_via_OVMF
[spice-guest-utils]: https://www.spice-space.org/download/windows/spice-guest-tools/spice-guest-tools-latest.exe
[virtio-win]: https://fedorapeople.org/groups/virt/virtio-win/direct-downloads/stable-virtio/virtio-win.iso
[virtio-dummy-disk]: https://forum.proxmox.com/threads/vm-wont-start-after-disk-set-to-virtio.94646/

# Known issues

### Application doesn't want to run in VM

Some applications or anticheats will refuse to run in a VM. In some cases, they can be fooled by configuring SMBIOS. Use `.smbios_auto()` to automatically read relevant values from the host system and build a credible config. Tested against VRChat EAC, others may or may not work.

### QEMU complains "Failed to mmap 0000:01:00.0 BAR 1. Performance may be slow"
dmesg has lines like this:
```
[Sun Mar 19 13:57:12 2023] x86/PAT: CPU 0/KVM:1329 conflicting memory types f800000000-fc00000000 write-combining<->uncached-minus
```
From what I've read, this seems to be a very specific issue with the nvidia drivers **while the amdgpu drivers are also loaded**. There's a [GitLab issue about it here][gitlab-ticket].

On my system, this happens when the nvidia drivers have already configured the GPUs iomem to use the `write-combining` cache strategy, when vfio wants `uncached-minus` (see [kernel documentation on PAT][pat]). Unloading the nvidia drivers *does not* remove these PAT entries, and vfio then complains that it doesn't match the cache strategy it was expecting.

#### PAT-dealloc

There is now an automated workaround that works without patching the kernel, see [pat-dealloc](https://github.com/thorio/pat-dealloc). When you have it installed, add `.pat_dealloc("0000:01:00.0")` to your config, substituting the PCI address of your GPU.

This will automatically clear PAT entries for the GPU when attaching or detaching, thus giving each driver a clean slate to work with.

#### Other Solutions

You can blacklist the nvidia kernel modules in modprobe's config, then run `vfio-run detach full` or `vfio-run attach full` after booting to choose between:
- working GPU in the VM, but bad GPU performance on the host (depending on workload)
- normal GPU performance on the host, but nonfunctional GPU passthrough

Rebooting clears the PAT, so you can choose again.

There are some other workarounds [here][workarounds-link].

[gitlab-ticket]: https://gitlab.freedesktop.org/drm/amd/-/issues/2794
[pat]: https://www.kernel.org/doc/Documentation/x86/pat.txt
[workarounds-link]: https://github.com/Kinsteen/win10-gpu-passthrough#compute-mode---vfio-fix

## Performance tuning

For best performance, you should use these cpu options:
```rust
.cpu("host,topoext,kvm=off,hv_frequencies,hv_time,hv_relaxed,hv_vapic,hv_spinlocks=0x1fff,hv_vendor_id=thisisnotavm")
```

Also look at CPU pinning, eg. `.cpu_affinity("0-5,8-13")` for 6 cores with corresponding hyperthreading pairs on Ryzen 5800X and 7800X3D.  
This will vary based on your CPU and alotted cores, see [taskset(1)][taskset] and [lstopo(1)][lstopo]

[taskset]: https://man7.org/linux/man-pages/man1/taskset.1.html
[lstopo]: https://linux.die.net/man/1/lstopo
