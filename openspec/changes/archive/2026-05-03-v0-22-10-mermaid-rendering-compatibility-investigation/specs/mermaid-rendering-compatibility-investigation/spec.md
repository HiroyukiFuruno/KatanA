## ADDED Requirements

### Requirement: Mermaid rendering inherits the useful `mmdc` output contract without depending on `mmdc`

システムは、Mermaid 図形を描画するとき、実行時に `mmdc` を必須とせず、旧 `mmdc` 経路が担っていた安定した出力条件を KatanA 管理下で採用した単一 Mermaid renderer 内で満たさなければならない（MUST）。

#### Scenario: Render without Mermaid CLI runtime dependency

- **WHEN** Mermaid 図形を描画する
- **THEN** システムは `mmdc` バイナリの存在を必須条件にしない
- **THEN** システムは OS アプリとしての Chrome / Chromium へ依存しない KatanA 管理下の単一 renderer 経路で表示結果を生成する
- **THEN** システムは Mermaid.js を使わない別実装へ既定で置き換えない

### Requirement: Default diagram preview and HTML export use an app-owned browser runtime

システムは、通常の Mermaid / Draw.io preview と HTML export において、OS にインストールされた Chrome / Chromium アプリではなく、Rust 管理 JS または KatanA 管理下の headless browser / WebView / Chromium runtime から選んだ単一経路を使用しなければならない（MUST）。

#### Scenario: Avoid launching the user's browser app for preview and export

- **WHEN** Mermaid または Draw.io の preview または HTML export が必要になる
- **THEN** システムはユーザーの OS にインストールされた Chrome / Chromium アプリを既定で起動しない
- **THEN** システムは Rust 管理 JS を先に検証し、満たせない場合は KatanA 管理下の高速な headless browser、WebView、Chromium runtime から単一の採用経路を選ぶ
- **THEN** システムは実行時の退避経路（fallback）を持たない
- **THEN** platform ごとの runtime 差分、配布サイズ、sandbox、CI 影響を設計判断として記録する

#### Scenario: Consider a Rust-owned fast headless browser for export

- **WHEN** Rust 製または Rust 管理の高速な headless browser が候補にある
- **THEN** システムは Mermaid / Draw.io preview だけでなく HTML export の diagram rendering でも同じ runtime を評価する
- **THEN** システムは HTML から PDF / PNG / JPEG へ変換する export 経路も、同じ所有境界で扱えるか確認する
- **THEN** システムは用途を満たす場合、その runtime を WebView / Chromium と同じ採用候補として扱う

#### Scenario: Evaluate Rust-owned JavaScript execution only when official JS remains in use

- **WHEN** Rust 管理の JS 実行環境を検討する
- **THEN** システムは公式 Mermaid.js / Drawio.js を実行対象にする
- **THEN** システムは DOM / SVG / layout API の不足が描画互換性を壊さないかを検証する

#### Scenario: Choose a single app-owned runtime from rendering evidence

- **WHEN** Rust 管理 JS が表示互換性または速度の基準を満たせない
- **THEN** システムは高速な headless browser、WebView、Chromium runtime を比較し、単一の採用経路を決める
- **THEN** システムは表示崩れの少なさと速度を主基準にしつつ、配布サイズ、platform 差分、sandbox、CI 安定性も採用理由に含める
- **THEN** システムは OS にインストールされた Chrome / Chromium アプリを採用候補に含めない
- **THEN** システムは不採用経路を実行時の退避経路（fallback）として残さない

#### Scenario: Preserve useful `mmdc` output assumptions

- **WHEN** KatanA の単一 Mermaid renderer が SVG を PNG 化する
- **THEN** renderer は viewport、container 幅、背景色、テーマ、余白、capture 対象、拡大率を明示的な policy として扱う
- **THEN** renderer は `mmdc` の既定出力に近い、過度に横長または余白過多にならない画像を生成する

### Requirement: Compatibility fixtures cover common Mermaid diagram types

システムは、採用した単一 Mermaid renderer の表示崩れを確認するため、代表的な Mermaid 図形 fixture を保持しなければならない（SHALL）。

#### Scenario: Cover common diagram families

- **WHEN** compatibility fixtures are prepared
- **THEN** they include at least flowchart, sequence, class, state, entity relationship, gantt, pie, journey, mindmap, and timeline examples
- **THEN** each fixture includes enough labels, edges, and theme-sensitive elements to reveal visible regressions

#### Scenario: Capture output evidence for user review

- **WHEN** a compatibility result is reported
- **THEN** it includes screenshot or image evidence generated from reproducible commands or `scripts/screenshot`
- **THEN** the evidence distinguishes `mmdc` reference behavior from KatanA Mermaid.js renderer behavior without making `mmdc` a runtime dependency

### Requirement: Fix decisions are prioritized from evidence

システムは、Mermaid.js 描画差分の修正優先度を、再現証跡とユーザー影響に基づいて決めなければならない（MUST）。

#### Scenario: Classify compatibility findings

- **WHEN** a rendering difference is found
- **THEN** it is classified as layout, size, theme, typography, marker, interaction, error handling, or cache behavior
- **THEN** the finding records whether it should be fixed in the renderer, deferred to a later versioned change, or accepted as a documented Mermaid.js difference

#### Scenario: Avoid uncontrolled SVG post-processing

- **WHEN** a proposed fix requires SVG post-processing
- **THEN** the design records why Mermaid initialization settings or container sizing cannot solve it
- **THEN** the post-processing scope is limited to the affected diagram type or SVG element pattern
