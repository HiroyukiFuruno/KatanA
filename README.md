<p align="center">
  <img src="assets/icon.iconset/icon_128x128.png" width="128" alt="KatanA Desktop">
</p>

<h1 align="center">KatanA Desktop</h1>

<p align="center">
  A fast, lightweight Markdown workspace for macOS, Windows, and Linux — built with Rust and egui.
</p>

<p align="center">
  <strong><a href="https://katana-desktop.katana-projects.org/">🌐 Official Website</a></strong> | <strong><a href="https://katana-desktop.katana-projects.org/docs/getting-started">📚 Documentation</a></strong>
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License: MIT"></a>
  <a href="https://github.com/HiroyukiFuruno/KatanA/actions/workflows/test-and-build.yml"><img src="https://github.com/HiroyukiFuruno/KatanA/actions/workflows/test-and-build.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/HiroyukiFuruno/KatanA/releases/latest"><img src="https://img.shields.io/github/v/release/HiroyukiFuruno/KatanA" alt="Latest Release"></a>
  <a href="https://github.com/HiroyukiFuruno/homebrew-katana"><img src="https://img.shields.io/badge/homebrew-cask-orange?logo=homebrew" alt="Homebrew"></a>
  <img src="https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey" alt="Platform: macOS | Windows | Linux">
</p>

<p align="center">
  English | <a href="README.ja.md">日本語</a>
</p>

---

## What is KatanA

The name **KatanA** comes from the Japanese word **"刀" (katana)** — a razor-sharp blade forged with precision and purpose.

Just as a katana cuts cleanly through its target, this tool is designed to **slice through the complexity of documentation workflows** with speed and clarity. The name reflects the creator's desire to build something that helps developers **cut through challenges decisively**, one problem at a time.

KatanA Desktop is a fast, lightweight Markdown workspace for macOS, Windows, and Linux, purpose-built for developers who work with specification documents and technical writing.

The trailing uppercase **A** stands for **"AI"**and**"Agent"**— KatanA is designed for the era of AI agent-assisted development, where Markdown specifications serve as the bridge between humans and AI.**Katana × AI Agent = KatanA.**

---

## Background

As of **2026**, software development is rapidly evolving with the rise of **AI agents** assisting engineers in writing, reviewing, and maintaining code.

Alongside this shift, **Spec-Driven Development (SDD)** is gaining attention as a development methodology where specifications, architecture descriptions, and tasks are defined before implementation. These specifications are typically written in **Markdown documents** and serve as the central source of truth for both developers and AI agents.

This repository is operated with an AI-agent-first workflow in mind.
Repository-local skills are maintained canonically under `.agents/skills/`.
If another AI agent expects a different skill directory layout, copy the same skill content from `.agents/skills/` into that agent's expected path instead of maintaining a separate variant.

