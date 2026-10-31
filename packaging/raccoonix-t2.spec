# raccoonix-t2: Fedora package for the t2refit lab catalog.
# This spec describes the package. It does not publish a GitHub release.
#
# Tag v0.1.0 downloads as t2refit-0.1.0.tar.gz and unpacks as t2refit-0.1.0.
# GitHub strips the leading v from the archive name and the top directory.

Name:           raccoonix-t2
Version:        0.1.0
Release:        1%{?dist}
Summary:        raccoonix T2 lab catalog for Fedora on T2 Intel Macs
License:        Apache-2.0
URL:            https://github.com/FALCONFOX64/t2refit
Source0:        %{url}/archive/refs/tags/v%{version}.tar.gz#/t2refit-%{version}.tar.gz

# The Rust inventory is an arch-specific binary. Skip the separate debuginfo
# package; the release profile strips symbols.
%global debug_package %{nil}
%global raccoonix_cargo_home %{_builddir}/raccoonix-cargo-home
%global raccoonix_cargo_target %{_builddir}/raccoonix-cargo-target

BuildRequires:  python3-devel
BuildRequires:  python3-pip
BuildRequires:  python3-setuptools >= 77
BuildRequires:  cargo
BuildRequires:  rust
BuildRequires:  gcc
BuildRequires:  binutils

Requires:       python3

%description
raccoonix T2 is the Fedora package of the t2refit offensive-lab catalog
for Intel Macs with an Apple T2 chip. The git repository is t2refit. The
RPM name is raccoonix-t2. The commands are t2offsec and t2offsec-inventory.

The RPM installs those two read-only checkers, the catalog, the user
manual, the Apache-2.0 license, and NOTICE. It does not include
third-party tool source, wordlists, firmware, or proprietary blobs.
Those tools keep the licenses in the manual.

%generate_buildrequires
%pyproject_buildrequires -d python

%prep
%autosetup -n t2refit-%{version} -p1

%build
%pyproject_wheel --directory python
export CARGO_HOME=%{raccoonix_cargo_home}
export CARGO_TARGET_DIR=%{raccoonix_cargo_target}
export CARGO_PROFILE_RELEASE_STRIP=symbols
# The rpm environment passes -Cstrip=none so a debuginfo package can be
# built. This package does not ship one, and the last -Cstrip value wins.
export RUSTFLAGS="${RUSTFLAGS:+${RUSTFLAGS} }-Cstrip=symbols"
cargo build --locked --release --manifest-path rust/t2offsec-inventory/Cargo.toml

%install
%pyproject_install
%pyproject_save_files t2offsec
install -D -p -m 0755 %{raccoonix_cargo_target}/release/t2offsec-inventory \
    %{buildroot}%{_bindir}/t2offsec-inventory
strip --strip-unneeded %{buildroot}%{_bindir}/t2offsec-inventory
# Installed catalog path: /usr/share/raccoonix-t2/catalog/tools.toml
# The description omits that path. rpmlint spell-checks it and rejects usr and toml.
install -D -p -m 0644 catalog/tools.toml \
    %{buildroot}%{_datadir}/%{name}/catalog/tools.toml

%check
%pyproject_check_import t2offsec
PYTHONPATH=python/src python3 -m unittest discover -s python/tests -v
export CARGO_HOME=%{raccoonix_cargo_home}
export CARGO_TARGET_DIR=%{raccoonix_cargo_target}
cargo test --locked --offline --manifest-path rust/t2offsec-inventory/Cargo.toml

%files -f %{pyproject_files}
%license LICENSE-Apache-2.0 NOTICE
%doc README.md
%doc SECURITY.md
%doc docs/offensive/manual.md
%doc docs/offensive/release-notes.md
%{_bindir}/t2offsec
%{_bindir}/t2offsec-inventory
%dir %{_datadir}/%{name}
%dir %{_datadir}/%{name}/catalog
%{_datadir}/%{name}/catalog/tools.toml

%changelog
* Sat Oct 31 2026 t2refit contributors <217426068+FALCONFOX64@users.noreply.github.com> - 0.1.0-1
- BuildRequire python3-pip for the pyproject generator and strip the inventory binary.
- Package the t2offsec entry-point script.
- Drop the catalog path from the description so rpmlint can spell-check it.
- Record that same path in a comment next to the install.
* Wed Oct 28 2026 t2refit contributors <217426068+FALCONFOX64@users.noreply.github.com> - 0.1.0-1
- Package t2offsec, t2offsec-inventory, the catalog, the manual, and NOTICE.
- Build the Python project with the pyproject macros. No release is published.
