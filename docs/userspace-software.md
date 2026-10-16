# Userspace software

Recorded 2026-10-16. Reference machine: MacBookPro16,1 on Fedora.

Approved desktop software for that machine. Install with Fedora `dnf`, or as a Flatpak from Flathub. Burp Suite Community comes from PortSwigger. OpenSnitch comes from the Fedora repositories or from GitHub.

None of these touch Touch ID, the T2 services, or the display mux. firewalld and SELinux are already on. Keep both on. The Touch ID setup depends on SELinux staying enforcing. Nothing was installed to record this list, and `/etc` was not modified.

Claim label: approved list. The note on each row is the reason it was approved. The 2026-10-06 scan in `docs/hardware-scan.md` did not measure these packages.

Home for each package is Fedora packaging, Flathub, GitHub, or PortSwigger, as the row says. This repo records the list. The named upstream ships the package.

## Constraints

- Use Fedora `dnf` or Flathub. OpenSnitch may also come from GitHub. Burp Suite Community comes from PortSwigger.
- Leave Touch ID, the T2 services, and the display mux out of these installs.
- Keep firewalld and SELinux on. SELinux stays enforcing, because the Touch ID setup depends on it. Touch ID enrollment on a macOS-free disk is documented as blocked in `docs/gap-analysis.md`.
- Use the testing tools only on networks and systems the person running them is authorized to test.
- On this Mac, do not install a third-party kernel module or disable SELinux to make a security tool run. If a tool asks for that, check with the team first.

## Email clients

| Application | Install | Note |
| --- | --- | --- |
| Thunderbird | `sudo dnf install thunderbird` | Best all-round choice. OpenPGP is built in. It works with Gmail, Microsoft 365, and IMAP, and it handles calendars. |
| Evolution | `sudo dnf install evolution evolution-ews` | GNOME-native option. The `evolution-ews` add-on connects to Exchange and Microsoft 365. |
| Geary | `sudo dnf install geary` | Simple and lightweight, for basic IMAP mail. |
| Betterbird | Flathub | Thunderbird with extra fixes and features. |
| Proton Mail Bridge | Flathub | Only needed when using Proton Mail with Thunderbird or Evolution. |

## Cybersecurity tools for this Mac

| Application | Install | Note |
| --- | --- | --- |
| firewalld and SELinux | Already on. Leave both on. | The Touch ID setup depends on SELinux staying enforcing. |
| KeePassXC | `sudo dnf install keepassxc` | Password manager. The vault stays on the Mac. |
| Bitwarden | Flathub | Alternative when the vault should sync to the cloud. |
| Lynis | `sudo dnf install lynis` | Security audit via `sudo lynis audit system`. |
| AIDE | `sudo dnf install aide` | Alerts when system files change. |
| ClamAV | `sudo dnf install clamav clamav-update` | On-demand malware file scans. |
| OpenSnitch | Fedora repositories or GitHub | Asks before any application makes an outbound connection. |
| GnuPG / Kleopatra | `sudo dnf install kleopatra` | Encryption and signing keys. |
| YubiKey Manager | `sudo dnf install yubikey-manager` | Sets up hardware security keys, if used. |

## Testing and analysis tools

Only on networks and systems the person running them is authorized to test.

| Application | Install | Note |
| --- | --- | --- |
| Wireshark | `sudo dnf install wireshark` | Packet capture. |
| Nmap | `sudo dnf install nmap` | Network scanning. |
| OWASP ZAP | Flathub | Web application testing. |
| Burp Suite Community | PortSwigger | Web application testing. |

## Caution

On this Mac, do not install a third-party kernel module or disable SELinux just to make a security tool run. If a tool asks for that, check with the team first.
