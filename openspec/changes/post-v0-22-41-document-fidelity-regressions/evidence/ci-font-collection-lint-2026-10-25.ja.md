# CIの新しい書体コレクションlintへの適合

## 一次情報

最新ef672a8dの通常pushは正式hookを通過し、Draftで重大なレビュー指摘なし・全99thread未解決0を確認してReady化した。CI40162257588ではmacOS/Linux lintが同じ原因で終了101となった。

- [macOSの失敗ログ](https://github.com/HiroyukiFuruno/KatanA/actions/runs/40162257588/job/121076080878)
- [Linuxの失敗ログ](https://github.com/HiroyukiFuruno/KatanA/actions/runs/40162257588/job/121076080903)
- CIの実Rustは1.99.1、ローカルは1.97.1。CIのclippy::chunks_exact_to_as_chunksがmetadata_collection.rsの固定4byte分割を検出した。
- 元ログはtmp/oct25-ef672a8d-ci-lint-failure.jsonへ保存した。

## 最小修復

PRをDraftへ戻してから、chunks_exact(4)をCIの助言どおりas_chunks::<4>().0.iter()へ変更する。元のcount×4の長さ、read_atの厳密読込、offset解釈、異常データ拒否は維持する。新しいallow・閾値/timeout/ignore変更、CI toolchainの降格は行わない。

## 検証

既存のfont_loader::office_faces::metadata_対象は正式just test-specificで21件成功、42suites・2230filtered・実exit0。tmp/oct25-ci-font-metadata-focused.log。形式検査実0・独立レビュー新P0/P1なし（実sourceSHA主査一致）。通常commit/push、新HEADの再レビューとReady後の全必須CIを回収する。旧ef672a8dの合格やDraft skipを新HEADのCI成功へ流用しない。
