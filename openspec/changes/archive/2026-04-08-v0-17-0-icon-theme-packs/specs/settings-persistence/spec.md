## ADDED Requirements

### Requirement: 選択中の icon pack は永続化される

システムは、ユーザーが選択した icon pack を設定として保存し、次回起動時に復元しなければならない（MUST）。

#### Scenario: icon pack selection を保存する

- **WHEN** ユーザーが settings で icon pack を変更する
- **THEN** selected icon pack は settings JSON に保存される

#### Scenario: icon pack selection を復元する

- **WHEN** ユーザーが再起動後に KatanA を開く
- **THEN** 前回選択した icon pack が active になる
