# Katana UI 設計思想

作成日: 2026-05-16  
更新日: 2026-05-16

## 結論

Katana 本体から UI 層を分離し、UI の変更は `katana-ui` 側で完結させる。

`katana-ui-core` は Katana 専用部品ではなく、atoms / molecules 単位の超汎用 UI ライブラリとして扱う。

`katana-document-viewer` は単なる preview 専用 crate ではなく、文書表示、preview、export、CLI から呼び出せる document pipeline を含む文書アーティファクト基盤として扱う。

そのため、`forge` は当面独立 crate にせず、`katana-document-viewer` 内の生成・変換・export サブシステムとして内包する。

最終的な理想形は、`katana` 本体が UI として管理すべきではない責務のみを持ち、メインパネルに `katana-ui` を組み込むだけの構成である。

---

## 基本方針

Katana の設計では、UI を単なる見た目の層としてではなく、独立したプロダクトとして扱う。

UI に関する変更、Widget の追加、レイアウト調整、テーマ変更、インタラクション改善は、可能な限り `katana-ui` / `katana-ui-core` 側で完結させる。

一方で、Katana 本体はアプリケーションコアとして振る舞い、UI の詳細を知らない状態を維持する。

また、文書表示、preview、export、CLI 用の出力処理は同じ文書アーティファクトの lifecycle に属するため、初期段階では `katana-document-viewer` に集約する。

---

## レイヤ構成

```text
katana
  └─ 起動 / 設定 / プロジェクト状態 / コマンド / 永続化 / アプリケーション境界

katana-ui
  └─ MainPanel / Dock / Panel composition / Layout orchestration / Theme application

katana-ui-core
  └─ atoms / molecules / primitive widgets / theme tokens / basic interactions

katana-language-editor
  └─ buffer / cursor / selection / diagnostics / editing model

katana-document-viewer
  ├─ document view / live preview / artifact view / scroll sync
  ├─ forge: build / transform / export / artifact generation
  └─ cli-facing API: CLI から呼び出せる document pipeline
```

---

## `katana` の責務

`katana` 本体は UI を所有しない。

主な責務は以下に限定する。

- アプリケーション起動
- 設定読み込み
- プロジェクト状態管理
- コマンド実行
- 永続化
- プラグイン境界
- ドメインモデル管理
- UI 層へ渡すアプリケーション状態の提供

`katana` は `katana-ui::MainPanel` を組み込み、必要な状態やイベントハンドラを渡すだけにする。

---

## `katana-ui` の責務

`katana-ui` は Katana 専用の画面合成レイヤである。

主な責務は以下。

- MainPanel
- Dock layout
- EditorPane / DocumentViewerPane の配置
- Split layout
- Panel orchestration
- Theme の適用
- UI 状態の接続
- `katana-language-editor` / `katana-document-viewer` の UI adapter 呼び出し

重要なのは、`katana-ui` が editor / document viewer の中身を所有しないこと。

`katana-ui` は editor / document viewer を画面上の構成要素として取り込み、配置・接続する側である。

---

## `katana-ui-core` の責務

`katana-ui-core` は **framework-neutral な UI Core** として扱う。

「widget (atoms / molecules)」だけでなく、**Application / window / surface を含む UI Core 全体**を所有する。利用側はこの crate に依存するだけで、Floem / GPUI / egui のような framework を直接知らずにアプリを起動から描画まで構築できる。

Katana 固有の概念を入れない。

想定する責務は以下。

### Runtime / Window / Surface

- Application (`Application::new().window(...).run()` のような entry point)
- AppConfig / AppHandle / AppLifecycle
- Window / WindowId / WindowConfig / WindowEvent / WindowCommand / WindowManager
- DisplayInfo (multi-monitor)
- Surface / FrameHandle / PaintRequest / SurfaceMetrics

window API の neutral 化粒度は **「中」**: title / size / close / focus / fullscreen / multi-window / icon を共通サポート。platform menu / IME / drag&drop は adapter 経由 escape hatch で対応。

### Widget primitives (atoms / molecules)

- Text / Button / Icon / Input / Checkbox / Radio / Badge / Divider / Spacer
- Card / List / Menu / Tooltip / Modal / Tabs / Toolbar / FormField / SplitPane / Breadcrumb
- Row / Column / Stack / Grid / ScrollArea
- Theme token / Primitive interaction
- Event model (UiEvent / PointerEvent / KeyboardEvent / FocusEvent / CommandEvent)
- Render model (UiTree / UiNode / UiNodeKind / UiProps)
- Adapter contract trait

### 公開 API の neutral 性

- public API は adapter 型 (Floem View / GPUI Element / egui Ui 等) を返さない。常に neutral DTO / trait を介する。
- adapter (Floem / GPUI / egui / 互換) は別 crate (`katana-ui-core-floem` 等) に分離し、core compile に framework 依存を引き込まない。

