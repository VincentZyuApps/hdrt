![hdrt](https://socialify.git.ci/VincentZyuApps/hdrt/image?custom_language=Rust&forks=1&issues=1&language=1&logo=https%3A%2F%2Favatars.githubusercontent.com%2Fu%2F250448479%3Fs%3D200%26v%3D4&name=1&owner=1&pulls=1&stargazers=1&theme=Auto)

# 🖥️ hdrt

English | [简体中文](README.zh-cn.md)

[![GitHub](https://img.shields.io/badge/GitHub-181717?style=for-the-badge&logo=github&logoColor=white)](https://github.com/VincentZyuApps/hdrt)
[![Gitee](https://img.shields.io/badge/Gitee-C71D23?style=for-the-badge&logo=gitee&logoColor=white)](https://gitee.com/vincent-zyu/hdrt)

> **[Build Docs](.github/workflows/build.md)**

`hdrt` is a cross-platform hardware information CLI/TUI.

Name meaning:

- `hd`: Hardware Device
- `rt`: Rust Ratatui

Full meaning:

- `hdrt`: Hardware Device Rust Ratatui
- `hard ratatui`: a Ratatui-based terminal tool for reading hardware details

Quick memory hint: you can remember `hdrt` as `"hard rata"`, which makes the short command easier to recall.

## 📦 Installation

### Windows (Scoop)

> [Scoop Bucket](https://github.com/VincentZyuApps/scoop-bucket/blob/main/bucket/hdrt.json)

```powershell
scoop bucket add vincentzyu https://github.com/VincentZyuApps/scoop-bucket
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
```

## 🔐 Permissions

`hdrt` should show as much as it can with the current permissions.

Some fields need elevated privileges or external tools:

- Linux SMART details usually need `smartctl`, often with `sudo`.
- Linux memory slot serial numbers usually need `dmidecode`, often with `sudo`.
- Windows board, BIOS, and disk serial fields may need an Administrator terminal.

Recommended checks:

```bash
hdrt doctor
hdrt doctor --bench
sudo hdrt disk --detail smart
sudo hdrt memory
```

## 🚧 Status

This project is in early development. The first usable target is a Linux-first CLI MVP, followed by Windows, macOS, Android/Termux, and a richer Ratatui UI.
