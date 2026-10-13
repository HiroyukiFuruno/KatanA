## ADDED Requirements

### Requirement: 共通 shell chrome から文書の共有更新を利用できる

システムは、共通 shell chrome から単一のアクティブ文書更新コントロールを提供しなければならず、そのコントロールはアクティブな文書が CodeOnly、PreviewOnly、Split のいずれの mode で表示されている場合でも利用可能でなければならない。

#### Scenario: CodeOnly mode で共有更新を使う

- **WHEN** アクティブな Markdown 文書が開かれており、ユーザーが CodeOnly mode に切り替えたとき
- **THEN** 共通 shell chrome で共有更新コントロールが表示されたままである
- **THEN** それを実行すると、他の view mode と同じ更新セマンティクスが適用される

#### Scenario: PreviewOnly または Split mode で共有更新を使う

- **WHEN** アクティブな Markdown 文書が開かれており、ユーザーが PreviewOnly または Split mode にいるとき
- **THEN** preview ローカルな代替手段を必要とせず、同じ共有更新コントロールを利用できる
- **THEN** 共有更新セマンティクスと異なる挙動を持つ第 2 の更新コントロールは存在しない

#### Scenario: アクティブな文書が選択されていない

- **WHEN** アクティブな文書が存在しないとき
- **THEN** 共有更新コントロールは無効化される
- **THEN** 更新を実行しても workspace や preview の状態は変更されない
