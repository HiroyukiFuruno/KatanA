## ADDED Requirements

### Requirement: V8 を使う図形プレビュー依存関係はバージョン整合している

システムは、Mermaid / Draw.io プレビュー（preview）で利用する V8 を使う描画依存関係（V8-backed renderer dependencies）を、作業領域（workspace）内で単一の互換 `v8` バージョンに揃えなければならない（MUST）。対応済み図形ブロック（diagram block）は、`katana-canvas-forge` と `katana-diagram-renderer` の `v8` 固定指定（pin）不整合によりワーカー（worker）起動前に失敗してはならない（MUST NOT）。

#### Scenario: 作業領域の依存関係が同じ V8 固定指定を使う

- **WHEN** KatanA v0.22.19 向けに作業領域の依存関係（workspace dependencies）を解決する
- **THEN** `katana-canvas-forge` は `0.1.7` として解決される
- **THEN** 作業領域の `v8` は `=147.4.0` として解決される
- **THEN** `katana-canvas-forge` と `katana-diagram-renderer` は競合する `v8` バージョンを要求しない

#### Scenario: Mermaid プレビューのワーカーは描画前に切断されない

- **WHEN** 開いている Markdown 文書に対応済み Mermaid ブロックが含まれる
- **THEN** プレビューは V8 を使う描画ワーカーをバージョン競合による panic なしで起動する
- **THEN** 描画を試みる前に、ブロックが `[Mermaid] Diagram render worker disconnected before producing a result.` へ置換されない

#### Scenario: Draw.io プレビューは整合した実行環境を使う

- **WHEN** 開いている Markdown 文書に対応済み Draw.io ブロックが含まれる
- **THEN** プレビューは Mermaid 描画と同じ、作業領域で整合した V8 実行環境（runtime）を使う
- **THEN** kcf と kdr の `v8` バージョン分裂によりブロックが失敗しない
