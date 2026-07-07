> **[📖English](README.md) | [📖简体中文](README.zh-cn.md)**
<br>
> **[📖Build Doc](.github/workflows/build.md)**

![hdrt](https://socialify.git.ci/VincentZyuApps/hdrt/image?custom_language=Rust&forks=1&issues=1&language=1&logo=https%3A%2F%2Favatars.githubusercontent.com%2Fu%2F250448479%3Fs%3D200%26v%3D4&name=1&owner=1&pulls=1&stargazers=1&theme=Auto)

# 🖥️ hdrt

[![GitHub](https://img.shields.io/badge/GitHub-181717?style=for-the-badge&logo=github&logoColor=white)](https://github.com/VincentZyuApps/hdrt)
[![Gitee](https://img.shields.io/badge/Gitee-C71D23?style=for-the-badge&logo=gitee&logoColor=white)](https://gitee.com/vincent-zyu/hdrt)

`hdrt` is a cross-platform hardware information CLI/TUI.

Name meaning:

- `hd`: Hardware Device
- `rt`: Rust Ratatui

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
HDRT_VERSION=v0.1.7-alpha.10 bash -c "$(curl -fsSL https://raw.githubusercontent.com/VincentZyuApps/hdrt/main/docs/scripts/install/install.sh)"
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

## ⌨️ Commands

```bash
hdrt disk
hdrt memory
hdrt cpu
hdrt motherboard
hdrt all
hdrt doctor
hdrt doctor --bench
hdrt --backend native
hdrt --backend shell disk
hdrt --no-spinner all
hdrt --spinner-style ascii doctor --bench
hdrt tui
```

Aliases:

- `hdrt d` for `hdrt disk`
- `hdrt m` and `hdrt mem` for `hdrt memory`
- `hdrt c` for `hdrt cpu`
- `hdrt b` and `hdrt mb` for `hdrt motherboard`
- `hdrt a` for `hdrt all`

## 🧩 Backends

`hdrt` uses `--backend auto` by default. Backend selection is a global option, so it can be used with any command.

| Backend | Behavior | External commands |
|---------|----------|-------------------|
| `auto` | Native collectors first; shell collectors may fill missing fields. | May run shell commands |
| `native` | Rust/native system APIs only. Useful for checking what `hdrt` can collect without command helpers. | No |
| `shell` | Force shell-based collectors. Useful for comparing with system tools. | Yes |

Examples:

```bash
hdrt --backend auto all
hdrt --backend native disk
hdrt --backend shell memory
hdrt doctor --bench
```

Platform notes:

- Windows `native` uses Rust WMI/CIM and native fallback code; `shell` uses the PowerShell/CIM script.
- Linux `native` uses `/sys`, `/proc`, and DMI files; `shell` uses tools such as `lsblk`, `smartctl`, and `dmidecode`.
- Linux disk health is filled by `auto` / `shell` through `smartctl` when available. `native` keeps health unknown until native SMART/NVMe probing is implemented.
- Android / Termux and macOS accept `--backend`, but their backend split is still narrower than Windows/Linux.

## 🌀 Spinner

`hdrt` shows an interactive loading spinner by default when stderr is a terminal. The spinner writes to stderr, so JSON and Markdown stdout stay clean.

```bash
hdrt --no-spinner all
hdrt --spinner-style unicode all
hdrt --spinner-style ascii doctor --bench
hdrt --spinner-style dots disk
```

Spinner styles:

- `unicode` is the default and uses Braille frames.
- `ascii` uses `/ | \ -`.
- `dots` uses a dotted pulse.

## 🧾 Output Formats

```bash
hdrt disk --format table
hdrt disk --format wide
hdrt disk --format compact
hdrt disk --format json
hdrt disk --format markdown
hdrt all --lang zh-cn
hdrt disk --detail smart
```

Table styles:

- `table` and `wide` currently use the same rounded table layout.
- `compact` uses the modern bordered table style; it is a visual style, not a reduced-column view.

Display languages:

- `--lang en-us` is the default.
- `--lang zh-cn` localizes table, markdown, and TUI labels.
- Unknown display values are shown as `【--UNKNOWN--】` in English and `【--未知--】` in Simplified Chinese.

Detail levels:

- `--detail basic` is the default.
- `--detail smart` asks disk collectors for SMART and health details when the selected backend can provide them.
- `--detail full` is reserved for the richest supported detail level.

## 🔐 Permissions

`hdrt` should show as much as it can with the current permissions.

Some fields need elevated privileges or external tools:

- Linux SMART details usually need `smartctl`, often with `sudo`.
- Linux memory slot serial numbers usually need `dmidecode`, often with `sudo`.
- Linux `--backend native` avoids shell commands, so some fields such as disk health may stay unknown.
- Linux `--backend auto` and `--backend shell` can use tools such as `lsblk`, `smartctl`, and `dmidecode`.
- Android / Termux uses `/proc`, `df`, and `getprop`; Android may hide low-level disk, board, serial, firmware, and health fields.
- Windows board, BIOS, and disk serial fields may need an Administrator terminal.

Recommended checks:

```bash
hdrt doctor
hdrt doctor --bench
hdrt --backend native disk
hdrt --backend shell disk --detail smart
sudo hdrt disk --detail smart
sudo hdrt memory
```

## 🚧 Status

This project is in early development. Current collectors cover Linux, Android/Termux, Windows, and placeholder macOS support, followed by a richer Ratatui UI.
