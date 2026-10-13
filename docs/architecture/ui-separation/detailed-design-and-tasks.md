# KatanA UI 分離 詳細設計・超細分化 Task

作成日: 2026-05-16  
更新日: 2026-05-17  
対象: KatanA ecosystem repositories  
基準文書: [`principles.md`](principles.md)

---

## 0. Document distribution policy

本ドキュメント (master) は **KatanA repo の `docs/architecture/ui-separation/`** に置く canonical 配置とする。各 repo には自分の Phase だけを切り出した抜粋を配置し、repo owner は抜粋を読めば自分の作業が完結する。

### canonical 配置

```
katana/docs/architecture/ui-separation/
  ├─ README.md                          # overview + 各 repo へのリンク
  ├─ principles.md                      # 設計原則
  └─ detailed-design-and-tasks.md       # 本ファイル (master)
```

### 各 repo 抜粋配置 (1 repo 1 ファイル)

| repo | 配置先 | 抜粋内容 |
| --- | --- | --- |
| `katana-ui-core` | `docs/ui-separation-plan.md` | Phase 1 (neutral core) + P4-0 (primary / 互換 adapter) + P1-K |
| `katana-document-viewer` | `docs/ui-separation-plan.md` | Phase 2 (viewer + forge) + Phase 7 (KDV 側視点) |
| `katana-language-editor` | `docs/ui-separation-plan.md` | Phase 3 (domain + ports + md adapter) |
| `katana-canvas-forge` | `docs/ui-separation-plan.md` | Phase 7 (KCF 側) + Phase 2 backend integration |
| `katana-diagram-renderer` | `docs/ui-separation-plan.md` | Phase 7 (KDR pure renderer 方針) |
| `katana-markdown-model` | `docs/ui-separation-plan.md` | Phase 6 (KMM canonical) + KME compatibility |
| `katana-chat-ui` | `docs/ui-separation-plan.md` | Phase 4 ChatPane 接続 (後続) |
| `katana` (KatanA 本体) | `docs/architecture/ui-separation/phase-5-katana-integration.md` | Phase 5 (KatanA integration) |

### 抜粋ファイルの構成

- repository's role: 該当 repo の本構想における役割
- 該当 Phase 一覧 + master へのリンク
- Phase 概要 (master 該当 Phase の抜粋)
- タスク全リスト (master から該当 Phase を mechanical に抜粋)
- 前提 (depends on) / 出力 (provides) — master の 6.5 Phase 依存グラフから抜粋
- Done criteria
- 末尾に canonical へのリンク + drift 検出方針

### 同期方針

- 抜粋ファイルは master が更新されたら反映する。drift を防ぐため、master と抜粋ファイルの両方に同じ task ID を持たせる。
- 抜粋ファイル単独で task を追加・修正してはならない。必ず master を先に更新する。
- master 更新 PR では、影響する各 repo の抜粋ファイル更新 PR も追従する。

---

## 1. 結論

今回の設計変更では、既存の **Floem 前提の UI 部品分離**をやめ、`katana-ui-core` を **framework-neutral な atoms / molecules UI Core** として再定義する。

`Floem`、`GPUI`、将来の `native-renderer` は core 依存ではなく adapter 対象にする。

`katana-document-viewer` は単なる preview crate ではなく、以下を所有する文書アーティファクト基盤にする。

- document view
- live preview
- artifact view
- scroll sync
- forge: build / transform / export / artifact generation
- CLI から呼び出せる document pipeline API

既存の `katana-canvas-forge` は即時削除せず、まず `katana-document-viewer::forge` の backend として取り込み、最終的に public API の所有権を `katana-document-viewer` へ移す。

実装順は以下に固定する。

```text
1. katana-ui-core
2. katana-document-viewer
   └─ forge を内部サブシステムとして含める
3. katana-language-editor
4. katana-ui
5. katana
```

---

## 1. 現存 repository 解析

### 1.1 `HiroyukiFuruno/KatanA`

#### 現状

- workspace は `katana-core`、`katana-linter`、`katana-platform`、`katana-ui` で構成されている。
- Rust edition は 2024、MSRV は `1.95.0`。
- UI は `eframe` / `egui` / `egui_commonmark` 依存。
- Markdown / diagram / lint 系として `katana-canvas-forge`、`katana-diagram-renderer`、`katana-ast-lint`、`katana-markdown-linter` を利用している。
- README 上の product scope は、workspace-based Markdown browsing、diagram support、split preview、scroll sync、native desktop performance。
- `crates/katana-ui` は `katana-core` / `katana-platform` / egui 系に強く依存しており、現状では UI shell と application state が密結合している。
- `KatanaApp` は `PreviewPane` cache、export task、dialog、update、cursor range などを同一 struct に保持しており、UI surface と application orchestration が混ざっている。

#### 設計上の意味

`KatanA` 本体は最終段階で薄くする。現在の `katana-ui` は UI shell 兼 application presenter なので、段階的に以下へ分離する。

```text
KatanA/crates/katana-ui 現状
  ├─ egui shell
  ├─ preview pane
  ├─ editor pane
  ├─ export task
  ├─ update dialog
  ├─ file dialog
  ├─ linter doc cache
  └─ app state wiring

移行後
  ├─ katana         : 起動 / 設定 / 永続化 / app boundary
  ├─ katana-ui      : MainPanel / Dock / Pane composition
  ├─ katana-ui-core
  ├─ katana-document-viewer
  └─ katana-language-editor
```

#### 主要リスク

- `katana-ui` に残る egui 依存を一括削除すると破壊範囲が大きい。
- preview / export / diagram / editor の責務が既に本体 UI に散っているため、型境界を先に作らないと移行が進まない。
- `egui_commonmark` vendor patch が残る限り、preview 層の独立性が弱い。

---

### 1.2 `katana-ui-core`

#### 現状

- workspace member は `crates/katana-ui-core`。
- package description は「Floem向け共通UI widget の共通基盤」。
- dependency として `floem = 0.2.0`、`floem_reactive = 0.2.0`、`floem_renderer = 0.2.0` が hard dependency になっている。
- `src/lib.rs` は `composite`、`layout`、`primitive`、`theme` を公開しつつ、内部に `floem_view`、`overlay_lifecycle` を持つ。
- Justfile は厳格で、format、type check、Clippy、AST lint、storybook、overlay lifecycle、menu button contract、coverage、release verify を持つ。

#### 設計上の意味

今回の設計変更で最も大きく変わる repository。

現状は `Floem 向け widget crate` だが、移行後は以下に変える。

```text
katana-ui-core
  ├─ framework-neutral public core
  ├─ atoms / molecules
  ├─ theme token
  ├─ layout primitive
  ├─ interaction model
  ├─ render model / view model
  └─ adapter crates or feature-gated adapter modules
```

#### 変更方針

- `floem` hard dependency を core crate から外す。
- Floem 実装は `katana-ui-core-floem`、または `adapter::floem` feature に移す。
- public API は `Floem View` を返さず、Katana UI Core の view model / component model を返す。
- `primitive` / `layout` / `composite` / `theme` の概念は維持するが、型を neutral 化する。
- Storybook は adapter 経由で動かす。core の品質ゲートは adapter に依存させない。

---

### 1.3 `katana-document-viewer`

#### 現状

- workspace members は `kdp-linter`、`katana-document-preview`、`katana-document-preview-egui`。
- 現在の中核 crate 名は `katana-document-preview` で、description は `Vendor-neutral Markdown preview interface`。
- `MarkdownPreview` trait は `MarkdownSource` と `PreviewConfig` を受け、`PreviewOutput` を返す。
- `RenderTarget` は neutral trait だが、まだ marker に近い。
- `PreviewOutput` は `scroll_offset`、`content_height`、`diagnostics` を持つ。
- egui 実装は scaffold 状態で、`PreviewError::NotImplemented` を返す。

#### 設計上の意味

現状は **preview interface** として始まっているが、今回の設計では **document artifact subsystem** に拡張する。

既存の `preview` は `viewer_surface` の一機能に格下げする。

```text
katana-document-viewer
  ├─ document_model
  ├─ artifact_model
  ├─ viewer_surface
  ├─ forge
  ├─ cli_api
  └─ adapter
```

#### 変更方針

- crate 名の public 概念を `preview` から `viewer` に寄せる。
- `katana-document-preview` は互換 facade として残し、最終的には deprecated にする。
- `katana-document-viewer` crate を新設するか、既存 `katana-document-preview` を内部 module 化する。
- `forge` はここに内包する。
- `katana-canvas-forge` は短期的には forge backend として委譲し、長期的には public API を `katana-document-viewer::forge` に一本化する。

---

### 1.4 `katana-language-editor`

#### 現状

- workspace members は `kle-linter`、`katana-language-editor`、`katana-language-editor-egui`。
- neutral crate は `TextContent`、`CursorPosition`、`Selection`、`EditorConfig`、`EditorEvent`、`EditorOutput`、`LanguageEditor` trait を持つ。
- egui 実装は `egui::TextEdit` を使う scaffold。
- `cursor()` は現時点で default を返している。

#### 設計上の意味

方向性は既に正しい。

KLE は **language-neutral な編集ドメイン**を所有し、markdown / Rust / TypeScript など特定言語に縛られない。`katana-language-editor` 自身は syntax highlight / jump / hover / completion / formatter を実装せず、**外部から inject される port (trait)** として宣言する。

editor domain としてはまだ interface が浅いため、以下を追加する。

- buffer identity
- revision
- incremental edit
- diagnostics 受け取り model
- selection range
- scroll request
- IME / composition event
- editor command
- 外部 port group:
  - `SyntaxHighlightProvider`
  - `JumpProvider`
  - `HoverProvider`
  - `CompletionProvider`
  - `FormatterProvider`
  - `DiagnosticsSource`
- port から受け取る view model DTO (syntax token / hover content など)
- source anchor DTO（言語非依存。markdown / KMM は anchor adapter として接続）

#### 変更方針

- `katana-language-editor` は UI framework を知らないまま維持する。
- `katana-ui-core` の component model には依存しない。
- UI adapter は `katana-ui` 側、または adapter crate 側で行う。
- `katana-markdown-model` (KMM) を含む特定言語 model に依存しない。markdown 利用は KatanA / KDV 側で port adapter を提供する形に閉じる。
- source mapping は KMM 直結ではなく、`SourceAnchorAdapter` trait 経由で受け取る。markdown 用 adapter のみ KMM に依存できる。
- default language implementation は提供しない。利用側 (KatanA など) が必ず port を inject する。

---

### 1.5 `katana-canvas-forge`

#### 現状

- workspace members は `kcf-linter`、`katana-canvas-forge`、`katana-canvas-forge-cli`。
- description は diagram rendering and document export runtime。
- `lib.rs` は Mermaid / Draw.io rendering、HTML / PDF / PNG / JPEG export responsibilities を所有すると説明している。
- CLI は `mermaid`、`drawio`、`export`、`export-debug` を持つ。
- Justfile では dependency leak guard があり、`egui`、`katana-core`、`katana-ui`、`katana-platform`、`katana-native` の leak を禁止している。
- Mermaid / Draw.io / ZenUML / Playwright / runtime asset checksum / reference compare などの品質ゲートが厚い。

#### 設計上の意味

`forge` という名前と責務は既にここに存在する。

ただし今回の方針では、public subsystem owner は `katana-document-viewer` にする。

そのため、`katana-canvas-forge` は短期的に以下の役割へ変える。

```text
katana-document-viewer::forge  // public owner
  └─ katana-canvas-forge       // backend / compatibility runtime
```

#### 変更方針

- KCF の品質ゲートと runtime asset 管理は維持する。
- KDV に `forge` facade を作り、最初は KCF を呼び出す。
- KCF CLI は互換 CLI として残すが、将来的には `katana-document-viewer::cli_api` へ委譲する。
- KCF の export model と KDV の artifact model を揃える。
- KCF の public API は段階的に縮小する。

---

### 1.6 `katana-diagram-renderer`

#### 現状

- workspace members は `kdr-linter`、`katana-diagram-renderer`、`katana-diagram-renderer-cli`。
- description は versioned diagram rendering runtime。
- `lib.rs` では Mermaid / Draw.io / ZenUML rendering を所有し、document export と viewer ownership は deliberately exclude とされている。

#### 設計上の意味

KDR は `forge` の純粋な diagram backend として扱いやすい。

KCF と KDR の責務が近いため、将来的には以下の整理が必要。

```text
katana-diagram-renderer
  └─ diagram rendering only

katana-document-viewer::forge
  └─ document artifact build / transform / export
```

