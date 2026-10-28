# t2refit

Open-source workbench for refitting older Intel Macs that have an Apple T2 chip (about 2018–2020) so Linux can be the daily driver.

Public repo: https://github.com/FALCONFOX64/t2refit

The reference machine is a MacBookPro16,1 running Fedora 44 with the sharpenedblade T2 kernel. This repository records what that machine actually does, which existing project should own each gap, and a read-only scanner you can run again later. It does not replace [t2linux](https://t2linux.org/). t2linux, tiny-dfr, t2bce, and t2-touchid already cover most of the hardware. This repo fills the gaps they do not, and it does not vendor their trees.

Phase 0 of the hardware work is a sanitized scan, a ranked gap list, a roadmap, and `scripts/scan-t2-host.sh`. It contains no driver code. A second track, [raccoonix-t2](#raccoonix-t2), is the Fedora offensive-lab notebook: a userspace catalog and two read-only checkers. The hardware tree stays as it is.

## What this project will do

- Re-scan a T2 Mac and keep a sanitized compatibility record.
- Name the right upstream for each gap before writing new code.
- Hold small, upstreamable fixes and userspace tools when no existing project owns the problem.

## What this project will not do

- Support Apple Silicon.
- Fork the T2 kernel, tiny-dfr, or t2-touchid.
- Ship Apple firmware blobs, macOS recovery images, or speaker DSP coefficient files.
- Bypass the T2 Secure Enclave, extract biometric templates, or circumvent firmware signature checks.
- Enroll Touch ID on a disk that has no macOS volume. On this machine that path is blocked by the SEP.

## Run the scanner

```bash
./scripts/scan-t2-host.sh
```

The script is read-only. It does not use sudo and it does not use the network. It omits serial numbers, UUIDs, MAC addresses, SSIDs, and passwords.

## Where to start

Read [docs/index.md](docs/index.md). The hardware notes, the approved desktop list, and raccoonix-t2 are separate.

- Hardware: [scan](docs/hardware-scan.md), [gaps](docs/gap-analysis.md), [roadmap](docs/roadmap.md), and the scanner command above.
- Lab notebook: [raccoonix-t2](#raccoonix-t2), then the [manual](docs/offensive/manual.md) and the [release note](docs/offensive/release-notes.md).

## raccoonix-t2

**A Fedora Intel T2 MacBook, set up as an authorized security notebook.**

raccoonix-t2 is an open-source Apache-2.0 lab track inside t2refit. It names the tools, the Fedora path, and the license, and it gives you two read-only checkers. The hardware work above stays the purpose of this repo. raccoonix-t2 sits beside it.

### What it is

- A userspace catalog for an authorized lab on Fedora.
- The package name `raccoonix-t2`. The [release note](docs/offensive/release-notes.md) and `packaging/raccoonix-t2.spec` describe version 0.1.0. No GitHub release and no public RPM have been published.
- One plain entry per tool in the [user manual](docs/offensive/manual.md).

### What it is not

- A replacement for the T2 hardware scan, the gap list, or the roadmap.
- Driver code, firmware, or a Secure Enclave project.
- An exploit guide. Each entry stops at the command that prints that tool's own help.

### Machines

Intel Macs with an Apple T2 chip, roughly 2018–2020, running Fedora. The reference machine is a MacBookPro16,1 on Fedora 44. Apple Silicon is out of scope.

### Quick start

1. Read the [user manual](docs/offensive/manual.md) and the [release note](docs/offensive/release-notes.md).
2. From a checkout, list the groups. This only reads the catalog:

```bash
PYTHONPATH=python/src python3 -m t2offsec groups
```

3. Package instructions for `raccoonix-t2` itself are in the manual and in `packaging/raccoonix-t2.spec`. Adding a group of lab tools on a Fedora machine you administer is [install.md](docs/offensive/install.md). A local lab is [lab.md](docs/offensive/lab.md).

### Tool groups

Groups follow Kali's menus where a Fedora path exists, then add lab, bench, and local-assistant groups: recon, web, vulnerability checks, wireless, Bluetooth, passwords, frameworks, tunnels, capture, databases, forensics, reverse engineering, fuzzing, crypto, radio, RFID, VoIP, detection, reporting, VMs, bench hardware, and local models and report helpers. The Windows-resources menu is a summary with no install rows. Kali's top 10 is a cross-reference. The full list is in the [manual](docs/offensive/manual.md) and the [catalog](docs/offensive/catalog.md).

### Python and Rust helpers

Both commands are read-only. They report whether a selected tool is installed.

| Helper | Command |
| --- | --- |
| Python `t2offsec` | `PYTHONPATH=python/src python3 -m t2offsec check --tool nmap` |
| Rust `t2offsec-inventory` | `cargo run --manifest-path rust/t2offsec-inventory/Cargo.toml -- --tool nmap` |

The Rust helper prints a version string only when the catalog lists a local version flag for that tool.

### Authorized use

Use raccoonix-t2 on machines, networks, and files you are allowed to test.

### T2 boundaries

- The Secure Enclave stays closed. This repo does not extract biometric templates or bypass firmware signature checks.
- Touch ID enrollment on a disk with no macOS volume stays blocked on the reference machine.
- Firmware stays out of this repo. The built-in Bluetooth controller is powered on and the patch file `BCM.hcd` is missing (observed, [hardware scan](docs/hardware-scan.md)). Use a separate adapter for Bluetooth lab work.
- The built-in BCM4364 Wi-Fi interface is a station device (observed). Monitor mode on this host is untested (open). Use a separate adapter for wireless lab work.
- Daily boot, audio, the Touch Bar, and the T2 kernel stay with the projects named in the [gap analysis](docs/gap-analysis.md).

### License and contributors

raccoonix-t2 is open source under Apache-2.0. Documentation, the catalog, and the checkers use that license (`LICENSE-Apache-2.0`). Third-party tools keep the licenses recorded in the manual and in `NOTICE`. Future kernel code, if this repo ever holds any, would be GPL-2.0-or-later. This track has no kernel code.

Contributors of every experience level are welcome. A catalog correction, a clearer sentence, and a careful test note all help. Read [CONTRIBUTING.md](CONTRIBUTING.md) before you open a pull request.

- Manual: [docs/offensive/manual.md](docs/offensive/manual.md)
- Release note: [docs/offensive/release-notes.md](docs/offensive/release-notes.md)

## License

| Path | License |
| --- | --- |
| Documentation, `scripts/`, `catalog/`, `python/`, `rust/`, `packaging/`, and future userspace | Apache-2.0 (`LICENSE-Apache-2.0`, notices in `NOTICE`) |
| Future kernel modules, when any exist | GPL-2.0-or-later (`LICENSE-GPL-2.0-or-later`) |

Kernel code uses GPL-2.0-or-later so it can be proposed upstream. There is no kernel code in this commit.

## Cursor

Open `t2refit.code-workspace` in Cursor. Project rules live in `.cursor/rules/` and travel with the repo:

- `project.mdc` applies to every chat.
- `scanner.mdc` applies while editing `scripts/**/*.sh`.
- `docs.mdc` applies while editing `docs/**/*.md`.

## Layout

```text
docs/index.md                     Start here
docs/hardware-scan.md             What this MacBookPro16,1 did on 2026-10-06
docs/gap-analysis.md              Ranked gaps and who should own them
docs/roadmap.md                   Hardware phases after the scaffold
docs/userspace-software.md        Approved userspace packages for this Mac
docs/offensive/manual.md          raccoonix T2 user manual (`raccoonix-t2`)
docs/offensive/                   Fedora lab catalog, install, and smoke test
packaging/raccoonix-t2.spec      RPM metadata for a later public package
catalog/tools.toml                Tool groups, Fedora path, license
python/                           Read-only `t2offsec` CLI
rust/t2offsec-inventory/          Read-only install and version inventory
scripts/scan-t2-host.sh           Read-only T2 hardware scanner
```
