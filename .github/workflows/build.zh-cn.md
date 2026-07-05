# 构建与发布工作流

> **[English](build.md)**
> **[简体中文](build.zh-cn.md)**

## 概述

`hdrt` 的 CI/CD 工作流由 commit message 中的关键词驱动。推送到 `main` 分支时，只要包含支持的关键词，GitHub Actions 就会执行对应的构建、发布或 Scoop 更新流程。

## 关键词

当前只启用下面四种工作流关键词。

| Commit 信息中的关键词 | 构建（8 平台） | GitHub Release | Scoop | AUR / npm | crates.io |
|----------------------|:---:|:---:|:---:|:---:|:---:|
| `build action` | ✅ | ❌ | ❌ | ❌ | ❌ |
| `build release` | ✅ | ✅ | ❌ | ❌ | ❌ |
| `build publish` | ✅ | ✅ | ✅ | ❌ | ❌ |
| `publish from release` | ❌ | ❌ | ✅ | ❌ | ❌ |

说明：

- Pull Request 始终会构建，但不会发布。
- `build publish` 会构建二进制、创建 GitHub Release，然后更新 Scoop bucket。
- `publish from release` 不重新构建，会复用当前 `Cargo.toml` 版本对应的已有 GitHub Release 产物。
- AUR、npm 和 crates.io 任务暂时预留，后续再接。

## 用法示例

```bash
# 构建所有已启用目标。
git commit --allow-empty -m "ci: test hdrt cross build (build action)"

# 构建并创建 GitHub Release。
git commit -m "release: v0.1.0 (build release)"

# 构建、创建 GitHub Release，并发布 Scoop manifest。
git commit -m "release: v0.1.0 (build publish)"

# 从已有 Release 重新发布 Scoop manifest。
git commit --allow-empty -m "ci: update scoop manifest (publish from release)"
```

## 构建目标

| 平台 | 架构 | Target | 产物 |
|------|:---:|--------|------|
| Windows | x86_64 | `x86_64-pc-windows-msvc` | `hdrt-windows-x86_64-vX.Y.Z.exe` |
| Windows | ARM64 | `aarch64-pc-windows-msvc` | `hdrt-windows-aarch64-vX.Y.Z.exe` |
| Linux | x86_64 | `x86_64-unknown-linux-musl` | `hdrt-linux-x86_64-vX.Y.Z` |
| Linux | ARM64 | `aarch64-unknown-linux-gnu` | `hdrt-linux-aarch64-vX.Y.Z` |
| macOS | x86_64 | `x86_64-apple-darwin` | `hdrt-macos-x86_64-vX.Y.Z` |
| macOS | ARM64 | `aarch64-apple-darwin` | `hdrt-macos-aarch64-vX.Y.Z` |
| Android / Termux | ARM64 | `aarch64-linux-android` | `hdrt-android-aarch64-vX.Y.Z` |
| Android / Termux | x86_64 | `x86_64-linux-android` | `hdrt-android-x86_64-vX.Y.Z` |

## 流水线

```text
check
  ├─ 解析 commit message
  └─ 从 Cargo.toml 提取版本

build
  ├─ 在 linux-x86_64 上运行 rustfmt / clippy
  ├─ 构建 8 个 release 二进制
  └─ 上传构建产物

release
  ├─ 下载构建产物
  ├─ 生成 release notes
  └─ 创建 GitHub Release

publish-scoop
  ├─ 从 GitHub Release 下载 Windows 二进制
  ├─ 计算 SHA256
  ├─ 生成 hdrt.json
  └─ 推送 bucket/hdrt.json 到 VincentZyuApps/scoop-bucket
```

## Scoop 发布

Scoop job 会发布名为 `hdrt.json` 的 manifest 到：

```text
VincentZyuApps/scoop-bucket
```

manifest 支持：

- Windows x86_64
- Windows ARM64

需要配置的密钥：

| Secret | 用途 |
|--------|------|
| `SCOOP_BUCKET_TOKEN` | 可推送到 `VincentZyuApps/scoop-bucket` 的 GitHub PAT |

## 版本号

版本号从根目录 `Cargo.toml` 提取：

```toml
version = "0.1.0"
```

工作流会转换成 Release tag：

```text
v0.1.0
```

构建产物文件名也会包含同一个 tag。