KCF はこの間の compatibility layer として扱う。

---

### 1.7 `katana-markdown-model` / `katana-markdown-engine`

#### 現状

- `katana-markdown-model` は renderer-neutral Markdown document model を名乗り、KatanA viewers / editors / export flows が共有する common interpretation layer と説明している。
- `katana-markdown-engine` もほぼ同じ責務説明を持つ。
- KatanA 側 OpenSpec には `adopt-kme-in-katana` があり、KME 名で統合計画が残っている。

#### 設計上の判断

canonical は `katana-markdown-model` に寄せる。

理由は、今回の責務分離では「engine」より「model」の方が責務名として狭く、`forge` や `viewer` と衝突しにくいため。

```text
katana-markdown-model  // canonical
katana-markdown-engine // migration / compatibility / deprecation target
```

#### 変更方針

- KatanA OpenSpec の `KME` 表記を `KMM` に読み替える変更を作る。
- `katana-document-viewer::forge` は KMM の document model を input として扱う。
- `katana-language-editor` は KMM の source position / node id を editor sync に使う。
- `katana-ui-core` は KMM 型に直接依存しない。表示 DTO のみ受ける。

---

### 1.8 `katana-chat-ui`

#### 現状

- framework-neutral AI chat UI state と lightweight render model を持つ。
- host applications render that model with their own UI framework という方針。

#### 設計上の意味

今回の `katana-ui-core` 方針と親和性が高い。

ただし、今回の主軸は widget / document viewer / editor / katana-ui なので、chat は後続で `katana-ui` の Panel として接続する。

---

## 2. 目標アーキテクチャ

```text
katana
  └─ app boundary
      ├─ startup
      ├─ config
      ├─ project state
      ├─ persistence
      ├─ command dispatch
      └─ plugin boundary

katana-ui
  └─ Katana screen composition
      ├─ MainPanel
      ├─ Dock
      ├─ TabBar
      ├─ Toolbar
      ├─ EditorPane adapter
      ├─ DocumentViewerPane adapter
      └─ ChatPane adapter

katana-ui-core
  └─ generic UI core
      ├─ atoms
      ├─ molecules
      ├─ layout primitives
      ├─ theme tokens
      ├─ event model
      ├─ render model
      └─ adapters
          ├─ floem adapter
          ├─ gpui adapter
          └─ native renderer adapter

katana-language-editor
  └─ editor domain
      ├─ buffer
      ├─ cursor
      ├─ selection
      ├─ diagnostics
      ├─ edit command
      └─ editor event

katana-document-viewer
  └─ document artifact subsystem
      ├─ document model integration
      ├─ artifact model
      ├─ viewer surface
      ├─ scroll sync
      ├─ forge
      │   ├─ build
      │   ├─ transform
      │   ├─ export
      │   └─ artifact generation
      └─ cli_api

katana-markdown-model
  └─ renderer-neutral Markdown interpretation

katana-diagram-renderer
  └─ diagram rendering backend
```

---

## 3. 依存方向

### 3.1 許可する依存

```text
katana -> katana-ui
katana -> katana-core
katana -> katana-platform

katana-ui -> katana-ui-core
katana-ui -> katana-language-editor
katana-ui -> katana-document-viewer
katana-ui -> katana-chat-ui

katana-document-viewer -> katana-markdown-model
katana-document-viewer -> katana-diagram-renderer
katana-document-viewer -> katana-canvas-forge   // transitional backend only

// katana-language-editor (core) は KMM / markdown 系 crate に一切依存しない
katana-language-editor-md -> katana-language-editor + katana-markdown-model

katana-ui-core -> serde / thiserror / small neutral deps only
katana-ui-core-floem -> katana-ui-core + floem     // primary adapter 候補
katana-ui-core-egui  -> katana-ui-core + egui      // 互換 adapter
katana-ui-core-gpui  -> katana-ui-core + gpui      // 互換 adapter

katana-language-editor-floem -> katana-language-editor + floem  // primary adapter 候補
katana-language-editor-egui  -> katana-language-editor + egui   // 互換 adapter
katana-language-editor-gpui  -> katana-language-editor + gpui   // 互換 adapter
```

### 3.2 禁止する依存

```text
katana-ui-core -> katana
katana-ui-core -> katana-ui
katana-ui-core -> katana-language-editor
katana-ui-core -> katana-document-viewer
katana-ui-core -> katana-markdown-model
katana-ui-core -> floem       // core crateでは禁止
katana-ui-core -> gpui        // core crateでは禁止

katana-document-viewer::forge -> katana-ui
katana-document-viewer::forge -> katana-ui-core
katana-document-viewer::forge -> egui

katana-language-editor -> katana-ui
katana-language-editor -> katana-ui-core
katana-language-editor -> egui
katana-language-editor -> floem                   // core crateでは禁止 (互換 adapter crate のみ可)
katana-language-editor -> gpui                    // core crateでは禁止 (互換 adapter crate のみ可)
katana-language-editor -> katana-markdown-model   // KLE core は language-neutral 維持
katana-language-editor -> markdown 系 crate any   // markdown 用 port impl は katana-language-editor-md に閉じる

katana -> egui_commonmark
katana -> preview implementation internals
katana -> editor implementation internals
```

---

## 4. Package / crate 再構成案

### 4.1 `katana-ui-core`

```text
katana-ui-core/
  Cargo.toml
  crates/
    katana-ui-core/              // framework-neutral core (no floem / no gpui / no egui)
      src/
        lib.rs
        atom/
        molecule/
        layout/
        theme/
        event/
        render_model/
        accessibility/
        adapter_contract/
    katana-ui-core-floem/        // primary adapter 候補 (KatanA 起動先)
    katana-ui-core-egui/         // 互換 adapter (外部利用者向け)
    katana-ui-core-gpui/         // 互換 adapter (外部利用者向け)
    katana-ui-core-storybook/    // adapter smoke / visual catalog
```

公開の中心は `katana-ui-core` core crate。

adapter は性格別に分離する。

- **primary adapter** (例: `katana-ui-core-floem`): KatanA が起動時に使う。core と同等の品質ゲート、全 widget 対応、release 同期。
- **互換 adapter** (例: `katana-ui-core-egui`、`katana-ui-core-gpui`): 外部利用者向け。opt-in feature / 別 crate。品質ゲートは storybook smoke + compile test を最低ライン。対応 widget / 未対応機能は README に明記。SemVer minor で追加・縮小可能。

core crate の compile に floem / gpui / egui を要求しない。互換 adapter 起因の breakage は primary release を止めない。

### 4.2 `katana-document-viewer`

```text
katana-document-viewer/
  Cargo.toml
  crates/
    katana-document-viewer/
      src/
        lib.rs
        source/
        document/
        artifact/
        viewer_surface/
        sync/
        forge/
        cli_api/
        error.rs
    katana-document-viewer-floem/   // future adapter
    katana-document-viewer-egui/    // temporary compatibility adapter only
    katana-document-preview/        // compatibility facade
    katana-document-preview-egui/   // deprecated path
```

### 4.3 `katana-language-editor`

```text
katana-language-editor/
  crates/
    katana-language-editor/          // language-neutral editor domain
      src/
        buffer/
        cursor/
        selection/
        command/
        event/
        diagnostics/
        ports/                       // SyntaxHighlight / Jump / Hover / Completion / Formatter / DiagnosticsSource / SourceAnchorAdapter
        source_anchor/               // SourceAnchor / LineColumnRange / Fingerprint (言語非依存)
    katana-language-editor-md/       // markdown 用 port adapter (任意配置 / KatanA repo 配置でも可)
    katana-language-editor-floem/    // primary adapter 候補
    katana-language-editor-egui/     // 互換 adapter
    katana-language-editor-gpui/     // 互換 adapter
```

`katana-language-editor` core は KMM / markdown を含む特定言語に依存しない。markdown 用の port implementation は `katana-language-editor-md` 等の別 crate (KatanA repo 内配置も可) として提供し、KatanA 側で port を inject する。

framework adapter (floem / egui / gpui) は KUC と同じく primary / 互換の区別で運用する。

### 4.4 `KatanA`

```text
KatanA/
  crates/
    katana-core/
    katana-platform/
    katana-ui/          // MainPanel composition only
    katana-linter/
```

`KatanA/crates/katana-ui` は最終的に external `katana-ui` と同一責務へ寄せる。完全分離するか workspace 内に残すかは後でよいが、責務は今すぐ狭める。

---

## 5. 詳細設計

### 5.1 `katana-ui-core` 詳細設計

#### 5.1.1 Core principles

- Framework-neutral
- Katana domain-neutral
- State-light
- Render-model oriented
- Adapter contract first
- Theme token first
- Accessibility DTO を最初から持つ
- **Application / window / surface API も core が持つ** (widget だけでなく UI Core として、起動から描画まで neutral API で提供する)
- public API は adapter 型 (Floem View / GPUI Element / egui Ui 等) を返さない。常に neutral DTO / trait を介する。

#### 5.1.2 Core modules

```text
runtime
  ├─ Application       (entry point. Application::new().window(...).run())
  ├─ AppConfig         (識別子 / persistence / locale / accessibility 設定)
  ├─ AppHandle         (起動中アプリへの参照。command 送信 / window 取得)
  ├─ AppLifecycle      (start / suspend / resume / shutdown event)
  └─ RuntimeAdapter    (event loop を adapter に委譲する trait)

window
  ├─ Window            (個別 window ハンドル)
  ├─ WindowId
  ├─ WindowConfig      (title / size / min_size / max_size / icon / decorations / fullscreen)
  ├─ WindowEvent       (Close / Resize / Move / Focus / Minimize / Maximize / Restore)
  ├─ WindowCommand     (SetTitle / SetSize / Focus / Minimize / Maximize / Close / Fullscreen)
  ├─ WindowManager     (multi-window 管理 / window 作成 / window list)
  └─ DisplayInfo       (multi-monitor 情報 read-only DTO)

surface
  ├─ Surface           (window 内の描画 root abstraction)
  ├─ FrameHandle
  ├─ PaintRequest
  └─ SurfaceMetrics    (logical size / scale_factor / dpi)

atom
  ├─ Text
  ├─ Icon
  ├─ Button
  ├─ Input
  ├─ Checkbox
  ├─ Radio
  ├─ Badge
  ├─ Divider
  └─ Spacer

molecule
  ├─ Card
  ├─ List
  ├─ Menu
  ├─ Tooltip
  ├─ Modal
  ├─ Tabs
  ├─ Toolbar
  ├─ SplitPane
  └─ FormField

layout
  ├─ Row
  ├─ Column
  ├─ Stack
  ├─ Grid
  ├─ DockSlot
  └─ SizePolicy

theme
  ├─ ColorToken
  ├─ FontToken
  ├─ SpacingToken
  ├─ RadiusToken
  ├─ ShadowToken
  └─ ThemeSnapshot

event
  ├─ UiEvent
  ├─ PointerEvent
  ├─ KeyboardEvent
  ├─ FocusEvent
  └─ CommandEvent

render_model
  ├─ UiNode
  ├─ UiNodeId
  ├─ UiNodeKind
  ├─ UiProps
  └─ UiTree

adapter_contract
  ├─ WidgetAdapter
  ├─ RenderContext
  ├─ EventSink
  └─ HostHandle
```

#### 5.1.3 Adapter design

Adapter は core の `UiTree` / `UiNode` / `ThemeSnapshot` / `EventSink` を各 UI framework に変換する。

```text
katana-ui-core core
  └─ UiTree
      ├─ floem adapter -> Floem View
      ├─ gpui adapter  -> GPUI Element
      └─ native adapter -> native render commands
```

#### 5.1.4 Naming policy

- `Widget` は public product name として維持する。
- 内部実装では `component` / `node` / `render_model` を使う。
- `floem_` prefix は adapter crate 内に閉じ込める。

---

### 5.2 `katana-document-viewer` 詳細設計

#### 5.2.1 Core principles

- Preview is a feature, not the owner.
- Viewer owns artifact lifecycle.
- Forge is UI-free.
- CLI calls the same forge API as GUI.
- Viewer surface displays artifacts; it does not build them.
- Export result is a first-class artifact.

#### 5.2.2 Core modules

