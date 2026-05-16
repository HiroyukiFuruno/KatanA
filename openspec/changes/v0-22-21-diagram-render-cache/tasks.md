# Tasks: v0.22.21 diagram render cache - KatanA

## 要件 (Verbatim Requirements)

```text
Katana の図形描画キャッシュを、OS依存の一時保存領域と図形内容ベースの checksum によって再設計する。
目的は、タブ再表示時や既存タブ復元時に不要なロード・再描画を避け、macOS 以外でも同等にキャッシュが効く状態にすること。
```

### この要件から導出される制約 (MUST)

- 図形内容のみから content checksum を生成し、UI 状態・OS・タブ状態・ファイルパスは含めない。
- 新規タブ作成、ドキュメント更新、アプリ起動時の既存タブ復元のタイミングでのみ checksum 判定を実行する。
- タブ移動、タブ切替、選択状態変更、viewport 操作では checksum 判定を実行しない。
- macOS / Windows / Linux で同一の cache decision ロジックを利用できる構造にする。
- cache miss / checksum mismatch / payload 破損 / OS cache path 解決失敗時は再描画 fallback を実行し、ユーザーに例外を露出させない。

## Definition of Ready (DoR)

- [ ] KatanA v0.22.21 が release/v0.22.21 上で対応対象として確定していること
- [ ] `diagram-render-cache` capability が既存 `openspec/specs/` に存在せず、ADDED として扱えること
- [ ] `katana-diagram-renderer (kdr)` の公開 API（neutral interface）を変更せずに統合できる範囲か確認済みであること
- [ ] OS 別 cache path の方針（macOS / Windows / Linux の Katana app 用一時保存領域）が確定していること

## Branch Rule

- **OpenSpec 文書作成ブランチ**: `release/v0.22.21`
- **実装ブランチ**: `release/v0.22.21`
- **コミット方針**: v0.22.21 リリース準備差分（commit `3fadb238`）と本 capability の差分を分離してコミットする。`Cargo.lock` の更新は本 capability に必要な範囲のみに限定する。

## Release Process

- 本 capability は v0.22.21 の patch リリースに含める。
- 実装完了後、`release-workflow` skill に従い release/v0.22.21 → master の PR を作成する。
- 検証コマンド: `just type-check` / `cargo test -p katana-core` / `cargo test -p katana-ui` / `./scripts/openspec validate v0-22-21-diagram-render-cache --strict`。

## 1. Checksum

- [ ] 1.1 図形内容のみを対象にする canonicalization ルールを定義する（Included / Excluded を spec と一致させる）。
- [ ] 1.2 `DiagramContentCanonicalizer` を追加する。
- [ ] 1.3 `DiagramChecksumService` を追加する。
- [ ] 1.4 UI 状態、OS、タブ状態、ファイルパスが checksum に入らないことを単体テストで確認する。

### Definition of Done (DoD)

- [ ] 同一図形内容で異なるタブ ID / viewport / 選択状態を与えても checksum が一致すること
- [ ] 図形内容を変更すると checksum が変化すること
- [ ] OS 名・ファイルパス・最終閲覧時刻が checksum に含まれないことが単体テストで担保されていること

## 2. Cache Storage

- [ ] 2.1 `PlatformCachePathResolver` を追加する。
- [ ] 2.2 macOS / Windows / Linux の Katana app 一時保存領域を解決できるようにする。
- [ ] 2.3 `CacheManifest` の保存・読み込みを実装する。
- [ ] 2.4 描画 payload の保存・読み込みを実装する。
- [ ] 2.5 manifest と payload の atomic write を実装する。

### Definition of Done (DoD)

- [ ] manifest が `documentId` / `contentChecksum` / `cacheSchemaVersion` / `rendererVersion` / `payloadPath` を保持していること
- [ ] partial write が発生しても manifest と payload が不整合にならないこと
- [ ] OS ごとの cache path 解決が単体テストで通ること

## 3. Cache Decision

- [ ] 3.1 `DiagramRenderCacheCoordinator` を追加する。
- [ ] 3.2 新規タブ作成時に checksum 判定を実行する。
- [ ] 3.3 ドキュメント更新時に checksum 判定を実行する。
- [ ] 3.4 app 起動時の既存タブ復元で checksum 判定を実行する。
- [ ] 3.5 タブ移動時に checksum 判定が実行されないようにする。

### Definition of Done (DoD)

- [ ] cache hit 時に描画ロジックが実行されないことを統合テストで確認していること
- [ ] checksum mismatch 時に再描画と cache 更新の両方が実行されること
- [ ] タブ移動シナリオで checksum 判定が呼ばれないことが統合テストで担保されていること

## 4. Fallback and Recovery

- [ ] 4.1 cache miss 時に従来の描画ロジックへ fallback する。
- [ ] 4.2 checksum mismatch 時に再描画し、cache を更新する。
- [ ] 4.3 payload 破損時に cache を破棄し、再描画する。
- [ ] 4.4 OS cache path 解決失敗時に in-memory cache へ fallback する。
- [ ] 4.5 checksum 生成失敗時に cache を使わず従来描画を実行する。

### Definition of Done (DoD)

- [ ] 全ての失敗パターンでユーザーに例外が露出しないこと
- [ ] in-memory fallback が現セッション内でのみ有効で、永続化しないこと
- [ ] payload 破損ケースで再描画後に cache が再構築されること

## 5. Observability and Validation

- [ ] 5.1 cache hit / miss / mismatch / corrupt_payload / redraw_executed / checksum_evaluated / checksum_skipped_by_tab_move の metric を追加する（snake_case 命名）。
- [ ] 5.2 macOS / Windows / Linux で cache path resolver のテストを追加する。
- [ ] 5.3 タブ移動では checksum 判定が発生しないことをテストする。
- [ ] 5.4 `./scripts/openspec validate v0-22-21-diagram-render-cache --strict` を実行し、エラーを解消する。

### Definition of Done (DoD)

- [ ] metric が `diagram_cache_*` の snake_case で揃っていること
- [ ] OS ごとの cache path テストが CI 上で通ること
- [ ] OpenSpec 検証（strict）がエラー 0 で通ること
- [ ] `rendererVersion` のバンプ運用ルールが PR description / CHANGELOG で明文化されていること
