> **[📖English](README.md) | [📖简体中文](README.zh-cn.md)**
> **[📖Build Doc](.github/workflows/build.md)**

![hdrt](https://socialify.git.ci/VincentZyuApps/hdrt/image?custom_language=Rust&forks=1&issues=1&language=1&logo=https%3A%2F%2Favatars.githubusercontent.com%2Fu%2F250448479%3Fs%3D200%26v%3D4&name=1&owner=1&pulls=1&stargazers=1&theme=Auto)

# 🖥️ hdrt

[![GitHub](https://img.shields.io/badge/GitHub-181717?style=for-the-badge&logo=github&logoColor=white)](https://github.com/VincentZyuApps/hdrt)
[![Gitee](https://img.shields.io/badge/Gitee-C71D23?style=for-the-badge&logo=gitee&logoColor=white)](https://gitee.com/vincent-zyu/hdrt)

`hdrt` is a cross-platform hardware information CLI/TUI.

Name meaning:

- `hd`: Hardware Device
- `rt`: Rust Ratatui

Full meaning:

- `hdrt`: Hardware Device Rust Ratatui
- `hard ratatui`: a Ratatui-based terminal tool for reading hardware details

Quick memory hint: you can remember `hdrt` as `"hard rata"`, which makes the short command easier to recall.

## 📦 Installation

### Linux / Android / Termux (script)

The install script supports:

- apt-based Linux distributions through `.deb`
- dnf-based Linux distributions through `.rpm`
- Android / Termux through the Android binary
- x86_64 and aarch64

GitHub:

```bash
curl -fsSL https://raw.githubusercontent.com/VincentZyuApps/hdrt/main/docs/scripts/install/install.sh | bash
hdrt doctor
```

Gitee mirror:

```bash
curl -fsSL https://gitee.com/vincent-zyu/hdrt/raw/main/docs/scripts/install/install_gitee.sh | bash
hdrt doctor
```

Install a specific version:

```bash
HDRT_VERSION=v0.1.5-alpha.8 bash -c "$(curl -fsSL https://raw.githubusercontent.com/VincentZyuApps/hdrt/main/docs/scripts/install/install.sh)"
```

### Windows (Scoop)

> [Scoop Bucket](https://github.com/VincentZyuApps/scoop-bucket/blob/main/bucket/hdrt.json)

```powershell
scoop bucket add vincentzyu https://github.com/VincentZyuApps/scoop-bucket
scoop install hdrt
hdrt doctor
```

Gitee mirror:

```powershell
scoop bucket add vincentzyu https://gitee.com/vincent-zyu/scoop-bucket
scoop install hdrt
hdrt doctor
```

On Windows, `hdrt` uses native Rust WMI/CIM by default, then falls back to the lightweight `sysinfo + registry` backend if WMI is unavailable. Use PowerShell/CIM explicitly only when you want a comparison or debug fallback. `--ps` and `--ps1` are aliases for `--powershell`:

```powershell
hdrt --powershell all
hdrt --ps disk
hdrt --ps1 memory
```

## ⌨️ Commands

```bash
hdrt disk
hdrt memory
hdrt cpu
hdrt motherboard
hdrt all
hdrt doctor
hdrt doctor --bench
hdrt tui
```

Aliases:

- `hdrt d` for `hdrt disk`
- `hdrt m` and `hdrt mem` for `hdrt memory`
- `hdrt c` for `hdrt cpu`
- `hdrt b` and `hdrt mb` for `hdrt motherboard`
- `hdrt a` for `hdrt all`

## 🧾 Output Formats

```bash
hdrt disk --format table
hdrt disk --format wide
hdrt disk --format json
hdrt disk --format markdown
hdrt all --lang zh-cn
```

Display languages:

- `--lang en-us` is the default.
- `--lang zh-cn` localizes table, markdown, and TUI labels.
- Unknown display values are shown as `【--UNKNOWN--】` in English and `【--未知--】` in Simplified Chinese.

## 🔐 Permissions

`hdrt` should show as much as it can with the current permissions.

Some fields need elevated privileges or external tools:

- Linux SMART details usually need `smartctl`, often with `sudo`.
- Linux memory slot serial numbers usually need `dmidecode`, often with `sudo`.
- Linux disk inventory uses `lsblk` by default and falls back to `df` logical volumes when `lsblk` is unavailable.
- Android / Termux uses `/proc`, `df`, and `getprop`; Android may hide low-level disk, board, serial, firmware, and health fields.
- Windows board, BIOS, and disk serial fields may need an Administrator terminal.

Recommended checks:

```bash
hdrt doctor
hdrt doctor --bench
sudo hdrt disk --detail smart
sudo hdrt memory
```

## 🚧 Status

This project is in early development. Current collectors cover Linux, Android/Termux, Windows, and placeholder macOS support, followed by a richer Ratatui UI.
