## 現状整理

`vendor-subtree` は KatanA の Markdown rendering fork を保守しやすくするためのメンテナンス change であり、ユーザー向け機能追加ではない。目的は、active な parser bug fix と repository 構造の入れ替えを同じ change に混ぜず、継続的な vendor 保守を追いやすくすることにある。

## 現状分析

### 1. vendor の実スコープは `vendor/egui_commonmark` より広い

- `Cargo.toml` は `egui_commonmark` と `egui_commonmark_backend` の両方を local path へ patch している
- `crates/katana-ui/src/svg_loader/mod.rs` は `vendor/egui_commonmark_backend/src` から `copy.svg` と `check.svg` を直接読み込んでいる
- 両 crate の upstream repository は同じ `https://github.com/lampsitter/egui_commonmark`
- upstream repository は `egui_commonmark`、`egui_commonmark_backend`、`egui_commonmark_macros` を sibling crate として持つ workspace layout である

結論として、`vendor/egui_commonmark` だけを対象にした計画では不十分である。

### 2. 現行 vendor directory には既に Katana-specific な挙動が入っている

upstream `v0.22.0` と比較した時点で、少なくとも次の local delta がある。

- `vendor/egui_commonmark/Cargo.toml.orig` に `katana-core` 依存が追加されている
- `vendor/egui_commonmark/src/parsers/pulldown.rs` に Katana 固有の inline emoji rendering がある
- `vendor/egui_commonmark/src/lib.rs` で内部 module 公開範囲が拡張されている
- `vendor/egui_commonmark_backend/src/alerts.rs` と `src/elements.rs` に UI / layout の local 調整がある
- `vendor/egui_commonmark_backend/src/check.svg` と `src/copy.svg` は local asset として使われている

つまり subtree migration は単なる file move ではなく、既存 patch stack を棚卸しし、保存し、分離する change として扱わなければならない。

### 3. product code と test は vendor fork に直接依存している

- `crates/katana-core/src/preview/mod.rs` は `egui_commonmark` 制約に由来する挙動を前提に説明している
- `crates/katana-ui/tests/underline_rendering.rs` は vendor parser override を期待動作として参照している
- `openspec/changes/v0-8-7-preview-refresh-and-tasklist-fixes/tasks.md` では `egui_commonmark` と `egui_commonmark_backend` の両方に新しい vendor-side 修正を入れる計画になっている

よって subtree 化自体には価値があるが、`v0-8-7` がまだ同じ表面を変更している最中に混ぜると churn が増え、review 品質も下がる。

### 4. 目標は「最新 upstream」ではない

- 現行 KatanA は `egui 0.33` と `egui_commonmark 0.22.0` を前提にしている
- upstream HEAD はすでに workspace version `0.23.0`、`egui 0.34.0`、新しい workspace metadata へ進んでいる

したがって本 change は、`0.22.x` 系の互換 revision、または明示的な互換 fork revision に pin しなければならない。subtree migration と dependency upgrade は別 change として扱う。

## あるべき状態

この change 完了後は、次の状態になっているべきである。

1. upstream `lampsitter/egui_commonmark` repository が `vendor/egui_commonmark_upstream/` 配下に 1 回だけ表現されている
2. `Cargo.toml` は `egui_commonmark` と `egui_commonmark_backend` をその subtree root 内の crate subdirectory から解決している
3. Katana-specific な変更は raw subtree import の上に乗る review 可能な commit として再導入され、opaque な copied snapshot に埋もれていない
4. SVG asset include などの direct product reference は新しい subtree layout を向いている
5. 将来の maintainer が会話履歴に頼らず upstream pull、patch 再適用、regression verification を実行できる runbook がある

## 非目標

- KatanA を upstream `egui_commonmark 0.23.x` や `egui 0.34.x` へ上げること
- KatanA の Markdown rendering architecture を設計し直すこと
- active な vendor bug fix を subtree migration branch へ同居させること

## 設計判断

### 1. 必要性は medium であり、blocking ではない

vendor-local patch が recurring な product work になっているため、整備する価値はある。ただし `v0-8-7` や他の user-facing feature を直接 unblock する change ではない。したがって、適切な実施タイミングは `v0-8-7` 安定化直後から次の vendor-touching branch 開始前までの間である。

### 2. subtree の単位は upstream repository root とする

upstream repository には `egui_commonmark`、`egui_commonmark_backend`、`egui_commonmark_macros` が同居している。そのため、subtree は `vendor/egui_commonmark_upstream/` のような単一 prefix 配下に upstream workspace root を取り込む必要がある。

