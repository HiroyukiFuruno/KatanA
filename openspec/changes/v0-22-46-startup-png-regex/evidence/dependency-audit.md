# 依存更新の dry-run 判定

## 結果

root workspace と `scripts/screenshot` の両方で `cargo update --dry-run --locked` は exit 0。実ファイルへは書き込まず、両方とも同じ2件だけを最新の互換版として提示した。

- `unicode-properties` 0.1.4 → 0.1.5
- `yazi` 0.2.1 → 0.2.2

実行ログは [`compatible-updates.log`](compatible-updates.log)。root は13件、screenshot は9件について「利用可能な新しい版はあるが今回は不変」と報告した。

## 保持した候補と根拠

- egui 基盤5件 (`eframe`, `egui`, `egui_extras`, `egui_kittest`, `epaint_default_fonts`) は 0.36.2 → 0.37.0 が存在するが、workspace/screenshot の manifest で `=0.36.2` に固定。vendor fork も0.36.2系列のため、単独更新せず揃った移行として評価する。
- `libc` は lock 0.2.190 に対し0.2.191が利用可能。現行registry `katana-document-viewer 0.5.12` は `libc =0.2.190` を要求するため、0.2.191指定のdry-runは解決失敗 (`precise-libc.log`)。
- `katana-ui-core` 0.4.4は利用可能だが、KDV 0.5.12が `=0.4.1` を要求する。0.4.4指定のdry-runは失敗 (`precise-katana-ui-core.log`)。
- `generic-array` 0.14.9は利用可能だが、KDV 0.5.12 → office2pdf 0.8.1 → umya-spreadsheet 2.3.3 → aes/cipher → crypto-common 0.1.7 が `=0.14.7` を要求する。0.14.9指定は失敗 (`precise-generic-array.log`)。
- `pest` 2.9.5は利用可能だが、追加依存 `pest_stack_guard` が `psm =0.1.24` を要求し、既存 graph の `stacker 0.1.25` が選択する `psm 0.1.32` と解決できない。2.9.5指定は失敗 (`precise-pest.log`)。
- `parking_lot` 0.13.0は利用可能だが、manifest が `=0.12.5` に固定している。

v0.22.45で更新済みの `zip 9.2.0`、`fs4 1.1.0`、`encoding_rs 0.8.44`、`log 0.4.35`、`webbrowser 1.2.6` は前回取得した crates.io index と一致。KDV/KRRの固定版更新やUI基盤の0.x minor更新は、このpatchのdry-run候補だけでは採用根拠にならないため保留。

## 採用と検証

診断後、unicode-properties 0.1.5 と yazi 0.2.2 を両lockへ適用した。採用後の必須品質ゲートはtasks.mdで追跡する。dry-runは品質ゲート成功の代用にしない。

## parking_lot 0.13.0 移行可否（read-only）

### 判定

利用APIは狭く、ソース上の機械的な移行量は小さそう。ただし現行lock graphでは `parking_lot 0.13.0` が要求する `parking_lot_core 0.10.0` の `libc ^0.2.191` と、公開 `katana-document-viewer 0.5.12` の `libc =0.2.190` が衝突する。2本のKatanA manifestだけを0.13へ置き換えた一時overlayの `cargo metadata` も解決失敗した（[`parking-lot-metadata.stderr`](parking-lot-metadata.stderr)）。したがって現行KDVでの採用は不可。KDV側の互換registry版が出るまで保留が妥当で、別repoは変更していない。

### KatanA内の利用範囲

- 直接依存は `crates/katana-ui/Cargo.toml` と `crates/katana-platform/Cargo.toml` の両方で `parking_lot = "=0.12.5"`。UIだけではない。
- 利用は `RwLock<T>`、`RwLockReadGuard` / `RwLockWriteGuard`、`RwLock::new`、`.read()`、`.write()`。場所は platform cache (`crates/katana-platform/src/cache/mod.rs`, `cache/default/mod.rs`, `cache/memory/mod.rs`, `cache/types.rs`) と UI locale state (`crates/katana-ui/src/i18n/logic.rs`)。
- `read_recursive`, `RecursiveRwLock`, raw lock traits, `const_new`, `hardware-lock-elision`, `lock_api/nightly`, `UnparkResult::be_fair`, fair-unlock APIs の利用は `crates/` で見つからなかった。static lockは `LazyLock(|| RwLock::new(...))`。
- Workspace MSRVは1.95.0で、新版のMSRV1.95は一致する。

### 公開版 0.13.0 の差分・移行コスト

公式CHANGELOGは、parking_lot 0.13.0 / parking_lot_core 0.10.0 / lock_api 0.5.0 を2026-10-21付でまとめ、MSRVを1.95へ引き上げ、recursive read APIsを `RecursiveRwLock` に分離、const constructor群・hardware-lock-elision・自動 eventual fairness を削除、raw lock/guard state APIを変更したと記録する。現コードに該当する削除API利用は見つからず、型付きRwLockの呼び出し側修正は限定的に見える。一方、公平性と競合時のwake-up時点が変わるため、実装コンパイルだけで十分とは言えず、lock contention / cache / i18n の自動回帰が必要。今回はcompile/build禁止のため互換性を確定していない。

公式資料: [parking_lot CHANGELOG](https://github.com/Amanieu/parking_lot/blob/master/CHANGELOG.md)。公開index上の0.13.0 metadataでは `lock_api ^0.5.0`, `parking_lot_core ^0.10.0`, Rust 1.95、yanked=false。

### lock graphへの影響

現行lockには parking_lot 0.12.5 → lock_api 0.4.14 / parking_lot_core 0.9.12 がある。逆引きでは `comemo`, `eframe`, `epaint` など複数の既存依存も0.12系を利用する。KatanAの2 direct pinだけを0.13へ上げても0.12.5と0.13.0、lock_api 0.4/0.5、parking_lot_core 0.9/0.10の並存が見込まれる。API型は通常別crate境界に閉じるが、lockfile/バイナリ増加と型の混在を確認する必要がある。

一時overlayでは両direct pinを `=0.13.0` にし、依存graphのみ確認した。`parking_lot_core 0.10.0` が `libc ^0.2.191` を要求する一方、保持中KDV 0.5.12のexact `libc =0.2.190` と現UI側診断pinが一致せず解決に失敗した。通常manifest/lockは変更していない。cargo metadataの失敗ログは具体的な実装失敗ではなく、この依存選択衝突の結果。

### 判断材料

0.13は少数API利用という理由だけで拒否する必要はないが、現在は上流KDVとの依存境界で解決できず、今回すぐ更新する案は採れない。KDVの公開版がlibc .191以降に追随した後、KatanAの両direct pinを一緒に更新し、0.12/0.13並存のlock graphとconcurrency回帰を確認するのが実行可能な再評価条件。
