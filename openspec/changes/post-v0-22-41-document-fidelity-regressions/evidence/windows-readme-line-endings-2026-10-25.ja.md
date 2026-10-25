# Windows原本見出し検査の改行差

## 実際の失敗

PR346の `b0ba969b7355f8d9a1af1cce60af01d721073d22`、Actions run `40129549676` のWindows job `120978129785` を回収した。unitは1307成功・既存ignore2件で、同mtime画像更新・親directory renameの新回帰も成功した。続くparallel integrationは141成功・1失敗・既存ignore2件だった。

失敗は `original_sample_readme_header_renders_text_without_quoted_svg_leak` の原本見出し抽出で、`original sample must contain its README header section`。描画assertには到達していない。テストhelperの見出しと区切りがLF固定だった。Git原本をCRLFにすると両方の旧区切りが一致しないことを再現した。ただし当該Windows jobは原本の改行bytesを採取していないため、実checkoutのCRLF自体を観測済みとは扱わない。

## 修復と検証

helperはLFとCRLFを明示して受け入れ、切り出したHTML本文を変換しない。原本の見出し・本文、不正namespace、実描画・文字漏出のassertは維持する。原本LF/CRLF両方に同じ描画assertを実行する回帰を加えた。

既存 `just JOBS=2 T=preview_pane::html_export test-specific` は4件成功、実終了0。fmtと差分検査も成功した。これはmacOSでの対象検証であり、修復後Windows native CIは別途必要。

## 証跡

- ローカルraw: `tmp/oct25-b0ba-windows-ci-job.log` / `.json`
- 診断・原文行: `tmp/oct25-b0ba-windows-ci-diagnosis.json`
- LF/CRLFの旧区切り対照: `tmp/oct25-b0ba-windows-ci-line-ending-repro.json`
- 対象検査: `tmp/oct25-b0ba-windows-ci-html-export-tests.log`
- [Windows実job](https://github.com/HiroyukiFuruno/KatanA/actions/runs/40129549676/job/120978129785)

PRは修復のためDraftへ戻した。古い合格結果を次のHEADへ流用せず、current reviewの後にReady・必須CIを再実行する。
