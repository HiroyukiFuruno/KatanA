# KatanA

> A fast, keyboard-driven Markdown editor for macOS, Windows, and Linux — built with Rust and egui.

KatanA is a native desktop application for writing and previewing Markdown documents.
It renders your content in real time alongside your editor, supports diagram-as-code
(Mermaid, PlantUML, Draw.io), and keeps the entire experience lightweight and snappy —
no Electron, no Node.js, just a single native binary.

---

## Features

- **Live split-view preview** — Edit on the left, rendered HTML on the right, scroll-synced
- **Diagram rendering** — First-class support for Mermaid, PlantUML, and Draw.io fenced code blocks
- **GitHub Flavored Markdown** — Tables, strikethrough, task lists, footnotes, autolinks
- **Workspace-aware** — Open a folder and navigate files from the integrated file tree
- **Tab bar** — Multiple documents open simultaneously with VSCode-style tabs
- **i18n** — UI strings fully localised (English / Japanese bundled)
- **Plugin system** — Extensible renderer extension points for future diagram backends
- **AI integration hooks** — Provider registry scaffold for future AI-assisted workflows
- **Cross-platform native** — Native menu bar (macOS), in-app command UI (Windows/Linux), CJK font support

---

## Getting Started (for Users)

> Supported on macOS (Apple Silicon & Intel), Windows, and Linux.

Pre-built binaries are not yet available. Please build from source (see below).

---

## Development Setup

### Prerequisites

- macOS 13 Ventura or later, Windows 10/11, or Linux (e.g. Ubuntu 22.04+)
- **macOS:** [Homebrew](https://brew.sh) (the setup script will install it if missing)
- **Windows / Linux:** Standard build tools (`build-essential` on Ubuntu, Visual Studio Build Tools on Windows)

### One-command Setup

Run `make init` from the project root. It will check for and install every
required tool interactively:

```sh
make init
```

`make init` is the documented entry point and currently delegates to `scripts/setup.sh`.

The script installs:

| Tool | Purpose |
| --- | --- |
| **Homebrew** | Package manager (macOS/Linux) |
| **git** (latest) | Version control |
| **rustup** | Rust toolchain manager |
| **Rust stable** + clippy / rustfmt / llvm-tools | Compiler and linters |
| **cargo-llvm-cov** | Line-coverage gate (100% enforced in CI) |
| **cargo-watch** | Auto-rebuild on file changes |
| **cargo-outdated** | Detect stale dependencies |
| **cargo-bloat** | Binary size analysis |
| **tokei** | Source line count |
| **lefthook** | Git hook runner (pre-commit + pre-push) |

### Common `make` Commands

```sh
make init         # Bootstrap the development environment interactively
make run          # Build and launch KatanA
make test         # Run the full test suite
make check-light  # Full CI check: fmt + clippy + tests
make watch-run    # Launch with auto-reload on file changes
make doc-open     # Build and open API docs in your browser
```

Run `make help` for a complete list.

### Project Structure

```text
katana/
├── crates/
│   ├── katana-core/      # Markdown pipeline, document model, workspace, AI registry
│   ├── katana-ui/        # egui application shell, preview pane, i18n, snapshots
│   └── katana-platform/  # Filesystem abstraction, settings persistence
├── scripts/
│   └── setup.sh          # Implementation behind `make init`
├── docs/                 # Architecture decisions, coding rules
├── fixtures/             # Test fixtures (sample Markdown files)
└── Makefile              # Developer task runner
```

### Quality Gates

| Gate | Trigger | Checks |
| --- | --- | --- |
| pre-commit | Every `git commit` | `cargo fmt --check`, `cargo clippy -D warnings` |
| pre-push | Every `git push` | Full test suite + `cargo llvm-cov` (100% line coverage) |
| CI | Every PR / push to `master` | fmt · clippy · tests · coverage · CodeQL security scan |

---

## Contributing

KatanA is in early, active development. All contributions are welcome.

### Ways to Contribute

- **Bug reports** — Open an [issue](../../issues) with a description and reproduction steps
- **Feature requests** — Open an issue to discuss ideas before writing code
- **Code** — Fork the repo, implement your change on a branch, and open a pull request
- **Documentation** — Corrections, clarifications, and translations are always appreciated

### Pull Request Guidelines

1. Fork the repository and create a branch from `master`
2. Ensure `make check` passes completely before opening a PR
3. Write tests for new behaviour (coverage gate: 100% lines)
4. Keep commits focused and atomic; write clear, descriptive messages
5. Open a draft PR early if you want feedback on an approach

### Code Style

The project enforces strict formatting and linting automatically via lefthook.
The key rules are:

- **rustfmt** — `max_width = 100`, Unix newlines (enforced by `cargo fmt`)
- **clippy** — All warnings are errors (`-D warnings`); function body ≤ 30 lines; cognitive complexity ≤ 10
- **i18n** — All user-visible strings must go through the `i18n!()` macro — no hardcoded UI text
- See `docs/coding-rules.md` for the complete coding standards

---

## Support & Sponsorship

### Sponsoring

Sponsor support is coming soon. If you would like to support the development of
KatanA financially, please check back later or watch the repository for announcements.

<!-- TODO: add GitHub Sponsors / Open Collective links once configured -->

### Donations

A donation page is currently in preparation. Thank you for your patience.

<!-- TODO: add donation link -->

### Other Ways to Support

- ⭐ Star this repository — it helps others discover KatanA
- Share KatanA with people who might find it useful
- Report bugs and suggest improvements through issues

---

## License

KatanA is released under the [MIT License](LICENSE).

---

## Cross-Platform Validation & Release Blockers

Because KatanA is primarily developed on macOS, maintaining stability across Windows and Linux requires strict verification gates.

### 1. CI Validation (Automated)

The GitHub Actions CI (`.github/workflows/ci.yml`) runs a cross-platform matrix (`macos-latest`, `windows-latest`, `ubuntu-latest`).
**Release Blocker:** A release must NOT be published if any of the CI jobs in the matrix fail. The `make check` equivalent must pass on all platforms.

### 2. Runtime Smoke Checklist (Manual)

Before a minor or major release, the maintainer must verify the compiled Windows (`KatanA-windows-x86_64.zip`) and Linux (`KatanA-linux-x86_64.tar.gz`) binaries. This can be done via VMs (e.g., Parallels/UTM) or physical machines.

#### Required Evidence for Windows and Linux

1. **Initial Launch:** The executable launches without crashing and displays the default interface.
2. **Workspace Open:** Successfully open a local folder via the in-app command UI "Open Workspace" button.
3. **Markdown Edit:** Edit an existing Markdown file and observe the preview updating in real-time.
4. **Settings / Update:** Invoke the settings/update dialog and confirm the OS-appropriate instructions are shown (and no macOS-specific behavior leaks).

**Release Blocker:** Failure in any of the Runtime Smoke Checklist items on Windows or Linux blocks the release. Automated CI alone is not sufficient to guarantee UI integrity.
