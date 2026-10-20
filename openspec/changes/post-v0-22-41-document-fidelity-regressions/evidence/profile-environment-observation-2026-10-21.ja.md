# Cargo profile設定の実入力記録

## 発見と修復範囲

- `8d3b97b2`のnormal実測をdirect bashで開始した際、既存Justfileの`CARGO_PROFILE_DEV_DEBUG=0`、`CARGO_PROFILE_TEST_DEBUG=0`、`CARGO_PROFILE_DEV_STRIP=none`、`CARGO_PROFILE_TEST_STRIP=none`を継承していなかった。workerの`unoptimized + debuginfo`と実compile JSONの`debuginfo:2`を主査が確認し、テスト開始前のcompile段階で自己session74998をSIGINTした。actual exitは130、normal.json未生成、coverage未開始。旧ログ`tmp/oct21-normal-8d3-matched.log`を保持し、正式なnormal/coverage比較や成功へ採用しない。
- 既存実行identityの許可済み環境記録に、上記の正確な4変数と`CARGO_INCREMENTAL`を追加した。実child環境から値を取り、未設定は既存同様Noneとする。Cargo argv・テスト選択・coverage閾値・false reuse policyは変更しない。
- 最初のテストは3変数の省略名を誤って期待していたため、そのGREENを採用せず、主査指摘後に実Justfileと同じ完全名へ訂正した。

## 対象検証

- 正確な環境名を期待する回帰RED: actual1、`tmp/oct21-native-identity-profile-exact-red.log`。
- 修復後identity suite: 6件/1.013秒/actual0、`tmp/oct21-native-identity-profile-exact-green.log`。
- 未設定5変数のNone保持回帰を追加し、identity/observation/record/compare/shadow関連consumerの72件を一回実行、7.355秒/actual0。`tmp/oct21-native-identity-consumers.log`。既存の意図的receipt失敗warningもrawへ保持する。
- strict入力集合の検査により、新5キーのない旧manifestは比較unavailableとなる。旧manifestを移行・再利用許可へ昇格させず、新実行の診断記録を使う。独立レビューでP0/P1なし、guard変更は不採用と確認した。
- 実normalとcoverageの一致、製品全ゲート、Windows実測、速度改善、公開の証明ではない。新しい正規実測は修復統合後の同一HEADで、既存Justfileの4profile変数と共通条件を明示して行う。

## 診断の訂正

- `target/html-startup-failures`のsample通常ファイル2件をguardが拒否するとの初期推測は誤り。旧TRIPLE検査は当該名を認識し、既存profile discoveryは`target/debug`を返す。未知子ファイル拒否の追加案は今回の容量回収修復として採用しない。
- 診断原本2件は`tmp/oct21-preserved-html-startup-failures/`へコピーし、原本とSHA一致を確認。SHAは`74cb4f924bd64cf595aa3df5b4425a6e1b082a0c94b793ee542cd229245b3b36`と`b19deba255234f591103275622b01e63af090e0d045c8ef946ebf5e4efd2a23e`。コピーを製品の受入証明には使わない。
