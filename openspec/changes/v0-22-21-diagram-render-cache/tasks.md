# Tasks: v0.22.21 diagram render cache - KatanA

## 要件 (Verbatim Requirements)

```text
Katana の図形描画キャッシュを、OS依存の一時保存領域と図形内容ベースの checksum によって再設計する。
目的は、タブ再表示時や既存タブ復元時に不要なロード・再描画を避け、macOS 以外でも同等にキャッシュが効く状態にすること。
```

## 合意済み制約 (MUST)

- 図形 cache の単位は、AST で抽出した図形系コードブロックとする。
- 永続 payload は SVG ファイルそのものとする。
- `manifest.json` / `cache.json` は導入しない。
- Markdown ファイルの絶対パスから安定ハッシュを生成し、文書ごとの cache 領域を分離する。
- 図形種別とコードブロック本文から content checksum を生成する。
- AST 上の順番や位置は永続 cache key に使わない。
- ドキュメント更新時は、現在の AST に存在しない checksum の SVG を削除する。
- 同じ Markdown 内に同一図形コードブロックが複数ある場合は、同じ SVG を共有してよい。
- PNG / JPEG 等の通常画像は永続 cache 対象外とする。
- タブ移動、タブ切替、選択状態変更、viewport 操作では checksum 判定を実行しない。
- kmm を使う場合、依存はプレビュー側の局所 adapter に閉じる。

## Definition of Ready (DoR)

- [x] KatanA v0.22.21 が現在の branch 上で対応対象として確定していること
- [x] `diagram-render-cache` capability が既存 `openspec/specs/` に存在せず、ADDED として扱えること
- [x] `katana-diagram-renderer (kdr)` の公開 API を変更しない方針で合意していること
- [x] cache payload を SVG ファイルそのものにする方針で合意していること
- [x] 順番ベースの cache key を使わない方針で合意していること
- [x] 実装前に OpenSpec を新設計へ更新すること

## Branch Rule

- **OpenSpec 文書作成ブランチ**: 現在の branch
- **実装ブランチ**: 現在の branch
- **コミット方針**: v0.22.21 リリース準備差分と本 capability の差分を分離してコミットする。
- **注意**: task ごとの branch は作成しない。

## Release Process

- 本 capability は v0.22.21 の patch リリースに含める。
- push 後の GitHub Release 実行はユーザーが行う。
- DoD は `just check-local` 全通過とユーザーの動作確認 OK とする。
- ユーザー動作確認の前に `just check-local` を実行する。動作確認後の lint 修正でデグレードを起こさないため。

## 0. Cleanup Previous Attempt

- [x] 0.1 既存の manifest / JSON payload 前提の実装差分を棚卸しする。
- [x] 0.2 新設計に反する `diagram_cache_key_ignores_document_path` 系のテストを削除または反転する。
- [x] 0.3 `PersistentKey::DiagramRender` など、既存 KV cache へ保存する設計を撤去または未使用化する。
- [x] 0.4 実装前に、仕様逸脱を検出する failing test を追加する。
- [x] 0.5 古い設計の残存確認として `rg "DiagramCacheManifest|PersistentKey::DiagramRender|diagram_cache_key_ignores_document_path|block_[0-9]+" crates scripts Cargo.toml -S` を実行する。

### Definition of Done (DoD)

- [x] OpenSpec と実装が `manifest.json` / `cache.json` 前提を持っていないこと
- [x] 異なる Markdown 絶対パスの同一図形が cache を共有しないことをテストで確認していること
- [x] AST 順番ベースの永続 key が実装に残っていないこと

## Implementation Guardrails

- [x] 仕様を再解釈せず、`spec.md` の Requirement / Scenario に対応するテストから着手する。
- [x] 実装タスクを完了扱いにする前に、対応するテスト名と確認した Scenario を tasks.md または報告に残す。
- [x] 迷った場合は実装を止め、ユーザーに確認する。推測で `manifest` / JSON payload / 順番 key へ戻さない。
- [x] `just check-local` 前に、古い設計の残存 grep を再実行する。

## 1. AST Diagram Block Extraction

- [x] 1.1 Markdown AST から図形系コードブロックを列挙する adapter を追加する。
- [x] 1.2 Mermaid / Draw.io / PlantUML の図形種別とコードブロック本文を取得する。
- [x] 1.3 kmm を使う場合、kmm 型を cache store / path resolver に漏らさない境界を作る。
- [x] 1.4 通常コードブロック、通常画像、PNG / JPEG が永続 SVG cache 対象外であることをテストする。

### Definition of Done (DoD)

- [x] AST から図形コードブロックだけを列挙できること
- [x] 図形の順番や source position が永続 key に使われていないこと
- [x] 将来の `katana-document-viewer` 分離時に移植対象が局所化されていること

## 2. SVG Cache Path and Checksum

