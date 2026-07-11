> **[📖English](README.md) | [📖简体中文](README.zh-cn.md)**

<br>

> **[📖Build Doc](.github/workflows/build.md)**

![hdrt](https://socialify.git.ci/VincentZyuApps/hdrt/image?custom_language=Rust&description=1&forks=1&issues=1&language=1&logo=https%3A%2F%2Favatars.githubusercontent.com%2Fu%2F250448479%3Fs%3D200%26v%3D4&name=1&owner=1&pulls=1&stargazers=1&theme=Auto&v=4)

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
HDRT_VERSION=vX.Y.Z bash -c "$(curl -fsSL https://raw.githubusercontent.com/VincentZyuApps/hdrt/main/docs/scripts/install/install.sh)"
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
hdrt physical-disk
hdrt logical-disk
hdrt memory
hdrt cpu
hdrt motherboard
hdrt all
hdrt doctor
hdrt bench
hdrt --backend native
hdrt --backend shell disk
hdrt -e all
hdrt -e tui
hdrt --no-spinner all
hdrt --spinner-style ascii bench
hdrt tui
hdrt tui -t 1000
hdrt tui --chart-mode bar
hdrt tui --border rounded
```

Aliases:

- `hdrt d` for `hdrt disk`, which shows both physical and logical disks
- `hdrt pd` for `hdrt physical-disk`
- `hdrt ld` for `hdrt logical-disk`
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
hdrt --backend native physical-disk
hdrt --backend shell memory
hdrt bench
```

Platform notes:

- Windows `native` uses Rust WMI/CIM and native fallback code; `shell` uses the PowerShell/CIM script.
- Linux `native` uses `/sys`, `/proc`, and DMI files; `shell` uses tools such as `lsblk`, `smartctl`, and `dmidecode`.
- Linux disk health is filled by `auto` / `shell` through `smartctl` when available. `native` keeps health unknown until native SMART/NVMe probing is implemented.
- Android / Termux and macOS accept `--backend`, but their backend split is still narrower than Windows/Linux.

## ✨ Emoji Mode

Emoji decorations are disabled by default. Use `-e` or `--emoji` to decorate CLI output, Markdown, JSON display labels, spinner messages, and TUI labels.

```bash
hdrt -e all
hdrt --emoji disk --format markdown
hdrt --emoji disk --format json
hdrt --emoji tui
```

`--emoji --format json` keeps the raw data under `data` and adds decorated `title` and `labels` fields for display.

## 🌀 Spinner

`hdrt` shows an interactive loading spinner by default when stderr is a terminal. The spinner writes to stderr, so JSON and Markdown stdout stay clean.

```bash
hdrt --no-spinner all
hdrt --spinner-style unicode all
hdrt --spinner-style ascii bench
hdrt --spinner-style dots disk
```

Spinner styles:

- `unicode` is the default and uses Braille frames.
- `ascii` uses `/ | \ -`.
- `dots` uses a dotted pulse.

## 🖥️ TUI

`hdrt tui` opens the live Ratatui interface. It combines the static hardware inventory with real-time CPU, memory, and disk telemetry.

```bash
hdrt tui
hdrt tui --tab cpu
hdrt tui --tab physical-disk --chart-mode gauge
hdrt tui --tab logical-disk --chart-mode bar -t 1000
hdrt tui --interval 1000
hdrt tui --border double
hdrt tui --tui-border thick
```

TUI controls:

- `Tab` / `Left` / `Right` / `WASD`: switch tabs.
- `z` / `c`: switch the global chart style in this fixed cycle: gauge, bar, sparkline, line, scatter.
- `j` / `k`: select disks in the physical/logical disk tabs.
- `r`: refresh the static hardware inventory and reset live samples.
- `q` / `Esc`: quit.

`--chart-mode` only sets the initial chart mode. After startup, `z` / `c` continue from that position in the same fixed cycle.

`--border` selects the Ratatui panel border: `rounded`, `plain`, `double`, or `thick`. `--tui-border` is an alias; `round` aliases `rounded`, and `square` aliases `plain`.

The default refresh interval is `1000` ms. Values below `250` ms are clamped to `250` ms.

## 🧾 Render Formats

```bash
hdrt disk --format table
hdrt disk --style modern
hdrt disk --style psql
hdrt disk --format json
hdrt disk --format markdown
hdrt physical-disk --format table
hdrt logical-disk --format table
hdrt all --lang zh-cn
hdrt physical-disk --detail smart
```

CLI render styles:

- `--format` selects the CLI render format: `table`, `json`, or `markdown`.
- `--style` selects the CLI table style: `rounded`, `modern`, `sharp`, `psql`, `ascii`, or `blank`.
- `--table-style` is an alias for `--style`; `round` aliases `rounded`, and `plain` aliases `ascii`.
- The old compact-style output is now `--style modern`.
- CLI `--style` does not affect Ratatui; use `hdrt tui --border <BORDER>` for TUI panels.
- `--no-color` disables ANSI colors, and `--no-bold` disables bold text.

Display languages:

- `--lang en-us` is the default.
- `--lang zh-cn` localizes help, table, markdown, and TUI labels.
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
hdrt bench
hdrt --backend native physical-disk
hdrt --backend shell physical-disk --detail smart
sudo hdrt physical-disk --detail smart
sudo hdrt memory
```

## 🚧 Status

This project is in early development. Current collectors cover Linux, Android/Termux, Windows, and placeholder macOS support, followed by a richer Ratatui UI.
