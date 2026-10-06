#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# Read-only inventory of a Linux host for Intel T2 Mac compatibility work.
# No sudo, no network, no firmware copies, no serials, MAC addresses, or
# Wi-Fi credentials on stdout.

set -u

redact() {
  sed -E \
    -e 's/([0-9A-Fa-f]{2}:){5}[0-9A-Fa-f]{2}/[mac]/g' \
    -e 's/\b[0-9A-Fa-f]{8}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{12}\b/[uuid]/g' \
    -e 's/\b[Ff][Ee]80:[0-9A-Fa-f:%.]+/[ipv6]/g' \
    -e 's#(root=UUID=)[^[:space:]]+#\1[uuid]#g' \
    -e 's/(psk|password|passwd|secret)=[^[:space:]]+/\1=[redacted]/Ig'
}

section() {
  printf '\n[%s]\n' "$1"
}

read_trim() {
  local path="$1"
  if [[ -r "$path" ]]; then
    tr -d '\n' <"$path"
  else
    printf 'unread'
  fi
}

section identity
printf 'product_name=%s\n' "$(read_trim /sys/class/dmi/id/product_name)"
printf 'product_version=%s\n' "$(read_trim /sys/class/dmi/id/product_version)"
printf 'sys_vendor=%s\n' "$(read_trim /sys/class/dmi/id/sys_vendor)"
printf 'board_name=%s\n' "$(read_trim /sys/class/dmi/id/board_name)"
printf 'kernel=%s\n' "$(uname -r)"
printf 'cmdline=%s\n' "$(redact </proc/cmdline)"

section packages
pkgs=(t2linux-config t2linux-audio t2fanrd tiny-dfr brcmfmac-firmware t2-touchid)
if command -v rpm >/dev/null 2>&1; then
  rpm -q "${pkgs[@]}" 2>&1 | redact || true
else
  printf 'rpm=missing\n'
fi

section services
units=(t2fanrd.service tiny-dfr.service kait2en-t2-touchid.service)
for unit in "${units[@]}"; do
  active="$(systemctl is-active "$unit" 2>/dev/null || true)"
  enabled="$(systemctl is-enabled "$unit" 2>/dev/null || true)"
  printf '%s active=%s enabled=%s\n' "$unit" "${active:-unknown}" "${enabled:-unknown}"
done

section modules
if [[ -r /proc/modules ]]; then
  awk '$1 ~ /^(t2bce_|brcmfmac|appletbdrm|applesmc|apple_gmux|hid_apple|amdgpu|i915|thunderbolt)/ {print $1}' /proc/modules | sort
fi

section pci-ids
if command -v lspci >/dev/null 2>&1; then
  lspci -nn | grep -E 'Apple|BCM4364|Navi|UHD Graphics|Thunderbolt' | redact || true
fi

section usb-apple
if command -v lsusb >/dev/null 2>&1; then
  lsusb | grep -E 'Apple|Linux Foundation' | redact || true
fi

section wifi
shopt -s nullglob
for iface_path in /sys/class/net/wl*; do
  iface="$(basename "$iface_path")"
  oper="$(read_trim "$iface_path/operstate")"
  printf 'iface=%s operstate=%s\n' "$iface" "$oper"
done
shopt -u nullglob
if command -v journalctl >/dev/null 2>&1; then
  journalctl -b -k --no-pager -o cat 2>/dev/null \
    | grep -E 'brcmfmac: brcmf_c_preinit_dcmds: Firmware:' \
    | redact \
    | tail -n 1 || true
fi
hcd=missing
for candidate in /usr/lib/firmware/brcm/BCM.hcd /lib/firmware/brcm/BCM.hcd; do
  if [[ -e "$candidate" ]]; then
    hcd=present
  fi
done
printf 'bcm_hcd=%s\n' "$hcd"

section bluetooth
if command -v bluetoothctl >/dev/null 2>&1; then
  bluetoothctl show 2>/dev/null \
    | awk -F: '/Name:|Alias:|Powered:|Discovering:/ {gsub(/^[ \t]+/, "", $2); print $1 "=" $2}' \
    | redact || true
fi
if command -v journalctl >/dev/null 2>&1; then
  journalctl -b -k --no-pager -o cat 2>/dev/null \
    | grep -E 'Bluetooth: hci0: BCM4364|brcm/BCM.hcd|Failed to set baudrate' \
    | redact \
    | tail -n 6 || true
fi

