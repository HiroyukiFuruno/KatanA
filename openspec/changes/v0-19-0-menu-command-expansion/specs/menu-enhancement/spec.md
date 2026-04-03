## ADDED Requirements

### Requirement: File menu は document and workspace commands を提供しなければならない

システムは、File menu から workspace と document の基本操作へ到達できなければならない（SHALL）。

#### Scenario: workspace と document commands を開く

- **WHEN** ユーザーが File menu を開く
- **THEN** Open Workspace、Save、Export、Refresh 相当の command に到達できる

#### Scenario: command が利用不能である

- **WHEN** active document または workspace が存在せず command の前提条件を満たさない
- **THEN** 該当 menu item は disabled で表示されるか、同等に実行不能であることが明示される

### Requirement: View menu は navigation and visibility commands を提供しなければならない

システムは、View menu から navigation と pane visibility の主要操作へ到達できなければならない（SHALL）。

#### Scenario: navigation commands を開く

- **WHEN** ユーザーが View menu を開く
- **THEN** Command Palette、TOC toggle、workspace toggle、diagnostics 相当の command に到達できる

### Requirement: Help menu は support and release information を提供しなければならない

システムは、Help menu から support resources と release information へ到達できなければならない（SHALL）。

#### Scenario: help resources を開く

- **WHEN** ユーザーが Help menu を開く
- **THEN** GitHub Repository、documentation、release notes、check for updates 相当の command に到達できる
