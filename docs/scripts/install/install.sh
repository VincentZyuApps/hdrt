#!/usr/bin/env bash
# hdrt installer: supports apt (deb), dnf (rpm), and Termux on x86_64/aarch64.
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/VincentZyuApps/hdrt/main/docs/scripts/install/install.sh | bash
# Install a specific version:
#   HDRT_VERSION=v0.1.5-alpha.8 bash -c "$(curl -fsSL https://raw.githubusercontent.com/VincentZyuApps/hdrt/main/docs/scripts/install/install.sh)"
set -e

REPO="VincentZyuApps/hdrt"
API_URL="https://api.github.com/repos/${REPO}/releases/latest"

IS_TERMUX=false
if [ -n "${PREFIX:-}" ] && [ -d "${PREFIX}/bin" ]; then
  IS_TERMUX=true
fi

ARCH=$(uname -m)
case "$ARCH" in
  x86_64|amd64) ARCH_NAME="x86_64" ;;
  aarch64|arm64) ARCH_NAME="aarch64" ;;
  *)
    echo "❌ Unsupported architecture: $ARCH"
    echo "   Only x86_64 and aarch64 are supported by this installer."
    echo "   📦 Manual downloads: https://github.com/${REPO}/releases"
    echo "   🛠️ Build from source: cargo install hdrt"
    exit 1
    ;;
esac

if $IS_TERMUX; then
  PKG_MGR="termux"
elif command -v apt-get >/dev/null 2>&1; then
  PKG_MGR="apt"
elif command -v dnf >/dev/null 2>&1; then
  PKG_MGR="dnf"
else
  echo "❌ Unsupported package manager."
  echo "   This installer supports apt, dnf, and Termux."
  echo "   📦 Manual downloads: https://github.com/${REPO}/releases"
  echo "   🛠️ Build from source: cargo install hdrt"
  exit 1
fi

echo "🔍 Detected: arch=$ARCH_NAME pkg_mgr=$PKG_MGR"

run_root() {
  if [ "$(id -u)" -eq 0 ]; then
    "$@"
  elif command -v sudo >/dev/null 2>&1; then
    sudo "$@"
  else
    echo "❌ Root permission is required for $PKG_MGR installation, but sudo was not found."
    echo "   Re-run this script as root, or install the binary manually from the Release page."
    exit 1
  fi
}

if [ -n "${HDRT_VERSION:-}" ]; then
  VERSION="$HDRT_VERSION"
  echo "📌 Using specified version: $VERSION"
else
  echo "📡 Fetching latest version from GitHub..."
  VERSION=$(curl -fsSL "$API_URL" | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')
  if [ -z "$VERSION" ]; then
    echo "❌ Failed to fetch latest version from GitHub API."
    echo "   Set HDRT_VERSION manually, for example: HDRT_VERSION=v0.1.5-alpha.8"
    exit 1
  fi
  echo "📦 Latest version: $VERSION"
fi

BASE_URL="https://github.com/${REPO}/releases/download/${VERSION}"
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

if [ "$PKG_MGR" = "termux" ]; then
  ASSET="hdrt-android-${ARCH_NAME}-${VERSION}"
  echo "📥 Downloading ${ASSET}..."
  curl -fSL -o "${TMP_DIR}/hdrt" "${BASE_URL}/${ASSET}"
  echo "📦 Installing to ${PREFIX}/bin/hdrt..."
  install -Dm755 "${TMP_DIR}/hdrt" "${PREFIX}/bin/hdrt"
elif [ "$PKG_MGR" = "apt" ]; then
  PKG_FILE="hdrt-linux-${ARCH_NAME}-${VERSION}.deb"
  echo "📥 Downloading ${PKG_FILE}..."
  curl -fSL -o "${TMP_DIR}/${PKG_FILE}" "${BASE_URL}/${PKG_FILE}"
  echo "📦 Installing via dpkg/apt..."
  run_root dpkg -i "${TMP_DIR}/${PKG_FILE}" || run_root apt-get install -f -y
elif [ "$PKG_MGR" = "dnf" ]; then
  PKG_FILE="hdrt-linux-${ARCH_NAME}-${VERSION}.rpm"
  echo "📥 Downloading ${PKG_FILE}..."
  curl -fSL -o "${TMP_DIR}/${PKG_FILE}" "${BASE_URL}/${PKG_FILE}"
  echo "📦 Installing via dnf..."
  run_root dnf install -y "${TMP_DIR}/${PKG_FILE}"
fi

echo ""
echo "✅ hdrt installed successfully."
echo "   Run: hdrt doctor"
echo ""
echo "🧹 Uninstall:"
if [ "$PKG_MGR" = "termux" ]; then
  echo "  rm ${PREFIX}/bin/hdrt"
elif [ "$PKG_MGR" = "apt" ]; then
  echo "  sudo apt remove hdrt"
elif [ "$PKG_MGR" = "dnf" ]; then
  echo "  sudo dnf remove hdrt"
fi
echo ""
echo "📖 GitHub: https://github.com/${REPO}"
echo "📖 Gitee mirror: https://gitee.com/vincent-zyu/hdrt"
echo ""
echo "🇨🇳 Gitee mirror install script:"
echo "   curl -fsSL https://gitee.com/vincent-zyu/hdrt/raw/main/docs/scripts/install/install_gitee.sh | bash"
