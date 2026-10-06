# Hardware scan

Scan date: 2026-10-06. Host product: MacBookPro16,1. No serial number, disk UUID, MAC address, or Wi-Fi SSID is recorded here.

Commands were read-only. Nothing was installed, and `/etc` was not modified.

## Identity

| Fact | Evidence | Label |
| --- | --- | --- |
| product_name MacBookPro16,1, vendor Apple Inc., product_version 1.0 | `cat /sys/class/dmi/id/product_name` and the sibling vendor and version files | observed |
| Model board-id `Mac-E1008331FDC96864` | `cat /sys/class/dmi/id/board_name` | observed |
| Running kernel `7.2.9-310.t2.fc44.x86_64` | `uname -r` | observed |
| Fallback kernel `6.19.12-210.t2.fc44` is installed and is not the running kernel | `rpm -q kernel` | observed |
| Command line contains `intel_iommu=on iommu=pt pm_async=off`, repeated twice, plus `rhgb quiet` and a btrfs root | `/proc/cmdline` with the root UUID redacted | observed |
| Dracut snippet adds `t2bce_dma`, `t2bce_core`, and `t2bce_vhci` | `/usr/lib/dracut/dracut.conf.d/t2linux-modules-install.conf` | observed |

The disk layout (EFI, ext4 `/boot`, btrfs root, no macOS volume) is a prior note. This scan did not re-walk the partition table. Label: open for a fresh `lsblk` pass. The running root is a btrfs subvolume, which matches that note.

## Packages and services

`rpm -q` on 2026-10-06:

| Package | Version |
| --- | --- |
| t2linux-config | 16.1.0-1.fc44 |
| t2linux-audio | 2.2.0-1.20260917git9527bff.fc44 |
| t2fanrd | 0.1.1-1.20260429git48baf96.fc44 |
| tiny-dfr | 0.3.7-3.fc44 |
| brcmfmac-firmware | 20260916-1.fc44 |
| t2-touchid | 0.1.0-1.20260929gitca705e4.fc44 |

`systemctl is-active`:

| Unit | State |
| --- | --- |
| t2fanrd.service | active, enabled |
| tiny-dfr.service | inactive (dead), static |
| kait2en-t2-touchid.service | active, enabled |
| t2-touchid.service | not found |

tiny-dfr did run earlier this boot. `systemctl status` shows `ExecStart=/usr/bin/tiny-dfr` ended with `code=killed, signal=TERM` at 09:47:48 EDT, `Result=success`, `NRestarts=0`. The journal lines just before the stop are `Lid Switch event: On` and then `Lid Switch event: Off`. The unit stayed dead afterward. The device units `dev-tiny_dfr_display.device`, `dev-tiny_dfr_backlight.device`, and `dev-tiny_dfr_display_backlight.device` were active at scan time. Label: observed.

## PCI and USB devices that matter

`lspci -nn`:

| Device | Id |
| --- | --- |
| Intel UHD Graphics 630 | 8086:3e9b |
| AMD Navi 14 (Radeon 5500M class) | 1002:7340 |
| AMD Navi HDMI audio | 1002:ab38 |
| Apple ANS2 NVMe | 106b:2005 |
| Apple T2 Bridge | 106b:1801 |
| Apple T2 Secure Enclave | 106b:1802 |
| Apple Audio Device | 106b:1803 |
| Broadcom BCM4364 rev 04 | 14e4:4464 |
| Intel JHL7540 Thunderbolt 3 bridges, NHI, and USB | 8086:15ea, 8086:15eb, 8086:15ec |

`lsusb` Apple devices: iBridge `05ac:8233`, FaceTime HD Camera `05ac:8514`, Ambient Light Sensor `05ac:8262`, Headset `05ac:8103`, Touch Bar Display `05ac:8302`, Touch Bar Backlight `05ac:8102`, Internal Keyboard / Trackpad `05ac:0340`. One Apple device `05ac:8104` had an empty product string.

Loaded modules included `t2bce_core`, `t2bce_dma`, `t2bce_vhci`, `t2bce_audio`, `brcmfmac`, `appletbdrm`, `applesmc`, `apple_gmux`, `hid_apple`, `hid_appletb_kbd`, `hid_appletb_bl`, `amdgpu`, `i915`, and `thunderbolt`. The kernel log warns that each `t2bce_*` module is from the staging directory. Label: observed.

## Three buckets

### Works on the stock T2 packages

