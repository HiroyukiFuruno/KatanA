# Tasks: v0.22.27 KDV / KRR dependency update

> `katana-diagram-renderer` は KRR の互換 wrapper として扱い、KatanA の現行依存から外す。  
> `katana-document-viewer` v0.1.1 は crates.io 公開後に取り込む。

## Branch Rule

`release/v0.22.27` ブランチを切って作業する。

---

## 準備完了条件（Definition of Ready）

- [x] `katana-render-runtime` v0.3.3 が crates.io に公開されていること
- [x] `katana-document-viewer` v0.1.1 が crates.io に公開されていること

2026-05-27 現在、`cargo info katana-document-viewer@0.1.1` で crates.io 公開済みを確認済み。

---

## 1. KRR を直接依存にする

- [x] 1.1 root `Cargo.toml` の workspace dependency を `katana-render-runtime = "0.3.3"` に切り替える
- [x] 1.2 `crates/katana-core` が `katana-render-runtime` を直接参照する
- [x] 1.3 `Cargo.lock` から `katana-diagram-renderer` が消えることを確認する

---

## 2. 図形 backend の識別情報を KRR に揃える

- [x] 2.1 backend id を `kdv-krr-*` に更新する
- [x] 2.2 backend version に `katana-render-runtime` の crate version と runtime checksum を含める
- [x] 2.3 runtime asset 解決を `katana_render_runtime::RuntimePathResolver` 経由にする
- [x] 2.4 KDR 名義の回帰テスト期待値を KRR 名義に更新する

---

## 3. 文書と仕様を KDV + KRR に同期する

- [x] 3.1 README の図形描画説明から KDR 名義を外す
- [x] 3.2 現行 OpenSpec specs / project の依存関係説明を `katana-render-runtime` に更新する
- [x] 3.3 `katana-diagram-renderer` は互換 wrapper または履歴文脈に限定して扱う

---

## 4. KDV v0.1.1 を取り込む

- [x] 4.1 `katana-document-viewer` v0.1.1 の crates.io publish を確認する
- [x] 4.2 root `Cargo.toml` の `katana-document-viewer` を `0.1.1` に更新する
- [x] 4.3 `cargo tree` で KDV / KRR の V8 dependency が分裂していないことを確認する

---

## 5. リリース準備と検証

- [x] 5.1 `just VERSION=0.22.27 release` でバージョン情報を同期する
- [x] 5.2 `CHANGELOG.md` と `CHANGELOG.ja.md` に v0.22.27 を追記する
- [x] 5.3 `cargo check -p katana-core --tests` が通ること
- [x] 5.4 `./scripts/openspec validate --all --strict` が通ること
- [x] 5.5 `./scripts/release/check-pr-ready.sh 0.22.27` が通ること
