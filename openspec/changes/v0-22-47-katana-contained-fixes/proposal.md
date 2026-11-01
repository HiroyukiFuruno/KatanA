## Why

グループ名入力、Enter確定、キャンセルが配布版で退行し、Open IssueにもKatanA内で実行できる未対応が残っている。ユーザーの指示に従い、上流の修正・公開を必要としない対応を一つのv0.22.47へまとめ、実操作と既存品質条件を満たして公開する。

## What Changes

- グループ作成・名前編集を本番アプリ全体の入力経路で修復し、名前あり/空欄のEnter確定、作成後のサブウィンドウclose、Cancel/Escを回帰検証する（#412）。
- 追加先menuに選択色を復旧する。名前は任意のまま、作成・後からの名前/色変更で「名前＋色」の重複だけを拒否する（#412）。
- rootの日英READMEへ公開fixtureを表示した実アプリ画像を掲載する。
- PR405のGitHub Actions更新と契約検査を正式統合する（#411）。CI-only版番号例外の未承認な追加は行わず、製品修正と同じ次patch・全CIを通す。
- 更新helper起動失敗時にエラーを表示してアプリを維持し、不要な一時ファイルを残さない（#409）。
- 公開後のremote削除を明示opt-inだけに限定し、manual bumpのsource結合を検証可能にする（#408、#403）。
- CIの同等性・cache計測と安全な証跡専用lane、KatanA所有の実行環境の整備を完了する（#343、#372）。
- 文書closeのhost側残件、現行公開KDV/KRRの受入、正本diagram captureを実証する（#386、#339、#338）。既存PNG refresh回帰を維持し、未再現の遅延を解決済みとは主張しない（#385）。
- 依存候補・ZIP例外の期限と実API利用を再評価し、既存ゲートを維持した最新互換版を採用する（#354）。上流API/公開が必要な#336・KDV #69/#70の残件とは分ける。
- 公開物を実取得して検証し、担当範囲でIssueを整理して対応ボードを同期する（#388）。

## Capabilities

### New Capabilities

- `tab-group-dialog-interaction`: 本番アプリのグループ入力・確定・キャンセルの操作契約。
- `canonical-diagram-reference`: exact source/version/inputへ結合した正本画像の採用・再現手順。

### Modified Capabilities

- `auto-update`: helper起動失敗時のアプリ継続とエラー表示。
- `generated-release-identity`: generated bump commitの厳密なsource/run/attempt結合。
- `release-ci-integrity`: remote削除の明示指定、入力同等性に基づくCI再利用・測定。
- `multi-format-document-preview`: host側closeの分離と現行公開依存による受入。
- `oss-repository-security`: KatanA所有環境の整備・依存と期限例外の再評価。

## Impact

KatanAのgroup UI/入力routing、更新action、release/CI scripts、Docker/Python実行環境、受入・画像captureハーネスを対象とする。別repoのsourceは編集せず、未公開path/git依存を採用しない。本人確認を公開・Issue Closeの停止条件にしない。PR405の既存headへ同じ対応を統合して通常レビュー・必須CIを通し、merge後のmasterから既存版再利用の明示dispatchでv0.22.47を一度だけ公開する。タグや成果物の強制上書きはしない。
