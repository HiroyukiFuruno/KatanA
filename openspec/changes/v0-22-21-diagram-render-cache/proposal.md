## Why

KatanA の図形（diagram）描画キャッシュは、現状 macOS でのみ部分的に効いており、Windows / Linux では新規タブを開くたびに描画ロジック（render logic）が再実行されている疑いがある。原因は、キャッシュ判定・キャッシュ保存先・再描画トリガーが設計上分離されておらず、ロード処理（load path）と描画処理（render path）が強く結合しているためと考えられる。

タブ移動・選択状態変更・viewport 操作などの「図形内容が変わらないイベント」でも cache invalidation が発生し得るため、新規タブ表示や復元時に体感速度が劣化する。KatanA v0.22.21 では、図形内容のみから生成する content checksum と OS 非依存の cache path resolver を導入し、全 OS で同等の cache hit を実現する。

## What Changes

- 図形内容のみを正規化（canonicalize）する `DiagramContentCanonicalizer` を追加する。
- 正規化済み図形内容から checksum を生成する `DiagramChecksumService` を追加する。
- macOS / Windows / Linux の Katana app 用一時保存領域を抽象化する `PlatformCachePathResolver` を追加する。
- cache manifest と描画 payload を atomic write で保存・取得する `DiagramRenderCacheStore` を追加する。
- 新規タブ作成、ドキュメント更新、アプリ起動時の既存タブ復元のタイミングで checksum を比較する `DiagramRenderCacheCoordinator` を追加する。
- タブ移動・タブ切り替え・選択状態変更・viewport 操作では checksum 判定と再描画を行わない。
- cache miss / checksum mismatch / payload 破損 / OS cache path 解決失敗時の安全な fallback 経路を定義する。
- cache 判定結果を診断（diagnostics）用に metric / log として記録する。

## Capabilities

### New Capabilities

- `diagram-render-cache`: 図形内容ベースの checksum と OS 非依存の cache 保存領域により、新規タブ表示・ドキュメント更新・アプリ起動時復元のタイミングで再描画を抑止する capability。

### Modified Capabilities

- なし

## Impact

- `crates/katana-core`: 図形描画経路（diagram render path）への cache coordinator の差し込み、検証
- `crates/katana-platform`（または新規 module）: OS 別 cache path resolver、atomic write helper
- `crates/katana-ui`: タブ作成・タブ復元・ドキュメント更新イベントから coordinator を呼び出す統合点
- `crates/katana-ui/tests/integration/preview_pane/`: cache hit / miss / mismatch / corrupt の回帰確認
- 観測性（observability）: `diagram_cache_hit` / `diagram_cache_miss` / `diagram_cache_mismatch` / `diagram_cache_corrupt_payload` / `diagram_cache_redraw_executed` / `diagram_cache_checksum_evaluated` / `diagram_cache_checksum_skipped_by_tab_move` の metric / log 追加
