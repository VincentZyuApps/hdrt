> **[📖English](README.md) | [📖简体中文](README.zh-cn.md)**

<br>

> **[📖Build Doc](.github/workflows/build.md)**

![hdrt](https://socialify.git.ci/VincentZyuApps/hdrt/image?custom_language=Rust&description=1&forks=1&issues=1&language=1&logo=https%3A%2F%2Favatars.githubusercontent.com%2Fu%2F250448479%3Fs%3D200%26v%3D4&name=1&owner=1&pulls=1&stargazers=1&theme=Auto&v=4)

# 🖥️ hdrt

[![GitHub](https://img.shields.io/badge/GitHub-181717?style=for-the-badge&logo=github&logoColor=white)](https://github.com/VincentZyuApps/hdrt)
[![Gitee](https://img.shields.io/badge/Gitee-C71D23?style=for-the-badge&logo=gitee&logoColor=white)](https://gitee.com/vincent-zyu/hdrt)

`hdrt` 是一个跨平台硬件信息 CLI / TUI 工具。

名称含义：

- `hd`: Hardware Device
- `rt`: Rust Ratatui

快速记忆：可以把 `hdrt` 记成 `"hard rata"`，这样更容易想起这个简写指令。

## 📦 安装

### Linux / Android / Termux（脚本）

安装脚本支持：

- apt 系 Linux 发行版，安装 `.deb`
- dnf 系 Linux 发行版，安装 `.rpm`
- Android / Termux，安装 Android 二进制
- x86_64 和 aarch64

GitHub：

```bash
curl -fsSL https://raw.githubusercontent.com/VincentZyuApps/hdrt/main/docs/scripts/install/install.sh | bash
hdrt doctor
```

Gitee 镜像：

```bash
curl -fsSL https://gitee.com/vincent-zyu/hdrt/raw/main/docs/scripts/install/install_gitee.sh | bash
hdrt doctor
```

安装指定版本：

```bash
HDRT_VERSION=vX.Y.Z bash -c "$(curl -fsSL https://gitee.com/vincent-zyu/hdrt/raw/main/docs/scripts/install/install_gitee.sh)"
```

### Windows (Scoop)

> [Scoop Bucket](https://github.com/VincentZyuApps/scoop-bucket/blob/main/bucket/hdrt.json)

```powershell
scoop bucket add vincentzyu https://github.com/VincentZyuApps/scoop-bucket
scoop install hdrt
hdrt doctor
```

Gitee 镜像：

```powershell
scoop bucket add vincentzyu https://gitee.com/vincent-zyu/scoop-bucket
scoop install hdrt
hdrt doctor
```

## ⌨️ 命令

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
hdrt tui -t 2000
hdrt tui --chart-mode bar
```

别名：

- `hdrt d` 对应 `hdrt disk`，会同时显示物理磁盘和逻辑磁盘
- `hdrt pd` 对应 `hdrt physical-disk`
- `hdrt ld` 对应 `hdrt logical-disk`
- `hdrt m` 和 `hdrt mem` 对应 `hdrt memory`
- `hdrt c` 对应 `hdrt cpu`
- `hdrt b` 和 `hdrt mb` 对应 `hdrt motherboard`
- `hdrt a` 对应 `hdrt all`

## 🧩 后端

`hdrt` 默认使用 `--backend auto`。后端选择是全局参数，可以和任意命令一起使用。

| 后端 | 行为 | 是否启动外部命令 |
|------|------|------------------|
| `auto` | 优先使用 native 采集器；字段缺失时允许 shell 采集器补齐。 | 可能会 |
| `native` | 只使用 Rust/native 系统接口。适合检查不依赖命令辅助时能采集到什么。 | 不会 |
| `shell` | 强制使用 shell 后端。适合和系统工具输出做对照。 | 会 |

示例：

```bash
hdrt --backend auto all
hdrt --backend native physical-disk
hdrt --backend shell memory
hdrt bench
```

平台说明：

- Windows `native` 使用 Rust WMI/CIM 和 native fallback 代码；`shell` 使用 PowerShell/CIM 脚本。
- Linux `native` 使用 `/sys`、`/proc` 和 DMI 文件；`shell` 使用 `lsblk`、`smartctl`、`dmidecode` 等工具。
- Linux 硬盘健康状态由 `auto` / `shell` 在可用时通过 `smartctl` 补齐。`native` 会暂时保持未知，后续再做原生 SMART/NVMe 探测。
- Android / Termux 和 macOS 接受 `--backend` 参数，但后端拆分还没有 Windows/Linux 完整。

## ✨ Emoji 模式

Emoji 装饰默认关闭。使用 `-e` 或 `--emoji` 可以装饰 CLI 输出、Markdown、JSON 展示标签、spinner 文案和 TUI 标签。

```bash
hdrt -e all
hdrt --emoji disk --format markdown
hdrt --emoji disk --format json
hdrt --emoji tui
```

`--emoji --format json` 会把原始数据保留在 `data` 中，并额外增加带 emoji 的 `title` 和 `labels` 展示字段。

## 🌀 Spinner

当 stderr 是交互式终端时，`hdrt` 默认显示加载 spinner。spinner 写入 stderr，所以 JSON 和 Markdown 的 stdout 仍然保持干净。

```bash
hdrt --no-spinner all
hdrt --spinner-style unicode all
hdrt --spinner-style ascii bench
hdrt --spinner-style dots disk
```

Spinner 样式：

- `unicode` 是默认值，使用 Braille 动画帧。
- `ascii` 使用 `/ | \ -`。
- `dots` 使用点状脉冲。

## 🖥️ TUI

`hdrt tui` 会打开实时 Ratatui 界面，把静态硬件清单和实时 CPU、内存、磁盘遥测放在一起展示。

```bash
hdrt tui
hdrt tui --tab cpu
hdrt tui --tab physical-disk --chart-mode gauge
hdrt tui --tab logical-disk --chart-mode bar -t 1000
hdrt tui --interval 2000
```

TUI 快捷键：

- `Tab` / `Left` / `Right` / `WASD`：切换标签页。
- `z` / `c`：按固定顺序全局切换图表模式：仪表、条形、火花、折线、散点。
- `j` / `k`：在物理磁盘 / 逻辑磁盘页选择磁盘。
- `r`：刷新静态硬件清单并重置实时采样。
- `q` / `Esc`：退出。

`--chart-mode` 只设置启动时的初始图表模式。启动后，`z` / `c` 会从该位置继续按同一个固定顺序循环。

默认刷新间隔是 `2000` ms。低于 `250` ms 的值会被钳制到 `250` ms。

## 🧾 输出格式

```bash
hdrt disk --format table
hdrt disk --format wide
hdrt disk --format compact
hdrt disk --format json
hdrt disk --format markdown
hdrt physical-disk --format table
hdrt logical-disk --format table
hdrt all --lang zh-cn
hdrt physical-disk --detail smart
```

表格样式：

- `table` 和 `wide` 目前使用相同的圆角表格布局。
- `compact` 使用 modern 边框表格样式；它是视觉样式，不是删列压缩视图。

显示语言：

- `--lang en-us` 是默认值。
- `--lang zh-cn` 会本地化帮助、表格、Markdown 和 TUI 标签。
- 未知显示值在英文下显示为 `【--UNKNOWN--】`，在简体中文下显示为 `【--未知--】`。

详情级别：

- `--detail basic` 是默认值。
- `--detail smart` 会让磁盘采集器在当前后端支持时读取 SMART 和健康详情。
- `--detail full` 预留给最完整的详情级别。

## 🔐 权限

`hdrt` 会尽量在当前权限下显示尽可能多的信息。

部分字段需要管理员权限或外部工具：

- Linux SMART 详情通常需要 `smartctl`，很多场景还需要 `sudo`。
- Linux 内存插槽序列号通常需要 `dmidecode`，很多场景还需要 `sudo`。
- Linux `--backend native` 不启动 shell 命令，所以硬盘健康状态等字段可能保持未知。
- Linux `--backend auto` 和 `--backend shell` 可以使用 `lsblk`、`smartctl`、`dmidecode` 等工具。
- Android / Termux 使用 `/proc`、`df` 和 `getprop`；Android 可能隐藏底层磁盘、主板、序列号、固件和健康状态字段。
- Windows 主板、BIOS、磁盘序列号等字段可能需要管理员终端。

推荐检查：

```bash
hdrt doctor
hdrt bench
hdrt --backend native physical-disk
hdrt --backend shell physical-disk --detail smart
sudo hdrt physical-disk --detail smart
sudo hdrt memory
```

## 🚧 状态

项目处于早期开发阶段。当前采集器覆盖 Linux、Android/Termux、Windows，并保留 macOS placeholder，后续继续完善 Ratatui UI。
