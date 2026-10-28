# raccoonix-t2 0.1.0

raccoonix T2 is the public Fedora package of the t2refit lab catalog for Intel Macs with an Apple T2 chip. The RPM name is `raccoonix-t2`. This note describes that package. It does not publish a GitHub release.

## Package

| Field | Value |
| --- | --- |
| RPM name | `raccoonix-t2` |
| Display name | raccoonix T2 |
| Version | 0.1.0 |
| License | Apache-2.0 (`LICENSE-Apache-2.0`, with `NOTICE`) |
| Spec | `packaging/raccoonix-t2.spec` |
| Manual | `docs/offensive/manual.md` |
| Intended git tag | `v0.1.0` |

The git repository is t2refit. The RPM name is `raccoonix-t2`. The commands are `t2offsec` and `t2offsec-inventory`.

A GitHub archive of tag `v0.1.0` is named `t2refit-0.1.0.tar.gz` and unpacks as `t2refit-0.1.0`. The leading `v` is only on the tag. That tag is a later release step. No public RPM has been published.

## What 0.1.0 contains

- `/usr/bin/t2offsec`, which lists groups and checks whether a tool is installed.
- `/usr/bin/t2offsec-inventory`, the Rust inventory.
- The catalog at `/usr/share/raccoonix-t2/catalog/tools.toml` after the RPM is installed.
- This note, the user manual, `LICENSE-Apache-2.0`, and `NOTICE`.

The user manual covers the local RPM build, update, uninstall, troubleshooting, how the groups are organized, and one entry per catalog tool.

## What stays outside the RPM

Lab tools named by the catalog are separate Fedora, COPR, or upstream packages. Install those on a machine you administer, using `docs/offensive/install.md`.

Build the RPM locally with the steps in the manual. There is no dnf repository for `raccoonix-t2` yet.

## License

Documentation, the catalog, and the checkers are Apache-2.0. Third-party tools are not vendored. Their licenses are in the manual. Redistributors keep `LICENSE-Apache-2.0` and `NOTICE` with the package.
