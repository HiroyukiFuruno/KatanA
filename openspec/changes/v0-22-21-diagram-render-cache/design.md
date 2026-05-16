## Context

KatanA は Mermaid / Draw.io / PlantUML 等の図形（diagram）プレビューを `katana-diagram-renderer (kdr)` 経由で描画する。現状の描画キャッシュは、判定ロジック・保存先・再描画トリガーが UI 層と強く結合しており、以下の挙動が観測されている。

- macOS 以外では、同一ドキュメントを再オープンしても描画ロジックが再実行され、cache hit にならない。
- タブ移動・タブ切替・viewport 操作など、図形内容が変わらないイベントでも cache invalidation が発生し得る。
- アプリ起動時の既存タブ復元（startup restore）で、図形内容が変わっていなくても描画が走るケースがある。

KatanA v0.22.21 では、`diagram-render-cache` capability として、図形内容のみから生成する content checksum と OS 非依存の cache 保存領域を導入し、上記を解消する。

### Assumptions（前提）

- 既存 OpenSpec に `diagram-render-cache` capability は存在しないため `ADDED Requirements` として記述する。
- 「現在の状態の checksum」は、UI 状態ではなく、最新ドキュメント状態から得られる図形内容（diagram content）の checksum を指す。
- `katana-diagram-renderer (kdr)` の公開 API（neutral interface）は変更せず、KatanA 側で cache coordinator を新設して呼び出し関係だけを更新する。
- 既存ドキュメント永続化フォーマット（document persistence format）は変更しない。

## Goals / Non-Goals

**Goals:**

- 図形内容のみから生成する content checksum を導入する。
- macOS / Windows / Linux で同等の cache hit を実現する。
- 新規タブ作成、ドキュメント更新、アプリ起動時の既存タブ復元に限定して checksum 判定を行う。
- cache miss / checksum mismatch / payload 破損 / OS cache path 解決失敗時の fallback 経路を一本化する。
- cache 判定結果を診断用に metric / log として可視化する。

**Non-Goals:**

- 図形そのものの描画アルゴリズム変更。
- ドキュメント永続化フォーマットの大幅変更。
- ネットワーク共有キャッシュ（network-shared cache）の導入。
- ユーザー操作履歴（user operation history）のキャッシュ。
- タブ移動、選択状態変更、viewport 変更のみを理由とした checksum 再評価。

## Architecture

図形描画キャッシュを、次の 4 つの責務に分離する。

1. `DiagramContentCanonicalizer`: 図形内容のみを安定した順序・形式に正規化する。
2. `DiagramChecksumService`: 正規化済み図形内容から checksum を生成する。
3. `DiagramRenderCacheStore`: OS ごとの Katana app 一時保存領域に cache manifest と描画 payload を保存・取得する。
4. `DiagramRenderCacheCoordinator`: タブ作成、ドキュメント更新、アプリ起動時復元のタイミングで checksum を比較し、cache hit / redraw を決定する。

```text
Tab Open / Document Update / App Startup Restore
        |
        v
DiagramRenderCacheCoordinator
        |
        +--> DiagramContentCanonicalizer
        |        |
        |        v
        |   DiagramChecksumService
        |
        +--> DiagramRenderCacheStore
        |        |
        |        +--> PlatformCachePathResolver
        |        +--> CacheManifest
        |        +--> RenderPayload
        |
        v
cache hit -> hydrate from payload
cache miss / mismatch / corrupt -> run render logic -> write cache atomically
```

## Decisions

1. content checksum は図形内容のみから生成する。

   - **Included**: 図形 ID、図形種別、座標、サイズ、回転、変形、stroke / fill / opacity 等の描画結果に影響する style、z-order、テキスト図形の文字列・フォント指定・文字装飾、グループ階層、図形間参照。
   - **Excluded**: タブ ID、選択状態、hover / focus、viewport / scroll / zoom、OS 名、cache 保存先、ファイルパス、最終閲覧時刻、一時的な renderer runtime state。
   - 理由: viewport・選択状態・タブ状態・OS 差分による不要な cache invalidation を避ける。

