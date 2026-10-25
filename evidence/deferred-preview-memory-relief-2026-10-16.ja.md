# 文書を先に閉じた場合のメモリ返却要求

## 対象と判断

PR346の指摘4291563800（thread `PRRT_kwDORm09y86tFowi`）を現実装で照合した。Office/PDF previewを閉じても他形式のタブが残ると返却要求がキャンセルされ、最後のMarkdown/HTML閉鎖時には文書削除の記録がなくなる。KatanAの終了順序に関する不備として今回修復する。

これはmacOSのReopen報告の原因確定、メモリリークの証明、上流の変換速度や永続cacheの修復ではない。

## 最小修正

- `KatanaApp`の内部一時boolをfalseで初期化する。設定JSON・公開APIへ追加しない。
- 閉じた文書previewの記録をboolへ保持し、他形式やpinnedタブが残る間も失わない。
- `open_documents`と`tab_previews`が両方空になった時に、既存の返却要求を一度だけ送りboolを消費する。
- HTML-only、繰り返しempty cleanup、残存タブ中の返却抑制を維持する。新allocator・FFI・testport・allow・予算変更はない。既存worker/intake idle条件も変更しない。

## 回帰証跡

- 担当agentの正式REDはsession48404、`cargo test --locked -p katana-ui --lib app::action::closed_preview_memory_tests::close_document_then_last_markdown_retains_relief_intent_until_all_previews_close -- --exact --nocapture`、exit101。コンパイル後の「最初の文書閉鎖後に保持boolがtrue」のassertで実失敗、0成功/1失敗/1310除外を回収した。初回のstruct literal不足によるcompile error、bare名とexactの不適切な指定はRED証跡に含めない。
- agent最終session8854は同caller群5件成功/exit0。mainは実diffを照合し、DOCX→Markdownに加えPDF→HTML、DOCX→PNG、途中の繰り返しcleanupと最後の消費を追加した。
- main73003のcaller/state10件はexit0、`tmp/deferred-preview-relief-caller-2026-10-16.log`。3749の既存worker/intake/macOS live allocation7件もexit0、`tmp/deferred-preview-relief-worker-2026-10-16.log`。返却byte数の検査ではない。
- 正式AST34016は30件/exit0/13.57秒、impacted strict Clippy24114はexit0/7.53秒。`tmp/deferred-preview-relief-ast-2026-10-16.log`と`tmp/deferred-preview-relief-strict-2026-10-16.log`。fmt-check/diff-checkも成功。除外や基準変更なし。
- 独立highレビューとmainの差分・実caller照合で必須指摘はなかった。SettingsServiceの既定実装はInMemoryRepositoryであり、回帰で本人の設定ファイルを書き換えない。
- 正式統合・通常push・新HEAD current reviewは後続の実結果を記録する。開始を成功扱いしない。

## 未完了の境界

この回帰は実AppAction経路と内部状態の検査であり、返却byte数・RSS・起動障害の改善を証明しない。既存の残タブ内容、HTML-only、worker/intake、macOS live allocation回帰と通常品質ゲートを維持し、最終packaged/clean-machine/公開確認は別工程として残す。本人アプリ、設定ファイル、Termsは操作しない。
