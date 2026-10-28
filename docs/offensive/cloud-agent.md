# Smoke-test a Cursor cloud-agent environment

A smoke test checks that a fresh environment can read this repo and run the two checkers. It does not install the offensive tools. `absent` is a successful result.

The environment does not have to be the MacBookPro16,1, and it does not have to be Fedora. `rpm` is used when it exists and skipped when it does not.

## From the repo root

```bash
PYTHONPATH=python/src python3 -m t2offsec groups
PYTHONPATH=python/src python3 -m t2offsec check --tool nmap
cargo run --manifest-path rust/t2offsec-inventory/Cargo.toml -- --tool nmap
```

Expect exit code 0 from each command. The first prints group ids. The other two print `nmap` as `present` or `absent`. Both are fine.

Then run the unit tests:

```bash
PYTHONPATH=python/src python3 -m unittest discover -s python/tests -v
cargo test --manifest-path rust/t2offsec-inventory/Cargo.toml
```

The first `cargo` invocation downloads the crate index and builds the inventory tool. That network use is the compiler. The inventory tool itself does not open a connection.

## What failure means

- `python3` or `cargo` missing: the environment image is incomplete. That is not a catalog bug.
- A unit test fails: the catalog or a checker regressed. Fix that before using the environment for other work.
- `check` exits 0 and lists tools as absent: the smoke test passed. Install groups later, on a Fedora machine, using [install.md](install.md). Do not `dnf install` or `apt install` the tool list as part of this test.

## What the agent must not do

Do not install packages, edit `/etc`, change boot configuration, rebuild the initramfs, load or unload modules, or download firmware. The hardware scanner (`./scripts/scan-t2-host.sh`) is a separate read-only command and is optional here. It only reports useful device facts on a T2 Mac.
