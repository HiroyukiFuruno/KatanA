## Active 整理メモ

- [x] 2026-04-25: この無印 change は active 対象から外す
- [x] 理由: version なしのまま preview adapter の先行 contract だけが残り、`v0-28-0-preview-adapter-migration` と実施単位が重複していたため
- [x] 後続: `v0-28-0-preview-adapter-migration` を正の実施単位とする
- [x] 引き継ぎ: `05341608 feat: preview adapter契約を追加` の initial DTO / contract 実装は、v0.28.0 Task 1 の既存前提として再確認する

## 1. Adapter Contract

- [x] 1.1 `PreviewInput`、`PreviewThemeSnapshot`、`PreviewWorkspaceContext`、`PreviewRenderMetadata`、`PreviewAction` 相当の型を定義する
- [x] 1.2 adapter 外へ出してよい型と出してはいけない renderer-specific 型を文書化する
- [x] 1.3 parser / vendor / renderer internals が public adapter contract に入らない compile-time boundary を追加する

## 2. Metadata Contract Tests

- [ ] 2.1 TOC に必要な heading anchor metadata を fixture test で固定する
- [ ] 2.2 scroll sync に必要な block anchor / source range metadata を fixture test で固定する
- [ ] 2.3 block highlight、search、action hook に必要な metadata を contract test に追加する

## 3. Current Renderer Migration

- [ ] 3.1 現行 renderer を adapter implementation として包む
- [ ] 3.2 preview call site を adapter API へ移す
- [ ] 3.3 Markdown、table、math、diagram、emoji、anchor の既存 integration test を通す

## 4. Vendor Ownership

- [ ] 4.1 preview 関連の `[patch.crates-io]` と `vendor/` 利用を棚卸しする
- [ ] 4.2 preview-specific fork API の direct call を adapter implementation 内へ閉じる
- [ ] 4.3 preview 外の vendor patch は owning concern と残存理由を maintenance note に記録する
- [ ] 4.4 `make check` と `openspec validate preview-adapter-contract` を通す
