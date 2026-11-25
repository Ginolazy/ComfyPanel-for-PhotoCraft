# ComfyPanel for PhotoCraft

**ComfyPanel for PhotoCraft** is a desktop application bridge designed to seamlessly integrate ComfyPanel's powerful AI image generation and editing capabilities into **PhotoCraft**.

It provides a floating, responsive panel that connects your local or remote ComfyUI environment directly to PhotoCraft canvas workflows.

---

## Features

* **Native Canvas Integration** — Read documents, active selections, and layers directly from PhotoCraft into ComfyUI workflows.
* **Instant Placement** — Generated results can be placed directly back onto PhotoCraft layers or opened as new documents.
* **Dual Mode Support** — Easily toggle between the comprehensive **Studio Mode** (full parameters & workflow controls) and the compact **Turbo Mode** (minimal floating bar).
* **Lightweight & High-Performance** — Powered by Tauri 2.0 with minimal memory and CPU footprint.
* **Cross-Platform** — Built for both macOS (Apple Silicon / Intel Universal) and Windows.

---

## Installation & First Launch

### macOS

1. Download the latest `.dmg` from [GitHub Releases](https://github.com/Ginolazy/ComfyPanel-for-PhotoCraft/releases).
2. Open the DMG and drag **ComfyPanel** into your `Applications` folder.
3. If macOS displays a message saying the application cannot be opened because the developer cannot be verified, open your **Terminal** and run:
   ```bash
   xattr -d com.apple.quarantine "/Applications/ComfyPanel.app"
   ```

### Windows

1. Download the latest `.exe` installer from [GitHub Releases](https://github.com/Ginolazy/ComfyPanel-for-PhotoCraft/releases).
2. Run the installer and follow the setup instructions.

---

## Requirements & Setup

### 1. PhotoCraft Configuration (Required)
Before connecting ComfyPanel, enable external agent control in PhotoCraft:
1. Open PhotoCraft and navigate to **Settings** > **Integrations**.
2. Check **Allow agent control**.
3. Keep the default **Control port** (`44600`).
4. **Restart PhotoCraft** to apply changes (*Applies at next launch*).

### 2. Backend Engine (Choose at least one)
* **ComfyUI Mode**: Local or remote ComfyUI with the [ComfyPanel](https://github.com/Ginolazy/ComfyPanel) custom node installed.
* **Cloud Mode**: Cloud AI engines (e.g. BizyAir, RunningHub, NanoBanana) — **no local ComfyUI installation required!**

---

## 💡 Why a Standalone Companion App Instead of an Embedded Plugin?

* **PhotoCraft's Native Architecture**: Built with Rust and wgpu for maximum canvas performance, PhotoCraft intentionally avoids heavy embedded Webview containers (like Adobe UXP/CEP), offering a fast native Agent Control / IPC protocol instead.
* **Process Isolation & Zero Overhead**: Running in a lightweight Tauri desktop process ensures heavy UI interactions and workflow processing never block or degrade PhotoCraft's GPU rendering pipeline.
* **Seamless Companion Experience**: Designed as a floating companion window (with support for Turbo bar and Studio mode), ComfyPanel stays conveniently accessible alongside PhotoCraft while keeping your workspace clean and flexible.

---

## 📌 Host Compatibility & Feature Adaptation Note

ComfyPanel has expanded its creative AI workflow suite to officially support **PhotoCraft**.

Because PhotoCraft operates on a distinct native architecture (an independent lightweight Tauri companion process rather than an embedded host panel) and its own host control API, some environment-specific UI options and certain extended features are currently undergoing tailored adaptation. The core AI generation, canvas extraction, and layer placement pipelines are fully operational, and dedicated optimizations for PhotoCraft will continue to be rolled out.

---

<br>

# ComfyPanel for PhotoCraft（中文介绍）

**ComfyPanel for PhotoCraft** 是一款专为 **PhotoCraft** 打造的桌面协同应用。它将 ComfyPanel 强大的 AI 图像生成与编辑工作流无缝桥接至 PhotoCraft 的画布设计环境中。

---

## 功能特性

* **画布直连交互** — 可直接读取 PhotoCraft 中当前打开的文档、图层以及选区图像，一键传入 ComfyUI 进行重绘、扩图或风格迁移。
* **快速回填置入** — 生成的高清图像可直接置入回 PhotoCraft 当前画布图层，或一键作为新文档打开。
* **双模式切换** — 支持在功能完备的 **Studio Mode（完整工作区）** 与轻量紧凑的 **Turbo Mode（快捷浮动条）** 之间自由切换。
* **极致轻量** — 基于 Tauri 2.0 构建，启动飞快，常驻系统托盘，几乎不占用多余系统资源。
* **跨平台兼容** — 针对 macOS（通用 Universal 架构）与 Windows 提供原生安装体验。

---

## 安装与首次使用说明

### macOS 用户

1. 前往 [GitHub Releases](https://github.com/Ginolazy/ComfyPanel-for-PhotoCraft/releases) 下载最新的 `.dmg` 安装包。
2. 打开 DMG，将 **ComfyPanel** 拖入 `Applications`（应用程序）文件夹。
3. 如果 macOS 弹出“无法打开，因为无法验证开发者”的安全拦截提示，请打开 **终端（Terminal）** 执行以下命令即可解除：
   ```bash
   xattr -d com.apple.quarantine "/Applications/ComfyPanel.app"
   ```

### Windows 用户

1. 前往 [GitHub Releases](https://github.com/Ginolazy/ComfyPanel-for-PhotoCraft/releases) 下载最新的 `.exe` 安装程序。
2. 运行安装程序并按引导完成安装。

---

## 环境要求与前置配置

### 1. PhotoCraft 配置（初次使用必须配置）
在首次连接 ComfyPanel 前，需在 PhotoCraft 中开启外部控制接口：
1. 打开 PhotoCraft，进入 **Settings（设置）** > **Integrations（集成）**。
2. 勾选 **Allow agent control**（允许代理控制）。
3. 确保 **Control port（控制端口）** 设置为默认的 `44600`。
4. **重启 PhotoCraft** 使设置生效（界面提示 *Applies at next launch*）。

### 2. 后端引擎（以下二选一或兼备即可）
* **ComfyUI 模式**：本地或局域网已启动 ComfyUI，并安装了 [ComfyPanel](https://github.com/Ginolazy/ComfyPanel) 自定义节点插件。
* **云端模式**：直接使用云端 AI 算力引擎（如 BizyAir、RunningHub、NanoBanana 等）—— **完全无需在本地安装 ComfyUI！**

---

## 💡 为什么采用独立伴侣进程，而不是像 Photoshop 那样做成插件？

* **PhotoCraft 原生轻量架构**：PhotoCraft 基于 Rust + wgpu 打造，追求极致的画布手感与渲染性能，本身未内置类似 Photoshop 的 Webview（UXP/CEP）容器，而是官方原生开放了高效的 IPC / Agent Control 控制接口。
* **独立进程更稳定，不抢占画布资源**：AI 面板的重度 UI 渲染与工作流状态在独立的 Tauri 原生轻量进程中运行，与 PhotoCraft 核心渲染管线物理隔离，确保修图手感丝滑，互不干扰，更避免面板异常影响画布主进程。
* **灵活自如的协同体验**：ComfyPanel 作为独立的伴侣浮窗运行（支持 Turbo 快捷浮动条与 Studio 完整工作区），既可在画布旁随时唤起并保持灵活停靠，又避免了挤占 PhotoCraft 宝贵的工作区空间。

---

## 📌 宿主兼容与功能适配说明

ComfyPanel 现已正式将 AI 创作套件扩展并兼容至 **PhotoCraft**。

由于 PhotoCraft 采用了独特的原生架构（高性能独立 Tauri 伴侣进程，而非传统的嵌入式宿主面板）及其专属控制接口，部分针对特定宿主环境的交互选项以及个别扩展服务目前正处于针对 PhotoCraft 的专属调优阶段。核心的图像生成、画布读取与图层置入等主干流程均已完整支持，更多针对 PhotoCraft 的深度适配与优化将持续迭代更新。

---

## 👨‍💻 Author & Community / 关于作者与交流

**ComfyPanel** 由 **[Gino @ LAZYet](https://lazyet.com)** 独立设计与开发。

- 💎 **[Upgrade to ComfyPanel Pro / 升级至 Pro 专业版](https://lazyet.com/products/comfypanel-pro)**
- 🛍️ **[Explore our Physical Designs at LAZYet Studio / 探索我们的物理设计产品](https://lazyet.com)**
- 🐛 **[Submit Feedback & Bug Reports / 提交反馈与 Bug 报告 (GitHub)](https://github.com/Ginolazy/ComfyPanel-for-PhotoCraft/issues)**

<p align="center">
  <img src="https://github.com/user-attachments/assets/775d47e7-cb82-4f71-a5b1-2726159592b8" alt="ComfyPanel 交流群" width="220" />
</p>

---

## ⚖️ License / 许可证

This project uses a **Dual License** model / 本项目采用 **双轨许可** 模式：

| Component / 组件 | Paths / 对应路径 | License / 许可条款 |
| :--- | :--- | :--- |
| **Tauri Bridge & Host Wrapper** | `/src-tauri/`, `/scripts/`, `build_photocraft.py` | MIT License — Free to use, modify & distribute |
| **ComfyPanel Web UI & Core Logic** | `/src/` | Proprietary — Free Community Edition forever; Pro requires license; No redistribution or reverse engineering |

See [LICENSE](./LICENSE) for full legal terms.

---
*Crafted slowly. Shared freely. 2026 © [LAZYet](https://lazyet.com)*



