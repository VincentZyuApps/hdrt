#!/bin/sh
set -eu

export LC_ALL=C

detail="${1:-basic}"
device="${2:?device path required}"

case "$detail" in
  smart|full)
    smartctl -a "$device"
    ;;
  basic)
    smartctl -i -H "$device"
    ;;
  *)
    echo "unknown smart detail: $detail" >&2
    exit 2
    ;;
esac