```text
source
  ├─ DocumentSource
  ├─ SourceUri
  ├─ SourceKind
  └─ SourceRevision

document
  ├─ DocumentId
  ├─ DocumentKind
  ├─ DocumentSnapshot
  ├─ DocumentOutline
  └─ DocumentMetadataView

artifact
  ├─ ArtifactId
  ├─ ArtifactKind
  ├─ ArtifactFormat
  ├─ ArtifactBytes
  ├─ ArtifactUri
  ├─ ArtifactManifest
  └─ ArtifactDiagnostics

viewer_surface
  ├─ ViewerState
  ├─ ViewerCommand
  ├─ ViewerEvent
  ├─ ViewerViewport
  ├─ PageModel
  ├─ ScrollAnchor
  └─ HighlightRange

sync
  ├─ SourceAnchor
  ├─ ArtifactAnchor
  ├─ ScrollSyncMap
  └─ SyncResolution

forge
  ├─ BuildRequest
  ├─ BuildProfile
  ├─ BuildGraph
  ├─ TransformStep
  ├─ ExportRequest
  ├─ ExportFormat
  ├─ ExportOutput
  ├─ ForgeDiagnostics
  └─ ForgeBackend

cli_api
  ├─ CliRequest
  ├─ CliOutput
  └─ CliDiagnostics
```

#### 5.2.3 Forge pipeline

```text
DocumentSource
  ↓
KMM parse / document model
  ↓
Artifact build graph
  ↓
Diagram / asset resolution
  ↓
Intermediate artifact model
  ↓
Export target
  ├─ preview artifact
  ├─ HTML
  ├─ PDF
  ├─ image
  ├─ Office placeholder / future adapter
  └─ bundle
```

#### 5.2.4 KCF integration

短期的には以下にする。

```text
katana-document-viewer::forge::backend::canvas_forge
  └─ calls katana-canvas-forge
```

長期的には以下に寄せる。

```text
katana-document-viewer::forge
  ├─ diagram backend: katana-diagram-renderer
  ├─ html export backend
  ├─ pdf export backend
  ├─ image export backend
  └─ bundle export backend

katana-canvas-forge
  └─ compatibility wrapper / deprecated public facade
```

#### 5.2.5 CLI policy

- `katana-document-viewer::cli_api` を public にする。
- 既存責務別 CLI は当面残す。
- `katana-canvas-forge-cli` は KDV CLI API への delegate に移行する。
- 将来的な `katana-cli` 統合では、CLI 本体は薄い routing layer に留める。

---

### 5.3 `katana-language-editor` 詳細設計

#### 5.3.1 Core principles

- Editor owns editing domain.
- Editor is **language-neutral**: markdown / Rust / TypeScript など特定言語に縛られない。
- Editor does not implement syntax / jump / hover / completion / formatter. それらは **port (trait)** として宣言し、利用側が inject する。
- Editor は default language implementation を持たない。
- Editor does not depend on `katana-markdown-model` (KMM) など特定言語 model。
- UI owns placement only.
- Editor does not own preview sync policy.
- Editor exposes source anchors and events via neutral DTO.
- Adapter renders editor state.

#### 5.3.2 Required model expansion

```text
buffer
  ├─ BufferId
  ├─ BufferSnapshot
  ├─ BufferRevision
  ├─ TextEdit
  └─ TextRange

cursor
  ├─ CursorPosition
  ├─ CursorAffinity
  └─ CursorRange

selection
  ├─ Selection
  ├─ SelectionMode
  └─ MultiSelection

command
  ├─ EditorCommand
  ├─ InsertText
  ├─ DeleteRange
  ├─ ReplaceRange
  ├─ MoveCursor
  ├─ ApplyFormat
  └─ OpenDocument

event
  ├─ ContentChanged
  ├─ CursorMoved
  ├─ SelectionChanged
  ├─ DiagnosticsChanged
  └─ SourceAnchorChanged

diagnostics
  ├─ Diagnostic
  ├─ DiagnosticSeverity
  ├─ DiagnosticSource
  └─ DiagnosticRange

ports (利用側が inject する trait 群; KLE は default 実装を持たない)
  ├─ SyntaxHighlightProvider
  │   ├─ fn highlight(snapshot, range) -> Vec<SyntaxToken>
  │   └─ SyntaxToken (kind / range / scope)
  ├─ JumpProvider
  │   ├─ fn definition(snapshot, position) -> Option<JumpTarget>
  │   ├─ fn references(snapshot, position) -> Vec<JumpTarget>
  │   └─ JumpTarget (uri / range)
  ├─ HoverProvider
  │   ├─ fn hover(snapshot, position) -> Option<HoverContent>
  │   └─ HoverContent (rich text DTO; framework neutral)
  ├─ CompletionProvider
  │   ├─ fn complete(snapshot, position) -> Vec<CompletionItem>
  │   └─ CompletionItem (label / detail / insert_text)
  ├─ FormatterProvider
  │   └─ fn format(snapshot, range) -> Vec<TextEdit>
  ├─ DiagnosticsSource
  │   └─ subscribe(BufferId) -> Stream<DiagnosticsBatch>
  └─ SourceAnchorAdapter
      ├─ fn resolve(snapshot, position) -> Option<SourceAnchor>
      └─ SourceAnchor (opaque id + LineColumnRange + Fingerprint)

source_anchor (言語非依存。markdown / KMM 接続は SourceAnchorAdapter 実装側で行う)
  ├─ SourceAnchor (opaque)
  ├─ LineColumnRange
  └─ Fingerprint
```

#### 5.3.3 Port wiring 方針

- port trait は KLE crate 内で定義する。
- markdown 用 implementation は KatanA / KDV 側で提供する別 crate (`katana-language-editor-md` 等) として配置候補。KLE 側には含めない。
- KLE の構築 API は port を必須引数として受け取り、port なしでは editor を起動できない作りにする。
- port 未 inject の状態でビルドできてしまうと「default が markdown」のような暗黙仕様が生まれる。これを防ぐため builder で全 port を required にする。
- port を minor で追加するときは default impl を `NoopXxxProvider` として用意し、明示的に渡せるようにする（必須/optional は port 単位で個別に決める）。

---

### 5.4 `katana-ui` 詳細設計

#### 5.4.1 Core principles

- Katana-specific composition only.
- `katana-ui` は **独自 UI 表現 (Component model / DSL)** を持ち、Floem / GPUI / native を adapter (出力先) として後ろに持つ。
- public API は adapter 型 (Floem View / GPUI Element 等) を返さず、neutral な `katana-ui-core` の `UiTree` / `Component model` を返す。
- Does not own widget primitives.
- Does not own editor internals.
- Does not own document build/export logic.
- Receives app state and event handlers from `katana`.
- どの runtime adapter で出力するかは KatanA 側が起動時に選ぶ。`katana-ui` 自体は adapter agnostic。

#### 5.4.2 Adapter strategy: primary と互換

- **primary adapter**: KatanA が起動時に使うもの。当面は 1 系統 (例: `katana-ui-core-floem`) を primary とし、`katana-ui` core と同等の品質ゲートを通す。
- **互換 adapter**: 外部利用者が既存環境に `katana-ui` を差し込むためのもの。`katana-ui-core-egui` / `katana-ui-core-gpui` / `katana-ui-core-floem` を framework 別 crate として併設する (primary に選ばれていないものも維持)。
- `katana-ui` は primary / 互換のどちらにも依存しない。`katana-ui-core` core の `UiTree` を出力する。
- 互換 adapter は opt-in feature / 別 crate のみ。`katana-ui` の compile に引き込まれない。
- 各互換 adapter の対応 widget / 未対応機能 / フォールバック挙動は adapter README に明記する。
- 互換 adapter は SemVer minor で追加・縮小可能。primary adapter の release を互換 adapter が止めない。

#### 5.4.3 MainPanel composition

```text
MainPanel
  ├─ AppFrame
  ├─ TopToolbar
  ├─ LeftSidebar
  ├─ DockLayout
  │   ├─ EditorPane
  │   ├─ DocumentViewerPane
  │   ├─ ChatPane
  │   └─ DiagnosticsPane
  ├─ StatusBar
  └─ ModalHost
```

#### 5.4.4 Pane ownership

```text
EditorPane
  └─ owns placement and KLE adapter call only
      KLE port (syntax / jump / hover / completion / formatter) は KatanA 側で inject
      katana-ui は port を保持しない

DocumentViewerPane
  └─ owns placement and viewer adapter call only

ChatPane
  └─ owns placement and chat render model adapter only
```

---

### 5.5 `katana` 詳細設計

#### 5.5.1 Core principles

- `katana` owns application lifecycle.
- `katana` does not know widget details.
- `katana` does not call forge internals directly unless through command boundary.
- `katana` does not hold framework-specific UI handles.

#### 5.5.2 Responsibilities

```text
katana
  ├─ startup
  ├─ config loading
  ├─ workspace open / close
  ├─ project state
  ├─ persistence
  ├─ command dispatch
  ├─ update orchestration
  ├─ plugin boundary
  └─ passes app state into katana-ui::MainPanel
```

---

## 6. 移行ロードマップ

### Phase 0: governance and freeze

目的: 既存設計のズレを止血する。

- Floem hard dependency を増やさない。
- egui 依存を新規 public API に入れない。
- KDV / KLE / KUC / KCF / KMM の責務表を各 repo README に反映する。
- KMM / KME の canonical 名を KMM に固定する。
- `forge` public owner を KDV に固定する。

### Phase 1: `katana-ui-core` core neutralization

目的: atoms / molecules の UI Core を作る。

- core crate から Floem を外す。
- Floem adapter を分離する。
- UI node / theme token / event model を確定する。
- Storybook は adapter 経由にする。

### Phase 2: `katana-document-viewer` expansion

目的: preview crate から document artifact subsystem へ拡張する。

- `katana-document-viewer` crate を作る。
- `forge` module を作る。
- KCF backend を delegate する。
- artifact model を作る。
- CLI API を作る。

### Phase 3: `katana-language-editor` domain expansion

目的: editor を real domain crate にする。

- buffer / edit / cursor / selection / diagnostics を拡張する。
- source anchor を KMM と接続する。
- egui implementation を compatibility adapter に閉じ込める。

### Phase 4: `katana-ui` composition layer

目的: MainPanel / Dock / Pane の合成に専念させる。

- EditorPane / DocumentViewerPane を adapter 経由にする。
- UI primitive を `katana-ui-core` に寄せる。
- export / preview / editor logic を持たない。

### Phase 5: `KatanA` integration

目的: KatanA 本体を app boundary に戻す。

- direct egui preview dependency を消す。
- direct export task を KDV command に移す。
- KatanA は `katana-ui::MainPanel` を組み込むだけにする。

---

## 6.5 Phase 依存グラフ

各 Phase の入出力と前提を以下に固定する。同じ Phase 番号でも、別 Phase の前提条件を満たしていない限り着手しない。

```text
P0 (governance / naming / ADR)
  ↓ provides: 略語、ADR (KMM canonical, forge owner, KCF transitional)
  │
  ├─→ P1 (KUC neutral core)
  │     ↓ provides: render model / theme token / event model
  │
  ├─→ P2 (KDV viewer + forge facade)
  │     prereq: P0 (forge owner ADR)
  │     scope: KDV 側のみ。KCF 側は変更しない。
  │     ↓ provides: KDV facade, KCF backend adapter, cli_api skeleton
  │
  ├─→ P3 (KLE domain expansion)
  │     prereq: P0
  │     ↓ provides: buffer / cursor / diagnostics / source mapping DTO
  │
  ├─→ P4 (katana-ui composition)
  │     prereq: P1 (render model), 一部 P2 / P3 (pane adapter target shape)
  │     ↓ provides: MainPanel / EditorPane / DocumentViewerPane
  │
  ├─→ P5 (KatanA integration)
  │     prereq: P2 + P3 + P4
  │     特に P5-F-007 (egui_commonmark patch 削除) は P5-B 完了が必須
  │     ↓ provides: preview / export / editor の KDV/KLE 経由化
  │
  ├─→ P6 (KMM canonical)
  │     prereq: P0-B (naming ADR)
  │     ↓ provides: KMM 参照を持つ crate の dependency 整理
  │
  ├─→ P7 (KCF / KDR / KDV forge 再編)
  │     prereq: P2-E (KCF backend adapter 完成)、できれば P5-C (KatanA export migration) 完了
  │     scope: KCF 側 public API 縮小 + CLI delegate 化
  │
  ├─→ P8 (documentation / OpenSpec)
  │     prereq: P0 (naming)、各 Phase の決定が ADR に記録されていること
  │
  └─→ P9 (release strategy)
        prereq: P2 / P3 / P5 / P6 / P7 完了見込みが立っていること
        version 番号は P9-0 (current version inventory) で各 crate の現状版を確認してから決める
```

`prereq` を満たさない状態で次 Phase を進めると、Phase 間で API shape が一致しなくなる。`scope` を超えた変更は別 Phase に分離する。

---

