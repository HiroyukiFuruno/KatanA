# Windows の読取中の描画記録を置換する

## 実際の不合格

[CI 40402573799 の Windows job121851995933](https://github.com/HiroyukiFuruno/KatanA/actions/runs/40402573799/job/121851995933) は、PR370 の head `c3e39dae2197b1960ee5412ab90300f7898ed520` に対する検査で 1349 成功・4 失敗・既存 ignore 2 件だった。

- 開いている旧記録の保持テストと、並行読取テストは、記録の置換時に `PermissionDenied` / OS error 5 となった。
- 既存の「DEBUG 無効時にも記録する」「規約画面でも最初の frame を記録する」テストも失敗した。両方とも親が保持する一時ファイルを宛先に使っている。
- 後続の headless 証跡がないエラーを、文書受入そのものの失敗とは混同しない。
- 同 head の Readiness は成功したが、CI 全体は cancelled。CodeQL・Linux/macOS の残る job も cancelled であり、全 CI 成功とは扱わない。PR370 は Draft に戻して修復する。

抜粋は `tmp/oct27-c3e39dae-windows-ci-failure-excerpt.json`、6829 bytes、SHA-256 `72d92f5f09b83818414d5f47c930f1558af8bc15fe8f73617f542bcf7192a0cc`。GitHub コネクターの実ログから端末制御文字を除いた。CLI の端末保護は無効化しておらず、保存に失敗した 0 byte ファイルは証跡に採用しない。

## 確認した実装と判断の範囲

解決済み依存 tempfile 3.27.0 の Windows `persist` は、属性設定後に `MoveFileExW(MOVEFILE_REPLACE_EXISTING)` を使う。Rust の既定のファイル共有は delete sharing を含むが、開いている宛先への置換には従来の Windows rename の制約がある。[Rust 標準ライブラリの記録 #123985](https://github.com/rust-lang/rust/issues/123985)、[修正 #131072](https://github.com/rust-lang/rust/pull/131072)、[Microsoft の仕様](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-fscc/4217551b-d2c0-42cb-9dc1-69a716cf6d0c) を参照。

現行 CI と開発環境の Rust 1.99.1 の標準 `std::fs::rename` には、アクセス拒否時に `FileRenameInfoEx` の POSIX 互換の置換を試す実装がある。プロジェクトの MSRV 1.95 も、この標準ライブラリ修正の導入以降である。

今回の `io::Error` は tempfile 内のどの OS 呼出しが返したものかを区別していないため、全 4 件の失敗を特定の OS 呼出しまで確定したとはしない。既知の制約と失敗条件が一致しており、標準 API で対応する候補を選ぶ。

## 最小修正

同一ディレクトリの一時ファイルへ全内容を書いた後、`into_temp_path()` で書込ハンドルを閉じて `std::fs::rename` へ渡す。成功時だけ一時パスの cleanup を解除し、パス用メモリは通常どおり解放する。失敗時は所有する一時ファイルの cleanup と既存のエラーログを維持する。

既存の旧記録保持・並行読取・相対パス・規約画面・DEBUG 無効時のテストは変更しない。新依存、unsafe、retry、sleep、試験用分岐、除外・ignore は追加しない。元 200ms×10 と各予算も変更しない。

対象の `startup_heartbeat` 検査 8 件、strict Clippy、rustfmt、diff check は成功した。修正の Windows CI・実配布検証はまだ未完了であり、今回の不合格が解消したとはまだ扱わない。Windows の表示同期候補の効果も別の未検証事項である。
