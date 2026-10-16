# Offensive workspace

This track is a Fedora userspace catalog for authorized lab work and for testing development environments. It sits beside the T2 hardware tree. It does not replace it.

## Two trees, one repo

| Track | Owns | Does not own |
| --- | --- | --- |
| Hardware | `docs/hardware-scan.md`, `docs/gap-analysis.md`, `docs/roadmap.md`, `scripts/scan-t2-host.sh` | Tool installs, exploit notes, drivers |
| Lab | `catalog/tools.toml`, `docs/offensive/`, `python/`, `rust/t2offsec-inventory/` | The T2 kernel, tiny-dfr, t2bce, audio DSP, Touch ID |

The reference machine is still the MacBookPro16,1 on Fedora 44 with the sharpenedblade T2 kernel. Daily boot, Wi-Fi, audio, the Touch Bar, and the Secure Enclave stay with the projects named in the gap analysis. t2linux, the sharpenedblade COPR, t2bce, tiny-dfr, t2-apple-audio-dsp, and t2-touchid are not vendored here.

Phase 0 of the hardware roadmap is unchanged: no driver code. A future kernel module still needs a gap entry that names this repo as the home, and it would be GPL-2.0-or-later. This track is Apache-2.0 documentation and userspace.

## What you can do with it

- See Kali's tool menus and the closest Fedora path for the notable tools in each.
- Ask a Python command which groups exist, and whether a tool is on the machine.
- Ask a Rust command for the same inventory, including a version string when that is a safe local flag.

Both commands are read-only. They do not install packages, edit `/etc`, load modules, or open a scan.

## What you will not find

- Exploit writeups, payloads, or attack steps.
- Firmware, DSP coefficients, or a Secure Enclave bypass.
- An autonomous attack agent.

Start with [install](install.md) if you are about to add packages on Fedora, [lab](lab.md) if you want a local range, or [the catalog](catalog.md) if you only want the map.
