## Context

KatanA は Mermaid / Draw.io / PlantUML 等の図形（diagram）プレビューを `katana-diagram-renderer (kdr)` 経由で SVG へ変換し、プレビュー画面に表示する。

ユーザー検証では `just run-release` を繰り返した際、タブを切り替えるたびに図形が再描画されているように見えた。これは、生成済み SVG を「図形コードブロック単位」で再利用できていないことが主因である。

KatanA v0.22.21 では、Markdown ファイルの絶対パスで cache 領域を分離し、その中で AST から抽出した図形コードブロック本文の checksum ごとに SVG ファイルを保存する。

### Assumptions（前提）

- 既存 OpenSpec に `diagram-render-cache` capability は存在しないため `ADDED Requirements` として記述する。
- 図形 cache の永続 payload は SVG ファイルそのものとする。
- `manifest.json` / `cache.json` は今回導入しない。
- PNG / JPEG 等の通常画像は永続 cache 対象外とし、表示時のメモリ上（in-memory）扱いに留める。
- `katana-diagram-renderer (kdr)` の公開 API（neutral interface）は変更しない。
- kmm の AST 解析を使う場合でも、依存はプレビュー側の局所 adapter に閉じる。将来の `katana-document-viewer` / `katana-ui` 分離を邪魔しない。

## Goals / Non-Goals

**Goals:**

- cache 単位を AST で抽出した図形コードブロックに合わせる。
- Markdown ファイルの絶対パスごとに cache 領域を分離する。
- 図形コードブロック本文と図形種別から content checksum を生成する。
- cache hit 時は kdr を呼ばず、保存済み SVG を使用する。
- ドキュメント更新時に、現在の AST に存在しない checksum の SVG を削除する。
- タブ切り替えやタブ移動だけでは checksum 判定も再描画も実行しない。

**Non-Goals:**

- 図形そのものの描画アルゴリズム変更。
- kdr の公開 API 変更。
- ドキュメント永続化フォーマットの変更。
- PNG / JPEG の永続 cache 化。
- ネットワーク共有 cache の導入。
- preview / editor / `katana-ui` 分離作業そのもの。

## Architecture

図形描画 cache を、次の責務に分離する。

1. `DiagramAstBlockExtractor`: Markdown AST から図形系コードブロックを列挙する。
2. `DiagramContentChecksum`: 図形種別とコードブロック本文から checksum を生成する。
3. `DiagramDocumentCachePath`: Markdown 絶対パスから文書別 cache ディレクトリを解決する。
4. `DiagramSvgCacheStore`: SVG ファイルを読み書きし、現在の AST に存在しない SVG を削除する。
5. `DiagramRenderCacheCoordinator`: cache hit / miss / prune / redraw の判断をまとめる。

```text
Markdown document path + Markdown source
        |
        v
DiagramAstBlockExtractor
        |
        v
current diagram blocks
        |
        +--> DiagramContentChecksum
        |
        +--> DiagramDocumentCachePath
        |        |
        |        v
        |   ${os_cache_dir}/KatanA/.cache/diagrams/doc_<absolute_path_hash>/
        |
        v
DiagramSvgCacheStore
        |
        +--> cache hit: read <checksum>_<renderer_version>_<theme_hash>.svg
        +--> cache miss: run kdr -> write svg atomically
        +--> prune: delete SVG files whose checksum is absent from current AST
```

## Cache Layout

```text
${os_cache_dir}/KatanA/.cache/diagrams/
  doc_<absolute_path_hash>/
    mermaid/
      <content_checksum>_<renderer_version>_<theme_hash>.svg
    drawio/
      <content_checksum>_<renderer_version>_<theme_hash>.svg
    plantuml/
      <content_checksum>_<renderer_version>_<theme_hash>.svg
```

- `absolute_path_hash`: Markdown ファイルの絶対パスから生成する安定ハッシュ。同じ絶対パスなら毎回同じ値になる。
- `content_checksum`: 図形種別とコードブロック本文から生成する。
- `renderer_version`: kdr または KatanA 側 SVG 生成互換性のバージョン。
- `theme_hash`: dark / light 等、SVG 出力に影響するテーマ情報のハッシュ。

同じ Markdown ファイル内に完全に同じ図形コードブロックが複数ある場合、同じ SVG ファイルを共有する。これは表示結果が同じであり、物理ファイルを重複保存する意味がないためである。

## Decisions

