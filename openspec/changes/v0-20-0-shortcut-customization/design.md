## Context

現行 shortcut は `shell_ui.rs` 内で個別に `consume_shortcut` されており、設定変更・一覧表示・競合検知の仕組みが存在しない。`v0.19.0` で command inventory を整備した前提で、`v0.20.0` では user-facing shortcut layer を追加する。

## Goals / Non-Goals

**Goals:**

- user が command ごとの shortcut を一覧表示・変更できる
- duplicate binding を保存前に防ぐ
- conflict 時に既存割当先を popup で提示する
- settings persistence に shortcut schema を追加する

**Non-Goals:**

- OS のすべての reserved shortcut を完全再現すること
- user-defined macro system
- command inventory 自体の再設計

## Decisions

### 1. shortcut schema は command inventory key を primary key にする

display label ではなく command key で永続化する。
さらに、v0.17.0 でのクロスプラットフォーム対応と連携し、ショートカットスキーマは設定ファイル内で OS ごと（macOS, Windows, Linux）に独立したキーマッププロファイルを持つ構造とし、OS 固有の Modifier 競合を防ぐ。

- 採用理由:
  - i18n や label 変更に影響されない
  - OS 間のショートカット差異をセキュアに吸収できる
- 代替案:
  - label ベースで保存する: rename で壊れるため不採用

### 2. duplicate binding は保存前 validation で拒否する

要求どおり、既に登録がある shortcut は再登録させない。

- 採用理由:
  - runtime ambiguity を防げる
- 代替案:
  - last write wins: 既存割当が見えなくなるため不採用

### 3. conflict popup には既存割当 command を表示する

ユーザー要望の「別途何に設定されているかをポップアップ」に合わせ、conflict 時は割当先 command を user-facing label で表示する。

## Risks / Trade-offs

- **[Risk] OS reserved shortcut の扱いが曖昧だと false conflict / missed conflict が起きる**  
  -> Mitigation: app-local collision を最低保証とし、OS reserved は known blocked set として段階導入する
- **[Risk] shortcut schema migration で default 更新が壊れる**  
  -> Mitigation: schema versioning と restore defaults を用意する

## Migration Plan

1. command inventory keys と default shortcuts を定義する
2. settings schema と persistence を追加する
3. runtime dispatcher を hard-coded shortcut から inventory-driven shortcut に移行する
4. settings UI と conflict popup を実装する

## Open Questions

- OS reserved shortcut の blacklist を `v0.20.0` でどこまで含めるか
- chord shortcut を許可するかは将来に回してよいか
