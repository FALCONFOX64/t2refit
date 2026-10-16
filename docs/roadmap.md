# Roadmap

Phase 0 is a local repo, a sanitized scan of the reference MacBookPro16,1, a ranked gap list, Cursor project rules, and a read-only scanner. Driver implementation is not in this roadmap.

A separate userspace track, the Fedora offensive-lab catalog, lives under `catalog/`, `python/`, `rust/t2offsec-inventory/`, and `docs/offensive/`. It does not close the gaps below and it does not add a kernel module. Start at `docs/index.md`.

## Phase 1 — Make the scan worth filing upstream

Build candidate: extend `scripts/scan-t2-host.sh` only if a new flag would have caught one of the top gaps automatically. The first flags already cover tiny-dfr inactive, missing `BCM.hcd`, and an over-long ALSA card id.

Then write two short notes, still in this repo, that a human can paste upstream:

- tiny-dfr: the unit is TERM'd across a lid event and does not restart, with the journal times from `docs/hardware-scan.md`.
- t2linux-audio: ALSA card ids are limited to 15 characters, so `t2-MacBookPro16,1` cannot be the id. The working id on this host is `t2-16_1`.

Why this waits: those notes should be checked once more after a reboot before they are sent. This phase still does not edit `/etc` or restart services.

## Phase 2 — Touch Bar resting layer, opt-in

Build candidate: a documented tiny-dfr config snippet with `MediaLayerDefault = true`, shipped as an example under `docs/` and not installed onto the machine.

Why this waits: installing it changes the running desktop. The owner asked for brightness and media keys as the resting layer. tiny-dfr remains the daemon. This repo does not draw its own Touch Bar.

Depends on Phase 1's restart finding. A config file does nothing while the service stays dead.

## Phase 3 — Firmware and Bluetooth, without blobs

Build candidate: scanner output that states `bcm_hcd=missing` clearly enough to open a Fedora `brcmfmac-firmware` issue. Document the legal source (a macOS recovery image via the existing `get-apple-firmware` tool) and do not commit the file.

Why this waits: Wi-Fi already works. Bluetooth is powered on but still on MFG firmware. Copying `BCM.hcd` into git is a hard refusal.

The P2P `err=-52` stays a note on the brcmfmac issue, not a project of its own.

## Phase 4 — Touch ID, upstream only

No build in this repo unless t2-touchid upstream asks for a patch that does not touch SEP keybags.

The blocked state is: no macOS volume, no enrolled finger, and on the 2026-10-06 boot the bridge could not reach the sensor. Linux-native enrollment is an upstream research problem. It is not a Phase 4 coding task here.

## Explicitly later, and not a fork

| Idea | Home | Why it is not Phase 1 |
| --- | --- | --- |
| Trackpad touch-jump | kernel input / t2linux | Needs a trace of the input device, not a new daemon |
| Take `t2bce_*` out of staging | deqrocks/t2bce | The modules already run; quality work belongs upstream |
| dGPU power toggle | t2linux hybrid-graphics notes | Not measured on this host yet |
| Speaker DSP coefficients | lemmyg/t2-apple-audio-dsp | Model-specific and proprietary-shaped; do not vendor |
| Other T2 board-ids | this repo's matrix, after a scan | MacBookPro16,1 is the only filled row |

## Done means

A phase is done when the scanner or the doc changed, the claim labels are still honest, and no firmware blob, SEP bypass, or unsolicited `/etc` edit landed in the tree.