2. cache manifest と描画 payload は分離し、manifest だけで checksum 判定する。

   ```text
   CacheManifest
   - documentId
   - contentChecksum
   - cacheSchemaVersion
   - rendererVersion
   - payloadPath
   - createdAt
   - updatedAt
   ```

   - 理由: payload を読まずに高速判定するため。`rendererVersion` または `cacheSchemaVersion` が変わった場合は checksum 一致でも cache miss として扱う。

3. cache 保存先は OS ごとに `PlatformCachePathResolver` 経由で解決する。

   - macOS: Katana app の macOS 用 cache/temp 領域
   - Windows: Katana app の Windows 用 cache/temp 領域
   - Linux: Katana app の Linux 用 cache/temp 領域
   - 理由: アプリロジックから OS 名を直接見ない構造にする。manifest と payload は atomic write とし、部分書き込み（partial write）時の不整合を防ぐ。

4. checksum 判定は次のタイミングに限定する。

   - 新規タブを開いたとき
   - ドキュメント更新が実行されたとき
   - app を開いたときに既に開かれているタブを復元するとき

   次のタイミングでは判定しない。

   - タブ移動
   - タブ切り替え
   - 同一内容の viewport 操作
   - 選択状態のみの変更

5. cache 取得失敗・破損は例外をユーザーに露出させず、必ず再描画 fallback する。

   - manifest が存在しない: cache miss として描画する。
   - payload が存在しない: cache miss として描画する。
   - payload が破損している: payload と manifest を破棄し、再描画する。
   - OS cache path が取得できない: 現セッションの in-memory cache に fallback する。
   - checksum 生成に失敗する: cache を使わず従来描画を実行する。

6. 観測性（observability）は metric / log で記録する。命名は snake_case で統一する。

   - `diagram_cache_hit`
   - `diagram_cache_miss`
   - `diagram_cache_mismatch`
   - `diagram_cache_corrupt_payload`
   - `diagram_cache_redraw_executed`
   - `diagram_cache_checksum_evaluated`
   - `diagram_cache_checksum_skipped_by_tab_move`

## Flows

### Update Flow

1. ドキュメント更新イベントを受ける。
2. 最新の図形内容を canonicalize する。
3. `currentContentChecksum` を生成する。
4. 保存済み `contentChecksum` と比較する。
5. 不一致の場合のみ描画ロジックを実行する。
6. 新しい payload と manifest を atomic write する。

### Tab Open Flow

1. タブ作成時に `documentId` を解決する。
2. manifest を読み込む。
3. current content checksum と manifest checksum を比較する。
4. 一致し、`cacheSchemaVersion` / `rendererVersion` も一致する場合は payload から復元する。
5. 不一致または cache miss の場合は描画ロジックを実行し、cache を更新する。

### Startup Restore Flow

1. app 起動時に復元対象タブを列挙する。
2. 各タブについて manifest を読み込む。
3. checksum が一致する場合は payload から復元する。
4. 不一致の場合のみ描画ロジックを実行する。

## Risks / Trade-offs

- **Risk**: content canonicalization の対象漏れにより、視覚的に変わったのに checksum が同じになる
  -> Included / Excluded を spec の Scenario で固定し、図形種別ごとの canonicalize テストを追加する。

- **Risk**: OS cache path の権限不足や容量不足で write が失敗する
  -> in-memory cache へ fallback する経路を Decision 5 で定義し、ユーザーに露出させない。

- **Risk**: `rendererVersion` のバンプ忘れにより古い payload を hydrate してしまう
  -> renderer 側の semantic-affecting change で `rendererVersion` を必ず変更するレビュー観点を tasks.md の DoD に含める。

- **Risk**: タブ移動を契機とした再描画抑止が、選択状態と連動した hover overlay 等の UI 更新まで止めてしまう
  -> 「描画 payload の cache 判定をスキップする」のみで、UI overlay の再描画は別経路として扱う。Scenario でも tab movement の影響範囲を限定する。