### 依存禁止

`katana-ui-core` は `katana`、`katana-language-editor`、`katana-document-viewer`、`katana-markdown-model` に依存してはならない。

この層に editor / document viewer / forge / markdown の概念が入ると、汎用 UI Core ではなく Katana 専用部品集になるため、再利用性が落ちる。

---

## `katana-language-editor` の責務

`katana-language-editor` は **language-neutral な編集ドメイン**を所有する。

markdown / Rust / TypeScript など特定言語に縛られない。利用側が「どの言語向けに使うか」を実行時に決める設計とする。default 言語実装すら持たない。

主な責務は以下。

- buffer
- cursor
- selection
- diagnostics 受け取り
- editing model
- editor 内部状態
- 外部から受け取る port 定義（syntax / jump / hover / completion / formatter など）

`katana-language-editor` 自身は syntax highlight / jump / semantic 解析を実装しない。これらは port (trait) として宣言し、利用側 (KatanA など) が言語固有の implementation を inject する。

```text
katana-language-editor
  ├─ editor domain (language-neutral)
  └─ ports (trait)
      ├─ SyntaxHighlightProvider
      ├─ JumpProvider
      ├─ HoverProvider
      ├─ CompletionProvider
      └─ DiagnosticsSource
        ↑
      利用側が言語固有 implementation を inject
        ↑
      例: KatanA から markdown 用 SyntaxHighlightProvider を渡す
```

`katana-language-editor` は `katana-ui` を知らない。
`katana-language-editor` は `katana-markdown-model` (KMM) にも依存しない。markdown 利用は KatanA / KDV 側で port adapter を提供する形で実現する。

UI 表示が必要な場合は、`katana-ui` 側が editor adapter を通じて `EditorPane` として組み込む。

---

## `katana-document-viewer` の責務

`katana-document-viewer` は、文書アーティファクトの表示と生成 pipeline を所有する。

単なる preview 専用ではなく、以下を扱う。

- live preview
- PDF view
- Office document view
- HTML / image / bundle artifact view
- document artifact model
- scroll sync
- export result の確認
- CLI から呼び出せる変換・出力 API

`preview` は `katana-document-viewer` の機能の一つとして扱う。

`viewer` という名前は、表示専用という意味ではなく、文書アーティファクトを扱う surface / subsystem の名前として使う。

---

## `forge` の扱い

`forge` は独立 crate ではなく、当面は `katana-document-viewer` の内部サブシステムとして扱う。

責務は以下。

- document build
- transform
- export
- artifact generation
- intermediate representation の生成
- PDF / HTML / Office / image / bundle などへの出力準備

構成は以下。

```text
katana-document-viewer
  ├─ viewer_surface
  │   └─ artifact display / preview / sync
  │
  ├─ forge
  │   └─ build / transform / export / artifact generation
  │
  └─ cli_api
      └─ CLI から forge を呼び出すための薄い API
```

`forge` は UI に依存しない。

`forge` が生成した artifact / document model を `viewer_surface` が表示する。

この境界を維持すれば、将来 `forge` を独立 crate に切り出す必要が出ても移行しやすい。

---

## editor / document viewer との関係

`katana-language-editor` と `katana-document-viewer` は分離設計する。

`katana-ui` はそれらを直接所有するのではなく、UI adapter を通じて表示対象として扱う。

```text
katana-ui
  ├─ EditorPane          ──> katana-language-editor adapter
  └─ DocumentViewerPane  ──> katana-document-viewer adapter
```

この構成により、editor / document viewer の内部実装を変更しても、`katana-ui-core` には影響しない。

また、`katana-ui` は Katana 専用の画面合成だけに集中できる。

---

## 作る順番

実装順は以下に固定する。

```text
1. katana-ui-core
   ↓
2. katana-document-viewer
   └─ forge を内部サブシステムとして含める
   ↓
3. katana-language-editor
   ↓
4. katana-ui
   ↓
5. katana
```

### 1. `katana-ui-core`

最初に atoms / molecules の UI 基盤を固める。

この段階では Katana 固有概念を入れない。

先に primitive widgets、theme token、layout primitive、basic interaction を固めることで、後続の editor / document viewer / katana-ui が同じ UI 語彙を使えるようにする。

### 2. `katana-document-viewer`

次に document viewer を作る。

この段階で `forge` も内部サブシステムとして実装する。

理由は、preview、export、CLI 用変換、PDF / Office 表示は、同じ document artifact pipeline を共有するためである。

この段階で最低限作るものは以下。

- document model
- artifact model
- preview surface
- forge pipeline
- export API
- CLI から呼び出せる thin API

### 3. `katana-language-editor`

次に editor を作る。