- [x] 2.1 OS 標準 cache root 配下の `${os_cache_dir}/KatanA/.cache/diagrams` を解決する。
- [x] 2.2 Markdown 絶対パスから `doc_<absolute_path_hash>` を生成する。
- [x] 2.3 図形種別とコードブロック本文から `content_checksum` を生成する。
- [x] 2.4 SVG ファイル名を `<content_checksum>_<renderer_version>_<theme_hash>.svg` にする。

### Definition of Done (DoD)

- [x] 同じ絶対パスは毎回同じ document cache directory になること
- [x] 異なる絶対パスは同じ図形内容でも別 document cache directory になること
- [x] 図形本文が変わると `content_checksum` が変わること
- [x] tab state / viewport / selection / AST order / source position が checksum に入らないこと

## 3. SVG Cache Store

- [x] 3.1 SVG ファイルの読み込みによる cache hit 判定を実装する。
- [x] 3.2 cache miss 時に kdr で SVG を生成して保存する。
- [x] 3.3 SVG 書き込みを atomic write にする。
- [x] 3.4 破損 SVG または読み込み失敗時は cache miss として再描画する。
- [x] 3.5 `manifest.json` / `cache.json` を使わないことをテストで固定する。

### Definition of Done (DoD)

- [x] cache hit 時に kdr が呼ばれないこと
- [x] cache miss 時に SVG が生成され、次回 hit すること
- [x] 破損 SVG がユーザー向け例外にならず、再描画 fallback されること

## 4. Prune Removed Diagram SVGs

- [x] 4.1 ドキュメント更新後の AST から現在存在する `kind + content_checksum` 集合を作る。
- [x] 4.2 文書別 cache directory 内で、現在存在しない checksum の SVG を削除する。
- [x] 4.3 7 個から 6 個へ減り、真ん中の図形が削除されたケースをテストする。
- [x] 4.4 図形順序だけが変わった場合に SVG を再利用するケースをテストする。
- [x] 4.5 同一 Markdown 内の完全一致図形が同じ SVG を共有するケースをテストする。

### Definition of Done (DoD)

- [x] 削除された図形の SVG が残らないこと
- [x] 並び替えだけで再描画されないこと
- [x] 末尾削除だけに依存しない prune になっていること

## 5. Preview Integration

- [x] 5.1 新規タブ作成時に SVG cache を評価する。
- [x] 5.2 アプリ起動時の既存タブ復元で SVG cache を評価する。
- [x] 5.3 ドキュメント更新時に miss 分だけ再描画し、prune を実行する。
- [x] 5.4 タブ移動、タブ切替、選択状態変更、scroll / zoom だけでは cache 判定を実行しない。
- [x] 5.5 cache hit 時に画面が `Rendering Mermaid...` 等の描画中表示へ戻らないことを確認する。

### Definition of Done (DoD)

- [x] タブ切り替えだけでは kdr が呼ばれないこと
- [x] 保存済み SVG がある図形は即時表示されること
- [x] ユーザー検証でタブ再表示時の再描画感が解消していること

## 6. Observability and Validation

- [x] 6.1 `diagram_cache_hit` / `diagram_cache_miss` / `diagram_cache_pruned` / `diagram_cache_corrupt_svg` / `diagram_cache_redraw_executed` / `diagram_cache_skipped_by_tab_switch` を記録する。
- [x] 6.2 `renderer_version` の更新ルールを PR description / CHANGELOG に記載する。
- [x] 6.3 `./scripts/openspec validate v0-22-21-diagram-render-cache --strict` を実行する。
- [x] 6.4 `just check-local` を実行する。
- [x] 6.5 ユーザー動作確認 OK を得る。

### Definition of Done (DoD)

- [x] metric が `diagram_cache_*` の snake_case で揃っていること
- [x] OpenSpec 検証（strict）がエラー 0 で通ること
- [x] `just check-local` が全通過すること
- [x] ユーザーの動作確認が OK であること

## Validation Log

- [x] `./scripts/openspec validate v0-22-21-diagram-render-cache --strict`
- [x] `just check-local`
- [x] `rtk cargo test -p katana-ui state::toc_tests --lib -- --nocapture`
- [x] `rtk cargo test -p katana-ui views::panels::toc::tests --lib -- --nocapture`
- [x] `rtk cargo check -p katana-ui --lib`
- [x] `rtk just ast-lint`
- [x] ユーザー動作確認 OK

## User Review Feedback

- [x] 図形描画崩れは kdr v0.1.2 取り込み後にユーザー検証で解消確認済み。
- [x] コード表示時の目次ジャンプと現在位置ずれを、共通の目次 current state と AST anchor 連携で修正済み。
- [x] 目次側の自動スクロール追従が、目次自体をスクロールする操作を阻害しないように修正済み。
- [x] 本文側の高速スクロール時に目次ハイライトがガタつく問題は、同一候補が 0.025 秒継続した場合だけ反映する安定化で調整済み。
