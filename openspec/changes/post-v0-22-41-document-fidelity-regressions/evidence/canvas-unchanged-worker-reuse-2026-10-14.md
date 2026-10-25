# 未変更Canvasの実workerモデル再利用

## 対象と実再現

- 既存Issue #258、PR #346 current5af review5513924147/comment4267507147。
- default auto-refreshの同path/nonforceでSHA一致後もJSON再解析する経路をmain/readonlyレビューで確認。
- UI適用後のモデル比較では、旧surfaceが同一revisionを保持するため誤GREENになる。初worker候補のそのfixtureは不採用。
- public `RefreshDocument(false)`後にUI適用前の実request.poll結果Arcを比較する回帰99412は、旧製品sourceでcompile成功後1assert/exit101、0.01秒。rawは`tmp/canvas-unchanged-worker-model-red-2026-10-14.log`。

## 候補の範囲

- 同path/nonforce/Ready/revisionありだけ、既存SHAとモデルArcのsnapshotをrequestへ渡す。
- workerは実ファイルを再読込し、regular/サイズ/UTF-8/取消とchunk SHAを維持。一致時だけArcを再利用し、JSON解析とモデル再構築を省略する。
- manual/path変更/Failed/Loading/本文変更では通常解析する。予約refreshは適用後の最新surfaceを参照。単一worker/queue・generation・取消を維持。
- 全文読込/SHAやworker占有をなくしたとの主張ではない。

## 実検証と留保

- 修復候補17239は実worker Arc回帰exact1成功/exit0、0.01秒。58515はCanvas関連69件成功/exit0、0.52秒。
- 通常lib strictClippy22358は、旧take_source未使用と冗長closureの計3件でexit101。allow/cfg逃げをせず不要method削除、既存source消費回帰をtake_input/loadへ移行した。Snapshot.loadも通常Fileと同じ実読込契約へ揃えた。
- 補正後1137はCanvas関連69件成功/exit0、0.53秒。正式AST14569は23件成功/exit0、22.57秒。テストを含むUI全target/external-fixture-acceptance strict Clippy7721とrustfmt/diff checkも成功。
- 最終readonlyレビューは同path/非force/Ready限定・coalescing・取消・既存source消費回帰・共通実入力検証と関数30行/ファイル200行を確認しP0/P1残なし。新allow・期限変更・testport・解析counterは追加しない。
- 通常統合/push・current review返信解決は未完。
- 全release gate・packaged・全OS・95点・公開の証明ではない。本人アプリ/設定/Termsは非操作。
