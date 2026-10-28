# raccoonix-t2: Fedora package for the t2refit lab catalog.
# The release tag is not published by the commit that adds this spec.

Name:           raccoonix-t2
Version:        0.1.0
Release:        1%{?dist}
Summary:        raccoonix T2 lab catalog for Fedora on T2 Intel Macs
License:        Apache-2.0
URL:            https://github.com/FALCONFOX64/t2refit
Source0:        %{url}/archive/refs/tags/v%{version}.tar.gz#/%{name}-%{version}.tar.gz
BuildArch:      noarch

BuildRequires:  python3-devel
BuildRequires:  python3-rpm-macros
BuildRequires:  python3-setuptools
Requires:       python3

%description
raccoonix T2 is the Fedora package of the t2refit offensive-lab catalog
for Intel Macs with an Apple T2 chip. It installs the read-only t2offsec
command, the catalog at /usr/share/raccoonix-t2/catalog/tools.toml, and
the user manual.

The RPM does not include third-party tool source, wordlists, firmware, or
proprietary blobs. Those tools keep the licenses recorded in the manual.
The Rust inventory stays in the source tree and is built with cargo. It
is not part of this noarch package.

%prep
%autosetup -n t2refit-v%{version} -p1

%build
pushd python
%py3_build
popd

%install
pushd python
%py3_install
popd
install -D -p -m 0644 catalog/tools.toml \
    %{buildroot}%{_datadir}/%{name}/catalog/tools.toml

%files
%license LICENSE-Apache-2.0
%doc NOTICE
%doc docs/offensive/manual.md
%doc docs/offensive/release-notes.md
%{_bindir}/t2offsec
%{python3_sitelib}/t2offsec/
%{python3_sitelib}/t2offsec-%{version}.dist-info/
%dir %{_datadir}/%{name}
%dir %{_datadir}/%{name}/catalog
%{_datadir}/%{name}/catalog/tools.toml

%changelog
* Wed Oct 28 2026 t2refit contributors <217426068+FALCONFOX64@users.noreply.github.com> - 0.1.0-1
- First public RPM metadata for raccoonix-t2 0.1.0.
- Ship the catalog, the user manual, NOTICE, and the t2offsec checker.