section audio
card_id="$(read_trim /sys/class/sound/card0/id)"
printf 'card0_id=%s\n' "$card_id"
printf 'card0_id_length=%s\n' "${#card_id}"
if [[ -r /proc/asound/cards ]]; then
  redact </proc/asound/cards
fi
if command -v pactl >/dev/null 2>&1; then
  sink="$(pactl get-default-sink 2>/dev/null || true)"
  source="$(pactl get-default-source 2>/dev/null || true)"
  case "$sink" in
    *t2*|*alsa*) printf 'default_sink=%s\n' "$sink" ;;
    "") printf 'default_sink=missing\n' ;;
    *) printf 'default_sink=present-redacted\n' ;;
  esac
  case "$source" in
    *t2*|*alsa*) printf 'default_source=%s\n' "$source" ;;
    "") printf 'default_source=missing\n' ;;
    *) printf 'default_source=present-redacted\n' ;;
  esac
fi
for overlay in \
  /etc/udev/rules.d/99-t2-audio-rename.rules \
  /etc/wireplumber/wireplumber.conf.d/99-t2-audio.conf \
  /etc/t2linux-audio/16_1
do
  if [[ -e "$overlay" ]]; then
    printf 'overlay=present %s\n' "$overlay"
  else
    printf 'overlay=absent %s\n' "$overlay"
  fi
done

section backlight
if [[ -d /sys/class/backlight ]]; then
  printf 'backlight=%s\n' "$(ls -1 /sys/class/backlight 2>/dev/null | tr '\n' ' ')"
fi
if [[ -d /sys/class/leds ]]; then
  ls -1 /sys/class/leds 2>/dev/null | grep -E 'kbd_backlight|apple' | redact || true
fi

section display
for status_path in /sys/class/drm/card*-*/status; do
  [[ -e "$status_path" ]] || continue
  name="$(basename "$(dirname "$status_path")")"
  state="$(read_trim "$status_path")"
  case "$name" in
    *eDP*|*USB*|*LVDS*) printf '%s status=%s\n' "$name" "$state" ;;
  esac
done
for card in /sys/class/drm/card[0-9]*; do
  [[ -e "$card/device/driver" ]] || continue
  base="$(basename "$card")"
  [[ "$base" == card[0-9]*-* ]] && continue
  driver="$(basename "$(readlink -f "$card/device/driver")")"
  printf '%s driver=%s\n' "$base" "$driver"
done

section touchbar
if [[ -r /usr/share/tiny-dfr/config.toml ]]; then
  default="$(awk -F= '/^MediaLayerDefault/ {gsub(/[[:space:]]/, "", $2); print $2; exit}' /usr/share/tiny-dfr/config.toml)"
  printf 'packaged_media_layer_default=%s\n' "${default:-unread}"
fi
if [[ -f /etc/tiny-dfr/config.toml ]]; then
  printf 'etc_tiny_dfr_config=present\n'
else
  printf 'etc_tiny_dfr_config=absent\n'
fi

section bridge
if [[ -e /sys/class/net/t2_ncm/operstate ]]; then
  printf 't2_ncm=%s\n' "$(read_trim /sys/class/net/t2_ncm/operstate)"
else
  printf 't2_ncm=absent\n'
fi

section camera
shopt -s nullglob
videos=(/dev/video*)
shopt -u nullglob
if ((${#videos[@]} == 0)); then
  printf 'video_devices=none\n'
else
  for node in "${videos[@]}"; do
    name_path="/sys/class/video4linux/$(basename "$node")/name"
    printf '%s name=%s\n' "$(basename "$node")" "$(read_trim "$name_path")"
  done
fi

section input
for name_path in /sys/class/input/input*/name; do
  [[ -r "$name_path" ]] || continue
  name="$(cat "$name_path")"
  case "$name" in
    *Apple*|*Touch*|*Headset*|*Lid*)
      printf '%s %s\n' "$(basename "$(dirname "$name_path")")" "$name"
      ;;
  esac
done

section flags
printf 'flag tiny-dfr-inactive=%s\n' "$([[ "$(systemctl is-active tiny-dfr.service 2>/dev/null || true)" == active ]] && printf no || printf yes)"
printf 'flag bcm-hcd-missing=%s\n' "$([[ "$hcd" == present ]] && printf no || printf yes)"
if [[ "$card_id" == unread || ${#card_id} -le 15 ]]; then
  printf 'flag alsa-card-id-long=%s\n' no
else
  printf 'flag alsa-card-id-long=%s\n' yes
fi

exit 0
