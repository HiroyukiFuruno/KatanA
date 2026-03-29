## Context

現在、KatanAの継続的なキャッシュ機構は `CacheFacade` トレイトによって抽象化され、その標準実装である `DefaultCacheService` が利用されています。この実装は内部状態を単一の `PersistentData` として保持し、`~/.cache/KatanA/cache.json` へまとめて永続化しています。

現状解析で確定している事実は以下です。

- `new()` 時にだけ `cache.json` を読み込み、以後 `get_persistent()` はメモリ上の `PersistentData` を参照する。つまり、ホットパスの問題は「毎回 JSON をパースしている」ことではない。
- 一方で `set_persistent()` はどのキーが更新されても `PersistentData` 全体を `serde_json::to_string_pretty()` で再シリアライズし、`cache.json` を丸ごと書き直している。
- `PersistentData.entries` は `Vec<(String, String)>` であり、キー探索も線形である。
- 実際の persistent key には `workspace_tabs:/path` のような小さい状態と `diagram_<hash>` のような大きい値が混在している。
- したがって、問題の主因は Rust の JSON エンジン速度ではなく、保存単位の粗さとキー設計の曖昧さによる I/O 増幅およびロジック上の扱いにくさである。

## Goals / Non-Goals

**Goals:**

- `DefaultCacheService` の runtime 永続化ロジックを per-key file store へ移行する。
- namespace-aware な key 設計を導入し、ファイル名 codec・削除・移行・テストを組みやすくする。
- `cache.json` は migration source としてのみ扱い、runtime の正規経路から外す。
- 既存の `CacheFacade` インターフェース（`get_persistent`, `set_persistent`）は維持しつつ、裏側の storage 単位を改善する。
- 変更理由を benchmark / workload 計測で検証可能にする。
- upgrade 時の互換性保証範囲を明確にし、workspace state を失わない移行契約を定義する。

**Non-Goals:**

- データベースツール（SQLiteなど）の導入によるオーバーヘッドの追加。
- `CacheFacade` トレイト自体のシグネチャ変更（非同期化やライフタイムの強制など）。
- `cache.json` と KVS の runtime hybrid 運用。
- 新 storage から旧版アプリへの downgrade 互換保証。

## Decisions

### 1. runtime の正規経路は per-key file store に統一し、hybrid 運用は採らない

`cache.json` を runtime 経路として残しつつ KVS も併用する案は採りません。旧 `cache.json` は初回 migration 用の入力としてのみ読み、以後の保存・読み出しは per-key file store に統一します。

- 採用理由:
  - hybrid にすると「どの key は monolith / どの key は KVS か」という分岐が実装全体へ漏れる
  - 現在の問題は storage 境界の曖昧さなので、runtime の正規経路は 1 つに寄せる方が保守しやすい
- 代替案:
  - `cache.json` と KVS の runtime 併用: 移行期は楽だが、負債を恒久化しやすい

### 2. key は namespace-aware に再設計し、filename codec と分離する

キーは単なる自由文字列ではなく、少なくとも `workspace_tabs`、`diagram` などの namespace を伴う論理キーとして扱います。その論理キーから filename-safe な表現を生成する codec を切り出します。ファイル名は `kv/<namespace>/<hash>.json` あるいは `kv/<namespace>_<hash>.json` のような安定した形式にします。

ただし、`hash` はあくまで address 用の filename codec であり、canonical key そのものではありません。各ファイルの内容には `storage_version` と canonical key metadata を残します。これにより、再起動後も通常の read path では「workspace path や diagram 入力から同じ logical key を再計算して同じ filename を引く」ことができ、将来の migration・検証・障害解析ではファイル内容から元の key を確認できます。

例:

- `kv/workspace_tabs/<hash>.json`
  - `{ "storage_version": 1, "key": { "namespace": "workspace_tabs", "workspace_path": "/abs/path" }, "value": ... }`
- `kv/diagram/<hash>.json`
  - `{ "storage_version": 1, "key": { "namespace": "diagram", "document_path": "/abs/path/file.md", "diagram_kind": "mermaid", "source_hash": "...", "theme": "dark" }, "value": ... }`

