# Security

raccoonix-t2 is the userspace lab track in the t2refit repository. The RPM name is `raccoonix-t2`. The commands are `t2offsec` and `t2offsec-inventory`.

## Reporting a vulnerability

Report a vulnerability in this repository in private. Use GitHub Security Advisories:

https://github.com/FALCONFOX64/t2refit/security/advisories/new

Include the version or commit, the file or command involved, and what an affected user can do. Give the project time to prepare a fix before any public write-up.

A public issue is fine for a catalog typo, a doc mistake, or a build failure. Keep unfixed vulnerability details out of public issues, pull requests, and chat.

## Authorized use

Use the catalog and the checkers on machines, networks, and files you are allowed to test. The manual names each tool and the command that prints that tool's own help. It stops there.

## Boundaries

- The T2 Secure Enclave stays closed. Reports that ask for biometric templates, a firmware-signature bypass, or Touch ID enrollment on a macOS-free disk are out of scope.
- This repository does not take firmware, DSP coefficient files, wordlists, or proprietary blobs.
- Exploit writeups, payloads, attack procedures, malware, and credential-theft steps are not accepted as issues or as patches.
- The checkers are read-only. A report that a checker ran a catalog tool, followed a path outside its binary name, or loaded a catalog from a parent directory is in scope for a private report.

## What a fix looks like

A fix lands in this repository under Apache-2.0 for the documentation and the userspace checkers. Third-party tools keep their own licenses. A future kernel patch, if one is ever in scope, would be GPL-2.0-or-later and would need a gap entry that names this repo as the home.
