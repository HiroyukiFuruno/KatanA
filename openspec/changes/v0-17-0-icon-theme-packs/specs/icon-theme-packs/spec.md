## ADDED Requirements

### Requirement: built-in icon pack を選択できる

システムは、git 管理下で同梱された built-in icon pack から active pack を選択できなければならない（MUST）。

#### Scenario: default pack は `katana-icon` である

- **WHEN** ユーザーが icon pack を未変更のまま KatanA を起動する
- **THEN** active pack は `katana-icon` である

#### Scenario: settings から icon pack を切り替える

- **WHEN** ユーザーが settings UI で別の built-in icon pack を選択する
- **THEN** UI icon は再起動なしで選択した pack に切り替わる

#### Scenario: 初期 curated pack が利用可能である

- **WHEN** ユーザーが icon pack 一覧を開く
- **THEN** `katana-icon` に加えて curated external pack 5 種類が選択肢として表示される

### Requirement: selected pack は icon contract を満たす

システムは、selected pack に対して KatanA が要求する icon contract を満たさなければならない（MUST）。

#### Scenario: selected pack が icon を提供する

- **WHEN** UI が `Icon` enum に対応する icon asset を解決する
- **THEN** selected pack の asset が使われる

#### Scenario: third-party pack に直接対応 icon がない

- **WHEN** selected pack の third-party source に KatanA が要求する icon が存在しない、または visual language が合わない
- **THEN** その pack 向けに用意された KatanA authored override icon が使われる

#### Scenario: asset 解決が壊れている

- **WHEN** selected pack の asset が破損している、または build artifact に含まれていない
- **THEN** システムは recoverable fallback として `katana-icon` の対応 icon を使う

### Requirement: icon pack は color-aware rendering policy を持つ

システムは、icon pack ごとに定義された rendering policy に従って icon を描画しなければならない（MUST）。

#### Scenario: monochrome tint pack を描画する

- **WHEN** active pack の rendering policy が `TintedMonochrome` である
- **THEN** icon は active theme に応じた tint を受ける

#### Scenario: colorful pack を描画する

- **WHEN** active pack の rendering policy が `NativeColor` である
- **THEN** icon は SVG 自身の色を維持して描画される

### Requirement: bundled icon pack の provenance を追跡できる

システムは、bundled icon pack ごとに source、license、override 範囲を repository で追跡できなければならない（MUST）。

#### Scenario: icon pack のライセンスを確認する

- **WHEN** maintainer が bundled icon pack の出典を確認する
- **THEN** source URL、license、KatanA authored override の有無が文書で分かる
