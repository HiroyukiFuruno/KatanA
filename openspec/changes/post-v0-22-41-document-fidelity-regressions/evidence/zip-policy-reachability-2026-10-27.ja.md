# 既存ZIP例外の経路判定を厳密にする

## 修正対象

0403の未接続案を検査した際、既存の3件用policyにも、KDV経由という条件の
判定不足が見つかった。対象ID0367/0368/0369、固定版、registry条件、期限
2026-11-07 00:00 UTCは変更しない。0403を有効化する修正ではない。

旧実装は旧ZIPへ向くedgeの親だけをKDV除外後の到達集合と照合していたため、
次の合成metadataを受理していた。

- 旧ZIP自体をworkspace起点へ追加し、KDVを通らず到達できるケース。
- KDVから親へのedgeを切り、旧ZIP鎖を実workspaceから到達不能にしたケース。

これは判定関数の入力境界の再現であり、実Cargoがこのmetadataを生成したことや、
Office入力で攻撃が成立したことを示すものではない。

## 最小修正

許可された旧ZIP edgeごとに、親と子の両方が実workspace起点から到達可能で
あることを要求する。KDVを除いた探索では、親か旧ZIP自身のいずれかへ到達したら
拒否する。未知の版・caller・非registry・欠損graphを拒否する既存条件は維持する。

新規advisoryの除外、期限延長、ライセンス・他の検査の緩和は行わない。
Git管理のdeny.tomlも変更しない。

## 実検証

- HEADの旧policyをそのまま読み込んだ同じ2入力は、いずれも誤ってACCEPT。
  `tmp/oct27-zip-policy-strictness-red.json`に保存した。
- 2回帰を正式な`test_zip_advisory_overlay.py`へ追加し、既存を含む18件が成功。
  元cargo-denyの失敗を保持する回帰と、除外対象が3件だけである回帰も成功。
- 本体・撮影ツール・描画計測の実locked/all-feature metadataを取得し、修正後の
  経路判定が3件とも成功することを確認。
  `tmp/zstd0403-proposal-metadata/receipt.json`と`strictness-validation.json`に保持した。
- 未接続0403案は別のdraft検査であり、通常SCAの合格とは扱わない。

通常pushのRUSTSEC-2026-0403不合格は残る。通常review/CI・公開の結果をこの
単体検査やgraph照合から代用しない。