- The machine is booted and the internal NVMe is the disk in use. Label: observed.
- Keyboard and trackpad input nodes exist: `Apple Inc. Apple Internal Keyboard / Trackpad` on `t2bce_vhci`. Label: observed. Palm rejection and force touch were not measured.
- Internal panel: `card3-eDP-1` status `connected`, driver `amdgpu`. `i915` is loaded on `card1`; its DP and HDMI connectors were `disconnected`. Label: observed. Which GPU should own the panel, and whether dGPU power switching works, is open.
- Wi-Fi interface `wlp5s0` operstate `up`, NetworkManager device state `connected`. Firmware line from the kernel journal: `BCM4364/4 wl0: Jul 10 2023 12:53:18 version 9.30.503.0.32.5.92 FWID 01-c06f991b`, requested as `brcm/brcmfmac4364b3-pcie`. Label: observed. The SSID is omitted.
- `t2fanrd` is active. Label: observed.
- FaceTime HD Camera exposes `/dev/video0` and `/dev/video1`. The logged-in user has an ACL of `rw` on `/dev/video0`. No frame was captured. Label: devices observed, picture quality open.
- `t2_ncm` operstate `up`, carrier 1. Label: observed.

### Works only because of local /etc overlays

- ALSA card 0 id is `t2-16_1` (`AppleT2x6 - Apple T2 Audio`). `/etc/udev/rules.d/99-t2-audio-rename.rules` sets `ATTR{id}="t2-16_1"` and comments that a full DMI id such as `t2-MacBookPro16,1` exceeds ALSA's 15-character limit and truncates. Label: observed.
- PipeWire default sink `audio_effect.t2-161-speakers` and source `audio_effect.t2-161-mic`. Overlay files present: `/etc/wireplumber/wireplumber.conf.d/99-t2-audio.conf` and `/etc/t2linux-audio/16_1`. Packaged graphs also exist under `/usr/share/t2linux-audio/` for several model ids (`15_1`, `15_4`, `16_1`, `16_2`, `16_3`, `16_4`, `8_1`, `8_2`, `9_1`). Coefficient files were not copied. Label: paths and default node names observed. Sound quality was not re-measured in this scan.
- A prior note said writing the card id `t2-16,1` (comma) returned `EINVAL`, while `t2-16_1` succeeded. The live udev comment explains the rename as a 15-character limit. Both can be true. The live rule is the one in force. Label: prior note inferred until someone repeats the sysfs write.

### Blocked

- Touch ID. `kait2en-t2-touchid.service` is active. This boot's unit journal says the sensor was not reachable (`BiometricKit was not advertised`, connect error 99, "Cannot assign requested address") and that no macOS user has a finger enrolled or the SEP is still locked. `fprintd-list` reports one virtual device and no fingers enrolled for the current user. The IPv6 address from the log is omitted. Label: observed.
- Touch Bar controls right now. `tiny-dfr.service` is dead after a lid cycle (see above). DRM connector `card0-USB-2` (`appletbdrm`) reported `enabled=disabled` and `status=unknown`. Packaged `/usr/share/tiny-dfr/config.toml` has `MediaLayerDefault = false`. `/etc/tiny-dfr/` exists and has no `config.toml`. The rpm file list names `/etc/tiny-dfr/config.toml`, but that file is not on disk. Label: observed.
- Bluetooth firmware. `bluetoothctl show` reports `Powered: yes` and name `fedora`. The kernel log names the controller `BCM4364B3 Trinidad Olympic GEN (MFG)`, says `brcm/BCM.hcd` was not found, and logs `Failed to set baudrate` (`-16`). `/usr/lib/firmware/brcm/BCM.hcd` and `/lib/firmware/brcm/BCM.hcd` are absent. Label: observed. Pairing a HID device was not retested.
- Wi-Fi P2P. The kernel log repeats `brcmf_cfg80211_add_iface: add iface p2p-dev-wlp5s0 type 10 failed: err=-52`. The station interface stayed up. Label: observed. Impact on normal STA use is low.

## Other observed behavior

- Suspend path in this boot's kernel journal: `t2bce_audio`, `t2bce_vhci`, and `t2bce_core` each logged suspend and resume with `status=0`. `amdgpu` completed a MODE1 reset and resumed. This is one journal sequence, not a suspend test plan. Label: observed in the log, open as a reliability claim.
- While tiny-dfr was running, libinput logged `kernel bug: Touch jump detected and discarded` for `Apple Inc. Apple Internal Keyboard / Trackpad`, more than once. Label: observed.
- Backlight class: `gmux_backlight`, `appletb_backlight`. LED `:white:kbd_backlight` is present. Label: observed. Brightness levels were not swept.

## Disagreements with prior notes

Prior notes live under the workspace memory topics for this machine. Live commands win.

| Prior note | Live result |
| --- | --- |
| The Touch ID bridge can reach the sensor over `t2_ncm` | This boot's `kait2en-t2-touchid` journal says it cannot reach the sensor (error 99) and that the SEP is locked or no finger was enrolled in macOS |
| `/etc/tiny-dfr` is empty and the daemon uses the packaged config while running | The directory is still empty, and `tiny-dfr.service` is dead. It ran for about an hour, received SIGTERM next to a lid event, and did not restart |
| Packaged audio card id `t2-16,1` was rejected by the kernel | Live card id is `t2-16_1` because the udev rule renames it. The rule's own comment cites ALSA's 15-character id limit |

Wi-Fi firmware version, missing `BCM.hcd`, kernel release, cmdline flags, and the DSP sink names match the prior notes.
