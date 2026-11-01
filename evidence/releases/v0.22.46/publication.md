# v0.22.46 公開・起動復旧結果

[GitHub Release](https://github.com/HiroyukiFuruno/KatanA/releases/tag/v0.22.46) / [実装PR407](https://github.com/HiroyukiFuruno/KatanA/pull/407) / [公開run41305568486](https://github.com/HiroyukiFuruno/KatanA/actions/runs/41305568486)

- 公開tagのsourceはmerge `e5360a39ecac101130b1b3ae34b91da249d42377`。イベントhead `f448bdae083483faa466136ffdcb168541ed1e0e` と区別し、実tag・配布物・3OS manifest・署名付き来歴のsource/run/attemptを照合した。
- Ready CI41298051115の全8jobとReadiness41298050975が成功し、必須7checksを確認してmergeした。source mergeの3OS push CI41305567734も成功した。12製品コミットはGitHub上ですべてVerified。
- 全5配布物とchecksums.txtを実ダウンロードし、サイズ・SHA256・GitHub API digest・正本CI receiptとの一致と、取得中に公開物が差し替わっていないことを確認した。
- 全6ファイルの署名付き来歴を暗号学的に検証した。GitHub-hosted runner、公開workflow、master ref、source/signer digest、run41305568486/attempt1、subject digestを固定して確認した。
- 配布版の起動検査はmacOS arm64/x86_64、Linux x86_64、Windows x86_64で成功した。条件付きのWindows Server2022比較診断はSKIPPEDであり、成功には含めない。
- 公開後Readiness41312259460の `published-asset-identity` を取得した。publication-identity.json、run-admission.json、3OS manifestは同CI artifactのbyteそのままである。これはasset identityの証跡であり、文書QA・画像の視覚品質の証明ではない。

## 公開した修正

高さ19,067pxの保存PNGを復元すると、GPUの上限8,192pxを超えるtexture作成で毎回終了することを旧配布版で再現した。単独画像とMarkdown内画像のupload寸法を現在の描画上限内に収め、原本・元の表示寸法・縦横比を保持する。上限を超える画像は表示textureを縮小するため、極端な拡大の精細度は原本と異なる。

通常検索の `*.png` は各階層のPNG basenameに一致する。標準正規表現、否定、大小文字、単語一致を維持し、照合と入力エラー検証を共通化した。Draftレビューで見つかったWindows区切りと、単語一致時に親directory末尾の記号へ依存する問題も修正し、各threadに直接reply/resolveした。最終製品headのCodex再レビューは追加指摘なし、未解決threadは0件。

対象回帰は画像146件、実PNG/alpha2件、Explorer29件、撮影runner58件が成功した。3OSの全体テスト、静的検査、macOS coverage、文書描画の受入検査もReady CIで成功した。依存候補を評価し、両lockの互換2件を更新した。採否と制約の実測は[公開sourceの依存評価](https://github.com/HiroyukiFuruno/KatanA/blob/v0.22.46/openspec/changes/v0-22-46-startup-png-regex/evidence/dependency-audit.md)に残す。

## インストール済みappの復旧

旧appを退避し、実公開ZIPから取り出したv0.22.46を `/Applications/KatanA Desktop.app` へ直接更新した。main/sidecarのSHA256は正本macOS manifestと一致した。元PNGのtabを残した36tabsと他profile fieldsを保ち、一時的なCONTRIBUTING選択だけを元PNGへ戻した。現在選択されている別workspaceのglobal設定は書き換えていない。

二つの検証を区別して実施した。

- **元PNGの自動復元**: 元の保存状態を複製し、複製側だけで対象workspaceとPNGを選択した。初回ChangeLog表示を抑えるseen-version変更も複製側だけで行った。実インストール済み公開binaryでmesh追加後にframe93から153へ進み、9.82秒の検査に成功した。検査前後で実global/settingsとPNG原本のSHAが不変だった。診断processは終了済み。
- **実設定の通常起動**: 診断環境変数を外し、実ユーザー設定で通常起動した。8.31秒の生存観測、診断無効、現在のworkspace選択と36tabsの保持を確認し、通常appを起動したままにした。履歴リセットやtab削除は行っていない。

## 配布経路と整理

公開jobのexternal-registry-sync receiptはHomebrew/Linuxbrewをupdated、WinGetをsubmission_completedと記録している。tapのcurrent/version-pinned各4ファイルを読取確認し、0.22.46・実公開SHA・URLの一致とRuby構文を確認した。これはbrew installの受入ではなく、WinGetのcatalog反映やMicrosoft側のmerge済みとも主張しない。

ローカルの実装branch `release/v0.22.46` は正式mergeと公開を確認して通常削除した。既存公開Actionsの自動cleanupがremoteの同branchを削除したため、元の検証済み `f448bdae` への参照を復元し、独立したAPI読取で一致を確認した。リモート削除の明示opt-in化は[後続Issue408](https://github.com/HiroyukiFuruno/KatanA/issues/408)で追跡する。追加のremote削除は行わない。

この記録branchの最終整理は正式merge後に行う。公開後記録では製品・lock・workflowを変更せず、公開sourceとの一致を維持する。本人実機確認はIssue Close条件に含めない。
