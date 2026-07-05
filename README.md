![hdrt](https://socialify.git.ci/VincentZyu233/hdrt/image?custom_language=Rust&forks=1&issues=1&language=1&logo=https%3A%2F%2Favatars.githubusercontent.com%2Fu%2F250448479%3Fs%3D200%26v%3D4&name=1&owner=1&pulls=1&stargazers=1&theme=Auto)

# hdrt

English | [简体中文](README.zh-cn.md)

[![GitHub](https://img.shields.io/badge/GitHub-181717?style=for-the-badge&logo=github&logoColor=white)](https://github.com/VincentZyu233/hdrt)
[![Gitee](https://img.shields.io/badge/Gitee-C71D23?style=for-the-badge&logo=gitee&logoColor=white)](https://gitee.com/vincent-zyu/hdrt)

> **[Build Docs](.github/workflows/build.md)**

`hdrt` is a cross-platform hardware information CLI/TUI.

Name meaning:

- `hd`: Hardware Device
- `r`: Rust
- `t`: Ratatui

Full meaning:

- `hdrt`: Hardware Device Rust Ratatui
- `hard ratatui`: a Ratatui-based terminal tool for reading hardware details

Quick memory hint: you can remember `hdrt` as `"hard rata"`, which makes the short command easier to recall.

## Commands

```bash
hdrt disk
hdrt mem
hdrt cpu
hdrt mb
hdrt all
hdrt doctor
hdrt tui
```

Aliases:

- `hdrt mem` and `hdrt memory`
- `hdrt mb` and `hdrt motherboard`

## Output Formats

```bash
hdrt disk --format table
hdrt disk --format wide
hdrt disk --format json
hdrt disk --format markdown
```

## Permissions

`hdrt` should show as much as it can with the current permissions.

Some fields need elevated privileges or external tools:

- Linux SMART details usually need `smartctl`, often with `sudo`.
- Linux memory slot serial numbers usually need `dmidecode`, often with `sudo`.
- Windows board, BIOS, and disk serial fields may need Administrator PowerShell.

Recommended checks:

```bash
hdrt doctor
sudo hdrt disk --detail smart
sudo hdrt mem
```

## Status

This project is in early development. The first usable target is a Linux-first CLI MVP, followed by Windows, macOS, Android/Termux, and a richer Ratatui UI.
