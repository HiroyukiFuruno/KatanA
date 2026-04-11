# Explorer Sidebar Specification

## Purpose

エクスプローラー（ファイルブラウザ）としてのサイドバーボタンとツリービューの振る舞いを定義します。

## Requirements

### Requirement: sidebar-icon

- **Given**: サイドバーが表示されている
- **When**: アクティビティレール（左端のアイコン列）を見る
- **Then**: エクスプローラーのボタンは「マルチファイル (Files)」アイコンを使用していること

### Requirement: i18n-label

- **Given**: 言語が設定されている（EN/JA）
- **When**: サイドバーボタンにホバーする
- **Then**: ツールチップに「Explorer」（英語）または「エクスプローラー」（日本語）と表示されること
