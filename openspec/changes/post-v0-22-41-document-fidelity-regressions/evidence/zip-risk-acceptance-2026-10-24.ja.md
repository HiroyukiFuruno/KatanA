# ZIPの利用経路と期限付きリスク受容（2026-10-24）

## 本人の承認と範囲

本人は、KatanA側で直せる箇所を修正し、残る脆弱性の実影響を確認して期限付きの例外として扱い、KDV/KRRの次版を待たずに公開する方針を明示承認した。追跡は [KatanA #354](https://github.com/HiroyukiFuruno/KatanA/issues/354)、上流の解消は [KDV #69](https://github.com/HiroyukiFuruno/katana-document-viewer/issues/69)。

対象はRUSTSEC-2026-0367/0368/0369のみ。未修正の依存がなくなったこと、全経路で悪用不能であること、安全な修正版を採用したことは主張しない。新規脆弱性、別の依存経路、ライセンス、build/test/lint/coverage/配布契約の条件を緩和する承認ではない。

## 現公開版と候補の比較

GitHub latestはv0.22.41、公開元commitは`af55949c4ee7ddad414ed93fa360a67e9464acf4`。Gitの当該Cargo.lockを直接解析し、候補`226b91a9`と比較した。これは公開元ソースの比較であり、全配布binaryの逆解析や実攻撃再現ではない。

| 項目 | 公開v0.22.41 | 調査時の候補 |
| --- | --- | --- |
| KDV | 0.5.5 | 0.5.12 |
| KRR | 0.4.17 | 0.4.23 |
| Office engine | office2pdf-katana0.6.10 | office2pdf0.8.1 |
| ZIP crate | 0.6.6/2.4.2/8.6.0 | 同3版とdirect9.0.5 |

候補だけに新規導入された脆弱性という説明は誤り。既存リスクの持越しと今回の公開による増分を別々に評価する。2026-10-24の通常互換更新で旧ZIP2系は2.6.1へ更新されるが、9未満を修正済みとは扱わない。

## 公式に報告された条件

- [0367](https://rustsec.org/advisories/RUSTSEC-2026-0367.html): ZIP展開時にSUID/SGID等を反映する問題。特権processで攻撃者のarchiveを展開した場合などに権限昇格へつながる。
- [0368](https://rustsec.org/advisories/RUSTSEC-2026-0368.html): 展開先外部へ向くsymlinkを生成し、後の読書きがprocess権限で外部対象へ到達する問題。
- [0369](https://rustsec.org/advisories/RUSTSEC-2026-0369.html): 展開したsymlink越しに外部対象のpermissionを変更する問題。RustSecの修正版範囲と[公式GHSA](https://github.com/zip-rs/zip2/security/advisories/GHSA-4684-h2q9-2vcp)の表記には9.0.0/9.0.1の差があり、direct9.0.5は両方を満たす。

ライブラリが依存に存在することは、これらの展開APIを製品で呼ぶ証明ではない。一方、呼出文字列が検索で見つからないことだけでも全実行経路の安全証明にはならない。

## 確認したOffice経路

公開registryから取得済みのKDV0.5.12、office2pdf0.8.1、ironcalc0.8.3、umya-spreadsheet2.3.3、docx-rs0.4.22のRust sourceを確認した。

- KDVの`multi_format/office_preflight_archive.rs`は`ZipArchive<Cursor<&[u8]>>`を生成し、`by_index`でentryを検査する。
- ironcalcの`import/mod.rs`、umyaの`reader/xlsx.rs`、docx-rsの`reader/read_zip.rs`はarchive内のXML/assetを`by_name`等で読み出す。
- 上記sourceで`ZipArchive::extract`/`ZipStreamReader`、展開したentryのpermissionを設定する該当callerは見つからなかった。KDVのworker出力検証にある`symlink_metadata`はリンクを作成する処理ではない。
- PDF/HTMLが直接この3件を悪用する経路も今回確認していない。KRRが今回のZIPゲートの原因であるとは扱わない。

この静的確認は既知3件の利用経路評価に限る。Office parser全般の安全保証、圧縮bombやXMLの別問題の否定、任意の外部converter全体の監査ではない。

## KatanAが所有する対処箇所

`crates/katana-core/src/update/download.rs`は、公開版と候補の両方で、`unix_mode()`の結果を`set_permissions`へ渡していた。ただし調査を進めると、候補のZIP9.0.5は`format/functions.rs`の`clear_specials_bits`でSUID/SGID/stickyを除去してから返すことを確認した。候補の当該処理まで特権bitをそのまま反映するとした初期説明は訂正する。ホストでの`0o777`制限は防御の補強であり、候補に新たな権限昇格を再現・解消した証明ではない。

通常の更新URLは公式GitHub Releaseに組み立て、展開先は専用tempdirである。したがって、任意のOffice文書を開く操作と更新archive展開は別の入口であり、普通のユーザー権限の起動だけでroot権限を得られるとは断定しない。攻撃者が更新archiveを制御できる等の前提を区別する。

ここは上流待ちにせず、ホストでのpermission制限、リンクentry・既存リンク出力先の拒否、通常の0755実行権限とnested bytesの維持を実ファイル回帰で検証する。

先行実行では、同じ実ZIP/filesystem回帰を旧実装へ適用し、archiveリンクentry、既存directoryリンク、既存最終pathリンクの3件が失敗した。修正後は通常Deflated/Xz・0755・bytes/progressを含む7件成功。`tmp/oct24-updater-zip-red-tests.log`と`tmp/oct24-updater-zip-green-final.log`へ保存した。これは公開APIへ渡された既存リンク先を拒否する補強の検証であり、製品の専用tempdirへ他ユーザーが侵入できたという証明ではない。規約対応の分割後は同じ検証を再実行し、最終sourceの結果を別記する。

## 解除条件と検証

例外期限は2026-11-07を上限とする。KDV #69を解消した公開registry版の採用、利用経路の変化、新たな悪用証拠の発見のいずれかがあれば期限前でも再評価する。自動延長せず、正式な供給網入口で期限切れ・許可していない依存経路を拒否する。

最終sourceの更新ZIP回帰は10件成功（`tmp/oct24-updater-zip-green-final-freeze.log`）。通常Deflated/Xz・bytes/progress・0755・nested出力とリンク拒否を確認した。既存macOS配布ZIPも実19entryを解析し、symlinkと特権bitを含まないことを確認した。既存ファイルへの上書きは拒否し、新規の専用展開先を使う通常更新を維持する。同一ユーザーが親pathを同時に入れ替える競合まで防げるという主張はしない。

期限・経路・source・固定上流版・未許可ZIPの拒否を含むpolicy単体16件が成功した。標準供給網入口へこの単体検査を接続し、実3manifestの全feature/locked graphと元cargo-denyのadvisories/bans/licenses/sourcesを実行してactual0を回収した（`tmp/oct24/check-supply-chain-connected.log`）。入口・3manifest・全検査維持のrelease契約10件も成功した（`tmp/oct24-release-flow-contract.log`）。直接cargo-denyは引き続きadvisoryを検出する。通常push、current review、全CI、公開は別の未完工程である。

検査toolは公開最新版のcargo-deny0.20.2を維持する。公式Web文書には`expiry`/`allow`があるが、実0.20.2は`unexpected-keys`で拒否し、相対期限の起点もadvisory DBのreported日付とWebのissued日付で異なる。未対応設定や日付推測に依存しないよう、正規`check-supply-chain.sh`でUTCの絶対期限とlocked全feature依存経路を検証した後だけ、一時設定へ3件を追加して元の全cargo-deny検査を実行する方式を検証中。Git管理された`deny.toml`は変更せず、直接cargo-denyを実行すれば引き続き対象advisoryで拒否する。固定期限は2026-11-07 00:00 UTC（日本時間09:00）であり自動延長しない。
