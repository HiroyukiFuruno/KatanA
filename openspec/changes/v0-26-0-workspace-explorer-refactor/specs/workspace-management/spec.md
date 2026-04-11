# Workspace Management Specification

## Purpose

ワークスペース（ディレクトリ履歴）管理としてのサイドバーボタンとビューの振る舞いを定義します。

## Requirements

### Requirement: sidebar-icon

- **Given**: サイドバーが表示されている
- **When**: アクティビティレールを見る
- **Then**: ワークスペース管理のボタンは「フォルダ (FolderOpen)」アイコンを使用していること

### Requirement: i18n-label

- **Given**: 言語が設定されている（EN/JA）
- **When**: ワークスペースボタンにホバーする
- **Then**: ツールチップに「Workspace History」（英語）または「ワークスペース履歴」（日本語）と表示されること
