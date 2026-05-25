## Context

KatanA v0.22.x では、Mermaid / Draw.io preview と HTML / PDF / PNG / JPEG export の境界が段階的に `katana-diagram-renderer`（kdr）へ寄せられている。一方で、workspace dependency と一部 spec には `katana-canvas-forge`（kcf）前提が残っている。

次期 `katana-document-viewer`（kdv）v0.1.0 は、KatanA の Markdown 表示と export 経路を受け持つ前提の crate である。v0.22.26 では、KatanA 本体から kcf 依存を排除し、文書表示・出力の integration point を kdv v0.1.0 へ完全に載せ替える。

## Goals / Non-Goals

**Goals:**

- KatanA workspace から `katana-canvas-forge` dependency と kcf adapter を削除する。
- `katana-document-viewer = "0.1.0"` を crates.io dependency として追加し、preview / export の主経路にする。
- `katana-diagram-renderer` は crates.io 経由の dependency として追加・維持し、git / path dependency にしない。
- 既存の図形 cache、theme snapshot、export parity の契約を kdv 経由で維持する。
- KatanA 側は kdv / kdr の adapter に限定し、文書描画・出力の本体実装を再作成しない。

**Non-Goals:**

- kdv repository 側の v0.1.0 実装・release 作業は本 change では行わない。
- kdr の公開 API 変更は本 change では行わない。
- Markdown syntax、ユーザー操作、export の保存先 UI は変更しない。
- kcf repository 側の削除・release は本 change では行わない。

## Decisions

1. **kdv を preview / export の主境界にする**

   KatanA は Markdown document、theme snapshot、diagram cache context を kdv adapter へ渡し、表示・出力結果を受け取る。これにより KatanA 本体の責務はアプリ状態管理と UI 接続に限定される。

   代替案として KatanA 側で kcf adapter を残して段階移行する案があるが、v0.22.26 の目的である kcf 依存排除を満たさないため採用しない。

2. **kdr は crates.io dependency として扱う**

   kdr は図形描画の実体であり、KatanA workspace には crates.io の semver dependency として追加・維持する。git dependency や sibling repository の path dependency は、リリース再現性と CI の独立性を下げるため使わない。

3. **cache key は kdv / kdr 由来の runtime 情報を使う**

   既存の図形 cache は、図形内容、theme fingerprint、renderer runtime / profile の差分で無効化される必要がある。kcf 由来の runtime string は撤去し、kdv が渡す kdr の `RenderOutput.runtime` / `RenderOutput.profile` と dependency version から組み立てる。

4. **V8 整合チェックは kcf 排除確認へ置き換える**

   旧 spec は kcf / kdr の V8 pin 整合を重視していた。v0.22.26 では kcf が依存グラフから消えるため、検証は `cargo tree` で kcf が存在しないこと、kdv / kdr 経由の V8-backed renderer が単一に解決されることを確認する。

## Risks / Trade-offs

- [Risk] kdv v0.1.0 の API が KatanA の既存 preview / export 状態を直接表現できない。  
  → Mitigation: adapter 層を薄く作り、足りない情報は kdv 側 API 追加ではなく、まず KatanA 側で必要 DTO を明文化して確認する。

- [Risk] kcf 削除により export の HTML / PDF / PNG / JPEG parity が落ちる。  
  → Mitigation: 既存 fixture と export 回帰テストを kdv 経由へ差し替え、HTML semantics と native output の比較を維持する。

- [Risk] kdr dependency を path / git で仮接続すると release 再現性が壊れる。  
  → Mitigation: tasks に crates.io semver dependency 確認を明示し、`cargo metadata` または `cargo tree` で source を検証する。

- [Risk] kcf 由来の spec / docs / task 記述が残り、今後の実装判断を誤らせる。  
  → Mitigation: active spec と tasks の kcf 前提を kdv / kdr 境界へ更新し、archive 内の履歴は変更対象外として扱う。