# 7. 超細分化 Task

## P0. 全体 governance / 設計固定

### P0-0. 略語

本セクション以降で使用する略語を以下に固定する。

| 略語 | 展開                       | 役割                                                |
| ---- | -------------------------- | --------------------------------------------------- |
| KUC  | `katana-ui-core`           | framework-neutral UI Core (runtime / window / surface / atoms / molecules) |
| KDV  | `katana-document-viewer`   | document artifact subsystem (viewer + forge + cli)  |
| KLE  | `katana-language-editor`   | editor domain crate                                 |
| KCF  | `katana-canvas-forge`      | transitional diagram / export backend               |
| KDR  | `katana-diagram-renderer`  | pure diagram rendering backend                      |
| KMM  | `katana-markdown-model`    | canonical renderer-neutral Markdown model           |
| KME  | `katana-markdown-engine`   | migration / compatibility target of KMM             |
| KCU  | `katana-chat-ui`           | framework-neutral chat render model                 |

### P0-A. Repository inventory

- [ ] P0-A-001: `KatanA` の workspace members を一覧化する。
- [ ] P0-A-002: `KatanA` の direct UI dependencies を一覧化する。
- [ ] P0-A-003: `KatanA` の direct preview/export dependencies を一覧化する。
- [ ] P0-A-004: `KatanA/crates/katana-ui` 内の egui import 箇所を一覧化する。
- [ ] P0-A-005: `KatanA/crates/katana-ui` 内の preview 関連 module を一覧化する。
- [ ] P0-A-006: `KatanA/crates/katana-ui` 内の export task 関連 module を一覧化する。
- [ ] P0-A-007: `KatanA/crates/katana-ui` 内の editor 関連 module を一覧化する。
- [ ] P0-A-008: `katana-ui-core` の Floem import 箇所を一覧化する。
- [ ] P0-A-009: `katana-document-viewer` の preview naming 箇所を一覧化する。
- [ ] P0-A-010: `katana-language-editor` の egui implementation 箇所を一覧化する。
- [ ] P0-A-011: `katana-canvas-forge` の export model を一覧化する。
- [ ] P0-A-012: `katana-diagram-renderer` と `katana-canvas-forge` の重複責務を一覧化する。
- [ ] P0-A-013: `katana-markdown-model` と `katana-markdown-engine` の重複型を一覧化する。
- [ ] P0-A-014: `katana-chat-ui` の render model を一覧化する。
- [ ] P0-A-015: 各 repo の `just check` 内容を一覧化する。

### P0-B. Canonical naming

- [ ] P0-B-001: `katana-markdown-model` を canonical model crate として ADR に記録する。
- [ ] P0-B-002: `katana-markdown-engine` を migration / compatibility target として ADR に記録する。
- [ ] P0-B-003: `katana-document-viewer` を document artifact subsystem として ADR に記録する。
- [ ] P0-B-004: `preview` は feature 名として残し、owner 名にしないことを ADR に記録する。
- [ ] P0-B-005: `forge` は KDV 内 subsystem とすることを ADR に記録する。
- [ ] P0-B-006: `katana-canvas-forge` は transitional backend とすることを ADR に記録する。
- [ ] P0-B-007: `katana-ui-core` は framework-neutral core とすることを ADR に記録する。
- [ ] P0-B-008: `Floem` は adapter 対象であり core dependency ではないことを ADR に記録する。
- [ ] P0-B-009: `GPUI` は adapter 対象であり core dependency ではないことを ADR に記録する。
- [ ] P0-B-010: `egui` は新規 core API に入れないことを ADR に記録する。
- [ ] P0-B-011: `katana-ui-widget` を `katana-ui-core` にリネームする方針を ADR `docs/adr/0002-katana-ui-core-rename.md` (KatanA repo) に記録する。
- [ ] P0-B-012: GitHub repo `katana-ui-widget` を `katana-ui-core` にリネームする (GitHub 側の old-URL redirect で link が維持されることを確認)。
- [ ] P0-B-013: Cargo crate 名を `katana-ui-widget` → `katana-ui-core` に変更する (root `Cargo.toml` / `crates/*/Cargo.toml`)。
- [ ] P0-B-014: adapter crate を `katana-ui-core-floem` / `katana-ui-core-egui` / `katana-ui-core-gpui` に rename する。
- [ ] P0-B-015: storybook crate を `katana-ui-core-storybook` に rename する。
- [ ] P0-B-016: Justfile / `scripts/` / `release/` の `katana-ui-widget` 参照を更新する。
- [ ] P0-B-017: README / CONTRIBUTING に rename と UI Core 責務 (runtime / window / surface 含む) を明記する。
- [ ] P0-B-018: 関連 repo (KatanA / KDV / KLE / KCF / KCU など) の Cargo.toml dep 表記を `katana-ui-core` に追従させる PR を起票する。
- [ ] P0-B-019: 過去 OpenSpec changes / handoff / tmp 文書の `katana-ui-widget` 表記は **触らない** (履歴として残す)。新規 OpenSpec changes は `katana-ui-core` 表記を使う。

### P0-C. Dependency policy

- [ ] P0-C-001: 各 repo に dependency leak policy を追加する。
- [ ] P0-C-002: `katana-ui-core` core から `floem` 依存を禁止する guardrail を作る。
- [ ] P0-C-003: `katana-ui-core` core から `gpui` 依存を禁止する guardrail を作る。
- [ ] P0-C-004: `katana-document-viewer::forge` から UI crate 依存を禁止する guardrail を作る。
- [ ] P0-C-005: `katana-language-editor` から UI crate 依存を禁止する guardrail を作る。
- [ ] P0-C-006: `katana-ui` から forge internal module 参照を禁止する guardrail を作る。
- [ ] P0-C-007: `katana` から preview implementation 参照を禁止する guardrail を作る。
- [ ] P0-C-008: `katana` から editor implementation 参照を禁止する guardrail を作る。
- [ ] P0-C-009: `workspace.dependencies` の shared version policy を整理する。
- [ ] P0-C-010: adapter crate の optional feature policy を整理する。

### P0-D. Quality gate alignment

- [ ] P0-D-001: 各 repo の `just check` を baseline gate として固定する。
- [ ] P0-D-002: `katana-ast-lint` の version を各 repo で統一する。
- [ ] P0-D-003: `fmt-check` を全 repo で必須化する。
- [ ] P0-D-004: `clippy -D warnings` を全 repo で必須化する。
- [ ] P0-D-005: `unwrap_used` / `expect_used` / `todo` / `unimplemented` / `dbg_macro` / `panic` policy を揃える。
- [ ] P0-D-006: coverage gate の最低値を repo ごとに記録する。
- [ ] P0-D-007: release dry-run の有無を repo ごとに記録する。
- [ ] P0-D-008: OpenSpec validation を KatanA 側の integration gate に入れる。
- [ ] P0-D-009: migration 中の temporary compatibility API に `deprecated` policy を付ける。
- [ ] P0-D-010: `rtk` wrapper usage を noisy command policy として明文化する。

---

## P1. `katana-ui-core` neutral core 化

### P1-A. Workspace restructuring

- [ ] P1-A-001: `katana-ui-core` root Cargo.toml の current members を確認する。
- [ ] P1-A-002: `crates/katana-ui-core` を core crate として再定義する。
- [ ] P1-A-003: `crates/katana-ui-core-floem` を追加する。
- [ ] P1-A-004: `crates/katana-ui-core-storybook` を追加する。
- [ ] P1-A-005: root `workspace.dependencies` の neutral deps と adapter-specific deps を分けて整理する。core crate がどの shared entry を参照してよいか policy 化する。
- [ ] P1-A-006: `floem` / `floem_reactive` / `floem_renderer` を adapter crate dependency に移す。
- [ ] P1-A-007: core crate の package description を「framework-neutral」に変更する。
- [ ] P1-A-008: Floem 前提の README 文言を削除する。
- [ ] P1-A-009: adapter 方針を README に追加する。
- [ ] P1-A-010: release metadata に adapter crate を含める。

### P1-B. Core module skeleton

- [ ] P1-B-001: `atom` module を作る。
- [ ] P1-B-002: `molecule` module を作る。
- [ ] P1-B-003: `layout` module を neutral 化する。
- [ ] P1-B-004: `theme` module を neutral 化する。
- [ ] P1-B-005: `event` module を作る。
- [ ] P1-B-006: `render_model` module を作る。
- [ ] P1-B-013: `runtime` module を作る (Application / AppConfig / AppHandle / AppLifecycle / RuntimeAdapter)。
- [ ] P1-B-014: `window` module を作る (Window / WindowId / WindowConfig / WindowEvent / WindowCommand / WindowManager / DisplayInfo)。
- [ ] P1-B-015: `surface` module を作る (Surface / FrameHandle / PaintRequest / SurfaceMetrics)。
- [ ] P1-B-007: `accessibility` module を作る。
- [ ] P1-B-008: `adapter_contract` module を作る。
- [ ] P1-B-009: `primitive` module を `atom` へ段階移行する。
- [ ] P1-B-010: `composite` module を `molecule` へ段階移行する。
- [ ] P1-B-011: `floem_view` module を core から削除する。
- [ ] P1-B-012: `overlay_lifecycle` module を Floem adapter へ移す。

### P1-C. Theme tokens

- [ ] P1-C-001: `ColorToken` を定義する。
- [ ] P1-C-002: `FontToken` を定義する。
- [ ] P1-C-003: `SpacingToken` を定義する。
- [ ] P1-C-004: `RadiusToken` を定義する。
- [ ] P1-C-005: `ShadowToken` を定義する。
- [ ] P1-C-006: `BorderToken` を定義する。
- [ ] P1-C-007: `ZIndexToken` を定義する。
- [ ] P1-C-008: `ThemeSnapshot` を定義する。
- [ ] P1-C-009: `ThemeId` を定義する。
- [ ] P1-C-010: light theme fixture を作る。
- [ ] P1-C-011: dark theme fixture を作る。
- [ ] P1-C-012: theme serialization test を作る。
- [ ] P1-C-013: theme diff test を作る。

### P1-D. Layout primitives

- [ ] P1-D-001: `SizePolicy` を定義する。
- [ ] P1-D-002: `Length` を定義する。
- [ ] P1-D-003: `EdgeInsets` を定義する。
- [ ] P1-D-004: `Alignment` を定義する。
- [ ] P1-D-005: `Row` model を定義する。
- [ ] P1-D-006: `Column` model を定義する。
- [ ] P1-D-007: `Stack` model を定義する。
- [ ] P1-D-008: `Grid` model を定義する。
- [ ] P1-D-009: `ScrollArea` model を定義する。
- [ ] P1-D-010: `SplitPane` model を定義する。
- [ ] P1-D-011: layout snapshot test を作る。
- [ ] P1-D-012: layout serialization test を作る。

### P1-E. Atom widgets

- [ ] P1-E-001: `Text` atom を定義する。
- [ ] P1-E-002: `Icon` atom を定義する。
- [ ] P1-E-003: `Button` atom を定義する。
- [ ] P1-E-004: `Input` atom を定義する。
- [ ] P1-E-005: `Checkbox` atom を定義する。
- [ ] P1-E-006: `Radio` atom を定義する。
- [ ] P1-E-007: `Badge` atom を定義する。
- [ ] P1-E-008: `Divider` atom を定義する。
- [ ] P1-E-009: `Spacer` atom を定義する。
- [ ] P1-E-010: disabled state を atom 共通に追加する。
- [ ] P1-E-011: focusable state を atom 共通に追加する。
- [ ] P1-E-012: accessibility label を atom 共通に追加する。
- [ ] P1-E-013: atom render model snapshot を作る。

### P1-F. Molecule widgets

- [ ] P1-F-001: `Card` molecule を定義する。
- [ ] P1-F-002: `List` molecule を定義する。
- [ ] P1-F-003: `Menu` molecule を定義する。
- [ ] P1-F-004: `Tooltip` molecule を定義する。
- [ ] P1-F-005: `Modal` molecule を定義する。
- [ ] P1-F-006: `Tabs` molecule を定義する。
- [ ] P1-F-007: `Toolbar` molecule を定義する。
- [ ] P1-F-008: `FormField` molecule を定義する。
- [ ] P1-F-009: `Breadcrumb` molecule を定義する。
- [ ] P1-F-010: molecule event routing を定義する。
- [ ] P1-F-011: molecule snapshot test を作る。

### P1-G. Event model