この判断が必要な理由:

- upstream の実 layout をそのまま保持できる
- version を共有する sibling crate 間の drift を防げる
- 現在使っている 2 crate を別々の疑似 upstream として管理せずに済む
- 将来 `egui_commonmark_macros` を使う場合でも、repository 構造変更をやり直さずに済む

### 3. migration は互換 upstream revision を対象にしなければならない

実装前に、`0.22.x` 系の revision、または別途用意した互換 revision を確定する。もし current `egui 0.33` stack と両立する upstream revision が存在しないと分かった場合は、次のどちらかを選ぶ。

- Katana fork branch を subtree remote の対象にする
- dependency upgrade を含む別 change に広げる前提で artifact を先に更新する

最新 upstream への upgrade に silently すり替えてはならない。

### 4. patch layering は明示的に分ける

migration の commit stack は次の責務境界を保つ。

1. 選定した upstream revision の raw subtree import
2. Katana 側の path 張り替え (`Cargo.toml`、asset include、direct reference)
3. Katana-specific vendor patch
4. sync/runbook 文書

この分離がないと、将来の maintainer が local divergence を確認する際に巨大な mixed commit を逆算することになる。

### 5. direct consumer の張り替えは同一 change で完了させる

subtree migration は、legacy vendor directory を使う direct consumer を残したままでは不完全である。最低限、次は同じ change で切り替える。

- `Cargo.toml` の `[patch.crates-io]`
- `crates/katana-ui/src/svg_loader/mod.rs`
- legacy vendor path を直書きした test や comment

### 6. 中核前提が崩れた場合は artifact を先に直す

inventory phase を越える前に、次のいずれかが崩れたと分かった場合は `proposal / design / spec / tasks` を先に更新する。

- 選定した upstream-compatible revision が実際には KatanA の依存関係と両立しない
- upstream workspace layout が想定した root + crate-subdirectory model と異なる
- Katana patch inventory が想定より大きく、この artifact では足りない
- legacy vendor path に依存する product-critical consumer が追加で見つかった

この change は、artifact が reality に先行している限りでのみ「実装可能」と見なせる。

## 実装設計

### Phase 1: スコープを固定し、現在の patch set を棚卸しする

- current `vendor/egui_commonmark` と `vendor/egui_commonmark_backend` を、選定した互換 upstream revision と diff する
- 各 delta を次のいずれかへ分類する
  - 保存すべき Katana patch
  - 除去してよい drift / generated-file noise
  - 新しい subtree root に伴う path/layout 差分
- raw subtree import を commit する前に、最終 patch inventory を記録する

### Phase 2: upstream repository root を subtree として取り込む

- upstream repository を `vendor/egui_commonmark_upstream/` 配下へ追加する
- upstream HEAD ではなく、選定した互換 revision に pin する
- raw import commit の中へ Katana local change を混ぜない

### Phase 3: Katana 側を subtree layout へ張り替える

- `Cargo.toml` の `[patch.crates-io]` path を `vendor/egui_commonmark_upstream/...` へ更新する
- `crates/katana-ui/src/svg_loader/mod.rs` と、他の direct file reference を新しい asset path へ更新する
- 旧 `vendor/egui_commonmark` と `vendor/egui_commonmark_backend` path への runtime / build 依存を除去する

### Phase 4: Katana-specific patch を上に再適用する

- `katana-core` 統合など、必要な crate-manifest 変更を戻す
- Phase 1 で棚卸しした parser / UI patch を再適用する
- underline rendering、inline emoji support、alert / code-block UI、`v0-8-7` で入る vendor fix など、vendor-dependent behavior の regression coverage を維持または拡張する

### Phase 5: 保守 runbook を文書化する

- 例えば `docs/vendor-egui-commonmark.md` のような runbook を追加し、次を明記する
  - subtree remote と prefix
  - 互換 upstream revision を pull する手順
  - Katana-specific patch がどこにあるべきかという考え方
  - merge 前の必須 verification command

## 検証方針

この change で最低限必要な verification は次のとおり。

- `make check`
- `katana-ui` と `katana-core` の vendor regression test
- 削除した legacy vendor directory を参照する code path が残っていないことの manual audit
- artifact を広げずに進める限り、subtree revision が `0.22.x` 系の互換 line に留まっていることの確認

## 推奨実施タイミング

- `v0-8-7-preview-refresh-and-tasklist-fixes` が merge 済みかつ stable になった後
- 次に `vendor/*egui_commonmark*` を編集する branch が始まる前
- feature work と束ねない dedicated maintenance branch 上
