# 削除済み画像フォルダの通知範囲

## 発見と範囲

current68dbbe5bのUbuntu CI37516676587/job112452170844は `subscriber_overflow_retries_watched_and_pending_paths:85` の `watch_error.is_none` で失敗した（1132PASS/1FAIL/既存ignore2）。元ログにerror値はなく、この修復が同じ原因だったとは断定しない。rawは `tmp/68dbbe5b-live-snapshot/refresh/ubuntu-job-112452170844-logs.txt`。

登録時にはcanonical parentを使う一方、削除後の通知pathはparentをcanonicalizeできない。従来のevent処理は、このpath解決エラーを全targetの登録失敗として送信していた。削除した別フォルダの遅延通知でも、正常な画像に監視エラーが残り得る。

## 修復と回帰

- 実TempDirに別々のPNGを保存し、一方のフォルダを削除してbackend event境界へ通知。旧製品の31409はactual101/1assertion REDで、無関係画像に `image watch parent unavailable: No such file or directory` を確認した。`tmp/deleted-parent-event-red.log`。
- path解決エラーを警告として保持し、通知pathに一致する既存targetだけをinvalidate。削除directoryのdescendantsも維持する。正式backend errorのFailed通知は従来どおり保持する。
- 追加2件は削除file/parent通知の独立画像保護と削除directory通知のdescendant更新を検証する。最終18386は全loader33PASS/40suites/1.52秒/actual0。`tmp/deleted-parent-event-loader-final.log`。
- overflow回帰のassertに既存watch_stateを加え、次の失敗でerror/deferred/pending状態を回収できるようにした。assertと元5秒期限は変更しない。
- 公式90382はAST23PASS、strict impacted Clippy/fmt-check成功、actual0。`tmp/deleted-parent-event-static.log`。rootの差分・caller確認とreadonlyレビューも重大指摘無し、diffcheck成功。
- generation、public API、cache上限、実登録失敗の保持、本人アプリ操作・Terms承認、上流固定依存は変更していない。

## 残作業

ソースは通常hook/署名Gの624cbefb769a154cc21883afc40b9899addc4892へ正式統合した（68105 actual0）。日時は2026-10-06 19:35:08UTC / 2026-10-07 04:35:08JST。同期変更履歴と証跡は別の通常commitに分ける。

静的検査、関心事別の通常署名commit/push、新HEAD current review/全CI/coverage、配布候補の再作成・全5assets実受入、公開後処理を継続する。68dbbe5bのcoverage/package成功や単独回帰成功を、新graphのCI成功・実機受入へ代用しない。
