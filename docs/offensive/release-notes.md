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

The spec expects a GitHub archive of tag `v0.1.0`, which unpacks as `t2refit-v0.1.0`. That tag is a later release step.

## What 0.1.0 contains

- The `t2offsec` command, which lists groups and checks whether a tool is installed.
- The catalog at `/usr/share/raccoonix-t2/catalog/tools.toml` after the RPM is installed.
- This note, the user manual, and `NOTICE`.

The user manual covers how to install the package, how the groups are organized, and one entry per catalog tool: what it is, the Fedora package name, the command that prints its own help, the upstream documentation, and the license.

## What 0.1.0 leaves in the source tree

`t2offsec-inventory` is the Rust inventory. Build it from the repository with cargo. The noarch RPM does not compile it.

Lab tools named by the catalog are separate Fedora, COPR, or upstream packages. Install those on a machine you administer, using `docs/offensive/install.md`.

## License

Documentation, the catalog, and the checkers are Apache-2.0. Third-party tools are not vendored. Their licenses are in the manual. Redistributors keep `LICENSE-Apache-2.0` and `NOTICE` with the package.