- [ ] P1-G-001: `UiEvent` を定義する。
- [ ] P1-G-002: `PointerEvent` を定義する。
- [ ] P1-G-003: `KeyboardEvent` を定義する。
- [ ] P1-G-004: `FocusEvent` を定義する。
- [ ] P1-G-005: `CommandEvent` を定義する。
- [ ] P1-G-006: `UiNodeId` を event target に使う。
- [ ] P1-G-007: event bubbling policy を定義する。
- [ ] P1-G-008: event capture policy を定義する。
- [ ] P1-G-009: event serialization test を作る。
- [ ] P1-G-010: event ordering test を作る。

### P1-H. Render model

- [ ] P1-H-001: `UiNodeId` を定義する。
- [ ] P1-H-002: `UiNodeKind` を定義する。
- [ ] P1-H-003: `UiProps` を定義する。
- [ ] P1-H-004: `UiNode` を定義する。
- [ ] P1-H-005: `UiTree` を定義する。
- [ ] P1-H-006: `UiTreeDiff` を定義する。
- [ ] P1-H-007: `UiCommand` を定義する。
- [ ] P1-H-008: `RenderContext` を定義する。
- [ ] P1-H-009: render model snapshot test を作る。
- [ ] P1-H-010: render model no-framework compile test を作る。

### P1-I. Primary adapter (Floem) migration

Floem は primary adapter 候補として最初に整備する。P4-0 で primary 確定後、本セクションを正式に primary として扱う。Floem が primary に選ばれなかった場合は本セクションのタスクを互換 adapter (P1-K) と同水準に降格する。

- [ ] P1-I-001: `katana-ui-core-floem` crate を作る。
- [ ] P1-I-002: core の `UiTree` を Floem view に変換する adapter skeleton を作る。
- [ ] P1-I-003: `Text` adapter を実装する。
- [ ] P1-I-004: `Button` adapter を実装する。
- [ ] P1-I-005: `Input` adapter を実装する。
- [ ] P1-I-006: `Row` / `Column` adapter を実装する。
- [ ] P1-I-007: `Tabs` adapter を実装する。
- [ ] P1-I-008: `Toolbar` adapter を実装する。
- [ ] P1-I-009: `SplitPane` adapter を実装する。
- [ ] P1-I-010: overlay lifecycle guard を Floem adapter 側に移す。
- [ ] P1-I-011: menu button contract を Floem adapter 側に移す。
- [ ] P1-I-012: adapter compile test を作る。

### P1-K. 互換 adapter (egui / gpui)

primary に選ばれていない framework 向けの互換 adapter を併設する。外部利用者が既存環境に `katana-ui-core` を差し込めるようにするのが目的。品質ゲートは primary より緩いが、core crate に依存リークさせない原則は同じ。

- [ ] P1-K-001: `katana-ui-core-egui` 互換 adapter crate を新設する。
- [ ] P1-K-002: `katana-ui-core-gpui` 互換 adapter crate を新設する。
- [ ] P1-K-003: 各互換 adapter で `UiTree` -> framework view 変換 skeleton を作る (Text / Button / Row / Column を最低ライン)。
- [ ] P1-K-004: 各互換 adapter の対応 widget / 未対応機能 / フォールバック挙動を README に明記する。
- [ ] P1-K-005: 各互換 adapter に opt-in feature gate (`workspace.dependencies` の optional 化) を設定し、`katana-ui-core` core compile に引き込まれないことを保証する。
- [ ] P1-K-006: 各互換 adapter の最低品質ゲート (compile test + storybook smoke) を CI に追加する。primary より緩く許容する。
- [ ] P1-K-007: 互換 adapter の release が primary release を止めない policy を CI / release script に反映する。
- [ ] P1-K-008: 互換 adapter のサポート範囲・SemVer minor 追加縮小 policy を `docs/release/compat-adapters.md` に記録する。

### P1-L. Runtime / Window / Surface API

KUC を framework-neutral UI Core として完成させるために、起動 entry / window 管理 / 描画 surface の neutral API を整備する。adapter (Floem / GPUI / 互換 egui / gpui) はこの neutral API を変換する責務だけを持つ。

neutral 化の粒度は **「中」**: title / size / close / focus / fullscreen / multi-window / icon を共通サポートする。platform menu / IME / drag&drop は adapter 経由 escape hatch (`adapter_contract` 拡張) で対応。

- [ ] P1-L-001: `Application` を定義する (`Application::new() -> ApplicationBuilder`、`run(self) -> AppExitCode`)。
- [ ] P1-L-002: `AppConfig` を定義する (識別子 / persistence path / locale / accessibility option)。
- [ ] P1-L-003: `AppHandle` を定義する (`spawn_window` / `dispatch_command` / `current_windows`)。
- [ ] P1-L-004: `AppLifecycle` event (`Started` / `Suspended` / `Resumed` / `ShuttingDown`) を定義する。
- [ ] P1-L-005: `RuntimeAdapter` trait を定義する (event loop を adapter に委譲)。
- [ ] P1-L-006: `Window` / `WindowId` を定義する。
- [ ] P1-L-007: `WindowConfig` を定義する (title / size / min_size / max_size / icon / decorations / fullscreen)。
- [ ] P1-L-008: `WindowEvent` enum を定義する (Close / Resize / Move / Focus / Minimize / Maximize / Restore / DisplayChanged)。
- [ ] P1-L-009: `WindowCommand` enum を定義する (SetTitle / SetSize / SetPosition / Focus / Minimize / Maximize / Close / Fullscreen)。
- [ ] P1-L-010: `WindowManager` を定義する (multi-window 作成 / iteration / 1 window 終了でアプリ終了するかの policy)。
- [ ] P1-L-011: `DisplayInfo` DTO を定義する (multi-monitor read-only 情報)。
- [ ] P1-L-012: `Surface` / `FrameHandle` / `PaintRequest` / `SurfaceMetrics` を定義する。
- [ ] P1-L-013: runtime / window / surface module に対する framework 非依存 snapshot test を作る (Noop adapter で起動できることを確認)。
- [ ] P1-L-014: runtime / window / surface module の public API が adapter 型を返さないことを script で検査する。
- [ ] P1-L-015: primary adapter (Floem) で runtime / window / surface を実装する (`katana-ui-core-floem`)。
- [ ] P1-L-016: 互換 adapter (egui / gpui) で runtime / window / surface を実装する (機能差異を README に明記)。
- [ ] P1-L-017: platform menu / IME / drag&drop の escape hatch を `adapter_contract` 拡張として定義する (KUC 標準 API には入れない)。

### P1-J. Quality gate update

- [ ] P1-J-001: core crate が `floem` を含まないことを script で検査する。
- [ ] P1-J-002: core crate が `gpui` を含まないことを script で検査する。
- [ ] P1-J-003: core crate が `katana-*` domain crate を含まないことを script で検査する。
- [ ] P1-J-004: `just check` に dependency leak guard を追加する。
- [ ] P1-J-005: Storybook gate を adapter crate 対象に変更する。
- [ ] P1-J-006: release dry-run に core crate を含める。
- [ ] P1-J-007: release dry-run に Floem adapter crate を含める。
- [ ] P1-J-008: README に adapter policy を追加する。

---

## P2. `katana-document-viewer` 拡張 + forge 内包

### P2 スコープ宣言

Phase 2 は **KDV 側に KCF を呼ぶための facade を追加するだけ**にする。

- 触る対象: KDV のみ。
- KCF crate / KCF CLI / KCF public API は変更しない。
- KCF からの戻り値を KDV artifact 型に変換するレイヤを KDV 内部に持つ。
- KCF 側の public API 縮小、CLI delegate 化、deprecation は Phase 7 で行う。

この境界によって、Phase 2 で KCF を利用している既存利用者（CLI / 他 crate）に影響を出さない。

### P2-A. Workspace restructuring

- [ ] P2-A-001: root Cargo.toml の current members を確認する。
- [ ] P2-A-002: `crates/katana-document-viewer` を追加する。
- [ ] P2-A-003: `crates/katana-document-preview` を compatibility facade に変更する。
- [ ] P2-A-004: `crates/katana-document-preview-egui` を deprecated compatibility adapter として扱う。
- [ ] P2-A-005: `crates/katana-document-viewer-egui` を temporary adapter として追加するか判断する。判断基準: (a) KatanA 本体の preview が Phase 5 まで egui を保持する想定か / (b) `katana-document-preview-egui` の compatibility window で代替できるか。決定は ADR `docs/adr/kdv-egui-adapter.md` に記録する。
- [ ] P2-A-006: package descriptions を preview から viewer に更新する。
- [ ] P2-A-007: README の責務説明を document artifact subsystem に更新する。
- [ ] P2-A-008: OpenSpec を preview-only から viewer+forge に更新する。

### P2-B. Source and document model

- [ ] P2-B-001: `DocumentSource` を定義する。
- [ ] P2-B-002: `SourceUri` を定義する。
- [ ] P2-B-003: `SourceKind` を定義する。
- [ ] P2-B-004: `SourceRevision` を定義する。
- [ ] P2-B-005: `DocumentId` を定義する。
- [ ] P2-B-006: `DocumentKind` を定義する。
- [ ] P2-B-007: `DocumentSnapshot` を定義する。
- [ ] P2-B-008: `DocumentOutline` を定義する。
- [ ] P2-B-009: `DocumentMetadataView` を定義する。
- [ ] P2-B-010: KMM input conversion を作る。
- [ ] P2-B-011: KMM parse result conversion を作る。
- [ ] P2-B-012: source serialization test を作る。
- [ ] P2-B-013: document snapshot test を作る。

### P2-C. Artifact model

- [ ] P2-C-001: `ArtifactId` を定義する。
- [ ] P2-C-002: `ArtifactKind` を定義する。
- [ ] P2-C-003: `ArtifactFormat` を定義する。
- [ ] P2-C-004: `ArtifactBytes` を定義する。
- [ ] P2-C-005: `ArtifactUri` を定義する。
- [ ] P2-C-006: `ArtifactManifest` を定義する。
- [ ] P2-C-007: `ArtifactDiagnostics` を定義する。
- [ ] P2-C-008: preview artifact を定義する。
- [ ] P2-C-009: export artifact を定義する。
- [ ] P2-C-010: image artifact を定義する。
- [ ] P2-C-011: PDF artifact を定義する。
- [ ] P2-C-012: Office artifact placeholder を定義する。
- [ ] P2-C-013: artifact manifest serialization test を作る。

### P2-D. Forge API

- [ ] P2-D-001: `forge` module を作る。
- [ ] P2-D-002: `BuildRequest` を定義する。
- [ ] P2-D-003: `BuildProfile` を定義する。
- [ ] P2-D-004: `BuildGraph` を定義する。
- [ ] P2-D-005: `TransformStep` を定義する。
- [ ] P2-D-006: `ExportRequest` を定義する。
- [ ] P2-D-007: `ExportFormat` を定義する。
- [ ] P2-D-008: `ExportOutput` を定義する。
- [ ] P2-D-009: `ForgeDiagnostics` を定義する。
- [ ] P2-D-010: `ForgeError` を定義する。
- [ ] P2-D-011: `ForgeBackend` trait を定義する。
- [ ] P2-D-012: `ForgePipeline` を定義する。
- [ ] P2-D-013: no-UI dependency test を作る。

### P2-E. KCF backend integration

- [ ] P2-E-001: `backend::canvas_forge` module を作る。
- [ ] P2-E-002: KCF `RenderInput` への変換を作る。
- [ ] P2-E-003: KCF `RenderOutput` から Artifact への変換を作る。
- [ ] P2-E-004: KCF export output から ExportOutput への変換を作る。
- [ ] P2-E-005: Mermaid render path を接続する。
- [ ] P2-E-006: Draw.io render path を接続する。
- [ ] P2-E-007: ZenUML render path を接続する。
- [ ] P2-E-008: HTML export path を接続する。
- [ ] P2-E-009: PDF export path を接続する。
- [ ] P2-E-010: PNG export path を接続する。
- [ ] P2-E-011: JPEG export path を接続する。
- [ ] P2-E-012: KCF dependency を transitional として README に記載する。
- [ ] P2-E-013: KCF compatibility tests を作る。

### P2-F. Viewer surface

- [ ] P2-F-001: `ViewerState` を定義する。
- [ ] P2-F-002: `ViewerCommand` を定義する。
- [ ] P2-F-003: `ViewerEvent` を定義する。
- [ ] P2-F-004: `ViewerViewport` を定義する。
- [ ] P2-F-005: `PageModel` を定義する。
- [ ] P2-F-006: `ScrollAnchor` を定義する。
- [ ] P2-F-007: `HighlightRange` を定義する。
- [ ] P2-F-008: preview artifact display model を作る。
- [ ] P2-F-009: PDF page display model を作る。
- [ ] P2-F-010: Office display placeholder model を作る。
- [ ] P2-F-011: image display model を作る。
- [ ] P2-F-012: bundle manifest display model を作る。

