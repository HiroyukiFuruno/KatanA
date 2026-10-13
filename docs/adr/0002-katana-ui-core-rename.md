# ADR-0002: `katana-ui-widget` を `katana-ui-core` にリネームし、UI Core の責務を明示する

ステータス: Accepted  
決定日: 2026-05-17  
関連: [UI 分離 master](../architecture/ui-separation/detailed-design-and-tasks.md), [principles.md](../architecture/ui-separation/principles.md)

## コンテキスト

`katana-ui-widget` は当初「Floem 向け共通 UI widget の共通基盤」として設計された。今回の UI 分離構想 (2026-05-16) で以下を整理した結果、本 repo の責務が「widget (atoms / molecules) の集合」を超えていることが明らかになった。

本 repo の実体は **framework-neutral な UI Core** であり、以下を所有する。

- Component model / render model (`UiTree` / `UiNode` 等)
- theme token
- event model
- adapter contract (Floem / GPUI / egui / native への変換 trait)
- **window / runtime / surface API** (Application::new().window(...).run() のような entry point)
- atoms / molecules (widget primitive はその一部)

「widget」という名称はこの全体像を表現しきれず、以下のリスクを生む。

- 利用側が「atoms / molecules しか入っていない crate」と誤認する
- window / runtime API の追加が「widget の範囲外」と判断され開発が遅れる
- 外部利用者の発見性が下がる (UI Core を探す人は `core` を検索する)

## 決定

1. **GitHub repo 名** を `katana-ui-widget` → `katana-ui-core` にリネームする。
2. **Cargo crate 名** も `katana-ui-core` に変更する。
3. **adapter crate 名** を `katana-ui-widget-floem` 等から `katana-ui-core-floem` / `katana-ui-core-egui` / `katana-ui-core-gpui` に変更する。
4. **storybook crate 名** を `katana-ui-core-storybook` に変更する。
5. **略語** を `KUW` → `KUC` に変更する。新規 docs では KUC を使う。
6. UI 分離構想の Phase 1 で **window / runtime / surface module** を core に追加する (window API の neutral 化粒度は「中」: title / size / close / focus / fullscreen / multi-window / icon)。

実施タイミング:

- ADR (本ドキュメント) + master / 抜粋 / overview README / principles の rename: **2026-05-17 完了**
- Cargo.toml / Justfile / scripts / GitHub repo rename / 関連 repo の dep 表記: P0-B 系タスクで追加実施 (別 PR)
- 過去 OpenSpec changes / 既存 README の `katana-ui-widget` 表記は **履歴として残す** (新規 docs のみ KUC 表記)

## 理由

- 未公開段階 (crates.io publish 未実施 / 外部利用者なし) なので rename コストが最小。
- 命名と実体の一致は長期的な保守性に効く。
- KUC 略語は「core」を直訳しており、UI 分離構想の他 crate 略語 (KDV / KLE / KCF / KDR / KMM / KCU) と長さが揃う。
- window / runtime / surface を core に置くことで、Floem / GPUI / egui の各 adapter が共通の neutral API を変換するだけになり、adapter 実装の複雑度が下がる。

## 代替案

### 案 A: rename せず `katana-ui-widget` のまま runtime / window module を追加

却下理由: 名前と実体のズレが大きくなり、外部利用者の発見性も悪化する。

### 案 C: 2 crate に分割 (`katana-ui-runtime` + `katana-ui-widget`)

却下理由: adapter crate も 2 系統 (runtime adapter + widget adapter) となり、利用側 dep 数が増える。1 crate に集約してもサイズ的に問題ない見込み。

### window API の neutral 化粒度を「最小」または「最大」にする

却下理由:

- 最小では editor 系プロダクトで実用上必要な fullscreen / multi-window がカバーできない。
- 最大では platform menu / IME / drag&drop まで neutral 化が必要となり、Floem / GPUI / egui で差異が大きく実装が破綻しやすい。
- 「中」は 3 adapter 共通サポート範囲とほぼ一致し、特殊機能は adapter 経由 escape hatch (`adapter_contract` 拡張) で対応可能。

## 影響

### 直ちに影響を受けるファイル (本 ADR と同時に更新)

- `katana/docs/architecture/ui-separation/principles.md`
- `katana/docs/architecture/ui-separation/detailed-design-and-tasks.md`
- `katana/docs/architecture/ui-separation/README.md`
- `katana/docs/architecture/ui-separation/phase-5-katana-integration.md`
- `katana-ui-widget/docs/ui-separation-plan.md` (内容のみ。ファイル位置は repo rename 時に追従)
- `katana-document-viewer/docs/ui-separation-plan.md`
- `katana-language-editor/docs/ui-separation-plan.md`
- `katana-canvas-forge/docs/ui-separation-plan.md`
- `katana-diagram-renderer/docs/ui-separation-plan.md`
- `katana-markdown-model/docs/ui-separation-plan.md`
- `katana-chat-ui/docs/ui-separation-plan.md`

### P0-B 系タスクとして別 PR で実施するもの

- `katana-ui-widget` repo の GitHub rename (`katana-ui-core`)
- 各 Cargo.toml の `name = "katana-ui-widget"` を `"katana-ui-core"` に変更
- adapter / storybook crate の Cargo.toml 同様変更
- Justfile / scripts / release / publish dry-run / verify-release-target の参照更新
- README / CONTRIBUTING / OpenSpec project.md (新規部分のみ) 更新
- 関連 repo (KDV / KLE / KCF / KDR / KMM / KCU / KatanA) の `Cargo.toml` dep 表記の追従

### 過去 OpenSpec changes / 履歴文書

既存の `openspec/changes/` 配下と過去 PR / handoff / tmp 文書は **触らない**。`katana-ui-widget` 表記を歴史的事実として残す。新規作成する OpenSpec changes は `katana-ui-core` 表記とする。

## 検証

- [ ] master / 抜粋ファイルで「新規 docs として書かれた `katana-ui-widget` 表記」が 0 件 (ADR ファイル内の歴史的言及を除く)。
- [ ] 略語表に KUC が登録され、新規 docs で KUW が使われていない。
- [ ] window / runtime / surface module が master 5.1 と P1 タスクに追加されている。
- [ ] P0-B に rename 実施タスクが起票されている。