editor は buffer / cursor / diagnostics / editing model を所有する。

UI 表示は `katana-ui-core` の語彙に合わせるが、`katana-ui` には依存しない。

### 4. `katana-ui`

最後に `katana-ui` を作る。

`katana-ui` は `EditorPane` と `DocumentViewerPane` を配置し、MainPanel / Dock / Split layout / Panel orchestration を担当する。

この段階まで `katana-ui` を後回しにする理由は、先に widget、viewer、editor の責務境界を固めないと、画面合成レイヤが過剰に責務を持ちやすいためである。

### 5. `katana`

最後に `katana` 本体へ組み込む。

`katana` は UI 詳細を知らず、`katana-ui::MainPanel` を起動・接続するだけにする。

---

## CLI の扱い

現時点では、責務ごとの CLI 提供を維持する。

ただし、`katana-document-viewer` は CLI から呼び出せる `cli_api` を提供する。

将来的に `katana-cli` へ統合する場合でも、CLI 本体は薄い入口に留め、実処理は `katana-document-viewer::forge` に委譲する。

```text
katana-cli or responsibility-specific CLI
  └─ katana-document-viewer::cli_api
       └─ katana-document-viewer::forge
            └─ artifact generation
```

この構成により、CLI の統合可否に関係なく、document pipeline の責務を一箇所に保てる。

---

## 独自 UI Core と adapter 方針

`katana-ui` 自身が **独自の UI 表現 (DSL / Component model)** を持つ。Floem / GPUI / native-renderer は **adapter (出力先) として katana-ui の後ろにある**だけで、`katana-ui` の API がそれらに引っ張られることはない。

汎用 UI として使ってもらうことを狙う場合、人気フレームワークに完全依存するより、独自 UI Core を持ちつつ adapter を提供する方がよい。

理由は、外部フレームワークを土台にすると、その思想、更新速度、破壊的変更に設計が引っ張られるためである。

理想構成は以下。

```text
KatanA (application)
        ↓ 「どの adapter で起動するか」を選んで katana-ui::MainPanel を呼ぶだけ
katana-ui (独自 UI DSL / Component model)
        ↓
katana-ui Runtime Core (UiTree / ThemeSnapshot / EventSink)
        ↓
gpui-adapter / floem-adapter / native-renderer
```

この構成での重要点。

- `katana-ui` の public API は adapter 型 (Floem View 等) を返さない。常に neutral な UiTree / Component model を返す。
- adapter crate は別 crate に分離し、利用側が選択する。core crate を compile しても floem / gpui への依存は引き込まれない。
- KatanA は `katana-ui` を直接使い、起動時に runtime adapter を 1 つ選ぶ。adapter 切り替えは KatanA 側の choice であって、`katana-ui` の責務ではない。
- adapter 自体も「neutral UiTree → framework view」の薄い変換層に閉じ、katana-ui Component model 側に framework の概念が逆流しないようにする。

この構成では、Katana 側は UI 設計の主導権を持ち続けられる。

利用者側には、既存環境へ差し込める adapter を提供できる。利用者は `katana-ui` の Component model でアプリを書き、好きな adapter で出力する。

### primary adapter と互換 adapter

adapter は性格別に 2 種類を併設する。

- **primary adapter**: KatanA 本体が起動時に使うもの。常に最新の `katana-ui` API に追従し、品質ゲートも core と同等に保つ。当面は 1 系統 (例: floem) を primary とする。
- **互換 adapter**: 外部利用者が既存環境に `katana-ui` を差し込めるようにするための adapter。`katana-ui-core-egui` / `katana-ui-core-gpui` / `katana-ui-core-floem` のように framework 別に crate を提供する。primary に選ばれていない adapter も含めて維持する。

互換 adapter の運用ルール。

- 各互換 adapter は独立 crate / opt-in feature とし、`katana-ui` core の compile には引き込まれない。
- 品質ゲートは core より緩く許容してよい（storybook smoke + adapter compile test を最低ライン）。完全機能網羅は primary adapter にのみ求める。
- 互換 adapter のサポート範囲（対応 widget 一覧、未対応機能、フォールバック挙動）は各 adapter README に明記する。
- 互換 adapter は SemVer の minor で追加・縮小してよい。互換 adapter が落ちても `katana-ui` core / primary adapter / KatanA の release は止めない。
- primary adapter を切り替える場合は ADR を残し、旧 primary は互換 adapter として降格扱いとする。

---

## GPUI / Floem との関係

GPUI や Floem は直接依存先として固定するのではなく、adapter の対象として見る。

- GPUI は Zed 実績があり、エディタ系 UI との親和性が高い
- Floem は Lapce 実績があり、signal ベースの declarative UI として有力
- ただし、どちらも外部利用者向けの安定 API としては注意が必要

