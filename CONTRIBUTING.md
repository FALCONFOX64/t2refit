# Contributing

Changes need evidence. A device id plus command output, or a quote from `scripts/scan-t2-host.sh`, is the minimum. Say whether each claim is observed, inferred, or still open.

## In bounds

- Scanner checks that stay read-only.
- Documentation of a gap, with the upstream project that should own it.
- A small patch written so it can be sent upstream, when this repo is the right place to draft it.
- The offensive-lab catalog (`catalog/tools.toml`), its docs under `docs/offensive/`, and the read-only Python and Rust checkers. New code there is Python or Rust.

## Out of bounds

- Unrelated refactors.
- Apple firmware blobs, `*.hcd`, `*.clm_blob`, `*.txcap_blob`, and proprietary DSP coefficient files.
- Serial numbers, board UUIDs, disk UUIDs, MAC addresses, Wi-Fi SSIDs, and passwords.
- Secure Enclave bypasses, biometric-template extraction, and firmware-signature bypasses.
- Vendored copies of t2linux, tiny-dfr, t2bce, or t2-touchid.
- Kernel modules until `docs/gap-analysis.md` names this repo as the home and the license header is GPL-2.0-or-later.
- Exploit writeups, payloads, attack procedures, malware, and credential-theft steps. The lab catalog names tools. It does not teach those steps.
- Vendored wordlists.

## Git

Author commits as `FALCONFOX64 <217426068+FALCONFOX64@users.noreply.github.com>`. Do not add a `Co-authored-by` trailer. Do not change the global git config. Do not push unless the maintainer asks.

## System changes

Do not install packages, edit `/etc`, change the kernel command line, rebuild the initramfs, or load and unload modules as part of a contribution unless the maintainer asked for that change on the machine under test.
