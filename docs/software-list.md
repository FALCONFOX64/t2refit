# Software list

Credit: hardware support for these Intel T2 Macs is [t2linux](https://t2linux.org/) ([GitHub](https://github.com/t2linux)). This list is written by falconfox ([FALCONFOX64](https://github.com/FALCONFOX64)).

Everything here installs with Fedora's `dnf` or as a Flatpak from Flathub. None of it touches Touch ID, the T2 services, or the display mux.

Keep firewalld and SELinux on. The Touch ID setup depends on SELinux staying enforcing. Do not install third-party kernel modules or disable SELinux to make a security tool run. If a tool asks for that, check with the team first.

## Email clients

- **Thunderbird** — `sudo dnf install thunderbird`. Best all-round choice: OpenPGP is built in, it works with Gmail, Microsoft 365, and IMAP, and it handles calendars too.
- **Evolution** — `sudo dnf install evolution evolution-ews`. GNOME-native option. The `-ews` add-on connects to Exchange and Microsoft 365.
- **Geary** — `sudo dnf install geary`. Simple and lightweight for basic IMAP mail.
- **Betterbird** — Flathub. Thunderbird with extra fixes and features.
- **Proton Mail Bridge** — Flathub. Only needed for Proton Mail with Thunderbird or Evolution.

## Cybersecurity tools for the Mac

- **firewalld** and **SELinux** — already part of the system. Keep both on.
- **KeePassXC** — `sudo dnf install keepassxc`. Password manager; the vault stays on this Mac. Bitwarden from Flathub works if you would rather sync to the cloud.
- **Lynis** — `sudo dnf install lynis`. Security audit: `sudo lynis audit system`.
- **AIDE** — `sudo dnf install aide`. Alerts when system files change.
- **ClamAV** — `sudo dnf install clamav clamav-update`. On-demand malware scan of files.
- **OpenSnitch** — Fedora repos or GitHub. Asks before any app makes an outbound connection.
- **GnuPG / Kleopatra** — `sudo dnf install kleopatra`. Encryption and signing keys.
- **YubiKey Manager** — `sudo dnf install yubikey-manager`. Hardware security keys, if you use them.

## Testing and analysis

Only on networks and systems you are authorized to test.

- **Wireshark** — `sudo dnf install wireshark`. Packet capture.
- **Nmap** — `sudo dnf install nmap`. Network scanning.
- **OWASP ZAP** — Flathub. Web app testing.
- **Burp Suite Community** — from PortSwigger. Web app testing.

## Install status

Read-only check. Queries: `rpm -q`, `dnf list --installed`, `flatpak list`, `command -v`, plus `systemctl is-active firewalld` and `getenforce`. `rpm -q` exited 21 because some named packages are absent; each package still returned a status. Flatpak 1.18.4 is present and `flatpak list` returned no applications. No other query failed.

- **Thunderbird** — dnf — not installed
- **Evolution** — dnf — not installed
- **evolution-ews** — dnf — not installed
- **Geary** — dnf — not installed
- **Betterbird** — Flathub — not installed
- **Proton Mail Bridge** — Flathub — not installed
- **firewalld** — dnf — installed, active
- **SELinux** — system — Enforcing
- **KeePassXC** — dnf — not installed
- **Bitwarden** — Flathub — not installed
- **Lynis** — dnf — not installed
- **AIDE** — dnf — not installed
- **ClamAV** (`clamav`) — dnf — not installed
- **clamav-update** — dnf — not installed
- **OpenSnitch** — Fedora repos or GitHub — not installed
- **GnuPG** (`gnupg2`) — dnf — installed
- **Kleopatra** — dnf — not installed
- **YubiKey Manager** (`yubikey-manager`) — dnf — not installed
- **Wireshark** — dnf — not installed
- **Nmap** — dnf — not installed
- **OWASP ZAP** — Flathub — not installed
- **Burp Suite Community** — PortSwigger — not installed
