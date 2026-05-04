# Tasks: v0.24.0 katana-canvas-forge 拡張 intake — KatanA

> PDF / 図表の生成・export 拡充はすべて `katana-canvas-forge`（kcf）repo 側で行う。
> kcf 側の実装タスクは [katana-canvas-forge openspec](https://github.com/HiroyukiFuruno/katana-canvas-forge) を参照。
> 本 tasks.md は KatanA 側の intake（kcf バージョン更新 + 新 API 呼び出し）のみを扱う。

## Branch Rule

`master` で直接作業する（バージョンブランチ不要）。

---

## 準備完了条件（Definition of Ready）

- [ ] kcf の対象バージョン release tag が切られていること
- [ ] LLM 連携によるドキュメント生成・export API が確定していること

---

## 1. kcf バージョンを更新する

- [ ] 1.1 root `Cargo.toml` の kcf tag を新バージョンに更新する
- [ ] 1.2 `cargo build` が通ることを確認する

---

## 2. 新機能を KatanA UI に接続する

- [ ] 2.1 document generation の導線（現在開いているドキュメントへの追記・新規ファイル生成）を KatanA UI に追加する
- [ ] 2.2 生成前確認 UI（対象・保存先・反映内容）を追加する

---

## 3. 検証と commit

- [ ] 3.1 `just check` がエラーなし（exit code 0）で通過すること
- [ ] 3.2 commit & push（`master` 直接）