1. Markdown 絶対パスは checksum ではなく cache 領域の分離に使う。

   - 別 Markdown ファイルに同じ図形本文があっても、cache は共有しない。
   - 既存 OS の検索や cache 管理を壊さないため、OS 標準 cache root 配下の `KatanA/.cache/diagrams` に閉じる。

2. AST 上の順番や位置は永続キーに使わない。

   - 図形が 7 個から 6 個になり、真ん中が削除されると後続の順番はずれる。
   - 順番を保存先に使うと、同じ図形を再利用できず、削除済み図形のゴミも判定しづらい。
   - 現在の AST に存在する checksum 集合を正とし、それ以外の SVG を prune 対象にする。

3. cache payload は SVG ファイルそのものにする。

   - `manifest.json` / `cache.json` は導入しない。
   - cache hit 判定は、期待される SVG ファイルが存在し、読み込めるかで行う。
   - renderer version や theme はファイル名に含める。

4. SVG 書き込みは atomic write にする。

   - 一時ファイルへ書き込み、成功後に正式ファイル名へ置き換える。
   - 途中書き込みや破損 SVG を読んだ場合は cache miss として扱い、再描画する。

5. prune はドキュメント更新時に実行する。

   - 現在の AST から `kind + content_checksum` の集合を作る。
   - 文書別 cache ディレクトリ内で、この集合に存在しない SVG を削除する。
   - テーマ違いは同じ checksum に属するため、現在存在する図形なら削除しない。

6. タブ操作だけでは checksum 判定しない。

   - タブ移動、タブ切り替え、選択状態、scroll / zoom 等は SVG 内容を変えない。
   - これらを理由に kdr 呼び出しや SVG cache 判定を実行しない。

## Implementation Guardrails

実装時は、既存の失敗実装や便利な既存 KV cache 経路に引っ張られないよう、次の逸脱検出を必須にする。

- 実装前に、仕様を守るための failing test を先に追加する。
- `manifest.json` / `cache.json` / 既存 KV cache を使う実装に戻してはならない。
- `block_0001` のような AST 順番ベースの永続 key を使ってはならない。
- `diagram_cache_key_ignores_document_path` のように、Markdown 絶対パスを無視する期待値を残してはならない。
- 実装完了前に `rg "DiagramCacheManifest|PersistentKey::DiagramRender|diagram_cache_key_ignores_document_path|block_[0-9]+" crates scripts Cargo.toml -S` を実行し、古い図形 cache 設計の残存を確認する。
- DoD はテスト通過だけではなく、上記 grep 確認とユーザー動作確認 OK を含める。

## Flows

### Tab Open / Startup Restore Flow

1. Markdown ファイルの絶対パスから `doc_<absolute_path_hash>` を解決する。
2. Markdown を AST 解析し、図形コードブロックを列挙する。
3. 各図形コードブロックから `content_checksum` を生成する。
4. 対応する SVG ファイルが存在し、読める場合はそれを使用する。
5. 存在しない、または破損している場合のみ kdr で SVG を生成し、cache へ保存する。

### Document Update Flow

1. 更新後の Markdown を AST 解析し、現在存在する図形 checksum 集合を作る。
2. cache hit / miss を各図形コードブロックごとに判定する。
3. miss した図形だけ kdr で SVG を生成する。
4. 文書別 cache ディレクトリ内で、現在の checksum 集合に存在しない SVG を削除する。

### Tab Switch / Tab Move Flow

1. 既に保持しているプレビュー状態を表示する。
2. Markdown AST の再解析、checksum 判定、kdr 呼び出し、SVG prune は実行しない。

## Risks / Trade-offs

- **Risk**: renderer version の更新漏れで古い SVG を使う
  -> SVG 出力互換性に影響する変更では `renderer_version` を変更することを DoD に含める。

- **Risk**: 同じ Markdown 内の同一図形コードブロックが SVG を共有する
  -> 出力が同一なので許容する。コードブロックごとに別ファイルを作る設計より、削除と再利用が安定する。

- **Risk**: kmm 依存が将来の分離を阻害する
  -> kmm を使う場合は抽出 adapter をプレビュー配下に閉じ、cache store / path resolver へ kmm 型を漏らさない。

- **Risk**: SVG rasterize が遅い場合、SVG cache だけでは体感改善が不足する
  -> 今回の対象は kdr による SVG 生成の抑止。必要なら次の change で rasterized payload の in-memory cache を扱う。
