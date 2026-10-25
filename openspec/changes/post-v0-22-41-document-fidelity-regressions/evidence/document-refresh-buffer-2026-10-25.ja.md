# 自動更新確認で文書全体を再確保しない修復

## 実ログと原因

`079b392c` の実配布アプリで、86,321,258-byteのXLSXを10回表示した際、同じPID・入力URI・サイズの原本全量読込が42回、worker sessionのopen/closeが各10回記録された。実行前後の原本hashは一致したが、旧イベントには各読込のhashと時刻がなく、個々の過去読込のhash・正確な時刻を遡及して証明したとは扱わない。

自動更新の設定は有効・2秒間隔だった。`tick_auto_refresh` から非forceの文書更新へ進むたび、既存のintakeは原本をowned Vecへ読み込み、完全hashが同じなら表示更新をせず破棄していた。描画中の追加確認も待機中の更新として残るため、表示に不要な大きな再確保が発生していた。

この実行では、資源カウンター0と正常終了を確認してもsteadyのRSS/physical予算は満たさなかった。[配布版の元基準結果](macos-seven-pair-baseline-2026-10-25.ja.md)を保持し、不要な再確保を上流待ちへ分類しない。

## 修復の範囲

非forceで既存の文書identityがある場合だけ、既存intake worker内で64KiB固定バッファによる完全SHA確認を行う。同じURI・revision・generationならpayloadを作らないUnchanged結果を返す。初回、変更がある場合、手動force reloadは従来の全量readerへ進み、その実payloadとhashを正本とする。

mtime・サイズだけの近似cacheは使わない。途中で内容が変わった場合に、二回のhash一致やファイルの静止を新たな条件として要求しない。変更を読み込んだ実payloadの検証、取消、入力上限、古いgenerationの結果拒否、待機中の手動reload昇格を維持する。UIスレッドへファイルIOを移さず、公開API、上流依存、allocator、合格予算を変更しない。

独立レビューで見つかったFIFOの二重openを修復し、単一writerで更新を完了する実FIFO回帰を加えた。regularからnonregularへの途中差替えも、黙って再openして待ち続けない。旧 `read_to_end` と同様にInterruptedだけを再試行し、取消を表すOtherエラーは再試行しない。

## 検証と正式履歴

通常署名commitは `08ec21a1f11890f8043da47a9cf311d15648eef1`。実ファイルを使う13回帰を追加し、正式文書surface対象161件、正式AST30件、既存の対象Clippy、fmt、差分検査が成功した。主査は独立レビュー対象14ファイルの実SHAをすべて照合し、未対応P0/P1がないことを確認した。

元target入口のworker推測先が不在だった途中検査は155成功/2失敗で保持し、成功へ読み替えていない。最終検査では、同workspaceで実際に生成されたworkerを既存の `KATANA_KDV_OFFICE_WORKER` で明示し、同じ正式対象を161件成功・終了0で回収した。コンパイルエラーで挙動に到達しなかった最初の検査や、中断した旧RED試行も、挙動上のRED成功とは扱わない。

主査の照合は `tmp/oct25-stream-main-final-review.json`、独立レビューは `tmp/oct25-source-intake-stream-independent-review.md`、最終対象/AST/fmt/Clippyは `tmp/oct25-source-buffer-retention-stream-{targeted-final,ast,fmt-final,clippy}.log`。14ファイルのmanifest digestは `78087b496cd8390abb023c29b831bbdb5e8d68f8bb03826e2fc13ae235058ebd`。

新配布版の元cold/steady予算、実reload操作、全OSのCI・配布検査、公開は未完である。不要な全量再確保を除いたことだけを、メモリ予算の達成やリリース完了の証明にしない。