そのため、Katana の UI 設計を GPUI / Floem の思想に完全には寄せない。

`katana-ui-core` と `katana-ui` の設計を独立させ、その出力先として adapter を考える。

---

## 非採用方針

以下は現時点では主軸にしない。

- egui
- WebView 前提の UI
- Dioxus / Freya 系
- 特定フレームワークに完全依存する UI 設計

理由は、Katana が狙う UI の方向性が、純 Rust / native / 独自 runtime / 汎用 adapter 提供に寄っているためである。

---

## 依存方向

依存方向は常に上位から下位へ向ける。

```text
katana
  ↓
katana-ui
  ↓
katana-ui-core
```

`katana-ui` は editor / document viewer を画面構成要素として参照する。

```text
katana-ui ──> katana-language-editor
katana-ui ──> katana-document-viewer
```

ただし、`katana-language-editor` や `katana-document-viewer` が `katana-ui` に依存する構成は避ける。

`katana-document-viewer::forge` は UI に依存しない。

```text
katana-document-viewer
  ├─ viewer_surface ──> katana-ui-core
  └─ forge          ──> UI 非依存
```

必要な場合は adapter trait を定義し、依存逆転で接続する。

---

## 最終的な理想形

最終的な構成は以下。

```text
katana
  └─ MainPanel に katana-ui を組み込むだけ

katana-ui
  └─ Katana 専用の画面合成レイヤ

katana-ui-core
  └─ 汎用 UI コンポーネントライブラリ

katana-language-editor
  └─ editor domain

katana-document-viewer
  ├─ document viewer domain
  └─ forge domain
```

Katana 本体は UI を管理しない。

UI として管理すべきものは `katana-ui` に寄せる。

汎用化できる UI 部品は `katana-ui-core` に寄せる。

editor のドメインロジックは `katana-language-editor` に閉じ込める。

文書表示、preview、export、CLI 用変換は `katana-document-viewer` に閉じ込める。

---

## 設計判断の根拠

この設計の根拠は、UI を汎用化したい場合に最も重要なのが Widget 単体ではなく、責務境界だからである。

UI 変更が Katana 本体のリリース、ドメイン設計、editor / document viewer の内部実装に影響しない状態を作ることで、変更容易性と再利用性が高くなる。

また、`katana-ui-core` を atoms / molecules レベルに固定することで、Katana 以外のプロダクトでも使える UI 基盤として育てられる。

`forge` を初期段階で独立させず `katana-document-viewer` に含める理由は、preview、export、CLI 変換が同じ document artifact pipeline を共有するためである。

---

## 確認すべき設計ルール

今後の実装では、以下を確認基準にする。

1. `katana-ui-core` が Katana 固有概念を持っていないこと
2. `katana-ui-core` が `katana-language-editor` / `katana-document-viewer` / `katana` に依存していないこと
3. `katana-ui` が editor / document viewer の中身を所有していないこと
4. `katana` 本体が layout / widget / theme の詳細を知らないこと
5. GPUI / Floem などは core 依存ではなく adapter 対象として扱うこと
6. UI 変更が Katana 本体のドメインロジック変更を要求しないこと
7. `katana-document-viewer::forge` が UI に依存していないこと
8. export / CLI 変換処理が `katana-ui` や `katana` に漏れていないこと
9. `katana-ui` の public API が adapter 型（Floem View / GPUI Element 等）を返していないこと
10. `katana-language-editor` が `katana-markdown-model` を含む特定言語 model に依存していないこと
11. `katana-language-editor` の syntax / jump / hover / completion / formatter が port (trait) として宣言され、default implementation を持っていないこと
12. markdown 用 implementation は KatanA / KDV 側で port adapter として inject されていること

---

## 短い判断基準

迷った場合は、以下で判断する。

```text
これは Katana 固有の画面構成か？
  → yes: katana-ui

これは汎用的な UI 部品か？
  → yes: katana-ui-core

これは editor の内部ロジックか？
  → yes: katana-language-editor

これは文書表示 / preview / export / artifact generation か？
  → yes: katana-document-viewer

これは export / transform / artifact build か？
  → yes: katana-document-viewer::forge

これはアプリケーション全体の状態・設定・永続化か？
  → yes: katana
```

---

## Cargo workspace 前提

この設計は Rust の Cargo workspace と相性がよい。

Cargo workspace は、複数の関連 package をまとめて管理する仕組みであり、workspace 内の package は共通の `Cargo.lock` と共通の出力ディレクトリを共有できる。

そのため、`katana`、`katana-ui`、`katana-ui-core`、`katana-language-editor`、`katana-document-viewer` を同一 workspace 内で並行開発する構成に向いている。

参考: The Cargo Book - Workspaces  
https://doc.rust-lang.org/cargo/reference/workspaces.html