### P2-G. Scroll sync

- [ ] P2-G-001: `SourceAnchor` を定義する。
- [ ] P2-G-002: `ArtifactAnchor` を定義する。
- [ ] P2-G-003: `ScrollSyncMap` を定義する。
- [ ] P2-G-004: `SyncResolution` を定義する。
- [ ] P2-G-005: KMM node id から viewer anchor への mapping を作る。
- [ ] P2-G-006: line-column から viewer anchor への mapping を作る。
- [ ] P2-G-007: fingerprint fallback mapping を作る。
- [ ] P2-G-008: unresolved anchor diagnostics を作る。
- [ ] P2-G-009: scroll sync fixture test を作る。
- [ ] P2-G-010: edited document re-resolution test を作る。

### P2-H. CLI API

- [ ] P2-H-001: `cli_api` module を作る。
- [ ] P2-H-002: `CliRequest` を定義する。
- [ ] P2-H-003: `CliOutput` を定義する。
- [ ] P2-H-004: `CliDiagnostics` を定義する。
- [ ] P2-H-005: markdown preview build CLI entry を作る。
- [ ] P2-H-006: export CLI entry を作る。
- [ ] P2-H-007: diagram render CLI entry を作る。
- [ ] P2-H-008: export-debug CLI entry を作る。
- [ ] P2-H-009: existing KCF CLI command compatibility table を作る。
- [ ] P2-H-010: KCF CLI delegate PR を作る。
- [ ] P2-H-011: CLI golden output fixtures を作る。

### P2-I. Adapter and compatibility

- [ ] P2-I-001: existing `MarkdownPreview` trait を KDV API へ adapter する。
- [ ] P2-I-002: `MarkdownSource` を `DocumentSource` に変換する。
- [ ] P2-I-003: `PreviewConfig` を `ViewerConfig` に変換する。
- [ ] P2-I-004: `PreviewOutput` を `ViewerOutput` に変換する。
- [ ] P2-I-005: `PreviewError` を `ViewerError` に変換する。
- [ ] P2-I-006: `katana-document-preview` に deprecated notice を入れる。
- [ ] P2-I-007: `katana-document-preview-egui` を temporary adapter として固定する。
- [ ] P2-I-008: KatanA から preview facade を使えるようにする。

### P2-J. Quality gate update

- [ ] P2-J-001: KDV `just check` に forge no-UI dependency guard を追加する。
- [ ] P2-J-002: KDV `just check` に KCF backend smoke を追加する。
- [ ] P2-J-003: KDV `just check` に CLI API smoke を追加する。
- [ ] P2-J-004: KDV release package に KDV crate を追加する。
- [ ] P2-J-005: KDV release package に preview facade を追加する。
- [ ] P2-J-006: KDV release package に adapter crate を追加する。
- [ ] P2-J-007: artifact fixture tests を CI に追加する。
- [ ] P2-J-008: export fixture tests を CI に追加する。

---

## P3. `katana-language-editor` domain 強化

### P3-A. Buffer model

- [ ] P3-A-001: `BufferId` を定義する。
- [ ] P3-A-002: `BufferRevision` を定義する。
- [ ] P3-A-003: `BufferSnapshot` を定義する。
- [ ] P3-A-004: `TextRange` を定義する。
- [ ] P3-A-005: `TextEdit` を定義する。
- [ ] P3-A-006: `EditBatch` を定義する。
- [ ] P3-A-007: `TextContent` を BufferSnapshot に接続する。
- [ ] P3-A-008: buffer serialization test を作る。
- [ ] P3-A-009: edit application test を作る。

### P3-B. Cursor / selection

- [ ] P3-B-001: `CursorAffinity` を定義する。
- [ ] P3-B-002: `CursorRange` を定義する。
- [ ] P3-B-003: `SelectionMode` を定義する。
- [ ] P3-B-004: `MultiSelection` を定義する。
- [ ] P3-B-005: cursor movement command を定義する。
- [ ] P3-B-006: selection changed event を拡張する。
- [ ] P3-B-007: cursor restore model を作る。
- [ ] P3-B-008: cursor fixture test を作る。

### P3-C. Commands and events

- [ ] P3-C-001: `EditorCommand` enum を作る。
- [ ] P3-C-002: `InsertText` command を作る。
- [ ] P3-C-003: `DeleteRange` command を作る。
- [ ] P3-C-004: `ReplaceRange` command を作る。
- [ ] P3-C-005: `MoveCursor` command を作る。
- [ ] P3-C-006: `ApplyFormat` command を作る。
- [ ] P3-C-007: `OpenDocument` command を作る。
- [ ] P3-C-008: `EditorEvent` に revision を含める。
- [ ] P3-C-009: event ordering test を作る。

### P3-D. Diagnostics

- [ ] P3-D-001: `Diagnostic` を定義する。
- [ ] P3-D-002: `DiagnosticSeverity` を定義する。
- [ ] P3-D-003: `DiagnosticSource` を定義する。
- [ ] P3-D-004: `DiagnosticRange` を定義する。
- [ ] P3-D-005: markdown linter diagnostics を受ける DTO を作る。
- [ ] P3-D-006: AST lint diagnostics を受ける DTO を作る。
- [ ] P3-D-007: diagnostics changed event を作る。
- [ ] P3-D-008: diagnostics snapshot test を作る。

### P3-E. Source anchor (language-neutral)

KLE core 側は markdown / KMM を含む特定言語に依存しない。source anchor は opaque DTO + adapter trait に閉じる。

- [ ] P3-E-001: `SourceAnchor` (opaque ID + position 情報を持つ言語非依存 DTO) を定義する。
- [ ] P3-E-002: `LineColumnRange` DTO を定義する。
- [ ] P3-E-003: text fingerprint DTO を定義する。
- [ ] P3-E-004: `SourceAnchorAdapter` trait を定義する (snapshot + position から SourceAnchor を返す)。
- [ ] P3-E-005: source anchor changed event を定義する (anchor は opaque)。
- [ ] P3-E-006: `SourceAnchorAdapter` 用の `NoopSourceAnchorAdapter` を明示提供する。
- [ ] P3-E-007: KLE crate が `katana-markdown-model` を含む markdown 系 crate を import していないことを script で検査する。
- [ ] P3-E-008: source anchor fixture test を作る (markdown 概念を含まないこと)。

### P3-G. External ports (language-neutral trait group)

KLE は syntax / jump / hover / completion / formatter / diagnostics / source anchor の各機能を **port (trait)** として宣言する。default 言語実装は提供しない。利用側 (KatanA / katana-language-editor-md など) が必ず inject する。

- [ ] P3-G-001: `SyntaxHighlightProvider` trait と `SyntaxToken` DTO (kind / range / scope) を定義する。
- [ ] P3-G-002: `JumpProvider` trait と `JumpTarget` DTO (uri / range) を定義する。
- [ ] P3-G-003: `HoverProvider` trait と `HoverContent` DTO (framework-neutral rich text) を定義する。
- [ ] P3-G-004: `CompletionProvider` trait と `CompletionItem` DTO を定義する。
- [ ] P3-G-005: `FormatterProvider` trait を定義する (snapshot + range -> Vec<TextEdit>)。
- [ ] P3-G-006: `DiagnosticsSource` trait を定義する (subscribe(BufferId) -> Stream<DiagnosticsBatch>)。
- [ ] P3-G-007: editor builder API を作り、上記 port を必須引数として受け取る。port なしでは editor を construct できないようにする。
- [ ] P3-G-008: `NoopSyntaxHighlightProvider` / `NoopJumpProvider` 等 default を明示提供する (`Default::default()` には依存させず明示渡しを強制)。
- [ ] P3-G-009: port trait が KLE crate 内に閉じ、markdown / KMM 概念を含まないことを script で検査する。
- [ ] P3-G-010: port fixture test を作る (Noop / Mock implementation で editor が動作することを検証)。
- [ ] P3-G-011: port wiring 方針を README に明文化する (`docs/usage/ports.md`)。

### P3-H. Markdown port adapter (`katana-language-editor-md`)

KLE 自体に markdown 知識を入れない代わりに、markdown 用の port implementation を別 crate として提供する。配置は新規独立 crate または KatanA repo 内 crate のどちらでもよい (ADR で決める)。

- [ ] P3-H-001: `katana-language-editor-md` crate を新設する。配置場所 (独立 repo / KatanA repo 内) を ADR `docs/adr/kle-md-adapter-placement.md` に記録する。
- [ ] P3-H-002: KMM document model を入力とする `SyntaxHighlightProvider` implementation を提供する。
- [ ] P3-H-003: KMM source span を返す `SourceAnchorAdapter` implementation を提供する。
- [ ] P3-H-004: markdown 用 `JumpProvider` implementation (heading / link target) を提供する。
- [ ] P3-H-005: markdown 用 `CompletionProvider` stub (heading / link suggestions) を提供する。
- [ ] P3-H-006: markdown 用 `FormatterProvider` を提供する (KMM 既存 formatter があれば委譲)。
- [ ] P3-H-007: `katana-language-editor-md` から KLE への dependency 方向が正しい (KLE が md adapter を知らない) ことを script で検査する。
- [ ] P3-H-008: KatanA から `katana-language-editor-md` を inject する flow を `docs/usage/katana-markdown-editor.md` に記載する。
- [ ] P3-H-009: md adapter の fixture test (KMM input -> port output) を作る。

### P3-F. Adapter containment

- [ ] P3-F-001: `katana-language-editor-egui` を compatibility adapter として README に記録する。
- [ ] P3-F-002: future `katana-language-editor-floem` の crate boundary を設計する。
- [ ] P3-F-003: core crate から egui import がないことを script で検査する。
- [ ] P3-F-004: adapter crate の only dependency policy を作る。
- [ ] P3-F-005: KatanA integration で adapter 型を直接保持しないようにする。

---

## P4. `katana-ui` composition layer 化

### P4-0. Primary adapter 選定

`katana-ui` 自身は独自 UI 表現を持ち adapter agnostic。Phase 4 で確定する必要があるのは「**KatanA が起動時に使う primary adapter として何を選ぶか**」と「**互換 adapter としてどれを併設するか**」の 2 軸。

primary adapter の候補:

| 候補 | 内容 | 利点 | 欠点 |
| ---- | ---- | ---- | ---- |
| A | floem を primary | signal ベース宣言 UI / Lapce 実績 / 長期設計と整合 | 安定 API としては発展途上 |
| B | gpui を primary | Zed 実績 / エディタ向け最適化 | 外部利用者向け安定度に注意 |
| C | egui を primary (短期) | KatanA 現状を維持 / 移行コスト最小 | 独自 UI Core 思想と乖離、後で必ず移行 |
| D | primary は当面確定せず adapter agnostic で進める | Phase 5 で再決定可能 | 複数 adapter の品質ゲートを並走させる必要 |

互換 adapter の維持方針: primary に選ばれていない adapter (egui / gpui / floem) も外部利用者向けに crate として併設する。これは P4-0 の選定とは別軸で全て維持する。

決定タスク:

- [ ] P4-0-001: 上記候補 A / B / C / D を比較する ADR `docs/adr/katana-ui-primary-adapter.md` を作る。
- [ ] P4-0-002: 比較基準 (API 安定度、エディタ系適合、移行コスト、Phase 5 整合、外部利用者向け魅力) を ADR に明記する。
- [ ] P4-0-003: primary adapter 選定結果を `katana-ui-core-<primary>` crate の品質ゲートに反映する (core と同等)。
- [ ] P4-0-004: 互換 adapter として併設する framework 一覧 (egui / gpui / floem から primary を除いた集合) を確定する。
- [ ] P4-0-005: 互換 adapter の品質ゲート (storybook smoke + compile test 最低ライン) を CI に追加する。
- [ ] P4-0-006: primary adapter の release を互換 adapter の breakage が止めない policy を CI / release script に反映する。
- [ ] P4-0-007: primary 切り替えが将来発生した場合の降格 flow (旧 primary → 互換) を ADR に記載する。
- [ ] P4-0-008: 選定結果を `katana-ui-design-principles.md` と詳細設計 5.4.2 に反映する。

primary が確定するまでは P4-D / P4-E は adapter agnostic に書く (UiTree 出力までで止める)。primary 確定後に adapter 接続層を実装する。

### P4-A. Current KatanA UI split

