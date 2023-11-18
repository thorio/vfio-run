### Disclaimer: This is intended for power users, you're expected to write a little rust and bring some technical knowledge for this to work.

For more digestible, beginner friendly methods see BlandManStudios on YouTube: [Single GPU][single-gpu], [Dual GPU][multi-gpu]

# vfio-run

This is a helper for running a qemu VM with VFIO GPU-passthrough in multiple configurations.  
It allows you to configure your VM in a simple manner, skipping the endless pages of XML.  
You can also pick and choose configs, devices and features in multiple profiles.

This isn't a complete project, it's more of a template for your own Helper.

Configuration is done in code, with the gory details abstracted away.  
See `src/config.rs`.

[single-gpu]: https://www.youtube.com/watch?v=eTWf5D092VY
[multi-gpu]: https://www.youtube.com/watch?v=eTWf5D092VY

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
  	.with_smp("sockets=1,cores=4,threads=2")
  	.with_ram("8G")
  	.with_raw_disk("/dev/sdd") // the block device you installed Windows on
  	.with_user_networking()
  	.with_window()
  	.with_vga(Vga::Standard);
  ```

- Install [Spice guest utils][spice-guest-utils] and [VirtIO drivers][virtio-win] on the guest

- Add OVMF bios, VirtIO networking, VirtIO disk, audio, then check if it works
  ```rust
  .with_ovmf_bios("/usr/share/edk2/x64/OVMF.fd")
  .with_vfio_user_networking()
  .with_virtio_disk("/dev/sdd")
  .with_pipewire("/run/user/1000") // your UID
  ```

  > Windows will likely refuse to boot from VirtIO at first, this requires [some fiddling][virtio-dummy-disk].  
  > You can also use another physical disk for this.

- If using VirtIO disks backed by physical SSDs, disable disk defragging in Windows

- Add your GPU and, optionally, the drivers that need to be unloaded. NVIDIA Example:
  ```rust
  .with_pci_device("0000:01:00.0") // PCI address(es) determined in the first step
  .with_pci_device("0000:01:00.1")
  .with_unloaded_drivers(vec!["nvidia_drm", "nvidia_uvm", "nvidia_modeset", "nvidia"])
  ```

  > Important: If you try to run this from your graphical session, it will probably fail due to your GPU being in use.  
  > Stop your graphical session and switch to a TTY, then run it.  
  > If you only have one GPU, this **will steal your screen(s) until the VM shuts down**. then try its best to put it back.  
  > Especially on NVIDIA cards, you might not get your TTY back on the screen after the VM stops. Starting your Xorg or Wayland server again should work, as long as you can do it blind.

- If you're doing Single-GPU passthrough, you also want to add your keyboard and mouse:
  ```rust
  .with_usb_device(0x046d, 0xc08b) // get these IDs from lsusb
  .with_usb_device(0x75fa, 0x0088)
  ```

- Boot the VM. You should see Windows start on the monitor(s) attached to the GPU you passed.

- If you have a second GPU, add looking glass and spice, then try connecting with the looking glass client.
  ```rust
  .with_looking_glass(1000, 1000) // your UID and GID
  .with_spice()
  ```

[single-gpu-passthrough]: https://github.com/QaidVoid/Complete-Single-GPU-Passthrough
[looking-glass]: https://looking-glass.io/docs/B6/install/
[iommu]: https://wiki.archlinux.org/title/PCI_passthrough_via_OVMF
[spice-guest-utils]: https://www.spice-space.org/download/windows/spice-guest-tools/spice-guest-tools-latest.exe
[virtio-win]: https://fedorapeople.org/groups/virt/virtio-win/direct-downloads/stable-virtio/virtio-win.iso
[virtio-dummy-disk]: https://forum.proxmox.com/threads/vm-wont-start-after-disk-set-to-virtio.94646/

## Performance tuning

For best performance, you should use these cpu options:
```rust
.with_cpu("host,topoext,kvm=off,hv_frequencies,hv_time,hv_relaxed,hv_vapic,hv_spinlocks=0x1fff,hv_vendor_id=thisisnotavm")
```

Also look at CPU pinning, eg. `.with_cpu_affinity("0-5,8-13")` for 6 cores with corresponding hyperthreading pairs on Ryzen 5800X and 7800X3D.  
This will vary based on your CPU and alotted cores, see [taskset(1)][taskset] and [lstopo(1)][lstopo]

[taskset]: https://man7.org/linux/man-pages/man1/taskset.1.html
[lstopo]: https://linux.die.net/man/1/lstopo
