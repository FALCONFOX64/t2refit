# Install tool groups on Fedora

The package that carries the catalog itself is `raccoonix-t2`. Its install steps, and one entry for every tool, are in the [user manual](manual.md). This page is the next step: adding a group of lab tools on a Fedora machine you administer.

Package names below come from `catalog/tools.toml`. Fedora 44 repository membership was read from mdapi on 2026-10-16 (observed via that public index, not by installing on the MacBookPro16,1). Re-check a name before you rely on it months later.

Do this on a Fedora machine you administer. The commands change that machine. They are not part of a contribution, and they are not part of the cloud-agent smoke test.

## See a group before you install it

From the repo root:

```bash
PYTHONPATH=python/src python3 -m t2offsec groups
PYTHONPATH=python/src python3 -m t2offsec list --group recon
PYTHONPATH=python/src python3 -m t2offsec packages --group recon
```

`packages` prints official Fedora package names for that group's own tools, one per line. It skips COPR-only and upstream-only tools.

To install that official set yourself:

```bash
sudo dnf install $(PYTHONPATH=python/src python3 -m t2offsec packages --group recon)
```

Repeat with another group id (`web`, `lab`, `forensics`, and so on). Installing `lab` is the usual first set on a fresh Fedora host: Podman, libvirt, and QEMU.

## COPR and upstream

Tools with no Fedora 44 package are listed by:

```bash
PYTHONPATH=python/src python3 -m t2offsec gaps
```

A COPR line means an unofficial repository had a Fedora 44 build on the check date. Enabling a COPR writes repo files under `/etc`. Example, for Ghidra only:

```bash
sudo dnf copr enable davidgf/ghidra
sudo dnf install ghidra
```

`r3pek/cybersec` and `rebus/infosec` are broader. Enabling either one adds every package in that project, not a single tool. `rebus/infosec` describes its contents as not ready for official Fedora. Read the catalog row before you enable one.

An `upstream` line means there is no Fedora 44 package and no COPR row in the catalog. Follow that project's own install notes. Do not pipe a remote script into a shell from this repo.

## Afterward

Check what landed. This still does not run the tools:

```bash
PYTHONPATH=python/src python3 -m t2offsec check --group recon
cargo run --manifest-path rust/t2offsec-inventory/Cargo.toml -- --group recon
```

`present` means a binary is on `PATH` (including `/usr/sbin`), the rpm is installed, or the Python distribution is installed. `absent` is a normal result for a group you have not installed.

The version column from the Rust command is a local `--version` style flag. Some tools are inventoried by package only, because running them would touch a radio, a proxy, or a framework console.
