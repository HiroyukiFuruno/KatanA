## Purpose

This is a legacy capability specification that was automatically migrated to comply with the new OpenSpec schema validation rules. Please update this document manually if more context is required.

## Requirements

### Requirement: テーマ設定（Dark / Light）

Dark / Light の2種類のテーマを提供し、ユーザーが切り替え可能にする。デフォルトはDark。 The system SHALL conform.

#### Scenario: テーマの切り替え

- **WHEN** ユーザーが設定メニューからテーマをDarkからLightに変更する
- **THEN** アプリ全体の配色がLightテーマに切り替わる

#### Scenario: デフォルトテーマ

- **WHEN** アプリを初回起動する
- **THEN** Darkテーマが適用されている

#### Scenario: テーマ設定の永続化

- **WHEN** ユーザーがテーマを切り替える
- **THEN** 選択したテーマが設定に保存され、次回起動時に復元される

#### Scenario: テーマ切替の即時反映

- **WHEN** テーマを切り替える
- **THEN** 再起動なしで即座にUI全体に反映される

### Requirement: テーマ変更は再起動なしでダイアグラムプレビューへ反映される

システムは、アプリケーションの再起動を要求せずに、runtime のテーマ変更をダイアグラムプレビュー描画へ反映しなければならない（SHALL）。

#### Scenario: テーマモードが切り替わる

- **WHEN** ユーザーが dark theme と light theme の間で切り替えた時
- **THEN** ダイアグラムプレビューは新しく active になったテーマスナップショットで更新される

#### Scenario: プレビュー文字色が変更される

- **WHEN** ユーザーが設定 UI から preview 専用の文字色を変更した時
- **THEN** ダイアグラムプレビューは新しい色設定で更新される
- **THEN** 結果は以前のテーマではなく、現在の preview theme と一致する
