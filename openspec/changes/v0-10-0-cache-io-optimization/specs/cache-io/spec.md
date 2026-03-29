## ADDED Requirements

### Requirement: ディレクトリ単位のKVSキャッシュストレージ

システムは、単一のJSONファイル（`cache.json`）の代わりに、専用ディレクトリ（例: `kv/`）内の複数ファイルを用いたKVS（Key-Value Store）形式で永続化キャッシュデータを保存しなければならない（SHALL）。

#### Scenario: キャッシュ値の個別ファイルへの書き込み

- **WHEN** システムが特定のキー（例: `workspace_tabs:/path/to/ws`）に対するキャッシュデータを保存する時
- **THEN** キー名称をSHA-256などでハッシュ化した安全な文字列をファイル名（例: `kv/<namespace>/<hash>.json`）とし、そのキーのデータのみが独立してディスクへ同期される
- **THEN** ファイル内容には `storage_version` と canonical key metadata が含まれ、filename hash だけに依存せず元の論理キーを検証できる
- **THEN** 他の persistent key の値を再シリアライズして同時に書き戻さない

#### Scenario: キャッシュ値のディスクからの遅延読み込み (Lazy Load)

- **WHEN** システムが特定のキーに対するキャッシュデータを要求した際、オンメモリのキャッシュマップに該当データが存在しない時
- **THEN** 該当するハッシュ名のファイルが存在すればディスクから読み込んでオンメモリに保持し、存在しなければ `None` を返す

### Requirement: Persistent cache keys are namespace-aware

システムは、persistent cache key を namespace-aware に扱い、内部ロジックが `workspace_tabs` や `diagram` などの用途境界を明示的に判別できなければならない（SHALL）。

#### Scenario: ワークスペース状態を保存する

- **WHEN** システムがワークスペースの open tabs 状態を保存する時
- **THEN** そのキーは `workspace_tabs` namespace として識別可能である
- **THEN** filename codec は namespace 情報を失わずに安全な永続化表現へ変換できる
- **THEN** restart 後は workspace path から同じ logical key と filename を再計算して復元できる

#### Scenario: 図解キャッシュを保存する

- **WHEN** システムが diagram render 結果を保存する時
- **THEN** そのキーは `diagram` namespace として識別可能である
- **THEN** workspace state と diagram cache の invalidate / clear / migration を独立して扱える
- **THEN** 必要に応じて file content の metadata から対象 document path や diagram kind を検証できる

### Requirement: 旧形式（単一ファイル）からの互換性マイグレーション

システムは新アーキテクチャの導入後、初回の起動時に従来の単一キャッシュファイルから新KVSアーキテクチャへのデータの受け渡しを行わなければならない（SHALL）。

#### Scenario: 旧 cache.json が存在する場合の起動処理

- **WHEN** `DefaultCacheService` の初期化時に、新KVSアーキテクチャでありながら旧形式の `cache.json` がディスク上に存在する時
- **THEN** `cache.json`内の全てのエントリが読み込まれ、それぞれKVSディレクトリの個別ファイルとして書き出された後、旧 `cache.json` は安全のため削除（または `.bak` 等にリネーム退避）される
- **THEN** `workspace_tabs` などのユーザー作業状態は upgrade で失われない
- **THEN** `diagram` cache は migration 不能な場合に限り再生成可能データとしてスキップできる

#### Scenario: migration 完了後の runtime 動作

- **WHEN** 旧 `cache.json` からの migration が正常に完了した後にアプリが動作する時
- **THEN** persistent cache の正規 runtime 経路は KVS ディレクトリのみを利用する
- **THEN** `cache.json` を通常の read/write backend として併用しない

### Requirement: 互換性保証の境界を明示する

システムは、永続化フォーマット変更に伴う互換性保証の範囲を明確に定義しなければならない（SHALL）。

#### Scenario: upgrade compatibility

- **WHEN** ユーザーが旧版から新 storage へアップグレードする時
- **THEN** 少なくとも `workspace_tabs` などのユーザーコンテキスト保持に必要な namespace は互換 migration の保証対象である
- **THEN** 新 storage entry には将来 migration のための `storage_version` が保存される

#### Scenario: downgrade compatibility is not guaranteed

- **WHEN** ユーザーが新 storage を生成した後に旧版アプリへ戻す時
- **THEN** 旧版アプリが新 storage を読めることは保証しない
- **THEN** その制約は proposal / design / release note のいずれかで明示される
