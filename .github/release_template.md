<div align=center>

[![Downloads](https://img.shields.io/github/downloads/__REPO__/__VERSION__/total?style=flat-square&logo=github)](https://github.com/__REPO__/releases/__VERSION__)

</div>

### Downloads

| OS / Arch | x86_64 | ARM64 |
|-----------|--------|-------|
| **Windows** | [![binary](https://img.shields.io/badge/binary-x64-0078D4.svg?logo=windows)](__BASE_URL__/hdrt-windows-x86_64-__VERSION__.exe) | [![binary](https://img.shields.io/badge/binary-ARM64-0099CC.svg?logo=windows)](__BASE_URL__/hdrt-windows-aarch64-__VERSION__.exe) |
| **Linux** | [![binary](https://img.shields.io/badge/binary-x64-E95420.svg?logo=linux)](__BASE_URL__/hdrt-linux-x86_64-__VERSION__)<br>[![deb](https://img.shields.io/badge/deb-x64-A81D33.svg?logo=debian)](__BASE_URL__/hdrt-linux-x86_64-__VERSION__.deb)<br>[![rpm](https://img.shields.io/badge/rpm-x64-EE0000.svg?logo=redhat)](__BASE_URL__/hdrt-linux-x86_64-__VERSION__.rpm) | [![binary](https://img.shields.io/badge/binary-ARM64-E95420.svg?logo=linux)](__BASE_URL__/hdrt-linux-aarch64-__VERSION__)<br>[![deb](https://img.shields.io/badge/deb-ARM64-A81D33.svg?logo=debian)](__BASE_URL__/hdrt-linux-aarch64-__VERSION__.deb)<br>[![rpm](https://img.shields.io/badge/rpm-ARM64-EE0000.svg?logo=redhat)](__BASE_URL__/hdrt-linux-aarch64-__VERSION__.rpm) |
| **macOS** | [![binary](https://img.shields.io/badge/binary-Intel-000000.svg?logo=apple)](__BASE_URL__/hdrt-macos-x86_64-__VERSION__) | [![binary](https://img.shields.io/badge/binary-Apple_Silicon-000000.svg?logo=apple)](__BASE_URL__/hdrt-macos-aarch64-__VERSION__) |
| **Android / Termux** | [![binary](https://img.shields.io/badge/binary-x64-96ed89.svg?logo=android)](__BASE_URL__/hdrt-android-x86_64-__VERSION__) | [![binary](https://img.shields.io/badge/binary-ARM64-168039.svg?logo=android)](__BASE_URL__/hdrt-android-aarch64-__VERSION__) |

### Quick Start

Linux / Android / Termux install script from GitHub:

```bash
curl -fsSL https://raw.githubusercontent.com/__REPO__/main/docs/scripts/install/install.sh | bash
```

Gitee mirror install script:

```bash
curl -fsSL https://gitee.com/vincent-zyu/hdrt/raw/main/docs/scripts/install/install_gitee.sh | bash
```

```bash
hdrt doctor
hdrt all
hdrt --lang zh-cn
hdrt tui
```

### Windows Scoop

```powershell
scoop bucket add vincentzyu https://github.com/VincentZyuApps/scoop-bucket
scoop update
scoop install hdrt@__PLAIN_VER__
```

### Gitee Scoop Mirror

```powershell
scoop bucket add vincentzyu https://gitee.com/vincent-zyu/scoop-bucket
scoop update
scoop install hdrt@__PLAIN_VER__
```

### Notes

- `hdrt` uses `--backend auto` by default. Use `--backend native` to avoid external commands, or `--backend shell` to force shell-based collectors.
- Linux native mode uses `/sys` and `/proc`; auto/shell modes can use `lsblk`, `dmidecode`, and optional `smartctl` for richer fields.
- Android / Termux uses `/proc`, `df`, and `getprop`; low-level board, serial, firmware, and disk health fields may be hidden by Android.
