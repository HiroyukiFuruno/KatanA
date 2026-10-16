# 資源解放計測・読込容量終端・Windows診断の自己レビュー

対象は `release/v0.22.42` の `9b0226ba` 以降の作業差分。
リリース・全体メモリ回帰の完了報告ではない。

## 要件と設計

- 文書タブ除去だけでは detached worker の完了を保証しないため、
  文書処理の論理寿命を RAII で計測する。`(work, guard)` の単一 capture と
  tuple の Drop 順序により、起動失敗時も所有入力の解放後に counter を下げる。
- 通常終了・panic・起動失敗・未起動 closure 破棄を検証する。
  OS TLS、スレッド自体、GPU、allocator の解放とは区別する。
- 公開 KDV 0.5.8 の8資源 counter を hidden hook へ投影し、単独アプリの
  close polling と idle assertion に追加する。並列 UT で全局 zero を要求しない。
- 固定2 intake worker が取消済み OS 読込で占有された場合、非active要求を
  typed capacity failure へ終端する。有効な両2読込の通常 queue は維持する。
  枠の回復後は再要求でき、pool を恒久失敗にしない。結果 channel を再確認し、
  既に到着した結果を容量失敗で上書きしない。
- Windows resolver の読込と metadata 解析を既存 `DebugLog::write` で区別する。
  DEBUG が有効でない場合は出力しない。候補順・顔選択・取消・サイズ上限・
  10秒期限は変更しない。

## 現時点の検証

- 論理寿命の軽量実スレッド/RAIIテスト6件成功。
- 資源判定の新2テストは旧判定で RED、追加判定で GREEN。
- lifecycle runner の全41テスト成功。
- AST 23件、format、diff-check、全workspace/all-target strict Clippy成功。
- 同じ source を凍結した full coverage は exit0。UI986、core215、platform115、
  実export13、UI parallel143、serial18が成功し、strict document surface100%/
  uncovered0を維持した。既存ignored件数・閾値・除外は変更していない。

## 実入力と残件

新計測でも提供大型PPTXの11open後、全worker/KDV資源0だが
RSS増分646448 KiBで既存65536 KiBを超過した。待機だけでは解消しない。
代表HTML/PPTXの1+10cycleは容量回復後の再実行で成功した。
先行の容量逼迫中のOffice response missing失敗は削除せず記録する。

別の実ファイル読込だけの診断で、rootの空Vec + bounded readerによる
幾何拡張がRSS保持を増やす経路を再現した。opened file長の有界容量予約は
対照実験ではplateauする。続く製品の有界容量予約後、同PPTX反復のwarm増分は
5536 KiBに改善したが、HTML混在cold増分254672 KiBとOffice5件の増分315504 KiBは
既存196608 KiB基準を超過した。全体解消ではない。

reader変更後の全テストは成功したが、厳密coverageは253/256、再実行256/257で
失敗した。実File型の上限overflow・容量予約失敗の伝搬テストを追加し、関連12件と
増分reportのsource_io310/310を確認した。製品コード・閾値・除外の変更はなく、
最終sourceの通常fullcoverageはexit0となった。UI995、core215、platform115、
実export13、parallel143、serial18が成功し、strict document surface100%/uncovered0を
維持した。全target strict Clippy・formatもexit0。増分reportではなくこの全体gateを
検証根拠とする。既存ignored・除外・閾値は変更していない。

既存evidence7文書の表区切り・見出し・コード表記だけを修正し、差分を照合した。
数値・判定・基準の変更はなく、OpenSpec change全体のKML検査はexit0。

PR #346 は Draft のまま。manual target互換性の判断、上流の実HTML/評価、
Windows実CI、配布物/clean-machine smoke、全ゲート、公開・後処理は未完了。
新worktree、master編集、stash、検査・閾値緩和、兄弟repo編集は行っていない。
