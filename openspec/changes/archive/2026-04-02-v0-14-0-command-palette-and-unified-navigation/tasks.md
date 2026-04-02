## 1. Palette 契約

- [x] 1.1 他の AI エージェントが provider 種別、result 種別、実行 payload を誤解なく導けるように command-palette 契約を定義する
- [x] 1.2 command result、file result、Markdown content result、recent/common entry が 1 つの palette flow に共存する方式を定義する
- [x] 1.3 open、selection movement、confirm、dismiss に関する keyboard interaction 契約を定義する

## 2. Provider 統合

- [x] 2.1 palette から共通 application action を表示・実行できる command provider を追加する
- [x] 2.2 palette から一致 file を開ける workspace file provider を追加する
- [x] 2.3 content search が利用可能な時に一致文書を開き一致位置へ遷移できる Markdown content provider を追加する
- [x] 2.4 利用できない provider があっても palette 全体の体験が劣化しないようにする

## 3. UX 整合

- [x] 3.1 既存 file-search modal を即時削除せずに、palette を主要な高速導線にする
- [x] 3.2 empty query 時に役に立たない blank state ではなく有用な recent/common entry を提供する
- [x] 3.3 各 row が command、file、Markdown content hit のどれかを result 表示上で明確にする

## 4. 検証

- [x] 4.1. Linter Pass (`make check`)
  - [x] Extract magic numbers (`COMMAND_PALETTE_WIDTH`, margins, max items) into named constants.
  - [x] Extract `No results found.` text to localization strings.
  - [x] Replace `Color32::TRANSPARENT` with corresponding theme colors.
- [x] 4.2 palette result からの command execution、file open、Markdown content navigation を検証するテストを追加する
- [x] 4.3 keyboard-first の movement、confirm、dismiss 挙動を検証するテストを追加する
- [x] 4.4 一部 provider が unavailable でも palette が有用なままであることを確認する
- [x] 4.5 既存 file-search modal が compatibility fallback として引き続き動作することを確認する