- 採用理由:
  - パフォーマンス差が限定的でも、namespace ごとの削除・移行・TTL・テストが組みやすくなる
  - `workspace_tabs:/path` と `diagram_<hash>` のような ad-hoc 混在を減らせる
  - hash filename だけでは失われる可観測性を、file content の metadata で補える
- 代替案:
  - 既存の生文字列 key をそのままハッシュして保存: ストレージ改善はできるが、内部ロジックの整理効果が弱い

### 3. persistent storage の最適化対象は JSON パースではなく「全量再書き込み」と「線形探索」

この change の根拠は「Rust の JSON が遅いから」ではなく、`set_persistent` のたびに全エントリを書き戻す構造と、`Vec` ベースの線形探索をやめることです。内部の値表現は JSON のままで構いませんが、保存単位は key ごとに分割します。

- 採用理由:
  - 現行の `get_persistent` はすでにメモリ参照であり、JSON パースがホットパスの中心ではない
  - `workspace_tabs` 更新のたびに `diagram` を含む全 cache を再書き込みする構造だけでも改善余地が大きい
- 代替案:
  - JSON をやめて別形式へ移行する: 本件の本質に比べて過剰

### 4. オンメモリ表現は `HashMap` 系へ変更し、必要に応じて lazy load を併用する

`PersistentData.entries: Vec<(String, String)>` はやめ、キー探索が O(1) に近い構造へ変更します。起動時一括ロードか key 単位 lazy load かは実装で決められますが、少なくともディスクの保存単位は key ごとに分離します。

- 採用理由:
  - ストレージを分けてもメモリ探索が `Vec` のままだとロジック負債が残る
  - lazy load は起動時コストの最適化として有効だが、本件の本丸は per-key write である
- 代替案:
  - `Vec` を維持したまま file だけ分割: 探索と codec の設計改善が弱い

### 5. 旧 `cache.json` からの migration は one-shot に限定する

大容量の図解キャッシュは破棄しても再レンダリングされるだけですが、ワークスペース状態（開いているタブなど）が消えるとユーザーの作業コンテキストが失われます。
シンプルかつ安全な移行処理として、「初回に `DefaultCacheService` を初期化した際、もし古い `cache.json` が存在していれば、その内容を読み込んで新しい `kv` ディレクトリ内の各ファイルへ分割保存し、完了後に `cache.json` を削除または退避する」という one-shot migration を実装します。migration 完了後に runtime が `cache.json` へフォールバックする設計は採りません。

互換性保証は以下の境界で定義します。

- 保証する:
  - 旧 `cache.json` から新 store への upgrade で、`workspace_tabs` などのユーザー作業状態が引き継がれること
  - 新 store の各 entry に `storage_version` を持たせ、将来の format migration を段階的に追加できること
- 保証しない:
  - 新 store へ移行した後、そのデータを旧版アプリが読める downgrade 互換
  - 再生成可能な `diagram` cache の完全維持。必要なら破棄して再構築してよい

また、migration の安全性として、workspace state の移送が成功する前に旧 `cache.json` を破棄しない方針を採ります。

### 6. performance claim は benchmark で裏付ける

proposal 内の改善主張は、`workspace_tabs` 更新・diagram cache 保存・起動時 restore を対象に benchmark または tracing 計測で裏付けます。性能差が限定的でも、key 設計改善と storage 境界の整理は独立した価値として扱います。

## Risks / Trade-offs

- **[Risk] キャッシュのディレクトリ内にファイル数が肥大化する**
  → Mitigation: 単一ディレクトリに数万ファイルあっても現代のファイルシステム（APFS, ext4）ではアクセス速度の劣化はほぼありません（今回はディレクトリ階層はフラットな `kv` を想定）。また、定期的なキャッシュ一括削除（すでに実装済み）から `clear_all_directories_in` にて `kv` も含めて消去可能なため、運用上ディスクを圧迫した際もリカバリは容易です。

- **[Risk] small-value key に対する per-file I/O で体感差が限定的かもしれない**
  → Mitigation: benchmark を先に追加し、性能差が限定的でも key 設計改善と削除・移行の単純化を主目的として評価する

- **[Risk] runtime hybrid を採らないことで migration 実装がやや厳密になる**
  → Mitigation: migration を one-shot に限定し、失敗時は旧 `cache.json` を温存して再試行可能にする
