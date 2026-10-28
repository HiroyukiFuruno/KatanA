# v0.22.42 公開後検証と整理の責務分離

## 公開済み成果物

[GitHub Release](https://github.com/HiroyukiFuruno/KatanA/releases/tag/v0.22.42) は2026-10-28 11:27:18 UTCに公開された。配布元は `8c89a1544e9ac693d5ba388792a843baa55c9363`、正式runは `40672472894` / attempt 1。

全3OSの配布build、macOS両CPUの実起動、元のWindows200ms間隔×10回の各回描画増加、Publish jobは成功した。公開後のcleanup jobだけが、install-actionに非対応の `cargo-sweep@0.8.0` をfallbackなしで指定したため失敗した。したがってrun全体のconclusionはfailureであり、これをsuccessへ読み替えない。

元の検証コードで全5公開assetを実取得し、checksum・size・URL・各本体/Office worker・producer manifestを照合した。根拠は `evidence/v0.22.42-public-download-40672472894.json` と隣接する3 build manifest。検証実行56766はexit0、証拠は通常署名commit `d580b88b60592ebaf843fdfc9479a7187a78ed96`へ保存した。

## 維持する品質条件

- preflight、Linux/macOS/Windows build、macOS x86_64/arm64 native smoke、Publishの7 jobは、同じsource/run/attemptで全件completed/successを必須とする。欠落・重複・未知job・未完・別source/attemptを拒否する。
- Server2022比較や公開後cleanupの結果は別記録とし、失敗や未実施を成功と書かない。公開済みファイルの受入と、整理の完了は別の条件として両方を追跡する。
- 元Windows200ms×10、GPU queue上限1、全OS、署名・保護、メモリ/フォント/workerの基準、全5asset/実payload照合は変更しない。初回frameが200ms以内という未計測の主張もしない。
- 本人だけの手動確認をIssue Closeや公開の停止条件にせず、未実施確認を成功と記録しない。

## 配布元と検証コードの区別

公開後の検証コード修正を、既に配布した本体のソースと混同しない。実Release/tagが指すcommitのGit objectから、元producerと同じ順序・byte framingで全SOURCE_ROOTSのSHA256、Cargo.lock、公開registry依存を再計算する。元ソースを変更したり、新worktree/cloneへ展開したりせず、公開producerに完全一致することを検査する。

元producerが読んだtracked license symlinkは、同じcommit内のtracked regular blobへだけ解決する。絶対path・repository外へのescape・欠落・cycle・過深度・不正UTF-8・submoduleを拒否する。filesystem上のリンクや外部ファイルは読まない。

現在の検証HEADと配布タグの差分も検査する。許すのは有限に列挙した非配布の公開後検証コードとそのテストだけ。製品・lock・vendor・preflight等の差分は拒否する。さらに以下をbyte単位で制限する。

- build-and-release workflow: 証拠だけの更新が再公開を起こさない `evidence/**` ignore一行と、cleanup toolの固定版を正規cargo installで導入する変更だけ。
- test-and-build workflowと対応するCI resource契約: 同じcargo-sweep0.8.0固定の一回の文字列置換だけ。他のテストや閾値は不変。
- Release Readiness: 公開済みasset検証stepより前の全required checksを完全維持する。

公開前の既定provenance検証は従来どおりstrictである。明示した公開後経路だけが実タグのGit objectを使い、公開前run_id/attemptと併用できない。dirty/untracked sourceを拒否する。検証ログには現在の検証HEADと配布元commitを別々に表示し、run admissionの全metadata/jobs/optional結果をartifactへ保存する。

## 修正範囲と検証

- cleanup installerの非対応指定を除き、正規 `cargo install cargo-sweep --version 0.8.0 --locked` を使用する。同じ固定版を通常3OS CIにも適用し、対応する契約の期待値だけを更新する。固定版導入の新CI実行結果は別途回収する。
- 実Git objectの正一致/拒否境界10件、job admission境界13件、controller差分/既定strict/公開前併用拒否9件が成功。
- 既存release flow20件、公開payload9件、既定provenance6件、host acceptance34件、公式CI resource契約、Actionlintが成功。Rust製品コード、依存lock、public APIは変更していない。
- 新job admission CLIを実GitHub APIで実行し、`40672472894 / 1 / 8c89a1544e9ac693d5ba388792a843baa55c9363` を取得した。optional cleanup failureはそのまま記録される。これは全workflow成功の主張ではない。
- 独立差分レビューのP0/P1は0件。低優先度のrun admission保存は対応し、Git fixture objectの生成と既存exclude-standard範囲は根拠付きで記録した。

## CIツール依存の実環境回帰

公開後admission CLIの初期案は、開発端末のrtkを内部呼出しに使っていた。GitHub CLIだけをPATHへ置いた隔離したプロセス環境で、実API照合がrtk不存在で失敗するREDを再現した。内部呼出しをCI標準のghへ合わせ、同じ環境から実GitHub APIで同じ7必須job/run40672472894/attempt1/sourceMを照合してGREENを確認した（実行17863 exit0）。tool shellのrtk利用方針と、配布用・CIコードの実行依存を混同しない。

admission境界13件も再成功。テスト都合のAPIやfallbackは追加せず、既存のread-only gh呼出しだけを使う。根拠は `tmp/oct28-postpublication-gh-only-red.log`、`tmp/oct28-postpublication-gh-only-green.log`、`tmp/oct28-postpublication-job-transport-tests.log`。

## 追跡先

公開後のGit整理・検証コード修復は [Issue #379](https://github.com/HiroyukiFuruno/KatanA/issues/379) で追跡する。source commitの初期参照#371は配布アプリの終了待ちIssueであり、修正の追跡先を本節と#379へ明示して履歴を訂正する。署名済み履歴は書き換えない。

## 続行する作業

通常署名source commit b72065b1/47942653/2f31a2eeを完了した。commit後の厳密controller scopeは成功し、修正版のfresh実取得78200 exit0でも元の全5公開asset ID/size/SHA256・タグM・全producer/本体workerが一致した。17件の製品Issue更新と限定3機能のspec sync/公式archiveは実行・確認済み。通常push hook・current review・Ready後の必須CI、Issue379更新、単一後処理PRの通常merge、安全なbranch/worktree整理は、root作業台帳で未完として継続する。公開済みtag/assetの再作成やforce更新は行わない。
