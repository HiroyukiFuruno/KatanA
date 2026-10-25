# 上流待ちと独立したTLS依存の修復

## 変更

rootとscreenshotのlockfileを、公開registryの互換修正版へ更新した。
rustls-webpkiは0.103.15から0.103.16、webpki-rootsは1.0.9から1.0.10。
manifest、供給網の拒否条件、検証範囲、閾値は変更していない。
日英CHANGELOGへ利用者向けの証明書検証修正を記録した。

## 回収した結果

- `tmp/oct22-tls-patched-http-regression.log`: 既存の実TCPダウンロード回帰1件成功、actual exit 0。TLS証明書の負例実測そのものではない。
- `tmp/oct22-tls-patched-supply-chain.log`: root供給網はactual exit 1。RUSTSEC-2026-0381/0382は消失し、既存の上流ZIP 0.6.6/2.4.2/8.6.0の拒否が残る。
- `tmp/oct22-tls-patched-screenshot-supply.log`: screenshotもactual exit 1、残存は同じ上流ZIP。全供給網合格ではない。
- `tmp/oct22-tls-patched-paint-supply.log`: paint-metrics供給網はactual exit 0。
- `tmp/oct22-tls-change-kml.log`: 変更文書の検査actual exit 0。
- `tmp/oct22-tls-graph-ownership.md`: 両graphのregistry取得・所有境界を確認。上流ZIPをこの担当で実装変更しない。
- 独立差分レビューはP0/P1および必須P2が0件。`git diff --check`成功。

## 未完了

旧HEAD 7dd843d6の全体テストは新たなTLS不備の発見後、所有processだけを安全に停止しactual exit 130を保持した。
ログは`tmp/oct22-current-native-gates-normal-before-tls.log`。合格には数えない。
通常コミット後の新HEADで全体normal、vendor契約、coverage、実invocation照合を回収する。
Windows実測、公開、上流ZIP拒否、最終配布受入は別の未完了条件である。
