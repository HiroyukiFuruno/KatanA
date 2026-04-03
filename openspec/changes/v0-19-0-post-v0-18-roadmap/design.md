## Context

現在の KatanA workspace は `0.15.1` を名乗っているが、OpenSpec 上では `v0.16.0` の slideshow と `v0.17.0` の cross-platform support が active change として進行している。`v0.18.0` は `windows-linux-support` change の target release として定義されており、post-`v0.18.0` の計画はまだ artifact 化されていない。

一方で、今回の 7 concern には既に次の土台がある。

- diagnostics:
  - `crates/katana-linter/src/markdown.rs` に MarkdownDiagnostic、HeadingStructureRule、BrokenLinkRule がある
  - `crates/katana-ui/src/views/panels/problems.rs` に Problems Panel がある
  - `crates/katana-ui/src/app/action.rs` に `RefreshDiagnostics` がある
- menu / command surface:
  - `crates/katana-ui/src/macos_menu.m` に File / View / Settings / Help の native menu がある
  - `crates/katana-ui/src/views/modals/command_palette.rs` と `state/command_palette_providers.rs` に unified command surface がある
  - `crates/katana-ui/src/shell_ui.rs` に hard-coded shortcut がある
- settings:
  - `crates/katana-platform/src/settings/types/*` と `crates/katana-ui/src/settings/*` に settings persistence と UI はある
  - ただし keyboard / AI / attachment strategy 用の dedicated setting model はまだない
- editor / asset:
  - `crates/katana-ui/src/views/panels/editor/ui.rs` は `egui::TextEdit::multiline` をベースにした plain text editor である
  - `crates/katana-core/src/preview/image.rs` と `openspec/specs/local-asset-preview/spec.md` に local image preview の基盤はある
  - ただし image attach / paste / asset placement / explorer reveal の ingest flow は存在しない
- AI:
  - `crates/katana-core/src/ai/mod.rs` に provider abstraction はある
  - しかし active provider の登録、settings、UI command、local runtime integration は存在しない

この状態から 7 concern を 1-2 release に詰め込むと、command registry、editor model、settings schema、provider integration が同時に揺れて破綻しやすい。そこで roadmap change では、scope を minor version ごとに切り、実装前に dependency chain を固定する。

## Goals / Non-Goals

**Goals:**

- post-`v0.18.0` の草案を `v0.19.0` から `v0.25.0` までの version map として固定する
- 各 minor version について、primary concern、dependencies、affected modules、DoR、DoD、open questions を明示する
- 現行コードに既にある土台と、未実装で追加設計が必要な部分を分離する
- implementation-ready な future changes へ分割する際の判断材料を残す

**Non-Goals:**

- 今回の change 自体で diagnostics、menu、shortcut、editor、LLM を実装すること
- `v0.16.0` / `v0.17.0` の active change を書き換えること
- full markdownlint parity、full WYSIWYG editor、bundled on-device inference runtime を初期スコープとして約束すること
- 各 future release の詳細実装 task をこの change 1 つにすべて書き込むこと

## Decisions

### 1. 7 concern は `v0.19.0` から `v0.25.0` まで 1 minor 1 primary concern で切る

release ごとの焦点が曖昧だと、menu と shortcut、editor と image workflow、LLM foundation と user-facing generation が相互依存で膨らむ。そこで今回の草案は 1 release 1 primary concern を原則にする。

| Version | Primary Concern | Why This Slot |
|---------|-----------------|---------------|
| `v0.19.0` | markdownlint-compatible diagnostics surface | 既存 Problems Panel / diagnostics 基盤を伸ばすだけで価値が出る |
| `v0.20.0` | menu expansion | 既存 menu / command palette の command inventory を整理しやすい |
| `v0.21.0` | customizable shortcuts | `v0.20.0` で整理した command registry を shortcut customization に接続できる |
| `v0.22.0` | editor authoring + image asset workflow | command / settings 基盤を使って authoring UX を足せる |
| `v0.23.0` | local LLM foundation + lint autofix | deterministic diagnostics の次段として価値が高い |
| `v0.24.0` | local LLM document generation | provider runtime が stabilized してから出す方が安全 |
| `v0.25.0` | translation overlay for dynamic/external English | generation と別 release にして UX regressions を切り分ける |

- 採用理由:
  - concern ごとの Done 判定が明確になる
  - regression source を release 単位で追いやすい
  - user feedback を version ごとに回しやすい
- 代替案:
  - `v0.20.0` までに menu + shortcut をまとめる: command registry 設計と user-customization を同時に進めることになり重い
  - `v0.24.0` で document generation と translation overlay をまとめる: provider runtime の不安定要素と UX 評価軸が混ざるため不採用

### 2. `v0.19.0` は full markdownlint parity ではなく official compatibility surface を先に出す

現行 linter は internal rule ID (`md-heading-structure`, `md-broken-link`) と簡易 message を返している。ユーザー要望は official number / message sync だが、full markdownlint engine parity まで要求すると別製品レベルの work になる。初手では「supported deterministic subset に official rule code と英語メッセージを与える」方針にする。

- 採用理由:
  - `katana-linter` と Problems Panel の既存 contract を活かせる
  - UI 価値が早く出る
  - local LLM autofix (`v0.23.0`) の prompt input にも使いやすい
