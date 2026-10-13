## ADDED Requirements

### Requirement: Global command palette を開ける

システムは、shortcut または UI action から開けて、直ちに keyboard focus を受け取る global command palette を提供しなければならない（SHALL）。

#### Scenario: Palette を開く

- **WHEN** ユーザーが command palette action を実行した時
- **THEN** システムは command palette を開く
- **THEN** palette input は直ちに入力できる focus 状態になる

#### Scenario: Palette を閉じる

- **WHEN** ユーザーが command palette を dismiss した時
- **THEN** palette は意図しない action を発火させずに閉じる

### Requirement: Command results を実行できる

システムは、command palette 上に実行可能な application command を表示し、result list から実行できなければならない（SHALL）。

#### Scenario: Query から command を実行する

- **WHEN** ユーザーが利用可能な application command に一致する query を入力した時
- **THEN** palette は一致する command result を表示する
- **THEN** その result を選択すると対応する command が実行される

#### Scenario: Empty query で common actions を表示する

- **WHEN** ユーザーが empty query の状態で command palette を開いた時
- **THEN** palette は common action または recent action を即時選択候補として表示できる

### Requirement: Workspace file navigation を palette から実行できる

システムは、command palette から workspace file を検索して開けなければならない（SHALL）。

#### Scenario: File result を開く

- **WHEN** ユーザーが workspace file path に一致する query を入力した時
- **THEN** palette は file navigation result を表示する
- **THEN** file result を選択すると、その文書が開かれる

### Requirement: Markdown content navigation を palette から実行できる

システムは、Markdown content search が利用可能な時、command palette から Markdown content search result へ遷移できなければならない（SHALL）。

#### Scenario: Markdown content result を開く

- **WHEN** ユーザーが indexed Markdown content に一致する query を入力した時
- **THEN** palette は識別に十分な context を伴う Markdown content result を表示する
- **THEN** content result を選択すると、対象文書を開いて一致位置へ遷移する

### Requirement: Palette results は keyboard-first に操作できる

システムは、command palette 内で keyboard-first な result navigation と execution をサポートしなければならない（SHALL）。

#### Scenario: Keyboard で結果を移動する

- **WHEN** command palette が開いていて複数 result が available な時
- **THEN** ユーザーは keyboard により active selection を result list 内で移動できる

#### Scenario: Keyboard で結果を確定する

- **WHEN** command palette が active な result selection を持つ時
- **THEN** ユーザーは keyboard により active result を実行できる
