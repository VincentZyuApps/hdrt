![hdrt](https://socialify.git.ci/VincentZyuApps/hdrt/image?custom_language=Rust&forks=1&issues=1&language=1&logo=https%3A%2F%2Favatars.githubusercontent.com%2Fu%2F250448479%3Fs%3D200%26v%3D4&name=1&owner=1&pulls=1&stargazers=1&theme=Auto)

# 🖥️ hdrt

[English](README.md) | 简体中文

[![GitHub](https://img.shields.io/badge/GitHub-181717?style=for-the-badge&logo=github&logoColor=white)](https://github.com/VincentZyuApps/hdrt)
[![Gitee](https://img.shields.io/badge/Gitee-C71D23?style=for-the-badge&logo=gitee&logoColor=white)](https://gitee.com/vincent-zyu/hdrt)

> **[构建文档](.github/workflows/build.zh-cn.md)**

`hdrt` 是一个跨平台硬件信息 CLI / TUI 工具。

名称含义：

- `hd`: Hardware Device
- `rt`: Rust Ratatui

完整解释：

- `hdrt`: Hardware Device Rust Ratatui
- `hard ratatui`: 面向硬件信息查看的 Ratatui 终端工具

快速记忆：可以把 `hdrt` 记成 `"hard rata"`，这样更容易想起这个简写指令。

## 📦 安装

### Windows (Scoop)

> [Scoop Bucket](https://github.com/VincentZyuApps/scoop-bucket/blob/main/bucket/hdrt.json)

```powershell
scoop bucket add vincentzyu https://github.com/VincentZyuApps/scoop-bucket
scoop install hdrt
hdrt doctor
```

Windows 上默认使用 Rust native WMI/CIM 后端；如果 WMI 不可用，会退回轻量 `sysinfo + registry` 后端。只有需要对照或 debug 时才显式启用 PowerShell/CIM 后端。`--ps` 和 `--ps1` 是 `--powershell` 的别名：

```powershell
hdrt --powershell all
hdrt --ps disk
hdrt --ps1 memory
```

## ⌨️ 命令

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

别名：

- `hdrt d` 对应 `hdrt disk`
- `hdrt m` 和 `hdrt mem` 对应 `hdrt memory`
- `hdrt c` 对应 `hdrt cpu`
- `hdrt b` 和 `hdrt mb` 对应 `hdrt motherboard`
- `hdrt a` 对应 `hdrt all`

## 🧾 输出格式

```bash
hdrt disk --format table
hdrt disk --format wide
hdrt disk --format json
hdrt disk --format markdown
```

## 🔐 权限

`hdrt` 会尽量在当前权限下显示尽可能多的信息。

部分字段需要管理员权限或外部工具：

- Linux SMART 详情通常需要 `smartctl`，很多场景还需要 `sudo`。
- Linux 内存插槽序列号通常需要 `dmidecode`，很多场景还需要 `sudo`。
- Windows 主板、BIOS、磁盘序列号等字段可能需要管理员终端。

推荐检查：

```bash
hdrt doctor
hdrt doctor --bench
sudo hdrt disk --detail smart
sudo hdrt memory
```

## 🚧 状态

项目处于早期开发阶段。第一版目标是 Linux 优先的 CLI MVP，然后逐步补 Windows、macOS、Android/Termux 和更完整的 Ratatui UI。