KatanA itself is also developed through a **Codex-centered AI-agent workflow**.
We primarily use **[Codex](https://openai.com/codex)** and combine it with other AI agents where they fit best, such as implementation, review, investigation, documentation, and release preparation.
This keeps the project aligned with the same human-and-agent collaboration model that KatanA is built to support.

### Mandatory Tools for AI-Agent Workflow

To minimize token consumption and manage context window bloat, the use of **[rtk](https://github.com/fission-ai/rtk)** is **mandatory** for all AI agents working in this repository.

AI agents MUST wrap noisy commands (tests, linters, builds) with `rtk` to ensure outputs are filtered and summarized before being ingested into the context.

However, existing Markdown tools are often either:

- general-purpose editors not optimized for technical documentation workflows, or
- heavy knowledge-management tools with unnecessary complexity.

# KatanA Desktop was created to solve this problem

The goal of KatanA is to provide a **simple, fast, and workspace-oriented Markdown environment** where developers can easily **browse and edit documentation used in SDD workflows**.

---

## Features

- **Live split-view preview** — Edit on the left, rendered HTML on the right, scroll-synced
- **Diagram rendering** — First-class support for Mermaid, PlantUML, and Draw.io fenced code blocks
- **GitHub Flavored Markdown** — Tables, strikethrough, task lists, footnotes, autolinks
- **Workspace-aware** — Open a folder and navigate files from the integrated file tree
- **Tab bar** — Multiple documents open simultaneously with VSCode-style tabs
- **i18n** — UI strings fully localised (English / Japanese bundled)
- **Fast native performance** — Built with Rust and egui, no Electron, no Node.js — just a single native binary

---

## Installation

> Available on macOS (Apple Silicon & Intel), Windows, and Linux.

### macOS

#### Homebrew (Recommended for macOS)

```sh
brew tap HiroyukiFuruno/KatanA && brew install --cask katana-desktop
```

No additional steps required — the app is ready to use immediately.

#### Manual Download (macOS)

1. Go to the [Releases page](https://github.com/HiroyukiFuruno/KatanA/releases/latest)
2. Download the latest `KatanA-Desktop-x.y.z.dmg`
3. Open the DMG and drag **KatanA Desktop.app** into your **Applications** folder
4. Run the following command once to allow the app to launch:

```sh
xattr -cr /Applications/KatanA\ Desktop.app
```

> **Note:** On macOS Sequoia (15.x), Gatekeeper requires this command for apps not notarized with Apple.
> Alternatively, go to **System Settings → Privacy & Security → "Open Anyway"** after the first launch attempt.

### Windows

#### Winget (Recommended)

On Windows 10 / 11, you can easily install KatanA Desktop using the official `winget` package manager:

```powershell
winget install HiroyukiFuruno.katana-desktop
```

#### Manual Download (Windows)

You can also download the files manually from the [Releases page](https://github.com/HiroyukiFuruno/KatanA/releases/latest).

- **Portable ZIP**: Extract the archive to your preferred location and run `katana-desktop.exe`.
- **MSI Installer**: Automatically creates Start Menu and Desktop shortcuts. (Note: Since the MSI is not code-signed, Windows SmartScreen may display a warning. Select "More info" -> "Run anyway" to proceed).

### Linux

#### Homebrew (Recommended)

```sh
brew tap HiroyukiFuruno/KatanA && brew install katana-desktop
```

#### Manual Download (Linux)

1. Go to the [Releases page](https://github.com/HiroyukiFuruno/KatanA/releases/latest)
2. Download the `KatanA-linux-x86_64.tar.gz`
3. Extract the archive and run the executable directly

### Updating KatanA

KatanA features a built-in auto-updater. Once installed (whether via Homebrew or manually), KatanA will notify you when a new release is available.

- **macOS:**You can update the app directly from within the UI by clicking the**"Update & Restart"** button. If you initially installed KatanA via Homebrew, the first time you update from within the app, KatanA will automatically untap itself from Homebrew to prevent double-management issues.
- **Windows / Linux:** The update dialog will provide a direct link to download the latest asset from the release page.

---

## Current Status

KatanA Desktop is under **active development**. See the [Releases page](https://github.com/HiroyukiFuruno/KatanA/releases/latest) for the latest version and changelog.

Core features available today:

- Workspace-based Markdown browsing
- Diagram support (Mermaid / PlantUML / draw.io)
- Split preview with synchronized scrolling
- Fast native desktop performance (Rust-based)

The project is evolving rapidly — new features and improvements are released frequently.

---

## Diagram Rendering Setup

KatanA supports Mermaid, PlantUML, and Draw.io diagrams in Markdown fenced code blocks. External tools are required only for Mermaid and PlantUML rendering.

### Mermaid

Install the Mermaid CLI globally:

```sh
npm i -g @mermaid-js/mermaid-cli
```

Once installed, `mmdc` will be detected automatically. No additional configuration is needed.

### PlantUML

1. Install a Java runtime:

```sh
brew install openjdk@25
```

1. Open a Markdown file containing a PlantUML block in KatanA and click the **⬇ Download** button that appears in the preview pane. The `plantuml.jar` will be downloaded automatically.

### Draw.io

Draw.io diagrams are rendered natively — no external tools required.

---

## Project Goals

KatanA aims to become a tool that helps developers:

- Read and navigate Markdown documentation efficiently
- Work with specification-driven workflows
- Integrate documentation with modern AI-assisted development

The long-term vision is to build a **lightweight documentation workspace** that complements modern development tools.

---

## We Want Your Ideas

This project is still in its early stages. We welcome features ideas, usability suggestions, and bug reports.
If you have thoughts on how KatanA can improve the documentation workflow for developers, please open an [issue](https://github.com/HiroyukiFuruno/KatanA/issues) or [discussion](https://github.com/HiroyukiFuruno/KatanA/discussions).

---

## For Contributors

If you would like to help shape the future of KatanA, please read our:

- 🤝 **[Contributing Guide](CONTRIBUTING.md)** — How to join discussions, provide design feedback, and collaborate via AI-agent-assisted workflows.

---

## Open Source Commitment

KatanA Desktop is an open source project.

We are committed to keeping the **core functionality available for free**, especially features that do not incur ongoing operational costs.

These include:

- Markdown viewing
- Workspace navigation
- Documentation browsing
- Diagram rendering

---

## Future Plans

Some advanced features may require external services or operational costs.

For sustainability, the project may introduce:

- optional paid features (e.g., AI-assisted tools)
- small advertisements within the application

However, the **core documentation functionality will remain free**.

---

## For Developers

If you want to build from source, contribute, or understand the architecture:

- 📖 **[Development Guide](docs/development-guide.md)** — Setup, build, test, and project structure
- 📐 **[Coding Rules](docs/coding-rules.md)** — Code style, conventions, and quality gates
- 🏗️ **[Architecture Decisions](docs/adr/)** — Design rationale and ADRs

---

## Support the Project

If you find KatanA useful and would like to support its development, you can do so through sponsorship.

<a href="https://github.com/sponsors/HiroyukiFuruno"><img src="https://img.shields.io/badge/Sponsor-❤️-ea4aaa?style=for-the-badge&logo=github-sponsors" alt="Sponsor"></a>

Support helps cover:

- development time
- infrastructure
- tooling costs

👉 **[Become a Sponsor](https://github.com/sponsors/HiroyukiFuruno)**

- ⭐ Star this repository — it helps others discover KatanA
- Share KatanA with people who might find it useful

---

## Appendix: Windows Environment Setup

Installation instructions for external tools required for diagram rendering (Mermaid / PlantUML) on Windows.

<details>
<summary>Installing Node.js (Required for Mermaid CLI)</summary>

Node.js is required to run the Mermaid CLI (`mmdc`).

```powershell
winget install -e --id OpenJS.NodeJS.LTS
```

After installation, open a new terminal and run:

```powershell
npm i -g @mermaid-js/mermaid-cli
```

> **If you encounter errors:** If `winget` fails, please download and install Node.js directly from the [Official Node.js Website](https://nodejs.org/).

</details>

<details>
<summary>Installing JDK (Required for PlantUML)</summary>

A Java runtime is required to render PlantUML diagrams.

```powershell
winget install -e --id Oracle.JDK.25
```

> **If you encounter errors:** Errors may occur if your `winget` sources are outdated. Keep your sources updated and retry:
>
> ```powershell
> winget source update
> winget install -e --id Oracle.JDK.25
> ```

</details>

---

## License

KatanA Desktop is released under the [MIT License](LICENSE).
