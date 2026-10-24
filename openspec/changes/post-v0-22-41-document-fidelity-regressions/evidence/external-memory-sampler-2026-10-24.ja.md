# 隔離配布検証用の外部メモリ観測

## 実装範囲

`scripts/release/process_memory_sampler.py` は対象PIDのRSSを外部OS経由で取得する。macOSではSDKの `proc_pid_rusage(RUSAGE_INFO_V4)` によりphysical footprintも別に取得し、Linuxでは未対応のphysical footprintをnullのまま保持する。RSSからの代入や欠測値の0補完はしない。

採取の前後に、実行ファイルpathとプロセス開始識別子を照合する。Linuxは `/proc/<pid>/stat` のfield22、macOSは `ri_proc_start_abstime` を使う。不一致・読取失敗では採取済み値も破棄して非成功を返す。既存配布provenanceとの接続、反復操作の集計、元性能基準への判定は別の未完作業である。

## 実検証

- macOS外部プロセス採取・境界検査: 主査 `tmp/oct24-memory-sampler-main-tests.log`、6件、実終了0。
- Linux observer内の外部採取・境界検査: 主査 `tmp/oct24-memory-sampler-main-linux-tests.log`、6件、実終了0。後続の主査テスト構造補正後も同じ実環境で再検証した。
- SDKレイアウト確認: `tmp/process-memory-rusage-layout.log`。resident offset64、physical offset72。開始識別子はSDKのoffset80と実値を確認した。
- 独立差分レビュー: `tmp/oct24-memory-sampler-independent-review.md`。初回はWindowsのテスト分岐・採取後PID変更の回帰不足を指摘し、補正後に再確認した。
- Windowsは未実行。Windows向け契約はunsupportedの実CLI応答を確認する分岐であり、Windowsの性能測定実装・実測の完了を意味しない。

この観測器の検査成功を、配布版本体の文書操作・通常診断OFF性能・全OS・独立95点・公開完了として扱わない。本人アプリ・設定・Termsへの操作は含まない。
