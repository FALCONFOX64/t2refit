# t2refit

Open-source workbench for refitting older Intel Macs that have an Apple T2 chip (about 2018–2020) so Linux can be the daily driver.

Public repo: https://github.com/FALCONFOX64/t2refit

The reference machine is a MacBookPro16,1 running Fedora 44 with the sharpenedblade T2 kernel. This repository records what that machine actually does, which existing project should own each gap, and a read-only scanner you can run again later. It does not replace [t2linux](https://t2linux.org/). t2linux, tiny-dfr, t2bce, and t2-touchid already cover most of the hardware. This repo fills the gaps they do not, and it does not vendor their trees.

Phase 0 of the hardware work is a sanitized scan, a ranked gap list, a roadmap, and `scripts/scan-t2-host.sh`. It contains no driver code. A second track, documented in [docs/index.md](docs/index.md), is a Fedora catalog of lab tools and two read-only checkers. That track does not change the hardware tree.

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

Read [docs/index.md](docs/index.md). Hardware notes, the approved desktop list, and the lab catalog are separate.

If you want the lab, start with the [raccoonix T2 user manual](docs/offensive/manual.md). The Fedora package name is `raccoonix-t2`. The map of tools is the [catalog](docs/offensive/catalog.md), and [overview](docs/offensive/overview.md) says how that track sits beside the hardware tree. From a checkout, list groups without installing anything:

```bash
PYTHONPATH=python/src python3 -m t2offsec groups
```

Install steps for Fedora are in [docs/offensive/install.md](docs/offensive/install.md). A local lab is [docs/offensive/lab.md](docs/offensive/lab.md). A cloud-agent smoke test is [docs/offensive/cloud-agent.md](docs/offensive/cloud-agent.md).

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
packaging/raccoonix-t2.spec     RPM metadata for the public package
catalog/tools.toml                Tool groups, Fedora path, license
python/                           Read-only `t2offsec` CLI
rust/t2offsec-inventory/          Read-only install and version inventory
scripts/scan-t2-host.sh           Read-only T2 hardware scanner
```