- 代替案:
  - app 内で official markdownlint engine をそのまま埋め込む: Rust desktop app として dependency / packaging コストが重い
  - internal rule ID のまま維持する: user-visible contract が弱く、official docs との往復がしにくい

### 3. `v0.20.0` は menu expansion の前提として command inventory を正規化する

menu、command palette、future shortcut editor は、同じ command catalog を参照しないと衝突と欠落が起きる。よって `v0.20.0` は「見た目の menu 拡張」だけでなく、`AppAction` と user-facing command metadata の中間層を定義する planning を含める。

- 採用理由:
  - `v0.21.0` の shortcut customization の土台になる
  - macOS native menu と non-macOS command surface の差分を埋めやすい
- 代替案:
  - menu item を ad-hoc に増やしてから shortcut system を別実装する: action label / availability / duplicate handling が散らばるため不採用

### 4. `v0.22.0` は Markdown source-first authoring に留め、full rich text editor へは踏み込まない

現行 editor は `egui::TextEdit::multiline` ベースであり、DOM を持つ rich text editor ではない。ここで full WYSIWYG を目指すと editor rewrite になるため、`v0.22.0` は snippet / formatting command / image insertion workflow に絞る。

- 採用理由:
  - 既存 preview / save / dirty buffer 契約を壊さず authoring UX を上げられる
  - image asset workflow を Markdown source と整合的に扱える
- 代替案:
  - full rich text editor を導入する: text model と preview contract の再設計が必要で `0.x` の 1 release では重すぎる

### 5. `v0.23.0` の local LLM は bundled runtime ではなく local endpoint integration を第一候補にする

`katana-core/src/ai/mod.rs` には provider abstraction があるが、runtime 実装はない。初手で llama.cpp や GGUF runtime の bundling まで行うと packaging、cross-platform、model management が一気に増えるため、まずは local endpoint integration を前提にする。

- 採用理由:
  - 既存 provider abstraction に自然に乗る
  - `v0.18.0` の cross-platform 方針とも相性がよい
  - Ollama や OpenAI-compatible local server を first target にしやすい
- 代替案:
  - runtime をアプリ内に同梱する: model 配布、GPU/CPU 差分、platform support が重く不採用

### 6. `v0.25.0` の translation overlay は既存 i18n を置き換えず、dynamic/external strings に限定する

KatanA には locale JSON による app i18n が既にあり、active `v0.17.0` でも system locale default が計画されている。translation overlay は missing locale を補う仕組みではなく、linter message や AI-generated text のような dynamic/external strings だけを対象にする。

- 採用理由:
  - 既存 i18n contract を壊さない
  - translation failure 時の fallback を英語に限定しやすい
- 代替案:
  - app UI 全体を local LLM translation で上書きする: deterministic locale files と衝突するため不採用

## Risks / Trade-offs

- **[Risk] `v0.22.0` の editor concern が広すぎて 1 release で収まらない**  
  -> Mitigation: future concrete proposal では formatting commands と image asset workflow を phase A/B に再分割できるようにする
- **[Risk] official markdownlint compatibility の「どこまで同期するか」が曖昧なまま進む**  
  -> Mitigation: `v0.19.0` の open question として full parity を除外し、supported subset を先に固定する
- **[Risk] shortcut customization が OS reserved shortcuts と衝突する**  
  -> Mitigation: `v0.21.0` の DoR に reserved shortcut policy の定義を入れる
- **[Risk] local LLM runtime choiceが確定しないまま `v0.23.0` に入る**  
  -> Mitigation: Ollama 互換か OpenAI-compatible local endpoint かを user decision として先に切る
- **[Risk] translation overlay が UI を不安定に見せる**  
  -> Mitigation: `v0.25.0` では opt-in と cache policy を明示し、raw English fallback を常に残す
- **[Risk] roadmap change が implementation task の代替になってしまう**  
  -> Mitigation: 本 change は roadmap source of truth に限定し、実装前に各 version を dedicated change へ分解する

## Migration Plan

1. この roadmap change で `v0.19.0` から `v0.25.0` の version map、DoR、DoD、open questions を固定する
2. user との対話で open questions を解消し、concern ごとに assumptions を確定する
3. 実装に入る前に、対象 version 用の dedicated OpenSpec change を新規作成し、この roadmap entry を proposal/design/spec/task へ落とし直す
4. `v0.16.0` / `v0.17.0` 完了後に only-next-version から着手し、後続 versions は再精査して scope freeze する

## Open Questions

- `v0.19.0`: official markdownlint compatibility は rule code / short message / docs link まででよいか、それとも rule behavior parity まで要求するか
- `v0.20.0`: File / View / Help に加えて Export、Window、Tools 相当の command grouping を roadmap 時点で決めるか
- `v0.21.0`: duplicate shortcut 判定に OS reserved shortcut を含めるか、app-local command collision のみに留めるか
- `v0.22.0`: rich text は「Markdown insertion helper」と解釈してよいか、それとも block WYSIWYG に寄せるか
- `v0.22.0`: image attach default は `asset/img/<timestamp>.<ext>` 固定を基本にし、rename dialog を opt-in にするか
- `v0.23.0`: first supported local LLM runtime は Ollama、LM Studio、OpenAI-compatible local server のどれを primary target にするか
- `v0.24.0`: document generation output は active document insertion、new file generation、template-based scaffolding のどれを優先するか
- `v0.25.0`: translation overlay は auto-translate、hover/on-demand translate、cached translated copy のどれを基本 UX にするか
