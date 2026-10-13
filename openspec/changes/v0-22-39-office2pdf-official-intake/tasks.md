## 1. Provenance and boundaries

- [x] 1.1 official `office2pdf 0.6.7` が #745 の merge commit を含む crates.io release であることを確認する
- [x] 1.2 published KDV `0.5.3` が official `office2pdf = "=0.6.7"` を使用し、KDV release / 3 OS CI が成功していることを確認する
- [x] 1.3 KatanA -> KDV -> KUC/KRR の dependency direction と KatanA direct KUC prohibition を design に固定する

## 2. Consumer intake

- [x] 2.1 workspace version、internal crate versions、bundle metadata、CHANGELOG EN/JA を v0.22.39 へ同期する
- [x] 2.2 KatanA を exact published KDV `0.5.3` へ更新し、Cargo.lock で official `office2pdf 0.6.7` と retired package absence を固定する
- [x] 2.3 KatanA direct Office/PDF engine dependency、parser、layout、font resolver、KUC direct dependencyを追加しないことを contract で検証する

## 3. Verification

- [x] 3.1 format、clippy、AST lint、workspace tests、strict coverage 100% / uncovered 0 を実行する
- [x] 3.2 PDF / DOCX / XLSX / PPTX corpus を macOS / Linux / Windows headless acceptance で実行し、PPTX paragraph / CJK evidence を確認する
- [x] 3.3 OpenSpec strict validation、release preflight、package / sidecar asset contract を実行する

## 4. Release

- [x] 4.1 verified evidence を記録し、既存の明示承認に従って commit、push、PR、merge、v0.22.39 release、public artifact verification、automation removal を順に実行する