出力先: `docs/inventory/katana-app-state-split.md` に table 形式で記録する。各 field について `current_owner`、`target_owner`、`migration_phase`、`notes` 列を持つ。すべての分類が同表に集まることが done 条件。

- [ ] P4-A-001: `KatanaApp` state fields を分類する。
- [ ] P4-A-002: app lifecycle field を `katana` 側候補に分類する。
- [ ] P4-A-003: UI composition field を `katana-ui` 側に分類する。
- [ ] P4-A-004: preview field を `katana-document-viewer` 側に分類する。
- [ ] P4-A-005: editor field を `katana-language-editor` 側に分類する。
- [ ] P4-A-006: export task field を `katana-document-viewer::forge` 側に分類する。
- [ ] P4-A-007: file dialog field を platform boundary に分類する。
- [ ] P4-A-008: update dialog field を app command boundary に分類する。
- [ ] P4-A-009: linter docs cache を domain cache に分類する。
- [ ] P4-A-010: cursor range state を editor domain に分類する。

### P4-B. MainPanel API

- [ ] P4-B-001: `MainPanelInput` を定義する。
- [ ] P4-B-002: `MainPanelState` を定義する。
- [ ] P4-B-003: `MainPanelEvent` を定義する。
- [ ] P4-B-004: `MainPanelCommand` を定義する。
- [ ] P4-B-005: `AppFrame` を定義する。
- [ ] P4-B-006: `DockLayout` を定義する。
- [ ] P4-B-007: `PaneId` を定義する。
- [ ] P4-B-008: `PaneKind` を定義する。
- [ ] P4-B-009: `PanePlacement` を定義する。
- [ ] P4-B-010: `MainPanel` snapshot test を作る。

### P4-C. Panes

- [ ] P4-C-001: `EditorPane` を定義する。
- [ ] P4-C-002: `DocumentViewerPane` を定義する。
- [ ] P4-C-003: `ChatPane` を定義する。
- [ ] P4-C-004: `DiagnosticsPane` を定義する。
- [ ] P4-C-005: `ExplorerPane` を定義する。
- [ ] P4-C-006: `SettingsPane` を定義する。
- [ ] P4-C-007: pane close event を定義する。
- [ ] P4-C-008: pane focus event を定義する。
- [ ] P4-C-009: pane split event を定義する。
- [ ] P4-C-010: pane reorder event を定義する。

### P4-D. Adapter calls

- [ ] P4-D-001: `EditorPane` が KLE adapter を呼ぶ boundary を作る。
- [ ] P4-D-002: `DocumentViewerPane` が KDV adapter を呼ぶ boundary を作る。
- [ ] P4-D-003: `ChatPane` が KCU render model adapter を呼ぶ boundary を作る。
- [ ] P4-D-004: `DiagnosticsPane` が common DTO を受ける boundary を作る。
- [ ] P4-D-005: `katana-ui` が editor buffer を直接編集しないようにする。
- [ ] P4-D-006: `katana-ui` が forge export を直接実行しないようにする。
- [ ] P4-D-007: `katana-ui` が artifact bytes を直接生成しないようにする。
- [ ] P4-D-008: `katana-ui` が KMM parser internal を見ないようにする。

### P4-E. Widget adoption

前提: P4-0 で primary adapter が確定していること。本セクションは KatanA UI を `katana-ui` の Component model で記述し直すタスク。adapter は `katana-ui-core` 経由で背後にあるが、KatanA のコードに floem / egui / gpui の型が直接現れてはならない。

- [ ] P4-E-001: KatanA toolbar を `katana-ui::Toolbar` (`katana-ui-core::Toolbar` molecule 利用) で記述する。
- [ ] P4-E-002: KatanA tab bar を `katana-ui::TabBar` で記述する。
- [ ] P4-E-003: KatanA split pane を `katana-ui::SplitPane` で記述する。
- [ ] P4-E-004: KatanA modal を `katana-ui::Modal` で記述する。
- [ ] P4-E-005: KatanA tooltip を `katana-ui::Tooltip` で記述する。
- [ ] P4-E-006: KatanA menu を `katana-ui::Menu` で記述する。
- [ ] P4-E-007: KatanA button 利用箇所を `katana-ui-core::Button` atom 経由で記述する。
- [ ] P4-E-008: KatanA text 利用箇所を `katana-ui-core::Text` atom 経由で記述する。
- [ ] P4-E-009: KatanA icon 利用箇所を `katana-ui-core::Icon` atom 経由で記述する。
- [ ] P4-E-010: KatanA の theme token 利用を `katana-ui-core::ThemeSnapshot` 経由に統一する。
- [ ] P4-E-011: KatanA コード上に floem / egui / gpui の型が直接現れていないことを script で検査する。primary adapter は起動 entrypoint でのみ参照される。
- [ ] P4-E-012: 旧 egui ベース UI コードの削除条件を ADR `docs/adr/katanaui-legacy-egui-removal.md` に記録する。Phase 5 の preview / export / editor 移行完了が前提。

### P4-F. State separation

- [ ] P4-F-001: UI layout state を `katana-ui` に移す。
- [ ] P4-F-002: document state を KDV に移す。
- [ ] P4-F-003: editor state を KLE に移す。
- [ ] P4-F-004: export task state を KDV forge に移す。
- [ ] P4-F-005: update state を application command layer に移す。
- [ ] P4-F-006: file dialog state を platform/service boundary に移す。
- [ ] P4-F-007: linter doc cache を domain/service boundary に移す。
- [ ] P4-F-008: persisted layout state schema を作る。
- [ ] P4-F-009: migration from old persisted layout を作る。

---

## P5. `KatanA` integration

### P5-A. Dependency update

- [ ] P5-A-001: `KatanA` root Cargo.toml に `katana-ui-core` dependency を追加する必要がないことを確認する。
- [ ] P5-A-002: `KatanA/crates/katana-ui` に `katana-ui-core` dependency を追加する。
- [ ] P5-A-003: `KatanA/crates/katana-ui` に `katana-document-viewer` dependency を追加する。
- [ ] P5-A-004: `KatanA/crates/katana-ui` に `katana-language-editor` dependency を追加する。
- [ ] P5-A-005: `KatanA` から direct `egui_commonmark` usage を減らす計画を作る。
- [ ] P5-A-006: `KatanA` から direct KCF export usage を KDV forge に置換する計画を作る。
- [ ] P5-A-007: `[patch.crates-io]` の egui_commonmark vendor patch 削除条件を定義する。
- [ ] P5-A-008: `katana-canvas-forge` direct dependency を transitional として記録する。

### P5-B. Preview migration

出力先: P5-B-001〜005 の「特定する」系タスクは `docs/inventory/katana-preview-current.md` に code path (file:line) + 役割を一覧記録する。

- [ ] P5-B-001: current `PreviewPane` construction point を特定する。
- [ ] P5-B-002: current preview cache key を特定する。
- [ ] P5-B-003: current scroll sync input を特定する。
- [ ] P5-B-004: current diagram rendering call を特定する。
- [ ] P5-B-005: current task list action flow を特定する。
- [ ] P5-B-006: `DocumentViewerPane` adapter を追加する。
- [ ] P5-B-007: `PreviewPane` を KDV viewer state に置換する。
- [ ] P5-B-008: task list action を KDV event に変換する。
- [ ] P5-B-009: scroll sync を KDV sync map に変換する。
- [ ] P5-B-010: preview fixture regression を追加する。

### P5-C. Export migration

出力先: P5-C-001〜004 の「特定する」系タスクは `docs/inventory/katana-export-current.md` に code path (file:line) + format / output policy を記録する。

- [ ] P5-C-001: current `ExportTask` creation point を特定する。
- [ ] P5-C-002: current export format list を特定する。
- [ ] P5-C-003: current export output path policy を特定する。
- [ ] P5-C-004: current open-on-complete policy を特定する。
- [ ] P5-C-005: `ExportTask` を KDV `ExportRequest` に変換する。
- [ ] P5-C-006: KDV `ExportOutput` を app notification に変換する。
- [ ] P5-C-007: export progress event を定義する。
- [ ] P5-C-008: export error handling を定義する。
- [ ] P5-C-009: export regression fixture を追加する。
- [ ] P5-C-010: KCF direct export call を削除する。

### P5-D. Editor migration

出力先: P5-D-001〜005 の「特定する」系タスクは `docs/inventory/katana-editor-current.md` に code path (file:line) + 周辺型を記録する。

- [ ] P5-D-001: current editor content model を特定する。
- [ ] P5-D-002: current cursor restore logic を特定する。
- [ ] P5-D-003: current syntax highlight layouter を特定する。
- [ ] P5-D-004: current markdown authoring actions を特定する。
- [ ] P5-D-005: current diagnostics flow を特定する。
- [ ] P5-D-006: KLE `BufferSnapshot` に接続する。
- [ ] P5-D-007: KLE `EditorEvent` を app command に接続する。
- [ ] P5-D-008: KLE diagnostics を diagnostics pane に接続する。
- [ ] P5-D-009: KMM source mapping を editor-viewer sync に接続する。
- [ ] P5-D-010: editor fixture regression を追加する。

### P5-E. Application shell thinning

- [ ] P5-E-001: `KatanaApp` から preview cache を削除する条件を定義する。
- [ ] P5-E-002: `KatanaApp` から export tasks を削除する条件を定義する。
- [ ] P5-E-003: `KatanaApp` から editor cursor raw egui type を削除する条件を定義する。
- [ ] P5-E-004: `KatanaApp` から settings preview pane を削除する条件を定義する。
- [ ] P5-E-005: `KatanaApp` に `MainPanelState` だけを持たせる中間状態を作る。
- [ ] P5-E-006: app action dispatch を `MainPanelEvent` 経由に変える。
- [ ] P5-E-007: persistence schema を app core に移す。
- [ ] P5-E-008: settings schema を app core に移す。
- [ ] P5-E-009: platform service を app boundary に移す。
- [ ] P5-E-010: UI framework-specific handles を app state から削除する。

### P5-F. Vendor / patch cleanup

出力先: P5-F-001〜003 の「特定する」系タスクは `docs/inventory/katana-vendor-current.md` に file:line で記録する。P5-F-004〜006 の「寄せる / 確定する」系タスクは判断結果を同 ADR section に記録する。判断基準: (a) syntax highlight は editor 表示 path のみで使われているか / (b) preview と editor の双方で使われているか — 後者の場合は KMM neutral DTO 経由を選択する。

- [ ] P5-F-001: `vendor/egui_commonmark_upstream` の使用箇所を特定する。
- [ ] P5-F-002: `egui_commonmark_backend` 使用箇所を特定する。
- [ ] P5-F-003: `CommonMarkCache` 使用箇所を特定する。
- [ ] P5-F-004: syntax highlight dependency を KLE / KDV どちらに寄せるか確定する。
- [ ] P5-F-005: preview rendering dependency を KDV に寄せる。
- [ ] P5-F-006: editor syntax highlight dependency を KLE に寄せる。
- [ ] P5-F-007: `[patch.crates-io] egui_commonmark` 削除 PR を準備する。
- [ ] P5-F-008: `[patch.crates-io] egui-winit` 削除条件を確認する。
- [ ] P5-F-009: mathjax_svg patch の所有者を KDV forge に寄せる。
- [ ] P5-F-010: vendor cleanup regression test を追加する。

---

## P6. KMM canonical 化

### P6-A. Canonical switch

- [ ] P6-A-001: `katana-markdown-model` README を canonical model として更新する。
- [ ] P6-A-002: `katana-markdown-engine` README に compatibility / migration notice を追加する。
- [ ] P6-A-003: KatanA OpenSpec `adopt-kme-in-katana` を `adopt-kmm-in-katana` に読み替える提案を作る。
- [ ] P6-A-004: KDV から KMM を参照する dependency を追加する。
- [ ] P6-A-005: KLE は KMM に **hard dependency を持たない**。markdown source mapping は `SourceAnchorAdapter` trait 経由で受け取り、KMM 用 implementation は `katana-language-editor-md` crate に閉じる。この方針を ADR `docs/adr/kle-kmm-dependency.md` に記録する。検査: P3-E-007 の script で KLE crate が KMM / markdown 系 crate を import していないことを確認する。
- [ ] P6-A-006: KUC が KMM に依存しないことを guardrail に追加する。
- [ ] P6-A-007: KMM document model fixture を KDV forge fixture に共有する。
- [ ] P6-A-008: KMM metadata target を viewer unresolved 表示 DTO に変換する。
- [ ] P6-A-009: KMM metadata target を editor anchor DTO に変換する。
- [ ] P6-A-010: KMM reconciliation を editor save flow に接続する。

---

