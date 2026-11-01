# 修正の検証記録

- 対象: Issue #406、release/v0.22.46。
- 配布版v0.22.45で実PNG復元がexit101: Dimension Y value 19067 exceeds the limit of 8192。
- 修正前のunit回帰はload_textureの上限で失敗した（crash-red.log）。
- 修正後の画像対象回帰146件、実PNGとalpha回帰2件、Explorer回帰20件成功。
- 修正を含むnativeソースビルドで保存profileの複製を使用。実PNGのmesh追加後もheartbeatがframe240まで進行、15.52秒の観測でプロセス継続。元設定・画像・タブ一覧は変更しない。
- 実PNGのuploadは549x8192、表示寸法は1280x19067を保持。
- フィルターはユーザー具体例 *.png によりwildcard期待とliteral照合の不一致を特定。通常モードのwildcard照合を追加し、root/nested/case/否定/regex切替の回帰が成功。標準regex構文は維持する。
- 独立画像レビューでP0/P1/P2指摘なし。
- 完全な実行ログは作業場所のtmp/v0-22-46/に保存。公開前・公開後の最終source/CI/assetsはtasks.mdへ追記する。

- 既存appの一時リカバリー: 保存状態をバックアップし、active tabだけCONTRIBUTING.mdへ切替。36tabsと履歴はそのまま、既存v0.22.45でfirst-frame/frame143まで進行。修正版の公開後に元PNGへ戻す。

- 画像capture driverは現在frameのlocal_image textureのmesh/textured rectを条件待機する。runner58件成功、両画像がreadyとなり撮影完了。検査環境上限2048では137x2048へ縮小されるため精細度の受入証拠にはしない。native8192の実測と区別する。
- wildcard整理後はExplorer24件成功、source全体buildもv0.22.46で成功。
