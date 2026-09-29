#!/bin/sh

set -eu

repo="kossnocorp/yatera"

case "$(uname -s):$(uname -m)" in
Darwin:x86_64) target="x86_64-apple-darwin" ;;
Darwin:arm64 | Darwin:aarch64) target="aarch64-apple-darwin" ;;
Linux:x86_64 | Linux:amd64) target="x86_64-unknown-linux-gnu" ;;
*)
  printf 'Unsupported platform: %s %s\n' "$(uname -s)" "$(uname -m)" >&2
  exit 1
  ;;
esac

latest_url="$(curl -fsSL -o /dev/null -w '%{url_effective}' "https://github.com/$repo/releases/latest")"
tag="${latest_url##*/}"

case "$tag" in
v[0-9]*) ;;
*)
  printf 'Could not determine the latest yatera release.\n' >&2
  exit 1
  ;;
esac

install_dir="${TERA_INSTALL_DIR:-${XDG_BIN_HOME:-$HOME/.local/bin}}"
asset="tera-$tag-$target"
temp_file="$(mktemp "${TMPDIR:-/tmp}/tera.XXXXXX")"
trap 'rm -f "$temp_file"' EXIT HUP INT TERM

curl -fL --progress-bar \
  "https://github.com/$repo/releases/download/$tag/$asset" \
  -o "$temp_file"
chmod +x "$temp_file"
mkdir -p "$install_dir"
mv "$temp_file" "$install_dir/tera"
trap - EXIT HUP INT TERM

printf 'Installed yatera to %s/tera\n' "$install_dir"
case ":$PATH:" in
*":$install_dir:"*) ;;
*) printf 'Add %s to PATH to run tera.\n' "$install_dir" ;;
esac
