# Gap analysis

Ranked for daily use of an Intel T2 Mac, with this MacBookPro16,1 as target 1. Each item names a home. "This repo" means new work belongs here. Otherwise the work belongs in the named project, and this repo only tracks it.

Sources for the family-wide picture: the t2linux state page at `https://wiki.t2linux.org/state/` and the kernel-module list on `https://t2linux.org/` (read 2026-10-06). Those rows are not re-tested on any machine except where this file cites a local command.

## Top gaps

### 1. Touch Bar daemon does not come back after a lid cycle

- Evidence: observed. `tiny-dfr.service` went inactive at 09:47:48 EDT with signal TERM, `NRestarts=0`, immediately after lid-switch journal lines. Touch Bar DRM connector `card0-USB-2` was `enabled=disabled`. Packaged `MediaLayerDefault = false`.
- Impact: the Touch Bar is the brightness and media control surface on this laptop. After sleep or a lid event it is blank until something starts the unit again.
- Home: tiny-dfr (package `tiny-dfr` 0.3.7-3.fc44, unit `/usr/lib/systemd/system/tiny-dfr.service`).
- Why: tiny-dfr already owns the Touch Bar display. The defect is restart policy across lid events, plus the resting layer the owner wants (`MediaLayerDefault = true`) is a config change in that project. A new daemon would duplicate it.
- Claim label: observed for the stop; open for the exact udev/device unit that failed to start it again.

### 2. Trackpad touch jumps

- Evidence: observed. libinput, via the tiny-dfr journal, logged `kernel bug: Touch jump detected and discarded` on `Apple Internal Keyboard / Trackpad` repeatedly during the hour tiny-dfr was up.
- Impact: cursor jumps while the palm or a second finger is near the pad. t2linux already documents the trackpad as usable and short of macOS (no force touch, weak palm rejection).
- Home: the kernel input path used for the T2 trackpad (t2linux / hid-apple / magicmouse), not a new userspace driver in this repo.
- Why: libinput attributes the jump to the kernel. Fixing it in a daemon would hide symptoms.
- Claim label: observed log line. The specific kernel function is open until someone traces that input device.

### 3. Speaker and mic DSP depend on a local card-id rename

- Evidence: observed. Card id is `t2-16_1` only because `/etc/udev/rules.d/99-t2-audio-rename.rules` forces it. The rule says `t2-MacBookPro16,1` does not fit ALSA's 15-character id. Default nodes are `audio_effect.t2-161-speakers` and `audio_effect.t2-161-mic`, fed by `/etc/t2linux-audio/16_1` and a WirePlumber drop-in. Upstream DSP work is lemmyg/t2-apple-audio-dsp, packaged as `t2linux-audio` 2.2.0.
- Impact: a stock install of the package does not attach the 16,1 speaker graph unless this rename exists. The graphs are model-specific. t2linux warns that the 16-inch 2019 six-speaker profile must not be copied to other models.
- Home: t2linux-audio packaging and lemmyg/t2-apple-audio-dsp.
- Why: the package already ships the graphs. The bug is the card id it expects versus the 15-character ALSA limit. This repo must not copy the coefficient files.
- Claim label: observed for the rename and the default nodes. Sound quality versus macOS is open (not measured in this scan).

### 4. Bluetooth is up on factory firmware

- Evidence: observed. Controller power is on. Kernel log: `BCM4364B3 Trinidad Olympic GEN (MFG)`, patch file `brcm/BCM.hcd` not found, `Failed to set baudrate` (`-16`). The hcd file is absent under `/usr/lib/firmware/brcm` and `/lib/firmware/brcm`. Wi-Fi STA is fine on `brcmfmac` with firmware 9.30.503.0.32.5.92. P2P interface creation fails with `err=-52` and does not take the STA link down.
- Impact: Bluetooth devices may pair poorly or at the wrong baud. The MFG name is what the controller reports when the patch file is missing. Wi-Fi daily use is not blocked.
- Home: `brcmfmac-firmware` packaging (and the upstream practice of taking the patch file from a macOS recovery image). Driver is mainline `brcmfmac`.
- Why: the kernel driver is already loaded. Shipping the blob from this repo would copy proprietary firmware, which this project refuses.
- Claim label: observed. Whether a given headset actually fails is open (no pairing test in this scan).

### 5. Touch ID cannot enroll, and this boot could not reach the sensor

- Evidence: observed. Service `kait2en-t2-touchid.service` is active. Journal: BiometricKit was not advertised, connect error 99, and either no macOS user has a finger enrolled or the SEP is locked. `fprintd-list` shows a virtual device and zero enrolled fingers. PCI device `106b:1802` is the T2 Secure Enclave.
- A prior note said the bridge had reached the sensor. This boot disagrees. See `docs/hardware-scan.md`.
- Impact: fingerprint unlock, sudo, and polkit via Touch ID are unavailable on this macOS-free disk.
- Home: t2-touchid (the installed bridge). The t2linux state page lists the T2 Secure Enclave, including Touch ID, as not working.
- Why: enrollment stores a biometric keybag in the SEP. Linux matching, where it works at all, needs a finger enrolled in macOS. Building a bypass in this repo is out of scope.
- Claim label: observed for this boot's failure. Whether error 99 is a missing IPv6 address on `t2_ncm` or a locked SEP is open. Both messages were logged.

## Kept with the current upstream

| Item | This host | Home | Label |
| --- | --- | --- | --- |
| Boot, NVMe, keyboard | Up | t2linux kernel and `t2bce` (`https://github.com/deqrocks/t2bce`) | observed |
| `t2bce_*` still staging | Kernel warns "quality is unknown" | deqrocks/t2bce, then mainline | observed |
| One suspend/resume of the bridge | Journal `status=0` | t2bce | observed once; reliability open |
| Fan | `t2fanrd` active | t2fanrd | observed |
| Wi-Fi STA | `wlp5s0` up, BCM4364/4 firmware loaded | brcmfmac + brcmfmac-firmware | observed |
| Internal panel | amdgpu `eDP-1` connected | amdgpu; hybrid power toggle is a t2linux open item | panel observed; power toggle open |
| Camera | V4L nodes present, user ACL rw | uvcvideo through the T2 USB bridge | devices observed; image open |
| Touch Bar silicon | `appletbdrm` bound, USB ids present | kernel `appletbdrm` (t2linux notes the input drivers upstreamed in 6.15; the bridge is still required) | devices observed |
| T2 audio/video encoder (Sidecar) | Not tested | t2linux state page: not working | open here; sourced as a family-wide gap |

## What this repo should build

Only the pieces no upstream already owns:

1. The read-only scanner and the sanitized record. That is this commit.
2. Later, scanner flags and a short repro note suitable to file against tiny-dfr (lid restart) and t2linux-audio (15-character card id). Drafts, not forks.
3. A model matrix for other T2 board-ids, filled only when someone runs the scanner on that machine. The t2linux wiki stays the source for machines this repo has not seen.

No new kernel module is justified by this scan.

## Other T2 models

Not verified here. t2linux's state page treats the family as one table: internal SSD, USB, keyboard, and Wi-Fi working; trackpad and audio partial; Touch Bar keyboard mode working where `apple-bce` / `t2bce` is present; Secure Enclave not working. Wi-Fi chipset rows on the t2linux wiki put MacBookPro16,1 on BCM4364 rev 4, island Bali, which matches the firmware file prefix `brcmfmac4364b3-pcie.apple,bali` seen on disk. Do not copy the 16,1 speaker graph to another model.
