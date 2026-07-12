#!/usr/bin/env bash
# hdrt Gitee 镜像安装脚本：支持 apt (deb)、dnf (rpm)、Termux，架构 x86_64/aarch64。
# 用法:
#   curl -fsSL https://gitee.com/vincent-zyu/hdrt/raw/main/docs/scripts/install/install_gitee.sh | bash
# 安装指定版本:
#   HDRT_VERSION=vX.Y.Z bash -c "$(curl -fsSL https://gitee.com/vincent-zyu/hdrt/raw/main/docs/scripts/install/install_gitee.sh)"
set -e

OWNER="vincent-zyu"
REPO="hdrt"
API_URL="https://gitee.com/api/v5/repos/${OWNER}/${REPO}/releases/latest"

IS_TERMUX=false
if [ -n "${PREFIX:-}" ] && [ -d "${PREFIX}/bin" ]; then
  IS_TERMUX=true
fi

ARCH=$(uname -m)
case "$ARCH" in
  x86_64|amd64) ARCH_NAME="x86_64" ;;
  aarch64|arm64) ARCH_NAME="aarch64" ;;
  *)
    echo "❌ 不支持的架构: $ARCH"
    echo "   当前脚本仅支持 x86_64 和 aarch64。"
    echo "   📦 手动下载: https://gitee.com/${OWNER}/${REPO}/releases"
    echo "   🛠️ 从源码构建: cargo install hdrt"
    exit 1
    ;;
esac

if $IS_TERMUX; then
  ANDROID_API=""
  if command -v getprop >/dev/null 2>&1; then
    ANDROID_API=$(getprop ro.build.version.sdk 2>/dev/null || true)
  elif [ -x /system/bin/getprop ]; then
    ANDROID_API=$(/system/bin/getprop ro.build.version.sdk 2>/dev/null || true)
  fi

  if [ -n "$ANDROID_API" ]; then
    echo "🤖 检测到 Android API：$ANDROID_API"
    if [ "$ANDROID_API" -lt 24 ] 2>/dev/null; then
      echo "❌ 需要 Android API 24（Android 7.0）或更高版本。"
      exit 1
    fi
  fi
fi

if $IS_TERMUX; then
  PKG_MGR="termux"
elif command -v apt-get >/dev/null 2>&1; then
  PKG_MGR="apt"
elif command -v dnf >/dev/null 2>&1; then
  PKG_MGR="dnf"
else
  echo "❌ 不支持的包管理器。"
  echo "   当前脚本支持 apt、dnf 和 Termux。"
  echo "   📦 手动下载: https://gitee.com/${OWNER}/${REPO}/releases"
  echo "   🛠️ 从源码构建: cargo install hdrt"
  exit 1
fi

echo "🔍 检测结果: 架构=$ARCH_NAME 包管理器=$PKG_MGR"

run_root() {
  if [ "$(id -u)" -eq 0 ]; then
    "$@"
  elif command -v sudo >/dev/null 2>&1; then
    sudo "$@"
  else
    echo "❌ 安装 $PKG_MGR 包需要 root 权限，但当前系统没有找到 sudo。"
    echo "   请切换到 root 后重新运行脚本，或到 Release 页面手动下载二进制。"
    exit 1
  fi
}

if [ -n "${HDRT_VERSION:-}" ]; then
  VERSION="$HDRT_VERSION"
  echo "📌 使用指定版本: $VERSION"
else
  echo "📡 正在从 Gitee API 获取最新版本..."
  VERSION=$(curl -fsSL "$API_URL" | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')
  if [ -z "$VERSION" ]; then
    echo "❌ 从 Gitee API 获取最新版本失败。"
    echo "   可以手动指定版本，例如: HDRT_VERSION=vX.Y.Z"
    exit 1
  fi
  echo "📦 最新版本: $VERSION"
fi

BASE_URL="https://gitee.com/${OWNER}/${REPO}/releases/download/${VERSION}"
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

if [ "$PKG_MGR" = "termux" ]; then
  ASSET="hdrt-android-${ARCH_NAME}-${VERSION}"
  echo "📥 正在从 Gitee 下载 ${ASSET}..."
  curl -fSL -o "${TMP_DIR}/hdrt" "${BASE_URL}/${ASSET}"
  echo "📦 正在安装到 ${PREFIX}/bin/hdrt..."
  install -Dm755 "${TMP_DIR}/hdrt" "${PREFIX}/bin/hdrt"
elif [ "$PKG_MGR" = "apt" ]; then
  PKG_FILE="hdrt-linux-${ARCH_NAME}-${VERSION}.deb"
  echo "📥 正在从 Gitee 下载 ${PKG_FILE}..."
  curl -fSL -o "${TMP_DIR}/${PKG_FILE}" "${BASE_URL}/${PKG_FILE}"
  echo "📦 正在通过 dpkg/apt 安装..."
  run_root dpkg -i "${TMP_DIR}/${PKG_FILE}" || run_root apt-get install -f -y
elif [ "$PKG_MGR" = "dnf" ]; then
  PKG_FILE="hdrt-linux-${ARCH_NAME}-${VERSION}.rpm"
  echo "📥 正在从 Gitee 下载 ${PKG_FILE}..."
  curl -fSL -o "${TMP_DIR}/${PKG_FILE}" "${BASE_URL}/${PKG_FILE}"
  echo "📦 正在通过 dnf 安装..."
  run_root dnf install -y "${TMP_DIR}/${PKG_FILE}"
fi

echo ""
echo "✅ hdrt 安装成功。"
echo "   运行: hdrt doctor"
echo ""
echo "🧹 卸载方式:"
if [ "$PKG_MGR" = "termux" ]; then
  echo "  rm ${PREFIX}/bin/hdrt"
elif [ "$PKG_MGR" = "apt" ]; then
  echo "  sudo apt remove hdrt"
elif [ "$PKG_MGR" = "dnf" ]; then
  echo "  sudo dnf remove hdrt"
fi
echo ""
echo "📖 Gitee: https://gitee.com/${OWNER}/${REPO}"
echo "📖 GitHub: https://github.com/VincentZyuApps/hdrt"
echo ""
echo "🌐 GitHub 安装脚本:"
echo "   curl -fsSL https://raw.githubusercontent.com/VincentZyuApps/hdrt/main/docs/scripts/install/install.sh | bash"
