# Run a local lab

A lab is a machine or a virtual network you own, used for work you are allowed to do. The catalog names the tools. This page is how to keep that work off the daily-driver network of the reference Mac.

## Keep the T2 host on its day job

Observed on this MacBookPro16,1 (`docs/hardware-scan.md`):

- Wi-Fi interface `wlp5s0` is a station link on the Broadcom BCM4364. Wireless assessment belongs on a separate adapter or a separate machine. Monitor mode was not tested here (open).
- Bluetooth is powered on and the patch file `BCM.hcd` is missing. Do not download firmware as part of standing up this workspace. Bluetooth lab work belongs on a separate adapter.
- Touch ID and the T2 Secure Enclave are blocked on a macOS-free disk. Nothing in this track enrolls a finger or talks to the SEP.

The internal keyboard, trackpad, NVMe, and the T2 bridge stay on the packages named in the gap analysis. Do not replace them with a lab metapackage.

## One isolated network

On a Fedora host where you are willing to install the lab group, create a virtual network that has no route to your home LAN, then put a guest on it. libvirt's isolated network mode does that. A typical shape, after `libvirt` and `qemu-system-x86` are installed, is a network named `lab` with DHCP and no forward mode:

```bash
virsh net-define /dev/stdin <<'EOF'
<network>
  <name>lab</name>
  <bridge name="virbr-lab"/>
  <ip address="192.168.199.1" netmask="255.255.255.0">
    <dhcp>
      <range start="192.168.199.10" end="192.168.199.50"/>
    </dhcp>
  </ip>
</network>
EOF
virsh net-start lab
virsh net-autostart lab
```

Those commands create a libvirt network. They do not attach the laptop's Wi-Fi to it. Point guests at that network only.

For a single service, Podman is enough and does not need a VM:

```bash
podman network create lab
podman run --rm --network lab docker.io/library/nginx:stable
```

Use an image you already trust. Stop the container when you are done. Do not publish it on the host's LAN interface unless that is the point of the exercise.

## Guests

Use an operating system image you are allowed to test. Keep it on the `lab` network. The catalog's `lab` group includes `virsh`, `virt-manager`, `qemu-system-x86_64`, `xfreerdp`, and `remmina` for opening those guests.

Scan and review only addresses inside that network. The tools in the catalog are the ones you would install for that work. Their command lines are not repeated here.

## What not to do on this repo's behalf

- Do not install the tool groups as part of editing this repo.
- Do not edit `/etc` overlays that the hardware scan already recorded for audio or the Touch Bar.
- Do not load or unload `t2bce` modules, rebuild the initramfs, or change the kernel command line.
- Do not copy firmware, wordlists, or exploit archives into git.