## P7. KCF / KDR / KDV forge 再編

### P7 スコープ宣言

Phase 7 は **KCF crate 側の public API を縮小し、CLI を KDV delegate に切り替える**フェーズ。

- 触る対象: KCF crate / KCF CLI / KCF README / KCF release note。
- 前提: Phase 2 で KDV 側 facade が完成し、KDV が KCF を内部 backend として呼べていること。
- Phase 2 で作成済みの「KCF→KDV 変換 (P2-E)」を canonical mapping として固定化し、KCF 側の旧 public 表記を deprecated に倒す。
- 重複と見える `RenderInput → BuildRequest` 等は、Phase 2 = 実装、Phase 7 = canonical 固定 + deprecation という別作業として扱う。

### P7-A. Responsibility split

- [ ] P7-A-001: KCF rendering responsibilities を一覧化する。
- [ ] P7-A-002: KCF export responsibilities を一覧化する。
- [ ] P7-A-003: KDR rendering responsibilities を一覧化する。
- [ ] P7-A-004: KCF と KDR の duplicated renderer types を一覧化する。
- [ ] P7-A-005: KDR を diagram rendering canonical として ADR に記録する。
- [ ] P7-A-006: KDV forge を export canonical として ADR に記録する。
- [ ] P7-A-007: KCF は compatibility runtime として ADR に記録する。

### P7-B. API migration

前提: Phase 2 (P2-E) で KDV 側に KCF→KDV の内部変換が完成していること。Phase 7 ではその mapping を canonical schema として固定し、KCF 側の重複 public 表記を deprecated に倒す。

- [ ] P7-B-001: `RenderInput → BuildRequest` mapping を KDV 側 canonical schema として ADR に記録する。KCF 側の `RenderInput` public 表記に `#[deprecated]` を付ける条件を明記する。
- [ ] P7-B-002: `RenderConfig → BuildProfile` mapping を canonical schema として ADR に記録する。KCF 側の `RenderConfig` 表記の deprecation 条件を明記する。
- [ ] P7-B-003: `RenderOutput → Artifact` mapping を canonical schema として ADR に記録する。KCF 側の `RenderOutput` 表記の deprecation 条件を明記する。
- [ ] P7-B-004: `RenderDiagnostics → ForgeDiagnostics` mapping を canonical schema として ADR に記録する。KCF 側 type の deprecation 条件を明記する。
- [ ] P7-B-005: KCF export command の input shape を `ExportRequest` 互換に縮小し、ADR に invariants を記録する。
- [ ] P7-B-006: KCF CLI 各 subcommand を KDV `cli_api` delegate 実装に置き換える PR を作る。
- [ ] P7-B-007: KCF README に「Phase 2 以降は KDV facade 経由の利用を推奨」「KCF public API は transitional」と policy を追加する。
- [ ] P7-B-008: KCF release note に future deprecation timeline (target version, removal candidate) を記載する。

### P7-C. Runtime assets

- [ ] P7-C-001: Mermaid runtime asset policy を KDV forge docs に移す。
- [ ] P7-C-002: Draw.io runtime asset policy を KDV forge docs に移す。
- [ ] P7-C-003: ZenUML runtime asset policy を KDV forge docs に移す。
- [ ] P7-C-004: checksum validation policy を KDV forge docs に移す。
- [ ] P7-C-005: reference comparison policy を KDV forge docs に移す。
- [ ] P7-C-006: KCF の runtime gate を KDV forge gate へ mirror する。
- [ ] P7-C-007: asset update command の ownership を整理する。

---

## P8. Documentation / OpenSpec

### P8-A. Shared documents

- [ ] P8-A-001: canonical 配置は `katana/docs/architecture/ui-separation/` (principles.md + detailed-design-and-tasks.md + README.md)。各 repo の `docs/ui-separation-plan.md` 抜粋ファイルとの drift 検査スクリプトを CI に追加する (master の task ID と抜粋ファイルの task ID 一致を確認)。
- [ ] P8-A-002: `katana-ui-core` README に neutral core 方針を追加する。
- [ ] P8-A-003: `katana-document-viewer` README に viewer+forge 方針を追加する。
- [ ] P8-A-004: `katana-language-editor` README に editor domain 方針を追加する。
- [ ] P8-A-005: `KatanA` README に architecture overview を追加する。
- [ ] P8-A-006: `katana-canvas-forge` README に transitional backend 方針を追加する。
- [ ] P8-A-007: `katana-diagram-renderer` README に pure renderer 方針を追加する。
- [ ] P8-A-008: `katana-markdown-model` README に canonical model 方針を追加する。

### P8-B. OpenSpec changes

- [ ] P8-B-001: `redefine-katana-ui-core-neutral-core` change を作る。
- [ ] P8-B-002: `expand-katana-document-viewer-forge` change を作る。
- [ ] P8-B-003: `strengthen-katana-language-editor-domain` change を作る。
- [ ] P8-B-004: `compose-katana-ui-main-panel` change を作る。
- [ ] P8-B-005: `thin-katana-app-boundary` change を作る。
- [ ] P8-B-006: `canonicalize-katana-markdown-model` change を作る。
- [ ] P8-B-007: `migrate-kcf-to-kdv-forge-backend` change を作る。
- [ ] P8-B-008: existing Floem phase docs を superseded として記録する。
- [ ] P8-B-009: existing preview intake docs を viewer+forge 方針に更新する。
- [ ] P8-B-010: existing chrome removal docs を MainPanel composition 方針に更新する。

---

## P9. Release strategy

### P9-A. Version sequence

- [ ] P9-A-001: `katana-ui-core` neutral core v0.2.0 を計画する。
- [ ] P9-A-002: `katana-ui-core-floem` v0.1.0 を計画する。
- [ ] P9-A-003: `katana-document-viewer` v0.2.0 を計画する。
- [ ] P9-A-004: `katana-document-preview` compatibility release を計画する。
- [ ] P9-A-005: `katana-language-editor` v0.2.0 を計画する。
- [ ] P9-A-006: `katana-canvas-forge` compatibility release を計画する。
- [ ] P9-A-007: `KatanA` integration release を計画する。

### P9-B. Compatibility policy

各タスクの判断基準: (a) 旧 API の現状利用者数 / 外部利用の有無、(b) Phase 2〜5 の完了見込み時期、(c) SemVer 互換性が保てる minor / patch window と breaking change を許容する major version 境界。決定結果は `docs/release/compatibility-windows.md` に記録する。

- [ ] P9-B-001: preview API (`MarkdownPreview`, `MarkdownSource`, `PreviewOutput` etc.) deprecation window を決める。
- [ ] P9-B-002: KCF CLI compatibility window を決める。delegate 移行完了 (P7-B-006) からの残存期間も明記する。
- [ ] P9-B-003: KME compatibility window を決める。canonical 切替 (P6-A) からの除却 timeline を含む。
- [ ] P9-B-004: Floem adapter compatibility policy を決める。supported Floem version 範囲、breaking change 取り扱いを含む。
- [ ] P9-B-005: egui adapter temporary support policy を決める。supported 終了の条件 (例: KatanA Floem 移行完了) を明示する。
- [ ] P9-B-006: KatanA app release cutover policy を決める。preview / export / editor / scroll sync の cutover を独立 / 同時のどちらにするかを含む。
- [ ] P9-B-007: KLE port trait の deprecation policy を決める。port 追加は SemVer minor、port 削除や signature 変更は major を要求する。`NoopXxxProvider` の有無や required / optional 区分も明記する。
- [ ] P9-B-008: `katana-language-editor-md` (markdown port adapter) の互換性 window を決める。KMM canonical 切替 (P6-A) との連動も含む。
- [ ] P9-B-009: 互換 adapter (egui / gpui) の SemVer minor 追加・縮小 policy を決める。互換 adapter 起因で primary release を止めない原則も明記する。
- [ ] P9-B-010: primary adapter 切り替え発生時の旧 primary 降格 (互換 adapter 化) policy を決める。降格後のサポート window も含む。

---

## 8. 最初の実装 PR の切り方

最初の PR は `katana-ui-core` の core neutral 化だけに限定する。

### PR-1: `katana-ui-core` architecture rewrite

- [ ] `README.md` を framework-neutral 方針に変更する。
- [ ] `Cargo.toml` の description を変更する。
- [ ] ADR を追加する。
- [ ] dependency leak guard を追加する。
- [ ] `floem` hard dependency をまだ削除せず、現状差分を記録する。
- [ ] `just check` が通ることを確認する。

### PR-2: Floem adapter crate scaffolding

- [ ] `crates/katana-ui-core-floem` を追加する。
- [ ] core から `floem_view` を移動する準備をする。
- [ ] adapter crate の compile test を追加する。
- [ ] Storybook の adapter dependency を確認する。

### PR-3: Core render model scaffolding

- [ ] `UiNodeId` を追加する。
- [ ] `UiNodeKind` を追加する。
- [ ] `UiNode` を追加する。
- [ ] `UiTree` を追加する。
- [ ] `ThemeSnapshot` を追加する。
- [ ] snapshot test を追加する。

### PR-4: Floem dependency removal from core

- [ ] core crate から `floem` dependency を削除する。
- [ ] core crate から `floem_reactive` dependency を削除する。
- [ ] core crate から `floem_renderer` dependency を削除する。
- [ ] all Floem types を adapter crate に移す。
- [ ] dependency leak guard が通ることを確認する。

---

## 9. Done criteria

### Repository-level done

- [ ] `katana-ui-core` core が Floem なしで compile できる。
- [ ] `katana-document-viewer::forge` が UI なしで compile できる。
- [ ] `katana-language-editor` core が UI なしで compile できる。
- [ ] `katana-ui` が editor / viewer internals を所有していない。
- [ ] `katana` が layout / widget / theme の詳細を知らない。

### Product-level done

- [ ] KatanA で Markdown edit が動く。
- [ ] KatanA で document preview が動く。
- [ ] KatanA で diagram rendering が動く。
- [ ] KatanA で export が KDV forge 経由で動く。
- [ ] KatanA で scroll sync が KMM source mapping 経由で動く。
- [ ] KatanA で UI composition が `katana-ui::MainPanel` から起動できる。

### Architecture-level done

- [ ] Floem は adapter 対象であり core dependency ではない。
- [ ] GPUI は adapter 対象であり core dependency ではない。
- [ ] egui は compatibility adapter 以外に残らない。
- [ ] forge は KDV subsystem として public API owner になる。
- [ ] KCF は backend / compatibility layer になる。
- [ ] KMM が canonical document model になる。

---

## 10. 参照した主な根拠

### Uploaded design

- `katana-ui-design-principles.md`
  - `katana-ui-core` を atoms / molecules の超汎用 UI とする。
  - `katana-document-viewer` に preview / export / CLI pipeline を集約する。
  - `forge` を KDV 内 subsystem とする。
  - 実装順を `widget -> viewer -> editor -> katana-ui -> katana` に固定する。

### Repository sources

- `HiroyukiFuruno/KatanA`
  - workspace: `crates/katana-core`, `crates/katana-linter`, `crates/katana-platform`, `crates/katana-ui`
  - current UI dependencies: `eframe`, `egui`, `egui_commonmark`
  - current product scope: Markdown workspace, split preview, diagram rendering, scroll sync

- `HiroyukiFuruno/katana-ui-core`
  - current description: Floem向け共通UI widget
  - current hard deps: `floem`, `floem_reactive`, `floem_renderer`
  - modules: `composite`, `layout`, `primitive`, `theme`, `floem_view`, `overlay_lifecycle`

- `HiroyukiFuruno/katana-document-viewer`
  - current crates: `katana-document-preview`, `katana-document-preview-egui`
  - current trait: `MarkdownPreview`
  - current state: egui implementation scaffold

- `HiroyukiFuruno/katana-language-editor`
  - current neutral trait surface exists
  - egui implementation is scaffold

- `HiroyukiFuruno/katana-canvas-forge`
  - current owner of Mermaid / Draw.io rendering and HTML / PDF / PNG / JPEG export
  - CLI already exists
  - dependency leak guard already exists

- `HiroyukiFuruno/katana-diagram-renderer`
  - diagram rendering only
  - explicitly excludes document export and viewer ownership

- `HiroyukiFuruno/katana-markdown-model`
  - renderer-neutral Markdown document model
  - common interpretation layer for viewers, editors, and export flows

### External references

- Cargo workspaces: https://doc.rust-lang.org/cargo/reference/workspaces.html
- Cargo features: https://doc.rust-lang.org/cargo/reference/features.html
- Rust API Guidelines: https://rust-lang.github.io/api-guidelines/

