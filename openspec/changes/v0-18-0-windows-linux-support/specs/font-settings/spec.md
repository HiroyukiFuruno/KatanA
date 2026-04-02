## MODIFIED Requirements

### Requirement: フォントファミリー設定

システムは、プレビューとエディタのフォントファミリーを変更できなければならない（MUST）。フォントファミリーが未設定または platform candidate が見つからない場合でも、対応 desktop OS 上で readable な fallback により表示を継続しなければならない（MUST）。

#### Scenario: フォントファミリーの変更

- **WHEN** ユーザーが設定でフォントファミリーを変更する
- **THEN** エディタとプレビューのフォントが即座に切り替わる

#### Scenario: デフォルトフォント

- **WHEN** フォントファミリーが未設定である
- **THEN** current platform で最初に解決できた default candidate が使用される
- **THEN** candidate が見つからない場合でも app は crash せず、renderer の既定フォントで継続する

#### Scenario: フォントファミリーの永続化

- **WHEN** ユーザーがフォントファミリーを変更する
- **THEN** 設定が保存され、次回起動時に復元される

## ADDED Requirements

### Requirement: emoji font fallback は対応 OS で recoverable に扱われる

システムは、emoji 用フォントが利用可能な場合はそれを優先し、利用できない場合でも crash せずに recoverable fallback で描画しなければならない（MUST）。

#### Scenario: macOS で color emoji font が利用可能である

- **WHEN** macOS 上で Apple Color Emoji が利用可能である
- **THEN** KatanA はそれを優先して emoji を描画する

#### Scenario: Windows / Linux で color emoji font が利用できない

- **WHEN** Windows または Linux 上で emoji font candidate が見つからない
- **THEN** アプリケーションは crash せずに既定 renderer で表示を継続する
