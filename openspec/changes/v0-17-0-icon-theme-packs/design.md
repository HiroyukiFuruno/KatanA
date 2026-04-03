## Context

現在の KatanA の icon system は、repository 配下の `assets/icons/*.svg` を `include_bytes!` で埋め込み、runtime では tint を前提に描画する構成になっている。

- `crates/katana-ui/src/icon.rs`
  - icon は enum と `include_bytes!` で固定されている
  - current implementation では single built-in set 前提である
- `crates/katana-ui/src/svg_loader/mod.rs`
  - SVG は rasterize して表示できる
  - icon ごとの color policy は持っていない
- `crates/katana-linter/src/rules/domains/assets/svg.rs`
  - `assets/icons` 直下の SVG を white / none / currentColor 前提で検証している

ユーザー要望は、git に入れない動的取り込みではなく、review 可能で安全な built-in pack 管理へ寄せること、既存 icon を `katana-icon` としつつ Material 系を含む複数 pack を選べるようにすること、さらに colorful icon にも対応することである。

## Goals / Non-Goals

**Goals:**

- icon assets を git 管理下の built-in pack として整理する
- `katana-icon` を既存 default pack として明示する
- curated external pack を 5 種類追加し、settings から切り替えられるようにする
- monochrome tint と native color の両方を扱える render policy を定義する
- selected pack で不足または非互換な icon を KatanA authored override で補完する
- license / provenance / override 方針を repository 内で監査可能にする

**Non-Goals:**

- ユーザーが任意の SVG pack を runtime import する機能
- ネットワーク経由の icon pack download
- icon marketplace 的な配布機能
- per-icon 単位の user customization

## Decisions

### 1. icon pack はすべて repository 同梱の built-in asset とする

動的 import は柔軟だが、license 事故、壊れた SVG、環境差分、サポート負荷を増やす。今回の目的は user customization の自由度よりも、品質と配布再現性を確保した pack selection にあるため、`v0.17.0` では repository 同梱に固定する。

- 採用理由:
  - CI / release / review で再現できる
  - license inventory を repository と一緒に監査できる
  - icon 容量が小さく、git 管理コストが低い
- 代替案:
  - user local import を初期スコープに含める: validation と support cost が増えすぎるため不採用

### 2. existing icon set は `katana-icon` として pack 化する

既存資産を置き換えるのではなく、pack system の baseline として明示的に名前を与える。default は `katana-icon` とし、既存画面の見た目を破壊しない。

- 採用理由:
  - regression を最小化できる
  - external pack 比較の基準になる
- 代替案:
  - existing icons を migration 없이別名 pack に置き換える: rollback しづらいため不採用

### 3. initial curated pack は `katana-icon` + external 5 種類に固定する

初期リリースでは「何でも入れられる」ではなく、審査済み curated set に限定する。初期 external pack は次を対象とする。

- `material-symbols`
- `lucide`
- `tabler-icons`
- `heroicons`
- `feather`

これらは commercial use と redistribution の観点で扱いやすい permissive license を持つ pack を優先し、license text と provenance を repository に同梱する。

- 採用理由:
  - 5 種類あれば選択肢として十分広い
  - いずれも広く使われており visual language が分かりやすい
- 代替案:
  - pack 数を無制限に増やす: review / override / license 管理コストが跳ねるため不採用
  - Material 系だけに絞る: variation が不足するため不採用

### 4. pack completeness は `Icon` enum 全件解決を基準にする

runtime fallback を `katana-icon` に寄せるだけだと、selected pack の visual language が部分的に壊れる。したがって shipping pack は原則として `Icon` enum 全件を解決できる状態にする。不足 icon や non-fit icon は pack ごとの override asset で埋める。runtime fallback は corruption や見落としに対する最後の safety net とする。

- 採用理由:
  - selected pack の統一感を保てる
  - 「切り替えたのに一部だけ別デザイン」を減らせる
- 代替案:
  - missing icon を常に `katana-icon` fallback に任せる: 見た目の一貫性が崩れるため不採用

### 5. render policy は pack metadata で `TintedMonochrome` と `NativeColor` を切り替える

現在は tint 前提のため、白基調 SVG 以外を綺麗に扱えない。pack ごとに render policy を持たせる。

- `TintedMonochrome`
  - 既存と同じく UI theme の text color 等で tint する
- `NativeColor`
  - SVG が持つ色をそのまま使う

必要であれば将来 `per-icon override` へ拡張できるが、`v0.17.0` では pack 単位を基本にする。

- 採用理由:
  - monochrome pack と colorful pack を同じ仕組みで扱える
  - API の複雑さを抑えられる
- 代替案:
  - 全 icon を native color にする: theme 一貫性と contrast 制御が崩れるため不採用
  - 全 icon を tint 維持にする: colorful support が達成できないため不採用

### 6. settings から pack selection を行い、即時反映する

icon pack は user-facing theme choice に近いため、settings から選択し、再起動なしで反映できる必要がある。selected pack は settings に保存する。

- 採用理由:
  - choice の結果をすぐ比較できる
  - app 再起動を要求しない方が UX がよい
- 代替案:
  - launch argument や config 手編集のみ: user-facing feature として弱いため不採用

### 7. license / provenance / override rationale は pack ごとに repository に残す

third-party icon を同梱する以上、出典、license、override の有無を後から監査できる必要がある。`docs/licenses/icon-packs.md` のような inventory を作り、各 pack の source、license、KatanA authored override の範囲を記録する。

- 採用理由:
  - later audit と pack 更新時の判断材料になる
  - 「どこまで third-party でどこからオリジナルか」が分かる
- 代替案:
  - LICENSE ファイルだけ置いて運用で覚える: provenance が散逸するため不採用

## Risks / Trade-offs

- **[Risk] 5 種類の external pack を同時追加すると override 作業量が大きい**  
  -> Mitigation: `Icon` enum 全件を coverage table で管理し、足りないものだけ KatanA authored override を作る
- **[Risk] colorful pack が light/dark theme で contrast を崩す**  
  -> Mitigation: pack metadata に preview policy を持たせ、必要なら colorful pack を native color でも palette 制約付きにする
- **[Risk] license inventory が更新されず陳腐化する**  
  -> Mitigation: pack 追加・更新 task に inventory 更新を必須化する
- **[Risk] current white-only linter が pack system と衝突する**  
  -> Mitigation: validation rule を pack policy aware に更新する

## Migration Plan

1. existing `assets/icons` を `katana-icon` pack として再編する
2. `Icon` registry と asset lookup を pack-aware に変更する
3. render policy を導入し、tint / native color の両方を扱えるようにする
4. settings に selected pack を追加し、preview と即時反映を実装する
5. curated external 5 pack を追加し、coverage table と override asset を揃える
6. license / provenance docs を整備し、validation と tests を追加する

## Open Questions

- colorful pack のうち、native color を全面許可する pack と accent 制約をかける pack をどう線引きするか
- future release で user import を追加するかどうか
